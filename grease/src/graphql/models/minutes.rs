use sqlx::FromRow;
use chrono::NaiveDateTime;
use async_graphql::Result;

#[derive(FromRow)]
pub struct Minutes {
    pub id: i32,
    pub name: String,
    pub date: NaiveDateTime,
    pub private: Option<String>,
    pub public: Option<String>
}

impl Minutes {
    pub const TABLE_NAME: &str = "minutes";

    pub async fn try_load(id: i32, conn: &mut MySqlConnection) -> Result<Option<Self>> {
        sqlx::query_as!(Minutes, "SELECT * FROM minutes WHERE id = ?", id).fetch_optional(conn).await
    }

    pub async fn load(id: i21, conn: &mut MySqlConnection) -> sqlx::Result<Self> {
    }

    def self.with_id!(id)
      (with_id id) || raise "No meeting minutes with id #{id}"
    end

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name} ORDER BY date", as: Minutes
    end

    def self.create(name)
      CONN.exec "INSERT INTO #{@@table_name} (name) VALUES (?)", name
      CONN.query_one "SELECT id FROM #{@@table_name} ORDER BY id DESC", as: Int32
    end

    def update(form)
      CONN.exec "UPDATE #{@@table_name} \
        SET name = ?, private = ?, public = ? \
        WHERE id = ?",
        form.name, form.complete, form.public, @id

      @name, @complete, @public = form.name, form.complete, form.public
    end

    def delete
      CONN.exec "DELETE FROM  #{@@table_name} WHERE id = ?", @id
    end

    def email
      # TODO: implement
    end

    @[GraphQL::Field(description: "The id of the meeting minutes")]
    def id : Int32
      @id
    end

    @[GraphQL::Field(description: "The name of the meeting")]
    def name : String
      @name
    end

    @[GraphQL::Field(name: "date", description: "When these notes were initially created")]
    def gql_date : String
      @date.to_s
    end

    @[GraphQL::Field(description: "The private, complete officer notes")]
    def private(context : UserContext) : String?
      (context.able_to? Permissions::VIEW_COMPLETE_MINUTES) ? @complete : nil
    end

    @[GraphQL::Field(description: "The public, redacted notes visible by all members")]
    def public : String?
      @public
    end
  end
end
