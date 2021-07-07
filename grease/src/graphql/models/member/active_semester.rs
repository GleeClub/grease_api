require "uuid"
require "graphql"

require "../../db"
require "./grades"
require "../permissions/constants"

module Models
  @[GraphQL::Object]
  class ActiveSemester
    include GraphQL::ObjectType

    class_getter table_name = "active_semester"

    @[GraphQL::Enum]
    enum Enrollment
      CLASS
      CLUB

      def self.mapping
        {
          "CLASS" => CLASS,
          "CLUB"  => CLUB,
        }
      end

      def to_rs
        Enrollment.mapping.invert[self].downcase
      end

      def self.from_rs(rs)
        val = rs.read
        enrollment = val.as?(String).try { |v| Enrollment.mapping[v.upcase]? }
        enrollment || raise "Invalid enrollment returned from database: #{val}"
      end

      def self.parse(val)
        Enrollment.mapping[val]? || raise "Invalid enrollment variant provided: #{val}"
      end
    end

    DB.mapping({
      member:     String,
      semester:   String,
      enrollment: {type: Enrollment, default: Enrollment::CLUB, converter: Enrollment},
      section:    String?,
    })

    def self.all_for_member(email)
      CONN.query_all "SELECT * FROM #{@@table_name} WHERE member = ?", email, as: ActiveSemester
    end

    def self.for_semester(email, semester_name)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE member = ? AND SEMESTER = ?",
        email, semester_name, as: ActiveSemester
    end

    def self.create_for_member(member, form, semester)
      if member.get_semester(semester.name)
        raise "#{member.full_name} is already active for the current semester"
      end

      CONN.exec "INSERT INTO #{@@table_name} (member, semester, enrollment, section)
        VALUES (?, ?, ?, ?)", member.email, semester.name, form.enrollment.to_rs, form.section
    end

    def self.update(email, semester_name, enrollment, section)
      active_semester = for_semester email, semester_name

      if enrollment
        if active_semester
          CONN.exec "UPDATE #{@@table_name} SET enrollment = ?, section = ? \
            WHERE member = ? AND semester = ?", enrollment.to_rs, section, email, semester_name
        else
          CONN.exec "INSERT INTO #{@@table_name} (member, semester, enrollment, section)
            VALUES (?, ?, ?, ?)", email, semester_name, enrollment.to_rs, section
        end
      elsif active_semester
        CONN.exec "DELETE FROM #{@@table_name} WHERE member = ? AND SEMESTER = ?", email, semester_name
      end
    end

    @[GraphQL::Field(description: "The grades for the member in the given semester")]
    def grades : Models::Grades
      Grades.for_member (Member.with_email! @member), (Semester.with_name! @semester)
    end

    @[GraphQL::Field]
    def semester : String
      @semester
    end

    @[GraphQL::Field]
    def enrollment : Models::ActiveSemester::Enrollment
      @enrollment
    end
  end
end
