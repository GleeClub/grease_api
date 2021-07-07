require "graphql"

require "../../db"

module Models
  @[GraphQL::Object]
  class Uniform
    include GraphQL::ObjectType

    class_getter table_name = "uniform"

    DB.mapping({
      id:          Int32,
      name:        String,
      color:       String?,
      description: String?,
    })

    def is_valid?
      if color = u.color
        match = /#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})/i.match(color)
        !match.nil?
      else
        true
      end
    end

    def validate!
      raise "Uniform color must be a valid CSS color string" unless is_valid?
    end

    def self.with_id(id)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE id = ?", id, as: Uniform
    end

    def self.with_id!(id)
      (with_id id) || raise "No uniform with id #{id}"
    end

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name} ORDER BY name", as: Uniform
    end

    def self.default
      default = CONN.query_one? "SELECT * FROM #{@@table_name} ORDER BY name", as: Uniform
      default || raise "There are currently no uniforms"
    end

    def self.create(form)
      CONN.exec "INSERT INTO #{@@table_name} (name, color, description) VALUES (?, ?, ?)",
        form.name, form.color, form.description
      CONN.query_one "SELECT id FROM #{@@table_name} ORDER BY id DESC", as: Int32
    end

    def update(form)
      CONN.exec "UPDATE #{@@table_name} \
      SET name = ?, color = ?, description = ? \
      WHERE id = ?",
        form.name, form.color, form.description, @id

      @name, @color, @description = form.name, form.color, form.description
    end

    def delete
      CONN.exec "DELETE FROM  #{@@table_name} WHERE id = ?", @id
    end

    @[GraphQL::Field(description: "The ID of the uniform")]
    def id : Int32
      @id
    end

    @[GraphQL::Field(description: "The name of the uniform")]
    def name : String
      @name
    end

    @[GraphQL::Field(description: "The associated color of the uniform (In the format \"#HHH\", where \"H\" is a hex digit)")]
    def color : String?
      @color
    end

    @[GraphQL::Field(description: "The explanation of what to wear when wearing the uniform")]
    def description : String?
      @description
    end

    def self.with_id(id)
      uniform = CONN.query_one? "SELECT * from #{@@table_name} where id = ?", id, as: Uniform
      uniform || raise "No uniform with id #{id}"
    end
  end
end
