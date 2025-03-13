use rocket_db_pools::{Database, sqlx};

#[cfg(debug_assertions)]
#[derive(Database)]
#[database("q_and_a")]
pub(crate) struct Db(sqlx::SqlitePool);

#[cfg(not(debug_assertions))]
#[derive(Database)]
#[database("q_and_a")]
pub(crate) struct Db(sqlx::PgPool);
