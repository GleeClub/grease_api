require "./member"

module Models
  class Session
    class_getter table_name = "session"

    DB.mapping({
      member: String,
      key:    String,
    })

    def self.for_token(token)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE `key` = ?", token, as: Session
    end

    def self.for_token!(token)
      (for_token token) || raise "No login tied to the provided API token"
    end

    def self.create(email, token)
      CONN.exec "INSERT INTO #{@@table_name} (member, `key`) VALUES (?, ?)", email, token
    end

    def self.get_or_generate_token(email)
      Member.with_email! email

      session = CONN.query_one? "SELECT * FROM #{@@table_name} WHERE member = ?", email, as: Session
      return session.key if session

      token = UUID.random.to_s
      Session.create email, token

      token
    end

    def self.remove_for(email)
      CONN.exec "DELETE FROM #{@@table_name} WHERE member = ?", email
    end

    def self.generate_for_forgotten_password(email)
      Member.with_email! email

      CONN.exec "DELETE FROM #{@@table_name} WHERE member = ?", email
      new_token = "#{UUID.random.to_s[0...32]}X#{Time.local.to_unix_ms}"
      Session.create email, new_token

      Emails::ResetPassword.send email, new_token
    end

    def self.reset_password(token, pass_hash)
      session = (Session.for_token token) || raise "No password reset request was found \
        for the given token. Please request another password reset."

      time_requested = session.key.split('X')[1]?.try &.to_i64.try { |ms| Time.unix_ms ms }
      if !time_requested || (time_requested.shift days: 1) < Time.local
        raise "Your token expired after 24 hours. Please request another password reset."
      end

      Session.remove_for session.member
      hash = Crypto::Bcrypt::Password.create(pass_hash, cost: 10)
      CONN.exec "UPDATE #{Member.table_name} SET pass_hash = ? WHERE email = ?",
        hash.to_s, session.member
    end
  end
end
