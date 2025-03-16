use rocket::{
    http::{Cookie, SameSite},
    time::{Duration, OffsetDateTime},
};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, query};

use crate::Db;

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum AuthType {
    GitHub,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub auth_type: AuthType,
    pub user_id: i64,
    pub name: String,
    pub avatar_url: String,
    pub gravatar_id: String,
}

impl<'c> From<User> for Cookie<'c> {
    fn from(user: User) -> Self {
        let value = serde_json::to_string(&user).unwrap();
        Cookie::build(("user", value))
            .expires(OffsetDateTime::now_utc().saturating_add(4 * Duration::HOUR))
            .http_only(true)
            .same_site(SameSite::Lax)
            .build()
    }
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
