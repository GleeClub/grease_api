//! # Grease API
//!
//! The backend for the Georgia Tech Glee Club's website.
//!
//! The backend is a JSON API, agnostic of the platform of interaction. This
//! leaves the site and its data to be available on other platforms in the future,
//! including potentially a phone app or a CLI.
//!
//! ## Basic JSON information:
//!
//! The types of data returned as fields are notated as such:
//!   * **string**: A string of characters
//!   * **integer**: An integer number
//!   * **float**: A floating-point number
//!   * **boolean**: `true` or `false`
//!   * **datetime**: An RFC 3339 datetime, formatted as a string
//!   * **date**: An RFC 3339 date, formatted as a string
//!   * **?**: Any field followed by a question mark can be null, otherwise it is never null
//!
//! ## All Routes:
//!
//! Below is the layout of the API. Click on the handler of any endpoint to find
//! out more about its usage.
//!
//! All routes have the following constraints:
//!   * Path Parameters: A list of parameters, their types, whether they are required, and
//!       a short description. If this section is missing, none are expected.
//!   * Query Parameters: The same as path paramters. Assume none are accepted if
//!       this section is missing.
//!   * Required Permissions: Describes what permissions are required to use the endpoint.
//!       If this section is missiong, anyone can use the endpoint.
//!   * Input Format: The expected format of all input data for POST requests. If this
//!       section is missing, POST data will be ignored.
//!   * Return Format: The format of data to be returned. If this section is missing,
//!       a [basic success](crate::routes::basic_success) is always returned by default
//!       on success.
//!
//! ### Authorization:
//!
//!   Method   | Route   | Handler
//! -----------|---------|------------------------------------------------
//! **POST**   | /login  | [login](crate::routes::member_routes::login)
//! **GET**    | /logout | [logout](crate::routes::member_routes::logout)
//!
//! ### Members:
//!
//!   Method   | Route                           | Handler
//! -----------|---------------------------------|-------------------------------------------------------------------------------------------------------
//! **GET**    | /user                           | [get_current_user](crate::routes::member_routes::get_current_user)
//! **GET**    | /members/{*email*}              | [get_member](crate::routes::member_routes::get_member)
//! **GET**    | /members/{*email*}/attendance   | [get_member_attendance_for_semester](crate::routes::event_routes::get_member_attendance_for_semester)
//! **GET**    | /members                        | [get_members](crate::routes::member_routes::get_members)
//! **POST**   | /members                        | [new_member](crate::routes::member_routes::new_member)
//! **POST**   | /members/confirm                | [confirm_for_semester](crate::routes::member_routes::confirm_for_semester)
//! **POST**   | /members/{*email*}/{*semester*} | [update_member_semester](crate::routes::member_routes::update_member_semester)
//! **DELETE** | /members/{*email*}/{*semester*} | [mark_member_inactive_for_semester](crate::routes::member_routes::mark_member_inactive_for_semester)
//! **POST**   | /members                        | [new_member](crate::routes::member_routes::new_member)
//! **POST**   | /members/profile                | [update_member_profile](crate::routes::member_routes::update_member_profile)
//! **POST**   | /members/{*email*}              | [update_member_as_officer](crate::routes::member_routes::update_member_as_officer)
//! **POST**   | /members/{*email*}/login_as     | [login_as_member](crate::routes::member_routes::login_as_member)
//! **DELETE** | /members/{*email*}              | [delete_member](crate::routes::member_routes::delete_member)
//!
//! ### Events:
//!
//!   Method   | Route          | Handler
//! -----------|----------------|----------------------------------------------------------
//! **GET**    | /events/{*id*} | [get_event](crate::routes::event_routes::get_event)
//! **GET**    | /events        | [get_events](crate::routes::event_routes::get_events)
//! **POST**   | /events        | [new_event](crate::routes::event_routes::new_event)
//! **POST**   | /events/{*id*} | [update_event](crate::routes::event_routes::update_event)
//! **DELETE** | /events/{*id*} | [delete_event](crate::routes::event_routes::delete_event)
//!
//! ### Event Details:
//!
//!   Method   | Route                                        | Handler
//! -----------|----------------------------------------------|------------------------------------------------------------------------------------------
//! **GET**    | /events/{*id*}/attendance                    | [get_attendance](crate::routes::event_routes::get_attendance)
//! **GET**    | /events/{*id*}/see_whos_attending            | [see_whos_attending](crate::routes::event_routes::see_whos_attending)
//! **GET**    | /events/{*id*}/attendance/{*member*}         | [get_member_attendance](crate::routes::event_routes::get_member_attendance)
//! **POST**   | /events/{*id*}/attendance/{*member*}         | [update_attendance](crate::routes::event_routes::update_attendance)
//! **POST**   | /events/{*id*}/rsvp/{*attending*}            | [rsvp_for_event](crate::routes::event_routes::rsvp_for_event)
//! **POST**   | /events/{*id*}/attendance/excuse_unconfirmed | [excuse_unconfirmed_for_event](crate::routes::event_routes::excuse_unconfirmed_for_event)
//! **GET**    | /events/{*id*}/carpools                      | [get_carpools](crate::routes::event_routes::get_carpools)
//! **POST**   | /events/{*id*}/carpools                      | [update_carpools](crate::routes::event_routes::update_carpools)
//! **GET**    | /events/{*id*}/setlist                       | [get_setlist](crate::routes::event_routes::get_setlist)
//! **POST**   | /events/{*id*}/setlist                       | [edit_setlist](crate::routes::event_routes::edit_setlist)
//!
//! ### Absence Requests:
//!
//!   Method   | Route                                            | Handler
//! -----------|--------------------------------------------------|-------------------------------------------------------------------------------
//! **GET**    | /absence_requests                                | [get_absence_requests](crate::routes::event_routes::get_absence_requests)-
//! **GET**    | /absence_requests/{*eventId*}                    | [get_absence_request](crate::routes::event_routes::get_absence_request)
//! **GET**    | /absence_requests/{*eventId*}/is_excused         | [member_is_excused](crate::routes::event_routes::member_is_excused)
//! **POST**   | /absence_requests/{*eventId*}/{*member*}/approve | [approve_absence_request](crate::routes::event_routes::approve_absence_request)
//! **POST**   | /absence_requests/{*eventId*}/{*member*}/deny    | [deny_absence_request](crate::routes::event_routes::deny_absence_request)
//! **POST**   | /absence_requests/{*eventId*}                    | [submit_absence_request](crate::routes::event_routes::submit_absence_request)
//!
//! ### Gig Requests:
//!
//!   Method   | Route                             | Handler
//! -----------|-----------------------------------|--------------------------------------------------------------------------------------------
//! **GET**    | /gig_requests/{*id*}              | [get_gig_request](crate::routes::event_routes::get_gig_request)
//! **GET**    | /gig_requests                     | [get_gig_requests](crate::routes::event_routes::get_gig_requests)
//! **POST**   | /gig_requests                     | [new_gig_request](crate::routes::event_routes::new_gig_request)
//! **POST**   | /gig_requests/{*id*}/dismiss      | [dismiss_gig_request](crate::routes::event_routes::dismiss_gig_request)
//! **POST**   | /gig_requests/{*id*}/reopen       | [reopen_gig_request](crate::routes::event_routes::reopen_gig_request)
//! **POST**   | /gig_requests/{*id*}/create_event | [create_event_from_gig_request](crate::routes::event_routes::create_event_from_gig_request)
//!
//! ### Variables:
//!
//!   Method   | Route              | Handler
//! -----------|--------------------|-------------------------------------------------------------
//! **GET**    | /variables/{*key*} | [get_variable](crate::routes::misc_routes::get_variable)
//! **POST**   | /variables/{*key*} | [set_variable](crate::routes::misc_routes::set_variable)
//! **DELETE** | /variables/{*key*} | [unset_variable](crate::routes::misc_routes::unset_variable)
//!
//! ### Announcements:
//!
//!   Method   | Route                 | Handler
//! -----------|-----------------------|------------------------------------------------------------------------------
//! **GET**    | /announcements/{*id*} | [get_announcement](crate::routes::officer_routes::get_announcement)
//! **GET**    | /announcements        | [get_announcements](crate::routes::officer_routes::get_announcements)
//! **POST**   | /announcements        | [make_new_announcement](crate::routes::officer_routes::make_new_announcement)
//! **POST**   | /announcements/{*id*} | [archive_announcement](crate::routes::officer_routes::archive_announcement)
//!
//! ### Google Docs:
//!
//!   Method   | Route                 | Handler
//! -----------|-----------------------|----------------------------------------------------------------------
//! **GET**    | /google_docs/{*name*} | [get_google_doc](crate::routes::officer_routes::get_google_doc)
//! **GET**    | /google_docs]         | [get_google_docs](crate::routes::officer_routes::get_google_docs)
//! **POST**   | /google_docs]         | [new_google_doc](crate::routes::officer_routes::new_google_doc)
//! **POST**   | /google_docs/{*name*} | [modify_google_doc](crate::routes::officer_routes::modify_google_doc)
//! **DELETE** | /google_docs/{*name*} | [delete_google_doc](crate::routes::officer_routes::delete_google_doc)
//!
//! ### Meeting Minutes:
//!
//!   Method   | Route                         | Handler
//! -----------|-------------------------------|----------------------------------------------------------------------------------
//! **GET**    | /meeting_minutes/{*id*}       | [get_meeting_minutes](crate::routes::officer_routes::get_meeting_minutes)
//! **GET**    | /meeting_minutes              | [get_all_meeting_minutes](crate::routes::officer_routes::get_all_meeting_minutes)
//! **POST**   | /meeting_minutes              | [new_meeting_minutes](crate::routes::officer_routes::new_meeting_minutes)
//! **POST**   | /meeting_minutes/{*id*}       | [modify_meeting_minutes](crate::routes::officer_routes::modify_meeting_minutes)
//! **GET**    | /meeting_minutes/{*id*}/email | [send_minutes_as_email](crate::routes::officer_routes::send_minutes_as_email)
//! **DELETE** | /meeting_minutes/{*id*}       | [delete_meeting_minutes](crate::routes::officer_routes::delete_meeting_minutes)
//!
//! ### Uniforms:
//!
//!   Method   | Route            | Handler
//! -----------|------------------|----------------------------------------------------------------
//! **GET**    | /uniforms/{*id*} | [get_uniform](crate::routes::officer_routes::get_uniform)
//! **GET**    | /uniforms]       | [get_uniforms](crate::routes::officer_routes::get_uniforms)
//! **POST**   | /uniforms]       | [new_uniform](crate::routes::officer_routes::new_uniform)
//! **POST**   | /uniforms/{*id*} | [modify_uniform](crate::routes::officer_routes::modify_uniform)
//! **DELETE** | /uniforms/{*id*} | [delete_uniform](crate::routes::officer_routes::delete_uniform)
//!
//! ### Todos:
//!
//!   Method   | Route         | Handler
//! -----------|---------------|------------------------------------------------------------------------------
//! **GET**    | /todos        | [get_todos](crate::routes::officer_routes::get_todos)
//! **POST**   | /todos        | [add_todo_for_members](crate::routes::officer_routes::add_todo_for_members)
//! **POST**   | /todos/{*id*} | [mark_todo_as_complete](crate::routes::officer_routes::mark_todo_as_complete)
//!
//! ### Songs:
//!
//!   Method   | Route                          | Handler
//! -----------|--------------------------------|-------------------------------------------------------------------------------------
//! **GET**    | /repertoire/{*id*}             | [get_song](crate::routes::repertoire_routes::get_song)
//! **GET**    | /repertoire                    | [get_songs](crate::routes::repertoire_routes::get_songs)
//! **POST**   | /repertoire                    | [new_song](crate::routes::repertoire_routes::new_song)
//! **POST**   | /repertoire/{*id*}             | [update_song](crate::routes::repertoire_routes::update_song)
//! **POST**   | /repertoire/{*id*}/current     | [set_song_as_current](crate::routes::repertoire_routes::set_song_as_current)
//! **POST**   | /repertoire/{*id*}/not_current | [set_song_as_not_current](crate::routes::repertoire_routes::set_song_as_not_current)
//! **DELETE** | /repertoire/{*id*}             | [delete_song](crate::routes::repertoire_routes::delete_song)
//!
//! ### Song Links:
//!
//!   Method   | Route                     | Handler
//! -----------|---------------------------|---------------------------------------------------------------------------
//! **POST**   | /repertoire/{*id*}/links  | [new_song_link](crate::routes::repertoire_routes::new_song_link)
//! **GET**    | /repertoire/links/{*id*}  | [get_song_link](crate::routes::repertoire_routes::get_song_link)
//! **DELETE** | /repertoire/links/{*id*}  | [remove_song_link](crate::routes::repertoire_routes::remove_song_link)
//! **POST**   | /repertoire/links/{*id*}  | [update_song_link](crate::routes::repertoire_routes::update_song_link)
//! **POST**   | /repertoire/upload        | [upload_file](crate::routes::repertoire_routes::upload_file)
//! **GET**    | /repertoire/cleanup_files | [cleanup_song_files](crate::routes::repertoire_routes::cleanup_song_files)
//!
//! ### Semesters:
//!
//!   Method   | Route                           | Handler
//! -----------|---------------------------------|----------------------------------------------------------------------------
//! **GET**    | /semesters                      | [get_semesters](crate::routes::officer_routes::get_semesters)
//! **GET**    | /semester/current               | [get_current_semester](crate::routes::officer_routes::get_current_semester)
//! **GET**    | /semesters/{*name*}             | [get_semester](crate::routes::officer_routes::get_semester)
//! **POST**   | /semesters                      | [new_semester](crate::routes::officer_routes::new_semester)
//! **POST**   | /semesters/{*name*}             | [edit_semester](crate::routes::officer_routes::edit_semester)
//! **POST**   | /semesters/{*name*}/set_current | [set_current_semester](crate::routes::officer_routes::set_current_semester)
//! **DELETE** | /semesters/{*name*}             | [delete_semester](crate::routes::officer_routes::delete_semester)
//!
//! ### Permissions and Roles:
//!
//!   Method   | Route                             | Handler
//! -----------|-----------------------------------|--------------------------------------------------------------------------------------------
//! **GET**    | /role_permissions                 | [get_current_role_permissions](crate::routes::officer_routes::get_current_role_permissions)
//! **GET**    | /member_roles                     | [get_current_officers](crate::routes::officer_routes::get_current_officers)
//! **GET**    | /permissions/{*member*}           | [member_permissions](crate::routes::officer_routes::member_permissions)
//! **POST**   | /permissions/{*position*}/enable  | [add_permission_for_role](crate::routes::officer_routes::add_permission_for_role)
//! **POST**   | /permissions/{*position*}/disable | [remove_permission_for_role](crate::routes::officer_routes::remove_permission_for_role)
//! **POST**   | /roles/add                        | [add_officership](crate::routes::officer_routes::add_officership)
//! **POST**   | /roles/remove                     | [remove_officership](crate::routes::officer_routes::remove_officership)
//!
//! ### Fees and Transactions:
//!
//!   Method   | Route                         | Handler
//! -----------|-------------------------------|----------------------------------------------------------------------------------------------------
//! **GET**    | /fees                         | [get_fees](crate::routes::officer_routes::get_fees)
//! **POST**   | /fees/{*name*}/{*new_amount*} | [update_fee_amount](crate::routes::officer_routes::update_fee_amount)
//! **POST**   | /fees/{*name*}/apply          | [apply_fee_for_all_active_members](crate::routes::officer_routes::apply_fee_for_all_active_members)
//! **GET**    | /transactions/{*member*}      | [get_member_transactions](crate::routes::officer_routes::get_member_transactions)
//! **POST**   | /transactions                 | [add_transactions](crate::routes::officer_routes::add_transactions)
//!
//! ### Static Data:
//!
//!   Method   | Route              | Handler
//! -----------|--------------------|------------------------------------------------------------------------------
//! **GET**    | /media_types       | [get_media_types](crate::routes::repertoire_routes::get_media_types)
//! **GET**    | /permissions       | [get_permissions](crate::routes::officer_routes::get_permissions)
//! **GET**    | /roles             | [get_roles](crate::routes::officer_routes::get_roles)
//! **GET**    | /event_types       | [get_event_types](crate::routes::event_routes::get_event_types)
//! **GET**    | /section_types     | [get_section_types](crate::routes::event_routes::get_section_types)
//! **GET**    | /transaction_types | [get_transaction_types](crate::routes::officer_routes::get_transaction_types)
//! **GET**    | /static_data       | [static_data](crate::routes::misc_routes::static_data)

#![recursion_limit = "512"]
#![feature(drain_filter, backtrace)]

mod auth;
mod cron;
mod db;
mod error;
pub mod routes;
mod util;
mod graphql;

fn main() {
    dotenv::dotenv().ok();

    if std::env::var("REQUEST_METHOD").is_ok() {
        cgi::handle(routes::handle_request);
    } else {
        cron::send_event_emails(None).ok();
    }
}
