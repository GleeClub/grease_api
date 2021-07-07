use sqlx::MySqlConnection;
use async_graphql::Result;

pub struct Fee {
    pub name: String,
    pub description: String,
    pub amount: i32,
}

impl Fee {
    pub const DUES: &'static str = "dues";
    pub const LATE_DUES: &'static str = "latedues";

    pub const DUES_NAME: &'static str = "Dues";
    pub const DUES_DESCRIPTION: &'static str = "Semesterly Dues";
    pub const LATE_DUES_DESCRIPTION: &'static str = "Late Dues";

    pub async fn load_all(conn: &mut MySqlConnection) -> Result<Vec<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM fee ORDER BY NAME")
            .query_all(conn).await
    }

    pub async fn load_opt(name: &str, conn: &mut MySqlConnection) -> Result<Option<Self>> {
        sqlx::query_as!(Self, "SELECT * FROM fee WHERE name = ?", name)
            .query_optional(conn).await
    }

    pub async fn load(name: &str, conn: &mut MySqlConnection) -> Result<Self> {
        Self::load_opt(name, conn).await.and_then(|fee| fee.ok_or_else(|| format!("No fee named {}", name)))
    }

    pub async fn set_amount(name: &str, new_amount: i32, conn: &mut MySqlConnection) -> Result<()> {
        sqlx::query!("UPDATE fee SET amount = ? WHERE name = ?", new_amount, name).exec(conn).await
    }

    pub async fn charge_dues_for_semester(conn: &mut MySqlConnection) -> Result<()> {
        let dues = Self::load(Self::DUES, conn).await?;
        conn.begin(|tx| {
            let members_who_havent_paid = sqlx::query!(
                "SELECT member FROM active_semester WHERE semester = ? AND email NOT IN \
                 (SELECT member FROM transaction WHERE type = ? AND description = ?)",
                 Semester::current()?.name, DUES_NAME, DUES_DESCRIPTION)
                .query_all(conn).await?;

            for email in members_who_havent_paid {
                sqlx::query!(
                    "INSERT INTO transaction (member, amount, type, description, semester)
                     VALUES (?, ?, ?, ?, ?)", email, 

            }


    def self.charge_dues_for_semester

          (SELECT member FROM #{ActiveSemester.table_name} WHERE semester = ?) \
        AND email NOT IN \
          (SELECT member FROM #{ClubTransaction.table_name} \
            WHERE type = ? AND description = ?)",
        Semester.current.name, DUES_NAME, DUES_DESCRIPTION, as: String

      members_who_have_not_paid.each do |email|
        CONN.exec "INSERT INTO #{ClubTransaction.table_name} \
          (member, amount, type, description, semester) VALUES (?, ?, ?, ?, ?)",
          email, dues.amount, DUES_NAME, DUES_DESCRIPTION, Semester.current.name
      end
    end

    def self.charge_late_dues_for_semester
      late_dues = with_name! LATE_DUES

      members_who_have_not_paid = CONN.query_all "SELECT email FROM #{Member.table_name} \
        WHERE email IN \
          (SELECT member FROM #{ActiveSemester.table_name} WHERE semester = ?) \
        AND email NOT IN \
          (SELECT member FROM #{ClubTransaction.table_name} \
            WHERE type = ? AND description = ?)",
        Semester.current.name, DUES_NAME, DUES_DESCRIPTION, as: String

      members_who_have_not_paid.each do |email|
        CONN.exec "INSERT INTO #{ClubTransaction.table_name} \
          (member, amount, type, description, semester) VALUES (?, ?, ?, ?, ?)",
          email, late_dues.amount, DUES_NAME, LATE_DUES_DESCRIPTION, Semester.current.name
      end
    end
    
    @[GraphQL::Field(description: "The short name of the fee")]
    def name : String
      @name
    end

    @[GraphQL::Field(description: "A longer description of what it is charging members for")]
    def description : String
      @description
    end

    @[GraphQL::Field(description: "The amount to charge members")]
    def amount : Int32
      @amount
    end
  end

  class TransactionType
    class_getter table_name = "transaction_type"

    DB.mapping({
      name: String,
    })

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name} ORDER BY name", as: TransactionType
    end

    def self.with_name(name)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE name = ?", name, as: TransactionType
    end

    def self.with_name!(name)
      (with_name name) || raise "No transaction type named #{name}"
    end
  end

  @[GraphQL::Object]
  class ClubTransaction
    include GraphQL::ObjectType

    class_getter table_name = "transaction"

    DB.mapping({
      id:          Int32,
      member:      String,
      time:        {type: Time, default: Time.local},
      amount:      Int32,
      description: String,
      semester:    String?,

     CONN.query_one? "SELECT * FROM #{@@table_name} WHERE id = ?", id, as: ClubTransaction
    end

    def self.with_id!(id)
      (with_id id) || raise "No transaction with id #{id}"
    end

    def self.for_semester(semester_name)
      CONN.query_all "SELECT * FROM #{@@table_name} \
        WHERE semester = ? ORDER BY time", semester_name, as: ClubTransaction
    end

    def self.for_member_during_semester(email, semester_name)
      CONN.query_all "SELECT * FROM #{@@table_name} \
        WHERE semester = ? AND member = ? ORDER BY time", semester_name, email, as: ClubTransaction
    end

    def self.add_batch(batch)
      type = TransactionType.with_name! batch.type

      batch.members.each do |email|
        CONN.exec "INSERT INTO #{@@table_name} \
          (member, amount, type, description, semester) \
          VALUES (?, ?, ?, ?, ?)",
          email, batch.amount, type.name, batch.description, Semester.current.name
      end
    end

    def resolve(resolved)
      CONN.exec "UPDATE #{@@table_name} SET resolved = ? WHERE id = ?",
        resolved, @id

      @resolved = resolved
    end

    @[GraphQL::Field(description: "The ID of the transaction")]
    def id : Int32
      @id
    end

    @[GraphQL::Field(description: "The email of the member this transaction was charged to")]
    def member : Models::Member
      Member.with_email! @member
    end

    @[GraphQL::Field(name: "time", description: "When this transaction was charged")]
    def gql_time : String
      @time.to_s
    end

    @[GraphQL::Field(description: "How much this transaction was for")]
    def amount : Int32
      @amount
    end

    @[GraphQL::Field(description: "A description of what the member was charged for specifically")]
    def description : String
      @description
    end

    @[GraphQL::Field(description: "Optionally, the name of the semester this transaction was made during")]
    def semester : String?
      @semester
    end

    @[GraphQL::Field(description: "The name of the type of transaction")]
    def type : String
      @type
    end

    @[GraphQL::Field(description: "Whether the member has paid the amount requested in this transaction")]
    def resolved : Bool
      @resolved
    end
  end
end
