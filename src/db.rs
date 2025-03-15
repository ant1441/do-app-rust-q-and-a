use rocket_db_pools::{Database, sqlx};

#[derive(Database)]
#[database("q_and_a")]
pub(crate) struct Db(sqlx::PgPool);
