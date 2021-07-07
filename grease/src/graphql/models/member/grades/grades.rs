require "graphql"

require "./week"
require "../../event/*"

module Models
  @[GraphQL::Object]
  class GradeChange
    include GraphQL::ObjectType

    def initialize(@reason : String, @change : Float64, @partial_score : Float64)
    end

    @[GraphQL::Field(description: "The reason the grade change was incurred.")]
    def reason : String
      @reason
    end

    @[GraphQL::Field(description: "How much the grade changed.")]
    def change : Float64
      @change
    end

    @[GraphQL::Field(description: "What the final grade was up to this event.")]
    def partial_score : Float64
      @partial_score
    end
  end

  @[GraphQL::Object]
  class EventWithGradeChange
    include GraphQL::ObjectType

    def initialize(@event : Models::Event, @change : GradeChange)
    end

    @[GraphQL::Field]
    def event : Models::Event
      @event
    end

    @[GraphQL::Field]
    def change : Models::GradeChange
      @change
    end
  end

  @[GraphQL::Object]
  class Grades
    include GraphQL::ObjectType

    def initialize(@final_grade : Float64,
                   @volunteer_gigs_attended : Int32,
                   @events_with_changes : Array(Models::EventWithGradeChange))
    end

    @[GraphQL::Field]
    def final_grade : Float64
      @final_grade
    end

    @[GraphQL::Field]
    def events_with_changes : Array(Models::EventWithGradeChange)
      @events_with_changes
    end

    @[GraphQL::Field]
    def volunteer_gigs_attended : Int32
      @volunteer_gigs_attended
    end

    def self.for_member(member, semester)
      events_with_attendance = Event.for_member_with_attendance member.email, semester.name
      grade, gigs_attended, events_with_changes = 100.0, 0, [] of EventWithGradeChange

      for_each_week events_with_attendance do |week|
        week.each do |event, attendance|
          change = calculate_grade_change event, attendance, week, grade
          grade = change.partial_score

          gigs_attended += 1 if week.attended_volunteer_gig event, attendance
          events_with_changes << EventWithGradeChange.new event, change
        end
      end

      Grades.new grade, gigs_attended, events_with_changes
    end

    def self.for_each_week(events_with_attendance : Array({Event, Attendance?}), &block)
      return if events_with_attendance.empty?

      start = events_with_attendance.first[0].call_time.at_beginning_of_week
      finish = events_with_attendance[-1][0].call_time

      while start <= finish
        week = events_with_attendance.select do |(event, attendance)|
          event.call_time >= start && event.call_time < start.shift weeks: 1
        end

        yield WeekOfEvents.new week unless week.empty?

        start = start.shift weeks: 1
      end
    end

    def self.calculate_grade_change(event : Event, attendance : Attendance?, week, grade)
      return GradeChange.new "No grades for inactive members", 0.0, grade unless attendance

      change, reason = if event.call_time > Time.local
                         event_hasnt_happened_yet
                       elsif attendance.did_attend
                         if week.missed_rehearsal? && event.is_gig?
                           missed_rehearsal event
                         elsif attendance.minutes_late && event.type != Models::Event::OMBUDS
                           late_for_event event, attendance, grade, (week.is_bonus_event? event, attendance)
                         elsif week.is_bonus_event? event, attendance
                           attended_bonus_event event, grade
                         else
                           attended_normal_event
                         end
                       elsif attendance.should_attend
                         should_have_attended event, attendance, week
                       else
                         didnt_need_to_attend
                       end

      new_grade = (grade + change).clamp 0.0, 100.0
      GradeChange.new reason, change, new_grade
    end

    def self.event_hasnt_happened_yet
      {0.0, "Event hasn't happened yet"}
    end

    def self.didnt_need_to_attend
      {0.0, "Did not attend and not expected to"}
    end

    def self.attended_normal_event
      {0.0, "No point change for attending required event"}
    end

    def self.late_for_event(event, attendance, grade, is_bonus_event)
      late_penalty = points_lost_for_lateness event, attendance

      if is_bonus_event
        if grade + event.points - late_penalty > 100
          return 100 - grade, "Event would grant #{event.points}-point bonus, \
					  but #{late_penalty} points deducted for lateness (capped at 100%)"
        else
          return event.points - late_penalty, "Event would grant #{event.points}-point \
		  			bonus, but #{late_penalty} points deducted for lateness"
        end
      elsif attendance.should_attend
        return -late_penalty, "#{late_penalty} points deducted for lateness to required event"
      else
        return 0.0, "No point change for attending required event"
      end
    end

    def self.missed_rehearsal(event)
      # If you haven't been to rehearsal this week, you can't get points or gig credit
      if event.type == Models::Event::VOLUNTEER_GIG
        {0.0, "#{event.points}-point bonus denied because this week's rehearsal was missed"}
      else
        {-event.points.to_f64, "Full deduction for unexcused absence from this week's rehearsal"}
      end
    end

    def self.attended_bonus_event(event, grade)
      # Get back points for volunteer gigs and and extra sectionals and ombuds events
      if grade + event.points > 100
        return 100 - grade, "Event grants #{event.points}-point bonus, but grade is capped at 100%"
      else
        return event.points.to_f64, "Full bonus awarded for attending volunteer or extra event"
      end
    end

    def self.should_have_attended(event, attendance, week)
      # Lose the full point value if did not attend
      if event.type == Models::Event::OMBUDS
        return {0.0, "You do not lose points for missing an ombuds event"}
      elsif event.type == Models::Event::SECTIONAL && week.attended_sectional?
        return {0.0, "No deduction because you attended a different sectional this week"}
      end

      if event.type == Models::Event::SECTIONAL
        if excuse = excused_from_sectional_penalty(event, attendance, week)
          return excuse
        end
      end

      if attendance.approved_absence
        {0.0, "No deduction because an absence request was submitted and approved"}
      else
        {-event.points.to_f64, "Full deduction for unexcused absence from event"}
      end
    end

    def self.excused_from_sectional_penalty(event, attendance, week)
      first_missed_sectional = week.first_missed_sectional
      if first_missed_sectional && first_missed_sectional[0].call_time < event.call_time
        return {0.0, "No deduction because you already lost points for one sectional this week"}
      end

      if last_sectional = week.last_sectional
        call_time : Time = last_sectional[0].call_time
        if call_time > event.call_time && call_time > Time.local
          return {0.0, "No deduction because not all sectionals occurred yet"}
        end
      end
    end

    # Lose points equal to the percentage of the event missed, if they should have attended
    def self.points_lost_for_lateness(event, attendance)
      call_time, release_time = event.call_time, event.release_time
      duration = if release_time && release_time > call_time
                   (release_time - call_time).minutes
                 else
                   60
                 end

      (attendance.minutes_late.to_f64 / duration) * event.points
    end
  end
end
