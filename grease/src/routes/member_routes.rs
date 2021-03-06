//! All member-focused routes.

use super::basic_success;
use crate::auth::User;
use crate::check_for_permission;
use crate::db::models::member::MemberForSemester;
use crate::db::*;
use crate::error::*;
use db::schema::Enrollment;
use serde_json::{json, Value};
use std::collections::HashSet;

/// Log in to the API.
///
/// ## Input Format:
///
/// Expects a [LoginInfo](crate::db::models::LoginInfo).
///
/// ## Return Format:
///
/// ```json
/// {
///     "token": string
/// }
/// ```
///
/// Returns an object with an API token unique to the member. Logging in
/// multiple times will return the existing token instead of generating
/// another one.
pub fn login(form: LoginInfo) -> GreaseResult<Value> {
    let conn = connect_to_db()?;
    if let Some(_member) = Member::check_login(&form.email, &form.pass_hash, &conn)? {
        if let Some(existing_session) = Session::load_for_email(&form.email, &conn)? {
            Err(GreaseError::AlreadyLoggedIn(existing_session.key))
        } else {
            Ok(json!({
                "token": Session::generate(&form.email, &conn)?
            }))
        }
    } else {
        Err(GreaseError::BadRequest(
            "login info was incorrect".to_owned(),
        ))
    }
}

/// Log out of the API.
///
/// ## Required Permissions:
///
/// The user must be logged in.
pub fn logout(user: User) -> GreaseResult<Value> {
    Session::delete(&user.member.member.email, &user.conn).map(|_| basic_success())
}

pub fn forgot_password(email: String) -> GreaseResult<Value> {
    let conn = connect_to_db()?;

    Session::generate_for_forgotten_password(email, &conn).map(|_| basic_success())
}

pub fn reset_password(token: Option<String>, reset_form: PasswordReset) -> GreaseResult<Value> {
    let conn = connect_to_db()?;
    let token = token.ok_or(GreaseError::BadRequest(
        "You must provide a reset token to reset your password.".to_owned(),
    ))?;

    Session::reset_password(token, reset_form, &conn).map(|_| basic_success())
}

/// Get a single member.
///
/// ## Path Parameters:
///   * email: string (*required*) - The email of the member
///
/// ## Query Parameters:
///   * grades: boolean (*optional*) - Whether to include the member's grades.
///   * details: boolean (*optional*) - Whether to include extra details.
///
/// ## Required Permissions:
///
/// The user must be logged in. If they are retrieving data about another member,
/// they will need to be able to "view-users" generally. If `details=true` is passed,
/// they will need to be able to "view-user-private-details" generally.
///
/// ## Return Format:
///
/// If `details = true`, then the format from
/// [to_json_full](crate::db::models::Member::to_json_full())
/// is used to return info on all semesters the member was active. Otherwise,
/// if `grades = true`, then the format from
/// [to_json_with_grades](crate::db::models::Member::to_json_with_grades())
/// is used. Otherwise, the simple format from
/// [to_json](crate::db::models::member::MemberForSemester::to_json())
/// is used.
pub fn get_member(
    email: String,
    grades: Option<bool>,
    details: Option<bool>,
    user: User,
) -> GreaseResult<Value> {
    let grades = grades.unwrap_or(false);
    let details = details.unwrap_or(false);

    if &email != &user.member.member.email {
        check_for_permission!(user => "view-users");
        if details || grades {
            check_for_permission!(user => "view-user-private-details");
        }
    }

    if details {
        let member = Member::load(&email, &user.conn)?;
        member.to_json_full_for_all_semesters(&user.conn)
    } else {
        let current_semester = Semester::load_current(&user.conn)?;
        let member = MemberForSemester::load(&email, &current_semester.name, &user.conn)?;

        if grades {
            member.to_json_with_grades(&current_semester, &user.conn)
        } else {
            Ok(member.to_json())
        }
    }
}

/// Get the member associated with the current user.
///
/// ## Return Format:
///
/// If the current user is logged in, then the
/// [to_json_with_grades](crate::db::models::Member::to_json_with_grades())
/// format of the current member is returned. Otherwise,
/// null is returned.
pub fn get_current_user(user: GreaseResult<User>) -> GreaseResult<Value> {
    match user {
        Ok(user) => user.member.to_json_full(&user.conn),
        Err(GreaseError::Unauthorized) => Ok(json!(null)),
        Err(other) => Err(other),
    }
}

