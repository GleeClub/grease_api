require "graphql"
require "../models/member"

module Input
  def self.parse_datetime(str, value_name)
    Time::Format::ISO_8601_DATE_TIME.parse str
  rescue
    raise "#{value_name} was not a valid datetime"
  end

  def self.parse_date(str, value_name)
    Time::Format::ISO_8601_DATE.parse str
  rescue
    raise "#{value_name} was not a valid datetime"
  end

  @[GraphQL::InputObject]
  class LoginInfo
    include GraphQL::InputObjectType

    getter email, pass_hash

    @[GraphQL::Field]
    def initialize(@email : String, @pass_hash : String)
    end
  end

  @[GraphQL::InputObject]
  class PasswordReset
    include GraphQL::InputObjectType

    getter pass_hash

    @[GraphQL::Field]
    def initialize(@pass_hash : String)
    end
  end

  @[GraphQL::InputObject]
  class NewMember
    include GraphQL::InputObjectType

    @enrollment : Models::ActiveSemester::Enrollment?

    getter email, first_name, preferred_name, last_name,
      pass_hash, phone_number, picture, passengers, location,
      on_campus, about, major, minor, hometown, arrived_at_tech,
      gateway_drug, conflicts, dietary_restrictions,
      enrollment, section

    @[GraphQL::Field]
    def initialize(
      @email : String,
      @first_name : String,
      @preferred_name : String?,
      @last_name : String,
      @pass_hash : String,
      @phone_number : String,
      @picture : String?,
      @passengers : Int32,
      @location : String,
      @on_campus : Bool?,
      @about : String?,
      @major : String?,
      @minor : String?,
      @hometown : String?,
      @arrived_at_tech : Int32?,
      @gateway_drug : String?,
      @conflicts : String?,
      @dietary_restrictions : String,
      enrollment : String?,
      @section : String?
    )
      @enrollment = enrollment.try { |e| Models::ActiveSemester::Enrollment.parse e }
    end
  end

  @[GraphQL::InputObject]
  class RegisterForSemesterForm
    include GraphQL::InputObjectType

    @enrollment : Models::ActiveSemester::Enrollment

    getter location, on_campus, conflicts,
      dietary_restrictions, enrollment, section

    @[GraphQL::Field]
    def initialize(
      @location : String,
      @on_campus : Bool?,
      @conflicts : String,
      @dietary_restrictions : String,
      enrollment : String,
      @section : String
    )
      @enrollment = Models::ActiveSemester::Enrollment.parse enrollment
    end
  end

  @[GraphQL::InputObject]
  class NewEvent
    include GraphQL::InputObjectType

    getter event, gig, repeat

    @[GraphQL::Field]
    def initialize(@event : Input::NewEventFields, @gig : Input::NewGig?,
                   @repeat : Input::NewEventPeriod?)
    end
  end

  @[GraphQL::InputObject]
  class NewEventFields
    include GraphQL::InputObjectType

    @call_time : Time
    @release_time : Time?

    getter name, semester, type, call_time,
      release_time, points, comments,
      location, gig_count, default_attend

    @[GraphQL::Field]
    def initialize(
      @name : String,
      @semester : String,
      @type : String,
      call_time : String,
      release_time : String?,
      @points : Int32,
      @comments : String?,
      @location : String?,
      @gig_count : Bool?,
      @default_attend : Bool
    )
      @call_time = Input.parse_datetime call_time, "Call time"
      @release_time = release_time.try { |t| Input.parse_datetime t, "Release time" }
    end
  end

  @[GraphQL::InputObject]
  class NewGig
    include GraphQL::InputObjectType

    @performance_time : Time

    getter performance_time, uniform, contact_name,
      contact_email, contact_phone, price,
      public, summary, description

    @[GraphQL::Field]
    def initialize(
      performance_time : String,
      @uniform : Int32,
      @contact_name : String?,
      @contact_email : String?,
      @contact_phone : String?,
      @price : Int32?,
      @public : Bool,
      @summary : String?,
      @description : String?
    )
      @performance_time = Input.parse_datetime performance_time, "Performance time"
    end
  end

  @[GraphQL::InputObject]
  class NewEventPeriod
    include GraphQL::InputObjectType

    @period : Input::Period
    @repeat_until : Time?

    getter period, repeat_until

    @[GraphQL::Field]
    def initialize(period : String, repeat_until : String?)
      @period = Input::Period.parse period
      @repeat_until = repeat_until.try { |u| Input.parse_date u, "Repeat until" }
    end
  end

  @[GraphQL::Enum]
  enum Period
    NO
    DAILY
    WEEKLY
    BIWEEKLY
    MONTHLY
    YEARLY
  end

  @[GraphQL::InputObject]
  class AttendanceForm
    include GraphQL::InputObjectType

    getter should_attend, did_attend, confirmed, minutes_late

    @[GraphQL::Field]
    def initialize(@should_attend : Bool, @did_attend : Bool,
                   @confirmed : Bool, @minutes_late : Int32)
    end
  end

  @[GraphQL::InputObject]
  class UpdatedCarpool
    include GraphQL::InputObjectType

    getter driver, passengers

    @[GraphQL::Field]
    def initialize(@driver : String, @passengers : Array(String))
    end
  end

  @[GraphQL::InputObject]
  class NewGigRequest
    include GraphQL::InputObjectType

    @start_time : Time

    getter name, organization, contact_name, contact_email,
      contact_phone, start_time, location, comments

    @[GraphQL::Field]
    def initialize(
      @name : String,
      @organization : String,
      @contact_name : String,
      @contact_email : String,
      @contact_phone : String,
      start_time : String,
      @location : String,
      @comments : String?
    )
      @start_time = Input.parse_datetime start_time, "Start time"
    end
  end

  @[GraphQL::InputObject]
  class NewSemester
    include GraphQL::InputObjectType

    @start_date : Time
    @end_date : Time

    getter name, start_date, end_date, gig_requirement

    @[GraphQL::Field]
    def initialize(@name : String, start_date : String,
                   end_date : String, @gig_requirement : Int32)
      @start_date = Input.parse_date start_date, "Start date"
      @end_date = Input.parse_date end_date, "End date"
    end
  end

  @[GraphQL::InputObject]
  class UpdatedMeetingMinutes
    include GraphQL::InputObjectType

    getter name, complete, public

    @[GraphQL::Field]
    def initialize(@name : String, @complete : String?, @public : String)
    end
  end

  @[GraphQL::InputObject]
  class NewUniform
    include GraphQL::InputObjectType

    getter name, color, description

    @[GraphQL::Field]
    def initialize(@name : String, @color : String?, @description : String?)
    end
  end

  @[GraphQL::InputObject]
  class NewSong
    include GraphQL::InputObjectType

    getter title, info

    @[GraphQL::Field]
    def initialize(@title : String, @info : String?)
    end
  end

  @[GraphQL::InputObject]
  class SongUpdate
    include GraphQL::InputObjectType

    @key : Models::Song::Pitch?
    @starting_pitch : Models::Song::Pitch?
    @mode : Models::Song::Mode?

    getter title, current, info, key,
      starting_pitch, mode

    @[GraphQL::Field]
    def initialize(
      @title : String,
      @current : Bool,
      @info : String?,
      key : String?,
      starting_pitch : String?,
      mode : String?
    )
      @key = key.try { |k| Models::Song::Pitch.parse k }
      @starting_pitch = starting_pitch.try { |sp| Models::Song::Pitch.parse sp }
      @mode = mode.try { |m| Models::Song::Mode.parse m }
    end
  end

  @[GraphQL::InputObject]
  class NewSongLink
    include GraphQL::InputObjectType

    getter type, name, target, content

    @[GraphQL::Field]
    def initialize(@type : String, @name : String,
                   @target : String, @content : String?)
    end
  end

  @[GraphQL::InputObject]
  class SongLinkUpdate
    include GraphQL::InputObjectType

    getter name, target

    @[GraphQL::Field]
    def initialize(@name : String, @target : String)
    end
  end

  @[GraphQL::InputObject]
  class TransactionBatch
    include GraphQL::InputObjectType

    getter members, type, amount, description

    @[GraphQL::Field]
    def initialize(@members : Array(String), @type : String,
                   @amount : Int32, @description : String)
    end
  end
end
