require "../../event/*"

module Models
  class WeekOfEvents
    def initialize(@events_with_attendance : Array({Event, Attendance?}))
    end

    def each(&block)
      @events_with_attendance.each do |(event, attendance)|
        yield event, attendance
      end
    end

    def missed_rehearsal?
      @events_with_attendance.any? do |(event, attendance)|
        event.type == Event::REHEARSAL && attendance.try &.deny_credit?
      end
    end

    def first_missed_sectional
      @events_with_attendance.find do |(event, attendance)|
        event.type == Event::SECTIONAL && attendance.try &.deny_credit?
      end
    end

    def attended_sectional?
      @events_with_attendance.any? do |(event, attendance)|
        event.type == Event::SECTIONAL && attendance.try &.deny_credit?
      end
    end

    def sectionals
      @events_with_attendance.select do |(event, attendance)|
        event.type == Event::SECTIONAL
      end
    end

    def last_sectional
      sectionals = self.sectionals
      sectionals.empty? ? nil : sectionals[-1]
    end

    def is_bonus_event?(event, attendance)
      event.type == Event::VOLUNTEER_GIG || event.type == Event::OMBUDS ||
        (event.type == Event::OTHER && !attendance.should_attend) ||
        (event.type == Event::SECTIONAL && self.first_missed_sectional.nil?)
    end

    def attended_volunteer_gig(event, attendance)
      attendance.try &.did_attend &&
        !self.missed_rehearsal? &&
        event.type == Event::VOLUNTEER_GIG &&
        event.gig_count
    end
  end
end
