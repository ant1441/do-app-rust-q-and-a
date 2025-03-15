use rocket_db_pools::Connection;
use sqlx::{postgres::PgQueryResult, query};

use crate::Db;

#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum AuthType {
    GitHub,
}

#[derive(Clone, Debug)]
pub struct User {
    pub auth_type: AuthType,
    pub user_id: i64,
    pub name: String,
    pub avatar_url: String,
    pub gravatar_id: String,
}

impl User {
    pub async fn upsert(&self, mut db: Connection<Db>) -> sqlx::Result<PgQueryResult> {
        query!(
            r"INSERT INTO users (auth_type, user_id, name, avatar_url, gravatar_id)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (auth_type, user_id) DO UPDATE
SET
    name = EXCLUDED.name,
    avatar_url = EXCLUDED.avatar_url,
    gravatar_id = EXCLUDED.gravatar_id;
",
            match self.auth_type {
                AuthType::GitHub => "GitHub", // Convert enum to string
            },
            self.user_id,
            self.name,
            self.avatar_url,
            self.gravatar_id
        )
        .execute(&mut **db)
        .await
    }
}
