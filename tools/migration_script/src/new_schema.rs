use crate::error::{MigrateError, MigrateResult};
use crate::migrate::Insert;
use chrono::{NaiveDate, NaiveDateTime};
use mysql::{params, Pool};

macro_rules! impl_insert {
    ($table_name:expr, pub struct $type_name:ident { $(pub $field_names:ident: $field_types:ty,)* }) => {
        #[allow(dead_code)]
        #[derive(Debug)]
        pub struct $type_name {
            $(
                pub $field_names: $field_types,
            )*
        }

        impl Insert for $type_name {
            fn insert(new_db: &Pool, new_values: &Vec<Self>) -> MigrateResult<()> {
                let field_names_str = vec![ $( stringify!($field_names), )* ];
                let query = format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    $table_name,
                    field_names_str
                        .iter()
                        .map(|&field_name| if field_name == "type_" { "`type`".to_owned() } else { format!("`{}`", field_name) })
                        .collect::<Vec<String>>()
                        .join(", "),
                    field_names_str
                        .iter()
                        .map(|&field_name| format!(":{}", field_name))
                        .collect::<Vec<String>>()
                        .join(", "),
                );
                for mut stmt in new_db.prepare(&query).into_iter() {
                    for new_value in new_values {
                        stmt.execute(params!( $( stringify!($field_names) => &new_value.$field_names, )* ))
                            .map_err(MigrateError::MySqlError)?;
                    }
                }

                Ok(())
            }
        }
    };
}

// CREATE TABLE member (
//   email varchar(50) NOT NULL PRIMARY KEY,
//   first_name varchar(25) NOT NULL,
//   preferred_name varchar(25) DEFAULT NULL,
//   last_name varchar(25) NOT NULL,
//   pass_hash varchar(64) NOT NULL,
//   phone_number varchar(16) NOT NULL,
//   picture varchar(255) DEFAULT NULL,
//   passengers int NOT NULL DEFAULT '0',
//   location varchar(50) NOT NULL,
//   on_campus tinyint(1) DEFAULT NULL,
//   about varchar(500) DEFAULT NULL,
//   major varchar(50) DEFAULT NULL,
//   minor varchar(50) DEFAULT NULL,
//   hometown varchar(50) DEFAULT NULL,
//   arrived_at_tech int DEFAULT NULL, -- year
//   gateway_drug varchar(500) DEFAULT NULL,
//   conflicts varchar(500) DEFAULT NULL,
//   dietary_restrictions varchar(500) DEFAULT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "member",
    pub struct NewMember {
        pub email: String,
        pub first_name: String,
        pub preferred_name: Option<String>,
        pub last_name: String,
        pub pass_hash: String,
        pub phone_number: String,
        pub picture: Option<String>,
        pub passengers: i64,
        pub location: String,
        pub on_campus: Option<bool>,
        pub about: Option<String>,
        pub major: Option<String>,
        pub minor: Option<String>,
        pub hometown: Option<String>,
        pub arrived_at_tech: Option<i64>,
        pub gateway_drug: Option<String>,
        pub conflicts: Option<String>,
        pub dietary_restrictions: Option<String>,
    }
}

// CREATE TABLE semester (
//   name varchar(32) NOT NULL PRIMARY KEY,
//   start_date datetime NOT NULL,
//   end_date datetime NOT NULL,
//   gig_requirement int NOT NULL DEFAULT '5',
//   current boolean NOT NULL DEFAULT '0'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "semester",
    pub struct NewSemester {
        pub name: String,
        pub start_date: NaiveDateTime,
        pub end_date: NaiveDateTime,
        pub gig_requirement: i64,
        pub current: bool,
    }
}

// CREATE TABLE role (
//   name varchar(20) NOT NULL PRIMARY KEY,
//   `rank` int NOT NULL,
//   max_quantity int NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "role",
    pub struct NewRole {
        pub name: String,
        pub rank: i64,
        pub max_quantity: i64,
    }
}

// CREATE TABLE member_role (
//   member varchar(50) NOT NULL,
//   role varchar(20) NOT NULL,

//   PRIMARY KEY (member, role),
//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (role) REFERENCES role (name) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "member_role",
    pub struct NewMemberRole {
        pub member: String,
        pub role: String,
    }
}

// CREATE TABLE section_type (
//   name varchar(20) NOT NULL PRIMARY KEY
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "section_type",
    pub struct NewSectionType {
        pub name: String,
    }
}