/// Get all members.
///
/// ## Query Parameters:
///   * grades: boolean (*optional*) - Whether to include uniform and attendance.
///   * include: string (*optional*) - Which members to include. Expects a comma-delimited
///       list of types from the allowed values of "class", "club", and "inactive".
///       If `include` isn't provided, defaults to returning only all currently active members.
///
/// ## Return Format:
///
/// If `grades = true`, then the format from
/// [to_json_with_grades](crate::db::models::event::EventWithGig::to_json_with_grades())
/// will be returned. Otherwise, the format from
/// [to_json](crate::db::models::event::EventWithGig::to_json())
/// will be returned.
pub fn get_members(
    grades: Option<bool>,
    include: Option<String>,
    user: User,
) -> GreaseResult<Value> {
    let current_semester = Semester::load_current(&user.conn)?;
    let (include_class, include_club, include_inactive) = if let Some(include) = include {
        let mut included = include.split(",").collect::<HashSet<&str>>();
        included.remove("");
        let include_class = included.remove("class");
        let include_club = included.remove("club");
        let include_inactive = included.remove("inactive");
        if included.len() > 0 {
            return Err(GreaseError::BadRequest(
                "For the include param, only 'class', 'club', and 'inactive' are allowed."
                    .to_owned(),
            ));
        }

        (include_class, include_club, include_inactive)
    } else {
        (true, true, false)
    };

    MemberForSemester::load_all(&current_semester.name, &user.conn).and_then(|members| {
        members
            .into_iter()
            .filter_map(|member| {
                if let Some(ref active_semester) = member.active_semester {
                    if !(include_class && &active_semester.enrollment == &Enrollment::Class)
                        && !(include_club && &active_semester.enrollment == &Enrollment::Club)
                    {
                        return None;
                    }
                } else if !include_inactive {
                    return None;
                }

                let json_val = if grades.unwrap_or(false) {
                    member.to_json_full(&user.conn)
                } else {
                    Ok(member.to_json())
                };
                Some(json_val)
            })
            .collect::<GreaseResult<Vec<_>>>()
            .map(|members| json!(members))
    })
}

/// Register a new member.
///
/// ## Input Format:
///
/// Expects a [NewMember](crate::db::models::NewMember).
pub fn new_member(new_member: NewMember) -> GreaseResult<Value> {
    let conn = connect_to_db()?;

    Member::create(new_member, &conn).map(|_| basic_success())
}

/// Confirms that an inactive member will be active for the current semester.
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Input Format:
///
/// Expects a [RegisterForSemesterForm](crate::db::models::RegisterForSemesterForm).
pub fn confirm_for_semester(form: RegisterForSemesterForm, user: User) -> GreaseResult<Value> {
    Member::register_for_semester(user.member.member.email, form, &user.conn)
        .map(|_| basic_success())
}

/// Mark a member as no longer active for a given semester.
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-user" generally.
///
/// ## Path Parameters:
///   * member: string (*required*) - The email of the member
///   * semester: string (*required*) - The name of the semester
pub fn mark_member_inactive_for_semester(
    member: String,
    semester: String,
    user: User,
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-user");
    Member::mark_inactive_for_semester(&member, &semester, &user.conn).map(|_| basic_success())
}

/// Update a member's activity for a semester.
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-user" generally.
///
/// ## Path Parameters:
///   * member: string (*required*) - The email of the member
///   * semester: string (*required*) - The name of the semester
///
/// ## Input Format:
///
/// Expects an [ActiveSemesterUpdate](crate::db::models::ActiveSemesterUpdate).
pub fn update_member_semester(
    member: String,
    semester: String,
    update: ActiveSemesterUpdate,
    user: User,
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-user");
    ActiveSemester::update(&member, &semester, update, &user.conn).map(|_| basic_success())
}

/// Update a member's account from their profile.
///
/// ## Required Permissions:
///
/// The user must be logged in.
///
/// ## Input Format:
///
/// Expects a [NewMember](crate::db::models::NewMember).
pub fn update_member_profile(update: NewMember, user: User) -> GreaseResult<Value> {
    Member::update(&user.member.member.email, true, update, &user.conn).map(|_| basic_success())
}

/// Update a member's account as an officer.
///
/// ## Path Parameters:
///   * member: string (*required*) - The email of the member
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "edit-user" generally.
///
/// ## Input Format:
///
/// Expects a [NewMember](crate::db::models::NewMember).
pub fn update_member_as_officer(
    member: String,
    update: NewMember,
    user: User,
) -> GreaseResult<Value> {
    check_for_permission!(user => "edit-user");
    Member::update(&member, false, update, &user.conn).map(|_| basic_success())
}

/// Log in as another member.
///
/// ## Path Parameters:
///   * member: string (*required*) - The email of the member
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "switch-user" generally.
///
/// ## Return Format:
///
/// ```json
/// {
///     "token": string
/// }
/// ```
///
/// Returns an object with a newly generated API token for login as that member.
pub fn login_as_member(member: String, user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "switch-user");
    if member == user.member.member.email {
        return Err(GreaseError::BadRequest(
            "Cannot re-login as self.".to_owned(),
        ));
    }

    let key = match Session::load_for_email(&member, &user.conn)? {
        Some(session) => session.key,
        None => Session::generate(&member, &user.conn)?,
    };

    Ok(json!({ "token": key }))
}

/// Delete a member from the site permanently.
///
/// WARNING! This is a permanent action, and cannot be undone. Make sure that
/// you know what you are doing. You must pass `confirm=true` to actually delete
/// a member.
///
/// ## Path Parameters:
///   * id: integer (*required*) - The email of the member
///
/// ## Query Parameters:
///   * confirm: boolean (*optional*) - Confirm the deletion
///
/// ## Required Permissions:
///
/// The user must be logged in and be able to "delete-user" generally.
pub fn delete_member(member: String, confirm: Option<bool>, user: User) -> GreaseResult<Value> {
    check_for_permission!(user => "delete-user");
    if confirm.unwrap_or(false) {
        Member::delete(&member, &user.conn).map(|_| basic_success())
    } else {
        Err(GreaseError::BadRequest(
            "You must pass 'confirm=true' to actually delete a member.".to_owned(),
        ))
    }
}
