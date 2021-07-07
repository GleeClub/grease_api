use async_graphql::{Context, Guard, Result};
use models::member::Member;
use sqlx::MySqlConnection;

mod input;
mod models;
mod mutation;
mod permission;
mod query;

pub struct LoggedIn;

#[async_trait::async_trait]
impl Guard for LoggedIn {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if ctx.data_opt::<Member>().is_some() {
            Ok(())
        } else {
            Err("User must be logged in".into())
        }
    }
}

pub fn get_conn(ctx: Context<'_>) -> Result<MysqlConnection> {
    ctx.data_unchecked::<MysqlConnection>()
}