// CREATE TABLE event_type (
//   name varchar(32) NOT NULL PRIMARY KEY,
//   weight int NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "event_type",
    pub struct NewEventType {
        pub name: String,
        pub weight: i64,
    }
}

// CREATE TABLE event (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   name varchar(64) NOT NULL,
//   semester varchar(32) NOT NULL,
//   `type` varchar(32) NOT NULL,
//   call_time datetime NOT NULL,
//   release_time datetime DEFAULT NULL,
//   points int NOT NULL,
//   comments text DEFAULT NULL,
//   location varchar(255) DEFAULT NULL,
//   gig_count boolean NOT NULL DEFAULT '1',
//   default_attend boolean NOT NULL DEFAULT '1',
//   section varchar(20) DEFAULT NULL,

//   FOREIGN KEY (semester) REFERENCES semester (name) ON UPDATE CASCADE ON DELETE CASCADE,
//   FOREIGN KEY (`type`) REFERENCES event_type (name) ON UPDATE CASCADE ON DELETE CASCADE,
//   FOREIGN KEY (section) REFERENCES section_type (name) ON UPDATE CASCADE ON DELETE SET NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "event",
    pub struct NewEvent {
        pub id: i64,
        pub name: String,
        pub semester: String,
        pub type_: String,
        pub call_time: NaiveDateTime,
        pub release_time: Option<NaiveDateTime>,
        pub points: i64,
        pub comments: Option<String>,
        pub location: Option<String>,
        pub gig_count: bool,
        pub default_attend: bool,
        pub section: Option<String>,
    }
}

// CREATE TABLE absence_request (
//   member varchar(50) NOT NULL,
//   event int NOT NULL,
//   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
//   reason varchar(500) NOT NULL,
//   state enum('pending', 'approved', 'denied') NOT NULL DEFAULT 'pending',

//   PRIMARY KEY (member, event),
//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "absence_request",
    pub struct NewAbsenceRequest {
        pub member: String,
        pub event: i64,
        pub time: NaiveDateTime,
        pub reason: String,
        pub state: String,
    }
}

// CREATE TABLE active_semester (
//   member varchar(50) NOT NULL,
//   semester varchar(32) NOT NULL,
//   enrollment enum('class', 'club') NOT NULL DEFAULT 'club',
//   section varchar(20) DEFAULT NULL,

//   PRIMARY KEY (member, semester),
//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (semester) REFERENCES semester (name) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (section) REFERENCES section_type (name) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "active_semester",
    pub struct NewActiveSemester {
        pub member: String,
        pub semester: String,
        pub enrollment: String,
        pub section: Option<String>,
    }
}

// CREATE TABLE announcement (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   member varchar(50) DEFAULT NULL,
//   semester varchar(32) NOT NULL,
//   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
//   content longtext NOT NULL,
//   archived bool NOT NULL DEFAULT '0',

//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE SET NULL ON UPDATE CASCADE,
//   FOREIGN KEY (semester) REFERENCES semester (name) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "announcement",
    pub struct NewAnnouncement {
        pub id: i64,
        pub member: Option<String>,
        pub semester: String,
        pub time: NaiveDateTime,
        pub content: String,
        pub archived: bool,
    }
}

// CREATE TABLE attendance (
//   member varchar(50) NOT NULL,
//   event int NOT NULL,
//   should_attend boolean NOT NULL DEFAULT '1',
//   did_attend boolean NOT NULL DEFAULT '0', -- TODO: null or not if an event hasn't passed
//   confirmed boolean NOT NULL DEFAULT '0',
//   minutes_late int NOT NULL DEFAULT '0',

//   PRIMARY KEY (member, event),
//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "attendance",
    pub struct NewAttendance {
        pub member: String,
        pub event: i64,
        pub should_attend: bool,
        pub did_attend: bool,
        pub confirmed: bool,
        pub minutes_late: i64,
    }
}

// CREATE TABLE carpool (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   event int NOT NULL,
//   driver varchar(50) NOT NULL,

//   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (driver) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "carpool",
    pub struct NewCarpool {
        pub id: i64,
        pub event: i64,
        pub driver: String,
    }
}

