require "graphql"

require "../db"

module Models
  @[GraphQL::Object]
  class Document
    include GraphQL::ObjectType

    class_getter table_name = "google_docs"

    DB.mapping({
      name: String,
      url:  String,
    })

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name} ORDER BY name", as: Document
    end

    def self.with_name(name)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE name = ?", name, as: Document
    end

    def self.with_name!(name)
      (with_name name) || raise "No document named #{name}"
    end

    def self.create(name, url)
      existing = CONN.query_one? "SELECT name FROM #{@@table_name} WHERE name = ?", name, as: String
      raise "A document already exists named #{name}" if existing

      CONN.exec "INSERT INTO #{@@table_name} (name, url) VALUES (?, ?)", name, url
    end

    def set_url(url)
      CONN.exec "UPDATE #{@@table_name} SET url = ? WHERE name = ?", url, @name

      @url = url
    end

    def delete
      CONN.exec "DELETE FROM #{@@table_name} WHERE name = ?", @name
    end

    @[GraphQL::Field(description: "The name of the document")]
    def name : String
      @name
    end

    @[GraphQL::Field(description: "A link to the document")]
    def url : String
      @url
    end
  end

  @[GraphQL::Object]
  class Variable
    include GraphQL::ObjectType

    class_getter table_name = "variable"

    DB.mapping({
      key:   String,
      value: String,
    })

    def self.with_key(key)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE key = ?", key, as: Variable
    end

    def self.with_key!(key)
      (with_key key) || raise "No value set at key #{key}"
    end

    def self.set(key, value)
      if CONN.query_one? "SELECT key FROM #{@@table_name} WHERE key = ?", key, as: String
        CONN.exec "UPDATE #{@@table_name} SET value = ? WHERE key = ?", value, key
      else
        CONN.exec "INSERT INTO #{@@table_name} (key, value) VALUES (?, ?)", key, value
      end
    end

    def unset
      CONN.exec "DELETE FROM #{@@table_name} WHERE key = ?", @key
    end

    @[GraphQL::Field(description: "The name of the variable")]
    def key : String
      @key
    end

    @[GraphQL::Field(description: "The value of the variable")]
    def value : String
      @value
    end
  end
end
