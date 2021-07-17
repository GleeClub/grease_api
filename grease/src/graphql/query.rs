use crate::graphql::LoggedIn;
use sqlx::MySqlPool;
use async_graphql::*;

pub struct Query;

#[Object]
impl Query {
    pub async fn user(&self, ctx: Context<'_>) -> Option<Member> {
        ctx.data_opt::<Member>()
    }

    #[async_graphql(guard(LoggedIn))]
    pub async fn member(&self, ctx: &Context<'_>, email: String) -> Result<Member> {
        Member::with_email(&email, ctx.data_unchecked::<MySqlPool>()).await.into()
    }

    #[async_graphql(guard(LoggedIn))]
    pub async fn members(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = true)] include_class: bool,
        #[graphql(default = true)] include_club: bool,
        #[graphql(default = false)] include_inactive: bool,
    ) -> Result<Vec<Member>> {
        let members = Member::load_all(ctx.data_unchecked::<MySqlPool>()).await?;

        Ok(members
            .into_iter()
            .filter(|member| {
                let enrollment = member
                    .get_semester(Semester::current().await?.name)
                    .await?
                    .enrollment;

                (include_class && enrollment == Some(Enrollment::Class))
                    || (include_club && enrollment == Some(Enrollment::Club))
                    || (include_inactive && enrollment.is_none())
            })
            .collect())
    }
}

//   @[GraphQL::Field]
//   def event(id : Int32, context : UserContext) : Models::Event
//     context.logged_in!

//     Models::Event.with_id! id
//   end

//   @[GraphQL::Field]
//   def events(context : UserContext) : Array(Models::Event)
//     context.logged_in!

//     Models::Event.for_semester Models::Semester.current.name
//   end

//   @[GraphQL::Field]
//   def public_events : Array(Models::PublicEvent)
//     Models::PublicEvent.all_for_current_semester
//   end

//   @[GraphQL::Field]
//   def absence_requests(context : UserContext) : Array(Models::AbsenceRequest)
//     context.able_to! Permissions::PROCESS_ABSENCE_REQUESTS

//     Models::AbsenceRequest.for_semester Models::Semester.current.name
//   end

//   @[GraphQL::Field]
//   def gig_request(id : Int32, context : UserContext) : Models::GigRequest
//     context.able_to! Permissions::PROCESS_GIG_REQUESTS

//     Models::GigRequest.with_id! id
//   end

//   @[GraphQL::Field]
//   def gig_requests(context : UserContext) : Array(Models::GigRequest)
//     context.able_to! Permissions::PROCESS_GIG_REQUESTS

//     Models::GigRequest.all
//   end

//   @[GraphQL::Field]
//   def variable(key : String, context : UserContext) : Models::Variable
//     context.logged_in!

//     Models::Variable.with_key! key
//   end

//   @[GraphQL::Field]
//   def meeting_minutes(id : Int32, context : UserContext) : Models::Minutes
//     context.logged_in!

//     Models::Minutes.with_id! id
//   end

//   @[GraphQL::Field]
//   def all_meeting_minutes(context : UserContext) : Array(Models::Minutes)
//     context.logged_in!

//     Models::Minutes.all
//   end

//   @[GraphQL::Field]
//   def current_semester(context : UserContext) : Models::Semester
//     context.logged_in!

//     Models::Semester.current
//   end

//   @[GraphQL::Field]
//   def semester(name : String, context : UserContext) : Models::Semester
//     context.logged_in!

//     Models::Semester.with_name! name
//   end

//   @[GraphQL::Field]
//   def semesters(context : UserContext) : Array(Models::Semester)
//     context.logged_in!

//     Models::Semester.all
//   end

//   @[GraphQL::Field]
//   def uniform(id : Int32, context : UserContext) : Models::Uniform
//     context.logged_in!

//     Models::Uniform.with_id! id
//   end

//   @[GraphQL::Field]
//   def uniforms(context : UserContext) : Array(Models::Uniform)
//     context.logged_in!

//     Models::Uniform.all
//   end

//   @[GraphQL::Field]
//   def documents(context : UserContext) : Array(Models::Document)
//     context.logged_in!

//     Models::Document.all
//   end

//   @[GraphQL::Field]
//   def song(id : Int32, context : UserContext) : Models::Song
//     context.logged_in!

//     Models::Song.with_id! id
//   end

//   @[GraphQL::Field]
//   def songs(context : UserContext) : Array(Models::Song)
//     context.logged_in!

//     Models::Song.all
//   end

//   @[GraphQL::Field]
//   def song_link(id : Int32, context : UserContext) : Models::SongLink
//     context.logged_in!

//     Models::SongLink.with_id! id
//   end

//   @[GraphQL::Field]
//   def public_songs : Array(Models::PublicSong)
//     Models::Song.all_public
//   end

//   @[GraphQL::Field]
//   def static(context : UserContext) : StaticData
//     context.logged_in!

//     StaticData.new
//   end

//   @[GraphQL::Field]
//   def transactions(context : UserContext) : Array(Models::ClubTransaction)
//     context.able_to! Permissions::VIEW_TRANSACTIONS

//     Models::ClubTransaction.for_semester Models::Semester.current.name
//   end

//   @[GraphQL::Field]
//   def fees(context : UserContext) : Array(Models::Fee)
//     context.able_to! Permissions::VIEW_TRANSACTIONS

//     Models::Fee.all
//   end

//   @[GraphQL::Field]
//   def officers(context : UserContext) : Array(Models::MemberRole)
//     context.able_to! Permissions::EDIT_OFFICERS

//     Models::MemberRole.current_officers
//   end

//   @[GraphQL::Field]
//   def current_permissions(context : UserContext) : Array(Models::RolePermission)
//     context.able_to! Permissions::EDIT_OFFICERS

//     Models::RolePermission.all
//   end
// end

// @[GraphQL::Object]
// class StaticData
//   include GraphQL::ObjectType

//   @[GraphQL::Field]
//   def media_types : Array(Models::MediaType)
//     Models::MediaType.all
//   end

//   @[GraphQL::Field]
//   def permissions : Array(Models::Permission)
//     Models::Permission.all
//   end

//   @[GraphQL::Field]
//   def roles : Array(Models::Role)
//     Models::Role.all
//   end

//   @[GraphQL::Field]
//   def event_types : Array(Models::EventType)
//     Models::EventType.all
//   end

//   @[GraphQL::Field]
//   def sections : Array(String)
//     Models::SectionType.all.map &.name
//   end

//   @[GraphQL::Field]
//   def transaction_types : Array(String)
//     Models::TransactionType.all.map &.name
//   end