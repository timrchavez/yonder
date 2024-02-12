#[macro_use]
extern crate rocket;

mod auth;
mod user;

use crate::auth::google::GoogleTokens;
use crate::auth::OAuth2Token;

use auth::Claims;
use rocket::http::{CookieJar, Status};
use rocket::request;
use rocket::{get, routes};

#[derive(Debug)]
struct User {
    pub id: String,
    pub state: String,
}

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for User {
    type Error = ();

    async fn from_request(request: &'r request::Request<'_>) -> request::Outcome<Self, ()> {
        let cookies = request
            .guard::<&CookieJar<'_>>()
            .await
            .expect("request cookies");
        if let (Some(auth_type), Some(serialized_tokens), Some(state)) = (
            cookies.get_private("auth-type"),
            cookies.get_private("user-tokens"),
            cookies.get_private("user-state"),
        ) {
            match state.value().to_string().as_str() {
                "logged_in" => match auth_type.value().to_string().as_str() {
                    "google" => {
                        let tokens: GoogleTokens =
                            serde_json::from_str(serialized_tokens.value()).unwrap();
                        return request::Outcome::Success(User {
                            id: tokens.claims().unwrap().email,
                            state: state.value().to_string(),
                        });
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        request::Outcome::Forward(Status::Unauthorized)
    }
}

#[get("/")]
fn index(user: User) -> String {
    format!("Hi, {}! Your state is: {}", user.id, user.state)
}

#[get("/", rank = 2)]
fn index_anonymous() -> &'static str {
    "Please login (/auth/google/login)"
}

#[get("/refresh")]
fn refresh(tokens: GoogleTokens) -> String {
    let claims: Claims = tokens.claims().unwrap();
    format!(
        "access_token={}, refresh_token={}, sub={}",
        tokens.access_token(),
        tokens.refresh_token(),
        claims.sub
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, index_anonymous, refresh])
        .attach(auth::google::fairing())
}
