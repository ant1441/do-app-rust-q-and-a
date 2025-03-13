#[macro_use]
extern crate rocket;

use std::{env, net::SocketAddr};

use rocket::{Request, fairing::AdHoc};
use rocket_db_pools::{Database, sqlx};
use rocket_prometheus::PrometheusMetrics;

mod db;
use db::Db;

#[derive(Debug)]

pub struct RequestSocketAddr {
    pub socket_addr: SocketAddr,
}

use rocket::request::{self, FromRequest};

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestSocketAddr {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let socket_addr = req.remote().unwrap();
        request::Outcome::Success(Self { socket_addr })
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

async fn run_migrations(rocket: rocket::Rocket<rocket::Build>) -> rocket::fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!().run(&**db).await {
            Ok(_) => Ok(rocket),
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
    let database_url = if cfg!(debug_assertions) {
        "".to_string()
    } else {
        env::var("DATABASE_URL").expect("Missing DATABASE_URL")
    };

    let figment = rocket::Config::figment().merge(("databases.q_and_a.url", database_url));

    let prometheus = PrometheusMetrics::new();

    rocket::custom(figment)
        .attach(prometheus.clone())
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .mount("/", routes![index])
        .mount("/metrics", prometheus)
}
