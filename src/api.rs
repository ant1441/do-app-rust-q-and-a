use rocket::{http::Status, serde::json::Json};
use rocket_db_pools::Connection;
use serde::Deserialize;

use crate::{
    db::Db,
    models::{self, Admin},
};

#[derive(Deserialize)]
pub struct SetTopic<'r> {
    topic: &'r str,
}

#[post("/set_topic", format = "json", data = "<topic_json>")]
pub async fn set_topic(
    db: Connection<Db>,
    admin_user: Admin,
    topic_json: Json<SetTopic<'_>>,
) -> Status {
    let topic = topic_json.topic;
    models::set_topic(db, admin_user, topic).await.unwrap();

    Status::Accepted
}
