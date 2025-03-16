use rocket_db_pools::{Database, sqlx};

#[derive(Database)]
#[database("q_and_a")]
pub struct Db(sqlx::PgPool);
