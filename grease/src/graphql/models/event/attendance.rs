require "graphql"

require "../../db"

module Models
  @[GraphQL::Object]
  class Attendance
    include GraphQL::ObjectType

    class_getter table_name = "attendance"

    DB.mapping({
      member:        String,
      event:         Int32,
      should_attend: {type: Bool, default: true},
      did_attend:    {type: Bool, default: false},
      confirmed:     {type: Bool, default: false},
      minutes_late:  {type: Int32, default: 0},
    })

    def self.for_member_at_event(email, event_id)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE member = ? && event = ?", email, event_id, as: Attendance
    end

    def self.for_member_at_event!(email, event_id)
      for_member_at_event(email, event_id) || raise "No attendance for member #{email} at event with id #{event_id}"
    end

    def self.for_event(event_id)
      CONN.query_all "SELECT * FROM #{@@table_name} WHERE event = ?", event_id, as: Attendance
    end

    def self.create_for_new_member(email)
      events = Event.for_semester Semester.current.name

      events.each do |event|
        should_attend = (event.call_time < Time.local) ? false : event.default_attend
        CONN.exec "INSERT IGNORE INTO #{@@table_name} (event, should_attend, member) VALUES (?, ?, ?)",
          event.id, should_attend, email
      end
    end

    def self.create_for_new_event(event_id)
      event = Event.with_id! event_id
      active_members = Member.active_during event.semester

      active_members.each do |member|
        CONN.exec "INSERT INTO #{@@table_name} (event, should_attend, member) VALUES (?, ?, ?)",
          event_id, event.default_attend, member.email
      end
    end

    def self.excuse_unconfirmed(event_id)
      Event.with_id! event_id

      CONN.exec "UPDATE #{@@table_name} SET should_attend = false \
        WHERE event = ? AND confirmed = false", event_id
    end

    def self.update(event_id, email, form)
      for_member_at_event! email, event_id

      CONN.exec "UPDATE #{@@table_name} SET \
        should_attend = ?, did_attend = ?, confirmed = ?, minutes_late = ? \
        WHERE member = ? AND event = ?",
        form.should_attend, form.did_attend, form.confirmed, form.minutes_late,
        email, event_id
    end

    def self.rsvp_for_event(event_id, member, attending)
      event = Event.with_id! event_id
      attendance = for_member_at_event! member.email, event_id
      event.ensure_no_rsvp_issue! member, attendance

      CONN.exec "UPDATE #{@@table_name} SET should_attend = ?, confirmed = true \
        WHERE event = ? AND member = ?", attending, event_id, member.email
    end

    def self.confirm_for_event(event_id, member)
      Event.with_id! event_id
      for_member_at_event! member.email, event_id

      CONN.exec "UPDATE #{@@table_name} SET should_attend = true, confirmed = true \
        WHERE event = ? AND member = ?", event_id, member.email
    end

    @[GraphQL::Field(description: "The email of the member this attendance belongs to")]
    def member : Models::Member
      Member.with_email! @member
    end

    @[GraphQL::Field(description: "Whether the member is expected to attend the event")]
    def should_attend : Bool
      @should_attend
    end

    @[GraphQL::Field(description: "Whether the member did attend the event")]
    def did_attend : Bool
      @did_attend
    end

    @[GraphQL::Field(description: "Whether the member confirmed that they would attend")]
    def confirmed : Bool
      @confirmed
    end

    @[GraphQL::Field(description: "How late the member was if they attended")]
    def minutes_late : Int32
      @minutes_late
    end

    @[GraphQL::Field]
    def absence_request : Models::AbsenceRequest?
      AbsenceRequest.for_member_at_event @member, @event
    end

    @[GraphQL::Field]
    def rsvp_issue : String?
      event = Event.with_id! @event
      event.rsvp_issue_for member, self
    end

    @[GraphQL::Field]
    def approved_absence : Bool
      absence_request.try &.state == Models::AbsenceRequest::State::APPROVED
    end

    @[GraphQL::Field(name: "denyCredit")]
    def deny_credit? : Bool
      @should_attend && !@did_attend && !approved_absence
    end
  end

  @[GraphQL::Object]
  class AbsenceRequest
    include GraphQL::ObjectType

    class_getter table_name = "absence_request"

    @[GraphQL::Enum(name: "AbsenceRequestState")]
    enum State
      PENDING
      APPROVED
      DENIED

      def self.mapping
        {
          "PENDING"  => PENDING,
          "APPROVED" => APPROVED,
          "DENIED"   => DENIED,
        }
      end

      def to_rs
        State.mapping.invert[self].downcase
      end

      def self.from_rs(rs)
        val = rs.read
        state = val.as?(String).try { |v| State.mapping[v.upcase]? }
        state || raise "Invalid absence request state returned from database: #{val}"
      end

      def self.parse(val)
        State.mapping[val]? || raise "Invalid absence request state variant provided: #{val}"
      end
    end

    DB.mapping({
      member: String,
      event:  Int32,
      time:   {type: Time, default: Time.local},
      reason: String,
      state:  {type: State, converter: State},
    })

    def self.for_member_at_event(email, event_id)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE member = ? AND event = ?", email, event_id, as: AbsenceRequest
    end

    def self.for_member_at_event!(email, event_id)
      (for_member_at_event email, event_id) || raise "No absence request for member #{email} at event with id #{event_id}"
    end

    def self.for_semester(semester_name)
      CONN.query_all "SELECT * FROM #{@@table_name} WHERE semester = ? ORDER BY time", semester_name, as: AbsenceRequest
    end

    def self.submit(event_id, email, reason)
      CONN.exec "INSERT INTO #{@@table_name} (member, event, reason) \
        VALUES (?, ?, ?)", email, event_id, reason
    end

    def self.set_state(event_id, email, state)
      for_member_at_event! email, event_id

      CONN.exec "UPDATE #{@@table_name} SET state = ? WHERE event = ? AND member = ?",
        state.to_rs, event_id, email
    end

    @[GraphQL::Field(description: "The member that requested an absence")]
    def member : Models::Member
      Member.with_email! @member
    end

    @[GraphQL::Field(description: "The event they requested absence from")]
    def event : Models::Event
      Event.with_id! @event
    end

    @[GraphQL::Field(name: "time", description: "The time this request was placed")]
    def gql_time : String
      @time.to_s
    end

    @[GraphQL::Field(description: "The reason the member petitioned for absence with")]
    def reason : String
      @reason
    end

    @[GraphQL::Field(description: "The current state of the request (See AbsenceRequestState)")]
    def state : Models::AbsenceRequest::State
      @state
    end
  end
end
