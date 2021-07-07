require "uri"
require "mysql"
require "graphql"

require "./event"
require "../utils"

module Models
  @[GraphQL::Object]
  class Song
    include GraphQL::ObjectType

    class_getter table_name = "song"

    @[GraphQL::Enum]
    enum Pitch
      A_FLAT
      A
      A_SHARP
      B_FLAT
      B
      B_SHARP
      C_FLAT
      C
      C_SHARP
      D_FLAT
      D
      D_SHARP
      E_FLAT
      E
      E_SHARP
      F_FLAT
      F
      F_SHARP
      G_FLAT
      G
      G_SHARP

      def self.mapping
        {
          "A_FLAT"  => A_FLAT,
          "A"       => A,
          "A_SHARP" => A_SHARP,
          "B_FLAT"  => B_FLAT,
          "B"       => B,
          "B_SHARP" => B_SHARP,
          "C_FLAT"  => C_FLAT,
          "C"       => C,
          "C_SHARP" => C_SHARP,
          "D_FLAT"  => D_FLAT,
          "D"       => D,
          "D_SHARP" => D_SHARP,
          "E_FLAT"  => E_FLAT,
          "E"       => E,
          "E_SHARP" => E_SHARP,
          "F_FLAT"  => F_FLAT,
          "F"       => F,
          "F_SHARP" => F_SHARP,
          "G_FLAT"  => G_FLAT,
          "G"       => G,
          "G_SHARP" => G_SHARP,
        }
      end

      def to_rs
        Pitch.mapping.invert[self].downcase
      end

      def self.from_rs(rs, nillable? = false)
        val = rs.read
        return nil if val.nil? && nillable?
        # There is a malformed pitch in the database, this handles that
        return nil if val.as?(String) == ""

        pitch = val.as?(String).try { |v| Pitch.mapping[v.upcase]? }
        pitch || raise "Invalid pitch returned from database: #{val}"
      end

      def self.parse(val)
        Pitch.mapping[val]? || raise "Invalid pitch variant provided: #{val}"
      end
    end

    class NillablePitchConverter
      def self.from_rs(rs)
        Pitch.from_rs(rs, nillable?: true)
      end
    end

    @[GraphQL::Enum]
    enum Mode
      MAJOR
      MINOR

      def self.mapping
        {
          "MAJOR" => MAJOR,
          "MINOR" => MINOR,
        }
      end

      def to_rs
        Mode.mapping.invert[self].downcase
      end

      def self.from_rs(rs, nillable? = false)
        val = rs.read
        return nil if val.nil? && nillable?

        mode = val.as?(String).try { |v| Mode.mapping[v.upcase]? }
        mode || raise "Invalid song mode returned from database: #{val}"
      end

      def self.parse(val)
        Mode.mapping[val]? || raise "Invalid song mode variant provided: #{val}"
      end
    end

    class NillableModeConverter
      def self.from_rs(rs)
        Mode.from_rs(rs, nillable?: true)
      end
    end

    DB.mapping({
      id:             Int32,
      title:          String,
      info:           String?,
      current:        {type: Bool, default: false},
      key:            {type: Models::Song::Pitch?, converter: Models::Song::NillablePitchConverter},
      starting_pitch: {type: Models::Song::Pitch?, converter: Models::Song::NillablePitchConverter},
      mode:           {type: Models::Song::Mode?, converter: Models::Song::NillableModeConverter},
    })

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name} ORDER BY title", as: Song
    end

    def self.with_id(id)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE id = ?", id, as: Song
    end

    def self.with_id!(id)
      (with_id id) || raise "No song with id #{id}"
    end

    def self.setlist_for_event(event_id)
      CONN.query_all "SELECT * FROM #{@@table_name} \
        INNER JOIN #{GigSong.table_name} ON #{@@table_name}.id = #{GigSong.table_name}.song
        WHERE #{GigSong.table_name}.event = ?
        ORDER BY #{GigSong.table_name}.order ASC", event_id, as: Song
    end

    def self.all_public
      all_links = SongLink.all.select { |link| link.type == SongLink::PERFORMANCES }

      Song.all.map do |song|
        videos = all_links
          .select { |link| link.song == song.id }
          .map { |link| PublicVideo.new link.name, link.target }

        PublicSong.new song.title, song.current, videos
      end
    end

    def self.create(form)
      CONN.exec "INSERT INTO #{@@table_name} (title, info) \
        VALUES (?, ?)", form.title, form.info

      CONN.query_one "SELECT id FROM #{@@table_name} ORDER BY id DESC", as: Int32
    end

    def update(form)
      CONN.exec "SET #{@@table_name} \
        title = ?, current = ?, info = ?, key = ?, starting_pitch = ?, mode = ?
        WHERE id = ?", form.title, form.current, form.info, form.key.try &.to_rs,
        form.starting_pitch.try &.to_rs, form.mode.try &.to_rs, @id
    end

    def delete
      CONN.exec "DELETE #{@@table_name} WHERE id = ?", @id
    end

    @[GraphQL::Field(description: "The ID of the song")]
    def id : Int32
      @id
    end

    @[GraphQL::Field(description: "The title of the song")]
    def title : String
      @title
    end

    @[GraphQL::Field(description: "Any information related to the song (minor changes to the music, who wrote it, soloists, etc.)")]
    def info : String?
      @info
    end

    @[GraphQL::Field(description: "Whether it is in this semester's repertoire")]
    def current : Bool
      @current
    end

    @[GraphQL::Field(description: "The key of the song")]
    def key : Models::Song::Pitch?
      @key
    end

    @[GraphQL::Field(description: "The starting pitch for the song")]
    def starting_pitch : Models::Song::Pitch?
      @starting_pitch
    end

    @[GraphQL::Field(description: "The mode of the song (Major or Minor)")]
    def mode : Models::Song::Mode?
      @mode
    end

    @[GraphQL::Field(description: "The links connected to the song sorted into sections")]
    def links : Array(Models::SongLinkSection)
      all_links = SongLink.for_song @id
      all_types = MediaType.all

      all_types.map do |type|
        SongLinkSection.new type.name, all_links.select { |l| l.type == type.name }
      end
    end
  end

  @[GraphQL::Object]
  class SongLinkSection
    include GraphQL::ObjectType

    def initialize(@name : String, @links : Array(SongLink))
    end

    @[GraphQL::Field]
    def name : String
      @name
    end

    @[GraphQL::Field]
    def links : Array(Models::SongLink)
      @links
    end
  end

  class GigSong
    class_getter table_name = "gig_song"

    DB.mapping({
      event: Int32,
      song:  Int32,
      order: Int32,
    })
  end

  @[GraphQL::Object]
  class MediaType
    include GraphQL::ObjectType

    class_getter table_name = "media_type"

    @[GraphQL::Enum]
    enum StorageType
      LOCAL
      REMOTE

      def self.mapping
        {
          "LOCAL"  => LOCAL,
          "REMOTE" => REMOTE,
        }
      end

      def to_rs
        StorageType.mapping.invert[self].downcase
      end

      def self.from_rs(rs)
        val = rs.read
        storage_type = val.as?(String).try { |v| StorageType.mapping[v.upcase]? }
        storage_type || raise "Invalid storage type returned from database: #{val}"
      end

      def self.parse(val)
        StorageType.mapping[val]? || raise "Invalid storage type variant provided: #{val}"
      end
    end

    DB.mapping({
      name:    String,
      order:   Int32,
      storage: {type: StorageType, converter: StorageType},
    })

    def self.with_name(name)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE name = ?", name, as: MediaType
    end

    def self.with_name!(name)
      (with_name name) || raise "No media type named #{name}"
    end

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name} ORDER BY order ASC", as: MediaType
    end

    @[GraphQL::Field(description: "The name of the type of media")]
    def name : String
      @name
    end

    @[GraphQL::Field(description: "The order of where this media type appears in a song's link section")]
    def order : Int32
      @order
    end

    @[GraphQL::Field(description: "The type of storage that this type of media points to")]
    def storage : Models::MediaType::StorageType
      @storage
    end
  end

  @[GraphQL::Object]
  class SongLink
    include GraphQL::ObjectType

    PERFORMANCES = "Performances"

    class_getter table_name = "song_link"

    DB.mapping({
      id:     Int32,
      song:   Int32,
      type:   String,
      name:   String,
      target: String,
    })

    def self.with_id(id)
      CONN.query_one? "SELECT * FROM #{@@table_name} WHERE id = ?", id, as: SongLink
    end

    def self.with_id!(id)
      (with_id id) || raise "No song link with id #{id}"
    end

    def self.for_song(song_id)
      CONN.query_all "SELECT * FROM #{@@table_name} WHERE song = ?", song_id, as: SongLink
    end

    def self.all
      CONN.query_all "SELECT * FROM #{@@table_name}", as: SongLink
    end

    def self.create(song_id, form)
      encoded_target = if content = form.content
                         file = Utils::FileUpload.new Path[form.target], content
                         file.upload
                         URI.encode form.target
                       else
                         form.target
                       end

      CONN.exec "INSERT INTO #{@@table_name} (song, type, name, target) VALUES (?, ?, ?, ?)",
        song_id, form.type, form.name, encoded_target

      CONN.query_one "SELECT id FROM #{@@table_name} ORDER BY id DESC", as: Int32
    end

    def update(form)
      media_type = MediaType.with_name! @type
      new_target = if media_type.storage == MediaType::StorageType::LOCAL
                     URI.encode form.target
                   else
                     form.target
                   end

      CONN.exec "UPDATE #{@@table_name} SET name = ?, target = ? WHERE id = ?",
        form.name, new_target, @id

      if @target != new_target && media_type.storage == MediaType::StorageType::LOCAL
        old_path = Utils::MUSIC_BASE_PATH / @target
        new_path = Utils::MUSIC_BASE_PATH / new_target

        raise "Song link #{@name} has no associated file" unless File.exists? old_path
        File.rename old_path.to_s, new_path.to_s
      end

      @name, @target = form.name, new_target
    end

    def delete
      media_type = MediaType.with_name! @type

      CONN.exec "DELETE FROM #{@@table_name} WHERE id = ?", @id

      if media_type.storage = MediaType::StorageType::LOCAL
        file_name = Utils::MUSIC_BASE_PATH / @target
        File.delete file_name if File.exists? file_name
      end
    end

    @[GraphQL::Field(description: "The ID of the song link")]
    def id : Int32
      @id
    end

    @[GraphQL::Field(description: "The ID of the song this link belongs to")]
    def song : Int32
      @song
    end

    @[GraphQL::Field(description: "The type of this link (e.g. MIDI)")]
    def type : String
      @type
    end

    @[GraphQL::Field(description: "The name of this link")]
    def name : String
      @name
    end

    @[GraphQL::Field(description: "The target this link points to")]
    def target : String
      @target
    end
  end

  @[GraphQL::Object]
  class PublicSong
    include GraphQL::ObjectType

    def initialize(@title : String, @current : Bool, @videos : Array(PublicVideo))
    end

    @[GraphQL::Field]
    def title : String
      @title
    end

    @[GraphQL::Field]
    def current : Bool
      @current
    end

    @[GraphQL::Field]
    def videos : Array(Models::PublicVideo)
      @videos
    end
  end

  @[GraphQL::Object]
  class PublicVideo
    include GraphQL::ObjectType

    def initialize(@title : String, @url : String)
    end

    @[GraphQL::Field]
    def title : String
      @title
    end

    @[GraphQL::Field]
    def url : String
      @url
    end
  end
end
