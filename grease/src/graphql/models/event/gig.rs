require "../../db"
require "../../schema/context"

module Models
  @[GraphQL::Object]
  class Gig
    include GraphQL::ObjectType

    class_getter table_name = "gig"

    DB.mapping({
      event:            Int32,
      performance_time: Time,
      uniform:          Int32,
      contact_name:     String?,
      contact_email:    String?,
      contact_phone:    String?,
      price:            Int32?,
      public:           {type: Bool, default: false},
      summary:          String?,
      description:      String?,
    })

    def self.for_event(event_id)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE event = ?", event_id, as: Gig
    end

    @[GraphQL::Field(description: "The ID of the event this gig belongs to")]
    def event : Int32
      @event
    end

    @[GraphQL::Field(name: "performanceTime", description: "When members are expected to actually perform")]
    def gql_performance_time : String
      @performance_time.to_s
    end

    @[GraphQL::Field(description: "The uniform for this gig")]
    def uniform : Models::Uniform
      Uniform.with_id! @uniform
    end

    @[GraphQL::Field(description: "The name of the contact for this gig")]
    def contact_name : String?
      @contact_name
    end

    @[GraphQL::Field(description: "The email of the contact for this gig")]
    def contact_email : String?
      @contact_email
    end

    @[GraphQL::Field(description: "The phone of the contact for this gig")]
    def contact_phone : String?
      @contact_phone
    end

    @[GraphQL::Field(description: "The price we are charging for this gig")]
    def price : Int32?
      @price
    end

    @[GraphQL::Field(description: "Whether this gig is visible on the external website")]
    def public : Bool
      @public
    end

    @[GraphQL::Field(description: "A summary of this event for the external site (if it is public)")]
    def summary : String?
      @summary
    end

    @[GraphQL::Field(description: "A description of this event for the external site (if it is public)")]
    def description : String?
      @description
    end
  end

  @[GraphQL::Object]
  class GigRequest
    include GraphQL::ObjectType

    class_getter table_name = "gig_request"

    @[GraphQL::Enum(name: "GigRequestStatus")]
    enum Status
      PENDING
      ACCEPTED
      DISMISSED

      def self.mapping
        {
          "PENDING"   => PENDING,
          "ACCEPTED"  => ACCEPTED,
          "DISMISSED" => DISMISSED,
        }
      end

      def to_rs
        Status.mapping.invert[self].downcase
      end

      def self.from_rs(rs)
        val = rs.read
        status = val.as?(String).try { |v| Status.mapping[v.upcase]? }
        status || raise "Invalid gig request status returned from database: #{val}"
      end

      def self.parse(val)
        Status.mapping[val]? || raise "Invalid gig request status variant provided: #{val}"
      end
    end

    DB.mapping({
      id:            Int32,
      event:         Int32?,
      time:          {type: Time, default: Time.local},
      name:          String,
      organization:  String,
      contact_name:  String,
      contact_phone: String,
      contact_email: String,
      start_time:    Time,
      location:      String,
      comments:      String?,
      status:        {type: Status, default: Status::PENDING, converter: Status},
    })

    def self.with_id(id)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE id = ?", id, as: GigRequest
    end

    def self.with_id!(id)
      (with_id id) || raise "No gig request with ID #{id}"
    end

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name} ORDER BY time", as: GigRequest
    end

    def self.submit(form)
      CONN.exec "INSERT INTO #{@@table_name} \
        (name, organization, contact_name, contact_phone, \
        contact_email, start_time, location, comments) \
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        form.name, form.organization, form.contact_name, form.contact_phone,
        form.contact_email, form.start_time, form.location, form.comments

      CONN.query_one "SELECT id FROM #{@@table_name} ORDER BY id DESC", as: Int32
    end

    def set_status(status)
      if @status == status
        return
      elsif @status == Status::ACCEPTED
        raise "Cannot change the status of an accepted gig request"
      elsif @status == Status::DISMISSED && status == Status::ACCEPTED
        raise "Cannot directly accept a gig request if it is dismissed (please reopen it first)"
      elsif @status == Status::PENDING && status == Status::ACCEPTED && @event.nil?
        raise "Must create the event for the gig request first before marking it as accepted"
      else
        CONN.exec "UPDATE #{@@table_name} SET status = ? WHERE id = ?", status.to_rs, @id

        @status = status
      end
    end

    def build_new_gig
      Input::NewGig.new @start_time.to_s, Uniform.default.id, @contact_name, @contact_email,
        @contact_phone, nil, false, nil, nil
    end

    @[GraphQL::Field(description: "The ID of the gig request")]
    def id : Int32
      @id
    end

    @[GraphQL::Field(name: "time", description: "When the gig request was placed")]
    def gql_time : String
      @time.to_s
    end

    @[GraphQL::Field(description: "The name of the potential event")]
    def name : String
      @name
    end

    @[GraphQL::Field(description: "The organization requesting a performance from the Glee Club")]
    def organization : String
      @organization
    end

    @[GraphQL::Field(description: "If and when an event is created from a request, this is the event")]
    def event : Models::Event?
      @event.try { |id| Event.with_id! id }
    end

    @[GraphQL::Field(description: "The name of the contact for the potential event")]
    def contact_name : String
      @contact_name
    end

    @[GraphQL::Field(description: "The email of the contact for the potential event")]
    def contact_email : String
      @contact_email
    end

    @[GraphQL::Field(description: "The phone number of the contact for the potential event")]
    def contact_phone : String
      @contact_phone
    end

    @[GraphQL::Field(name: "startTime", description: "When the event will probably happen")]
    def gql_start_time : String
      @start_time.to_s
    end

    @[GraphQL::Field(description: "Where the event will be happening")]
    def location : String
      @location
    end

    @[GraphQL::Field(description: "Any comments about the event")]
    def comments : String?
      @comments
    end

    @[GraphQL::Field(description: "The current status of whether the request was accepted")]
    def status : Models::GigRequest::Status
      @status
    end
  end
end
