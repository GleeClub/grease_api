require "../../db"
require "../../schema/context"

module Models
  @[GraphQL::Object]
  class PublicEvent
    include GraphQL::ObjectType

    DATETIME_FORMAT = "%Y%m%dT%H%M%SZ"

    def initialize(
      @id : Int32,
      @name : String,
      @time : Time,
      @location : String,
      @summary : String,
      @description : String,
      @invite : String
    )
    end

    def self.all_for_current_semester
      events = CONN.query_all "SELECT * FROM #{Event.table_name} WHERE id IN \
        (SELECT event FROM #{Gig.table_name} WHERE public = true) \
        ORDER BY call_time", as: Event

      events.map do |event|
        gig = event.gig || raise "All public events must have gigs"
        end_time = event.release_time || event.call_time.shift hours: 1

        calendar_event = String.build do |io|
          io << "VERSION:2.0\n"
          io << "PRODID:ICALENDAR-RS\n"
          io << "CALSCALE:GREGORIAN\n"
          io << "BEGIN:VEVENT\n"
          io << "DTSTAMP:" << (Time.utc.to_s DATETIME_FORMAT) << "\n"
          io << "DESCRIPTION:" << (gig.summary || "") << "\n"
          io << "DTEND:" << (end_time.to_s DATETIME_FORMAT) << "\n"
          io << "DTSTART:" << (event.call_time.to_s DATETIME_FORMAT) << "\n"
          io << "LOCATION:" << (event.location || "") << "\n"
          io << "SUMMARY:" << (gig.description || "") << "\n"
          io << "UID:" << UUID.random << "\n"
          io << "END:VEVENT\n"
          io << "END:VCALENDAR\n"
        end

        new event.id, event.name, event.call_time, (event.location || ""), (gig.summary || ""),
          (gig.description || ""), "data:text/calendar;base64,#{Base64.encode calendar_event}"
      end
    end

    @[GraphQL::Field]
    def id : Int32
      @id
    end

    @[GraphQL::Field]
    def name : String
      @name
    end

    @[GraphQL::Field(name: "time")]
    def gql_time : String
      @time.to_s
    end

    @[GraphQL::Field]
    def location : String
      @location
    end

    @[GraphQL::Field]
    def summary : String
      @summary
    end

    @[GraphQL::Field]
    def description : String
      @description
    end

    @[GraphQL::Field]
    def invite : String
      @invite
    end
  end
end
