require "graphql"

require "../../db"

module Models
  @[GraphQL::Object]
  class Carpool
    include GraphQL::ObjectType

    class_getter table_name = "carpool"

    DB.mapping({
      id:     Int32,
      event:  Int32,
      driver: String,
    })

    def self.for_event(event_id)
      CONN.query_all "SELECT * FROM #{@@table_name} WHERE event = ?", event_id, as: Carpool
    end

    def self.update(event_id, updated_carpools)
      Event.with_id! event_id

      CONN.exec "DELETE FROM #{@@table_name} WHERE event = ?", event_id

      updated_carpools.each do |carpool|
        CONN.exec "INSERT INTO #{@@table_name} (event, driver) VALUES (?, ?)", event_id, carpool.driver
        new_id = CONN.query_one "SELECT id FROM #{@@table_name} ORDER BY id DESC", as: Int32

        carpool.passengers.each do |passenger|
          CONN.exec "INSERT INTO #{@@table_name} (member, carpool) VALUES (?, ?)", passenger, new_id
        end
      end
    end

    @[GraphQL::Field(name: "driver", description: "The driver of the carpool")]
    def full_driver : Models::Member
      Member.with_email! @driver
    end

    @[GraphQL::Field(description: "The passengers of the carpool")]
    def passengers : Array(Models::Member)
      CONN.query_all "SELECT * FROM #{Member.table_name} WHERE email = \
        (SELECT member FROM #{RidesIn.table_name} WHERE carpool = ?)", @id, as: Member
    end
  end

  class RidesIn
    class_getter table_name = "rides_in"

    DB.mapping({
      member:  String,
      carpool: Int32,
    })
  end
end
