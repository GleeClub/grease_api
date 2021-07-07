require "mysql"
require "graphql"

require "../db"

module Models
  @[GraphQL::Object]
  class Semester
    include GraphQL::ObjectType

    class_getter table_name = "semester"
    class_getter current : Semester {
      semester = CONN.query_one? "SELECT * FROM #{@@table_name} WHERE current = true", as: Semester
      semester || raise "No current semester set"
    }

    DB.mapping({
      name:            String,
      start_date:      Time,
      end_date:        Time,
      gig_requirement: {type: Int32, default: 5},
      current:         {type: Bool, default: false},
    })

    def self.with_name(name)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE name = ?", name, as: Semester
    end

    def self.with_name!(name)
      (with_name name) || raise "No semester named #{name}"
    end

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name} ORDER BY start_date", as: Semester
    end

    def self.create(form)
      raise "A semester already exists named #{form.name}" if with_name form.name

      CONN.exec "INSERT INTO #{@@table_name} \
        (name, start_date, end_date, gig_requirement) \
        VALUES (?, ?, ?, ?)",
        form.name, form.start_date, form.end_date, form.gig_requirement
    end

    def self.update(name, form)
      if name == form.name
        with_name! name
      else
        raise "Another semester is already named #{form.name}" if with_name form.name
      end

      CONN.exec "UPDATE #{@@table_name} SET \
        name = ?, start_date = ?, end_date = ?, gig_requirement = ? \
        WHERE name = ?",
        form.name, form.start_date, form.end_date, form.gig_requirement,
        name
    end

    def self.set_current(name)
      with_name! name

      CONN.exec "UPDATE #{@@table_name} SET current = false"
      CONN.exec "UPDATE #{@@table_name} SET current = true WHERE name = ?", name
    end

    @[GraphQL::Field(description: "The name of the semester")]
    def name : String
      @name
    end

    @[GraphQL::Field(name: "startDate", description: "When the semester starts")]
    def gql_start_date : String
      @start_date.to_s
    end

    @[GraphQL::Field(name: "endDate", description: "When the semester ends")]
    def gql_end_date : String
      @end_date.to_s
    end

    @[GraphQL::Field(description: "How many volunteer gigs are required for the semester")]
    def gig_requirement : Int32
      @gig_requirement
    end

    @[GraphQL::Field(description: "Whether this is the current semester")]
    def current : Bool
      @current
    end
  end
end
