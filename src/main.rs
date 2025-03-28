#[macro_use]
extern crate rocket;

use std::time::Duration;
use std::{env, net::SocketAddr};

use auth::GitHub;
use models::{User, get_topic};
use rocket::fs::FileServer;
use rocket::http::CookieJar;
use rocket::request::{self, FromRequest};
use rocket::{Request, fairing::AdHoc};
use rocket_db_pools::{Connection, Database, sqlx};
use rocket_dyn_templates::{Template, context};
use rocket_oauth2::OAuth2;
use rocket_prometheus::PrometheusMetrics;

mod api;
mod auth;
mod db;
mod models;
mod types;

use db::Db;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug)]
pub struct RequestSocketAddr {
    pub socket_addr: SocketAddr,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestSocketAddr {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let socket_addr = req.remote().unwrap();
        request::Outcome::Success(Self { socket_addr })
    }
}

#[get("/")]
async fn index(cookies: &CookieJar<'_>, db: Connection<Db>) -> Template {
    let user = if let Some(cookie) = cookies.get_private("user") {
        let cookie_value = cookie.value();
        match serde_json::from_str::<User>(cookie_value) {
            Ok(user) => Some(user),
            Err(e) => {
                warn!("Invalid 'user' cookie found: {e}");
                cookies.remove_private("user");
                None
            }
        }
    } else {
        warn!("No 'user' cookie found");
        None
    };

    let is_admin = user.as_ref().map_or(false, User::is_admin);

    let is_auth = user.is_some();
    let topic = match get_topic(db).await {
        Ok(topic) => topic,
        Err(e) => {
            warn!("Failed to load topic: {e}");
            "Failed to load topic!".to_string().into()
        }
    };
    Template::render(
        "index",
        context! {is_auth: is_auth, is_admin: is_admin, user: user, topic: topic},
    )
}

async fn run_migrations(rocket: rocket::Rocket<rocket::Build>) -> rocket::fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!().run(&**db).await {
            Ok(_) => {
                info!("DB Migrations ran!");
                Ok(rocket)
            }
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

#[launch]
fn rocket() -> _ {
    let url = env::var("PUBLIC_URL").unwrap_or("http://localhost:8000/".to_string());
    let database_url = env::var("DATABASE_URL").expect("Missing DATABASE_URL");
    let static_path = env::var("STATIC_PATH").unwrap_or("static".to_string());

    let figment = rocket::Config::figment()
        .merge(("databases.q_and_a.url", database_url))
        .merge(("oauth.github.provider", "github"))
        .merge((
            "oauth.github.client_id",
            env::var("GITHUB_OAUTH_CLIENT_ID").expect("Expected GitHub Client ID"),
        ))
        .merge((
            "oauth.github.client_secret",
            env::var("GITHUB_OAUTH_CLIENT_SECRET").expect("Expected GitHub Client Secret"),
        ))
        .merge(("oauth.github.redirect_uri", format!("{url}auth/github")));

    let client = reqwest::ClientBuilder::new()
        .user_agent(APP_USER_AGENT)
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build reqwest client");
    let prometheus = PrometheusMetrics::new();

    rocket::custom(figment)
        .manage(client)
        .attach(prometheus.clone())
        .attach(Template::fairing())
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .attach(OAuth2::<GitHub>::fairing("github"))
        .mount("/", routes![index])
        .mount(
            "/auth",
            routes![
                auth::login,
                auth::logout,
                auth::github_login,
                auth::github_callback
            ],
        )
        .mount("/api", routes![api::set_topic])
        .mount("/static/", FileServer::from(static_path))
        .mount("/metrics", prometheus)
}
