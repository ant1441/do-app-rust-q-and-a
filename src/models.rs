use core::fmt;

use rocket::{
    http::{Cookie, SameSite, Status},
    request::{FromRequest, Outcome, Request},
    time::{Duration, OffsetDateTime},
};
use rocket_db_pools::Connection;
use rocket_db_pools::sqlx::Row;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, postgres::PgQueryResult, query};

use crate::Db;

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthType {
    GitHub,
}

impl fmt::Display for AuthType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthType::GitHub => f.write_str("GitHub"),
        }
    }
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = request.cookies();

        if let Some(cookie) = cookies.get_private("user") {
            let cookie_value = cookie.value();

            if let Ok(user) = serde_json::from_str::<User>(cookie_value) {
                debug!("User::from_request parsed 'user' cookie {user:#?}");

                return Outcome::Success(user);
            }
        }

        Outcome::Error((Status::Unauthorized, "User Unauthorized"))
    }
}

impl User {
    pub async fn get_id(&self, mut db: &mut Connection<Db>) -> sqlx::Result<i64> {
        let c = db.acquire().await?;
        let result = query!(
            r"SELECT id FROM users
            WHERE
            auth_type = $1 AND user_id = $2;
            ",
            match self.auth_type {
                AuthType::GitHub => "GitHub", // Convert enum to string
            },
            self.user_id,
        )
        .fetch_one(&mut *c)
        .await?;

        Ok(result.id as _)
    }

    pub async fn upsert(&self, mut db: Connection<Db>) -> sqlx::Result<PgQueryResult> {
        query!(
            r"
            INSERT INTO users (auth_type, user_id, name, avatar_url, gravatar_id)
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

    pub fn is_admin(&self) -> bool {
        if self.auth_type == AuthType::GitHub && self.user_id == 3115867 {
            true
        } else {
            false
        }
    }
}

pub struct Admin(User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = &'static str;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        trace!("Admin::from_request");
        let user_outcome = User::from_request(request).await;
        user_outcome.and_then(|user| {
            trace!("User::from_request parsed 'user' {user:#?}");
            if user.is_admin() {
                Outcome::Success(Admin(user))
            } else {
                warn!(
                    "Admin::from_request user '{}' ({}) is not admin",
                    user.name, user.auth_type
                );
                Outcome::Error((Status::Unauthorized, "Admin access required"))
            }
        })
    }
}

pub type Topic = String;

pub async fn get_topic(mut db: Connection<Db>) -> sqlx::Result<Option<Topic>> {
    let result = sqlx::query(r"SELECT title FROM topics WHERE cleared_at IS NULL;")
        .fetch_optional(&mut **db)
        .await?;

    if let Some(row) = result {
        let topic = row.get(0);
        Ok(Some(topic))
    } else {
        Ok(None)
    }
}

pub async fn set_topic(
    mut db: Connection<Db>,
    admin_user: Admin,
    topic: &str,
) -> sqlx::Result<PgQueryResult> {
    query!(r"UPDATE topics SET cleared_at = NOW() WHERE cleared_at IS NULL;")
        .execute(&mut **db)
        .await?;

    let id = admin_user.0.get_id(&mut db).await?;

    query!(
        r"
        INSERT INTO topics (title, created_by)
        VALUES ($1, $2)
        ON CONFLICT (id) DO UPDATE
        SET
            title = EXCLUDED.title,
            created_at = NOW();
        ",
        topic,
        id,
    )
    .execute(&mut **db)
    .await
}