// CREATE TABLE fee (
//   name varchar(16) NOT NULL PRIMARY KEY,
//   description varchar(40) NOT NULL PRIMARY KEY,
//   amount int NOT NULL DEFAULT '0'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "fee",
    pub struct NewFee {
        pub name: String,
        pub description: String,
        pub amount: i64,
    }
}

// CREATE TABLE google_docs (
//   name varchar(40) NOT NULL PRIMARY KEY,
//   url varchar(255) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "google_docs",
    pub struct NewGoogleDocs {
        pub name: String,
        pub url: String,
    }
}

// CREATE TABLE uniform (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   name varchar(32) NOT NULL PRIMARY KEY,
//   color varchar(4) DEFAULT NULL,
//   description text DEFAULT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "uniform",
    pub struct NewUniform {
        pub id: i64,
        pub name: String,
        pub color: Option<String>,
        pub description: Option<String>,
    }
}

// CREATE TABLE gig (
//   event int NOT NULL PRIMARY KEY,
//   performance_time datetime NOT NULL,
//   uniform int NOT NULL,
//   contact_name varchar(50) DEFAULT NULL,
//   contact_email varchar(50) DEFAULT NULL,
//   contact_phone varchar(16) DEFAULT NULL,
//   price int DEFAULT NULL,
//   public boolean NOT NULL DEFAULT '0',
//   summary text DEFAULT NULL,
//   description text DEFAULT NULL,

//   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (uniform) REFERENCES uniform (id) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "gig",
    pub struct NewGig {
        pub event: i64,
        pub performance_time: NaiveDateTime,
        pub uniform: i64,
        pub contact_name: Option<String>,
        pub contact_email: Option<String>,
        pub contact_phone: Option<String>,
        pub price: Option<i64>,
        pub public: bool,
        pub summary: Option<String>,
        pub description: Option<String>,
    }
}

// CREATE TABLE gig_request (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
//   name varchar(255) NOT NULL,
//   organization varchar(255) NOT NULL,
//   event int DEFAULT NULL,
//   contact_name varchar(255) NOT NULL,
//   contact_phone varchar(16) NOT NULL,
//   contact_email varchar(50) NOT NULL,
//   start_time datetime NOT NULL,
//   location varchar(255) NOT NULL,
//   comments text DEFAULT NULL,
//   status enum('pending', 'accepted', 'dismissed') NOT NULL DEFAULT 'pending',

//   FOREIGN KEY (event) REFERENCES event (id) ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "gig_request",
    pub struct NewGigRequest {
        pub id: i64,
        pub time: NaiveDateTime,
        pub name: String,
        pub organization: String,
        pub event: Option<i64>,
        pub contact_name: String,
        pub contact_email: String,
        pub contact_phone: String,
        pub start_time: NaiveDateTime,
        pub location: String,
        pub comments: Option<String>,
        pub status: String,
    }
}

// CREATE TABLE song (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   title varchar(128) NOT NULL,
//   info text DEFAULT NULL,
//   current boolean NOT NULL DEFAULT '0',
//   `key` enum('A♭', 'A', 'A#', 'B♭', 'B', 'B#', 'C♭', 'C', 'C♯', 'D♭', 'D', 'D♯', 'E♭',
//              'E', 'E#', 'F♭', 'F', 'F♯', 'G♭', 'G', 'G#') DEFAULT NULL,
//   starting_pitch enum('A♭', 'A', 'A#', 'B♭', 'B', 'B#', 'C♭', 'C', 'C♯', 'D♭', 'D', 'D♯',
//                       'E♭', 'E', 'E#', 'F♭', 'F', 'F♯', 'G♭', 'G', 'G#') DEFAULT NULL,
//   mode enum('major', 'minor') DEFAULT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "song",
    pub struct NewSong {
        pub id: i64,
        pub title: String,
        pub info: Option<String>,
        pub current: bool,
        pub key: Option<String>,
        pub starting_pitch: Option<String>,
        pub mode: Option<String>,
    }
}

// CREATE TABLE gig_song (
//   event int NOT NULL,
//   song int NOT NULL,
//   `order` int NOT NULL,

//   PRIMARY KEY (event, song),
//   FOREIGN KEY (event) REFERENCES event (id) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (song) REFERENCES song (id) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "gig_song",
    pub struct NewGigSong {
        pub event: i64,
        pub song: i64,
        pub order: i64,
    }
}

