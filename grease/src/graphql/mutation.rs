@[GraphQL::Object]
class Mutation
  include GraphQL::ObjectType
  include GraphQL::MutationType

  @[GraphQL::Field(description: "Gets a login token on successful login")]
  def login(email : String, pass_hash : String) : String
    raise "The given email or password is invalid" unless Models::Member.valid_login? email, pass_hash

    Models::Session.get_or_generate_token email
  end

  @[GraphQL::Field(description: "Logs the member out")]
  def logout(context : UserContext) : Bool
    Models::Session.remove_for context.user!.email
    true
  end

  @[GraphQL::Field]
  def forgot_password(email : String) : Bool
    Models::Session.generate_for_forgotten_password email
    true
  end

  @[GraphQL::Field]
  def reset_password(token : String, pass_hash : String) : Bool
    Models::Session.reset_password token, pass_hash
    true
  end

  @[GraphQL::Field]
  def register_member(form : Input::NewMember) : Models::Member
    Models::Member.register form
    Models::Member.with_email! form.email
  end

  @[GraphQL::Field]
  def register_for_semester(form : Input::RegisterForSemesterForm, context : UserContext) : Models::Member
    context.user!.register_for_current_semester form
    context.user!
  end

  @[GraphQL::Field]
  def update_profile(form : Input::NewMember, context : UserContext) : Models::Member
    context.user!.update form, as_self: true
    context.user!
  end

  @[GraphQL::Field]
  def update_member(email : String, form : Input::NewMember, context : UserContext) : Models::Member
    context.able_to! Permissions::EDIT_USER

    member = Models::Member.with_email! email
    member.update form, as_self: false
    member
  end

  @[GraphQL::Field(description: "Gets a login token for the given member")]
  def login_as(member : String, context : UserContext) : String
    context.able_to! Permissions::SWITCH_USER

    Models::Session.get_or_generate_token member
  end

  @[GraphQL::Field(description: "Deletes a member and returns their email")]
  def delete_member(email : String, context : UserContext) : String
    context.able_to! Permissions::DELETE_USER

    member = Models::Member.with_email! email
    member.delete
    email
  end

  @[GraphQL::Field]
  def create_event(form : Input::NewEvent, context : UserContext) : Models::Event
    context.able_to! Permissions::CREATE_EVENT, form.event.type

    new_id = Models::Event.create form
    Models::Event.with_id! new_id
  end

  @[GraphQL::Field]
  def update_event(id : Int32, form : Input::NewEvent, context : UserContext) : Models::Event
    context.able_to! Permissions::MODIFY_EVENT, form.event.type

    Models::Event.update id, form
    Models::Event.with_id! id
  end

  @[GraphQL::Field(description: "Deletes an event and returns its id")]
  def delete_event(id : Int32, context : UserContext) : Int32
    context.able_to! Permissions::DELETE_EVENT

    Models::Event.delete id
    id
  end

  @[GraphQL::Field]
  def update_attendance(event_id : Int32, member : String, form : Input::AttendanceForm, context : UserContext) : Models::Attendance
    event = Models::Event.with_id! event_id

    unless context.able_to? Permissions::EDIT_ATTENDANCE, event.type
      user_section = context.user!.get_semester(event.semester).try &.section
      member_section = (Models::ActiveSemester.for_semester member, event.semester).try &.section

      unless user_section == member_section && context.able_to? Permissions::EDIT_ATTENDANCE_OWN_SECTION, event.type
        raise "Permission #{Permissions::EDIT_ATTENDANCE} required"
      end
    end

    Models::Attendance.update event_id, member, form
    Models::Attendance.for_member_at_event! member, event_id
  end

  @[GraphQL::Field]
  def rsvp_for_event(id : Int32, attending : Bool, context : UserContext) : Models::Attendance
    Models::Attendance.rsvp_for_event id, context.user!, attending
    Models::Attendance.for_member_at_event! context.user!.email, id
  end

  @[GraphQL::Field]
  def confirm_for_event(id : Int32, context : UserContext) : Models::Attendance
    Models::Attendance.confirm_for_event id, context.user!
    Models::Attendance.for_member_at_event! context.user!.email, id
  end

  @[GraphQL::Field]
  def update_carpools(event_id : Int32, carpools : Array(Input::UpdatedCarpool), context : UserContext) : Array(Models::Carpool)
    context.able_to! Permissions::EDIT_CARPOOLS

    Models::Carpool.update event_id, carpools
    Models::Carpool.for_event event_id
  end

  @[GraphQL::Field]
  def respond_to_absence_request(event_id : Int32, member : String, approved : Bool, context : UserContext) : Models::AbsenceRequest
    context.able_to! Permissions::PROCESS_ABSENCE_REQUESTS

    state = approved ? Models::AbsenceRequest::State::APPROVED : Models::AbsenceRequest::State::DENIED
    Models::AbsenceRequest.set_state event_id, member, state
    Models::AbsenceRequest.for_member_at_event! member, event_id
  end

  @[GraphQL::Field]
  def submit_absence_request(event_id : Int32, reason : String, context : UserContext) : Models::AbsenceRequest
    Models::AbsenceRequest.submit event_id, context.user!.email, reason
    Models::AbsenceRequest.for_member_at_event! event_id, context.user!.email
  end

  @[GraphQL::Field]
  def submit_gig_request(form : Input::NewGigRequest) : Models::GigRequest
    new_id = Models::GigRequest.submit form
    Models::GigRequest.with_id! new_id
  end

  @[GraphQL::Field]
  def dismiss_gig_request(id : Int32, context : UserContext) : Models::GigRequest
    context.able_to! Permissions::PROCESS_GIG_REQUESTS

    request = Models::GigRequest.with_id! id
    request.set_status Models::GigRequest::Status::DISMISSED
    request
  end

  @[GraphQL::Field]
  def reopen_gig_request(id : Int32, context : UserContext) : Models::GigRequest
    context.able_to! Permissions::PROCESS_GIG_REQUESTS

    request = Models::GigRequest.with_id! id
    request.set_status Models::GigRequest::Status::PENDING
    request
  end

  @[GraphQL::Field]
  def create_event_from_gig_request(request_id : Int32, form : Input::NewEvent, context : UserContext) : Models::Event
    context.able_to! Permissions::CREATE_EVENT, form.event.type

    request = Models::GigRequest.with_id request_id
    new_id = Models::Event.create form, request
    Models::Event.with_id! new_id
  end

  @[GraphQL::Field]
  def set_variable(key : String, value : String, context : UserContext) : Models::Variable
    context.able_to! Permissions::EDIT_OFFICERS

    Models::Variable.set key, value
    Models::Variable.with_key! key
  end

  @[GraphQL::Field]
  def unset_variable(key : String, context : UserContext) : Models::Variable
    context.able_to! Permissions::EDIT_OFFICERS

    var = Models::Variable.with_key! key
    var.unset
    var
  end

  @[GraphQL::Field]
  def create_document(name : String, url : String, context : UserContext) : Models::Document
    context.able_to! Permissions::EDIT_LINKS

    Models::Document.create name, url
    Models::Document.with_name! name
  end

  @[GraphQL::Field]
  def update_document(name : String, url : String, context : UserContext) : Models::Document
    context.able_to! Permissions::EDIT_LINKS

    document = Models::Document.with_name! name
    document.set_url url
    document
  end

  @[GraphQL::Field]
  def delete_document(name : String, context : UserContext) : Models::Document
    context.able_to! Permissions::EDIT_LINKS

    document = Models::Document.with_name! name
    document.delete
    document
  end

  @[GraphQL::Field]
  def create_semester(form : Input::NewSemester, context : UserContext) : Models::Semester
    context.able_to! Permissions::EDIT_SEMESTER

    Models::Semester.create form
    Models::Semester.with_name! form.name
  end

  @[GraphQL::Field]
  def update_semester(name : String, form : Input::NewSemester, context : UserContext) : Models::Semester
    context.able_to! Permissions::EDIT_SEMESTER

    Models::Semester.update name, form
    Models::Semester.with_name! form.name
  end

  @[GraphQL::Field]
  def set_current_semester(name : String, context : UserContext) : Models::Semester
    context.able_to! Permissions::EDIT_SEMESTER

    Models::Semester.set_current name
    Models::Semester.with_name! name
  end

  @[GraphQL::Field]
  def create_meeting_minutes(name : String, context : UserContext) : Models::Minutes
    context.able_to! Permissions::EDIT_MINUTES

    new_id = Models::Minutes.create name
    Models::Minutes.with_id! new_id
  end

  @[GraphQL::Field]
  def update_meeting_minutes(id : Int32, form : Input::UpdatedMeetingMinutes, context : UserContext) : Models::Minutes
    context.able_to! Permissions::EDIT_MINUTES

    minutes = Models::Minutes.with_id! id
    minutes.update form
    minutes
  end

  @[GraphQL::Field]
  def email_meeting_minutes(id : Int32, context : UserContext) : Models::Minutes
    context.able_to! Permissions::EDIT_MINUTES

    minutes = Models::Minutes.with_id! id
    minutes.email
    minutes
  end

  @[GraphQL::Field]
  def delete_meeting_minutes(id : Int32, context : UserContext) : Models::Minutes
    context.able_to! Permissions::EDIT_MINUTES

    minutes = Models::Minutes.with_id! id
    minutes.delete
    minutes
  end

  @[GraphQL::Field]
  def create_uniform(form : Input::NewUniform, context : UserContext) : Models::Uniform
    context.able_to! Permissions::EDIT_UNIFORMS

    new_id = Models::Uniform.create form
    Models::Uniform.with_id! new_id
  end

  @[GraphQL::Field]
  def update_uniform(id : Int32, form : Input::NewUniform, context : UserContext) : Models::Uniform
    context.able_to! Permissions::EDIT_UNIFORMS

    uniform = Models::Uniform.with_id! id
    uniform.update form
    uniform
  end

  @[GraphQL::Field]
  def delete_uniform(id : Int32, context : UserContext) : Models::Uniform
    context.able_to! Permissions::EDIT_UNIFORMS

    uniform = Models::Uniform.with_id! id
    uniform.delete
    uniform
  end

  @[GraphQL::Field]
  def create_song(form : Input::NewSong, context : UserContext) : Models::Song
    context.able_to! Permissions::EDIT_REPERTOIRE

    new_id = Models::Song.create form
    Models::Song.with_id! new_id
  end

  @[GraphQL::Field]
  def update_song(id : Int32, form : Input::SongUpdate, context : UserContext) : Models::Song
    context.able_to! Permissions::EDIT_REPERTOIRE

    song = Models::Song.with_id! id
    song.update form
    song
  end

  @[GraphQL::Field(description: "Deletes a song and returns the id")]
  def delete_song(id : Int32, context : UserContext) : Int32
    context.able_to! Permissions::EDIT_REPERTOIRE

    song = Models::Song.with_id! id
    song.delete
    id
  end

  @[GraphQL::Field]
  def create_song_link(song_id : Int32, form : Input::NewSongLink, context : UserContext) : Models::SongLink
    context.able_to! Permissions::EDIT_REPERTOIRE

    new_id = Models::SongLink.create song_id, form
    Models::SongLink.with_id! new_id
  end

  @[GraphQL::Field]
  def update_song_link(id : Int32, form : Input::SongLinkUpdate, context : UserContext) : Models::SongLink
    context.able_to! Permissions::EDIT_REPERTOIRE

    link = Models::SongLink.with_id! id
    link.update form
    link
  end

  @[GraphQL::Field]
  def delete_song_link(id : Int32, context : UserContext) : Models::SongLink
    context.able_to! Permissions::EDIT_REPERTOIRE

    link = Models::SongLink.with_id! id
    link.delete
    link
  end

  @[GraphQL::Field]
  def add_permission_to_role(position : String, permission : String, event_type : String?, context : UserContext) : Bool
    context.able_to! Permissions::EDIT_PERMISSIONS

    Models::RolePermission.add position, permission, event_type
    true
  end

  @[GraphQL::Field]
  def remove_permission_from_role(position : String, permission : String, event_type : String?, context : UserContext) : Bool
    context.able_to! Permissions::EDIT_PERMISSIONS

    Models::RolePermission.remove position, permission, event_type
    true
  end

  @[GraphQL::Field]
  def add_officership(position : String, member : String, context : UserContext) : Models::MemberRole
    context.able_to! Permissions::EDIT_OFFICERS

    member_role = Models::MemberRole.new member, position
    member_role.add
    member_role
  end

  @[GraphQL::Field]
  def remove_officership(position : String, member : String, context : UserContext) : Models::MemberRole
    context.able_to! Permissions::EDIT_OFFICERS

    member_role = Models::MemberRole.new member, position
    member_role.remove
    member_role
  end

  @[GraphQL::Field]
  def update_fee_amount(name : String, amount : Int32, context : UserContext) : Models::Fee
    context.able_to! Permissions::EDIT_TRANSACTION

    fee = Models::Fee.with_name! name
    fee.set_amount amount
    fee
  end

  @[GraphQL::Field]
  def charge_dues(context : UserContext) : Array(Models::ClubTransaction)
    context.able_to! Permissions::EDIT_TRANSACTION

    Models::Fee.charge_dues_for_semester
    Models::ClubTransaction.for_semester Models::Semester.current.name
  end

  @[GraphQL::Field]
  def charge_late_dues(context : UserContext) : Array(Models::ClubTransaction)
    context.able_to! Permissions::EDIT_TRANSACTION

    Models::Fee.charge_late_dues_for_semester
    Models::ClubTransaction.for_semester Models::Semester.current.name
  end

  @[GraphQL::Field]
  def add_batch_of_transactions(batch : Input::TransactionBatch, context : UserContext) : Array(Models::ClubTransaction)
    context.able_to! Permissions::EDIT_TRANSACTION

    Models::ClubTransaction.add_batch batch
    Models::ClubTransaction.for_semester Models::Semester.current.name
  end

  @[GraphQL::Field]
  def resolve_transaction(id : Int32, resolved : Bool, context : UserContext) : Models::ClubTransaction
    context.able_to! Permissions::EDIT_TRANSACTION

    transaction = Models::ClubTransaction.with_id! id
    transaction.resolve resolved
    transaction
  end

  # TODO: sendEmail(since: NaiveDateTime!): Boolean!
end
