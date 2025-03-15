use reqwest::{StatusCode, header};
use rocket::{
    State,
    http::{Cookie, CookieJar, SameSite, Status},
    response::Redirect,
};
use rocket_db_pools::Connection;
use rocket_oauth2::{OAuth2, TokenResponse};

use crate::{
    Db,
    models::{AuthType, User},
    types::GitHubUser,
};

pub(crate) struct GitHub;

#[get("/login/github")]
pub(crate) fn github_login(oauth2: OAuth2<GitHub>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2.get_redirect(cookies, &["user:read"]).unwrap()
}

#[get("/auth/github")]
pub(crate) async fn github_callback(
    token: TokenResponse<GitHub>,
    cookies: &CookieJar<'_>,
    client: &State<reqwest::Client>,
    db: Connection<Db>,
) -> Result<Redirect, Status> {
    let token = token.access_token().to_string();
    cookies.add_private(
        Cookie::build(("token", token.clone()))
            .same_site(SameSite::Strict)
            .build(),
    );

    let resp = client
        // https://docs.github.com/en/rest/users/users?apiVersion=2022-11-28#get-the-authenticated-user
        .get("https://api.github.com/user")
        .bearer_auth(&token)
        .header(header::ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|_e| Status::InternalServerError)?;

    let resp_status = resp.status();
    if resp_status != StatusCode::OK {
        warn!("Unexpected response from GitHub API (/user): {resp_status}");
        return Ok(Redirect::to("/?failed_login"));
    }
    info!("Response from GitHub API {resp_status}");

    let github_user: GitHubUser = resp.json().await.unwrap();

    // Add user to DB
    let user = User {
        auth_type: AuthType::GitHub,
        user_id: github_user.id,
        name: github_user.name.unwrap_or("<GitHub User>".to_string()),
        avatar_url: github_user.avatar_url,
        gravatar_id: github_user.gravatar_id,
    };
    user.upsert(db).await;

    Ok(Redirect::to("/"))
}