// CREATE TABLE media_type (
//   name varchar(50) NOT NULL PRIMARY KEY,
//   `order` int NOT NULL UNIQUE,
//   storage enum('local', 'remote') NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "media_type",
    pub struct NewMediaType {
        pub name: String,
        pub order: i64,
        pub storage: String,
    }
}

// CREATE TABLE minutes (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   name varchar(100) NOT NULL,
//   `date` date NOT NULL,
//   private longtext DEFAULT NULL,
//   public longtext DEFAULT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "minutes",
    pub struct NewMinutes {
        pub id: i64,
        pub name: String,
        pub date: NaiveDate,
        pub private: Option<String>,
        pub public: Option<String>,
    }
}

// CREATE TABLE permission (
//   name varchar(40) NOT NULL PRIMARY KEY,
//   description text DEFAULT NULL,
//   `type` enum('static', 'event') NOT NULL DEFAULT 'static'
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "permission",
    pub struct NewPermission {
        pub name: String,
        pub description: Option<String>,
        pub type_: String,
    }
}

// CREATE TABLE rides_in (
//   member varchar(50) NOT NULL,
//   carpool int NOT NULL,

//   PRIMARY KEY (member, carpool),
//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (carpool) REFERENCES carpool (id) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "rides_in",
    pub struct NewRidesIn {
        pub member: String,
        pub carpool: i64,
    }
}

// CREATE TABLE role_permission (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   role varchar(20) NOT NULL,
//   permission varchar(40) NOT NULL,
//   event_type varchar(32) DEFAULT NULL,

//   FOREIGN KEY (role) REFERENCES role (name) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (permission) REFERENCES permission (name) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (event_type) REFERENCES event_type (name) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "role_permission",
    pub struct NewRolePermission {
        pub id: i64,
        pub role: String,
        pub permission: String,
        pub event_type: Option<String>,
    }
}

// CREATE TABLE song_link (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   song int NOT NULL,
//   `type` varchar(50) NOT NULL,
//   name varchar(128) NOT NULL,
//   target varchar(255) NOT NULL,

//   FOREIGN KEY (`type`) REFERENCES media_type (name) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (song) REFERENCES song (id) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "song_link",
    pub struct NewSongLink {
        pub id: i64,
        pub song: i64,
        pub type_: String,
        pub name: String,
        pub target: String,
    }
}

// CREATE TABLE todo (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   `text` varchar(255) NOT NULL,
//   member varchar(50) NOT NULL,
//   completed boolean NOT NULL DEFAULT '0',

//   FOREIGN KEY (member) REFERENCES member (email) ON UPDATE CASCADE ON DELETE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "todo",
    pub struct NewTodo {
        pub id: i64,
        pub text: String,
        pub member: String,
        pub completed: bool,
    }
}

// CREATE TABLE transaction_type (
//   name varchar(40) NOT NULL PRIMARY KEY
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "transaction_type",
    pub struct NewTransactionType {
        pub name: String,
    }
}

// CREATE TABLE transaction (
//   id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
//   member varchar(50) NOT NULL,
//   `time` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
//   amount int NOT NULL,
//   description varchar(500) NOT NULL,
//   semester varchar(32) DEFAULT NULL,
//   `type` varchar(40) NOT NULL,
//   resolved tinyint(1) NOT NULL DEFAULT '0',

//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (`type`) REFERENCES transaction_type (name) ON DELETE CASCADE ON UPDATE CASCADE,
//   FOREIGN KEY (semester) REFERENCES semester (name) ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "transaction",
    pub struct NewTransaction {
        pub id: i64,
        pub member: String,
        pub time: NaiveDateTime,
        pub amount: i64,
        pub description: String,
        pub semester: Option<String>,
        pub type_: String,
        pub resolved: bool,
    }
}

// CREATE TABLE session (
//   member varchar(50) NOT NULL PRIMARY KEY,
//   `key` varchar(64) NOT NULL,

//   FOREIGN KEY (member) REFERENCES member (email) ON DELETE CASCADE ON UPDATE CASCADE
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "session",
    pub struct NewSession {
        pub member: String,
        pub key: String,
    }
}

// CREATE TABLE variable (
//   `key` varchar(255) NOT NULL PRIMARY KEY,
//   value varchar(255) NOT NULL
// ) ENGINE=InnoDB DEFAULT CHARSET=utf8;
impl_insert! {
    "variable",
    pub struct NewVariable {
        pub key: String,
        pub value: String,
    }
}
