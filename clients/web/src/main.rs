#[macro_use]
extern crate rocket;

mod api;
mod auth;
mod user;

use crate::auth::google::GoogleTokens;
use crate::auth::OAuth2Token;
use crate::auth::Refreshable;

use auth::AuthenticationError;
use auth::Token;
use rocket::get;
use rocket::http::Cookie;
use rocket::http::CookieJar;
use rocket::http::SameSite;
use rocket::http::Status;
use rocket::request;
use rocket::routes;
use rocket_oauth2::OAuth2;
use user::User;

#[derive(Debug)]
struct Account {
    pub id: String,
    pub state: String,
    pub user: User,
}

async fn refresh_google_tokens(
    serialized_tokens: &str,
    oauth2: OAuth2<Token>,
) -> Result<GoogleTokens, AuthenticationError> {
    let mut tokens: GoogleTokens = serde_json::from_str(serialized_tokens).unwrap();
    // Checking to see if the access token is still valid
    match tokens.claims() {
        Ok(_) => {}
        Err(e) => match e {
            auth::AuthenticationError::Expired => {
                println!("refresh");
                tokens.refresh_access_token(oauth2).await;
            }
            auth::AuthenticationError::Decoding(_) => {
                return Err(e);
            }
        },
    }
    return Ok(tokens);
}

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for Account {
    type Error = ();

    async fn from_request(request: &'r request::Request<'_>) -> request::Outcome<Self, ()> {
        let cookies = request
            .guard::<&CookieJar<'_>>()
            .await
            .expect("request cookies");
        let oauth2 = request
            .guard::<OAuth2<Token>>()
            .await
            .expect("oauth2 client");
        if let (Some(auth_type), Some(serialized_tokens), Some(state), Some(user_account)) = (
            cookies.get_private("auth-type"),
            cookies.get_private("user-tokens"),
            cookies.get_private("user-state"),
            cookies.get_private("user-account"),
        ) {
            let user: User = serde_json::from_str(user_account.value()).unwrap();
            match state.value().to_string().as_str() {
                "logged_in" => match auth_type.value().to_string().as_str() {
                    "google" => {
                        // If this succeeds, the access token should be guaranteed to be valid
                        match refresh_google_tokens(serialized_tokens.value(), oauth2).await {
                            Ok(tokens) => {
                                cookies.add_private(
                                    Cookie::build((
                                        "user-tokens",
                                        serde_json::to_string(&tokens).unwrap(),
                                    ))
                                    .same_site(SameSite::Lax)
                                    .build(),
                                );
                                return request::Outcome::Success(Account {
                                    id: tokens.claims().unwrap().email,
                                    state: state.value().to_string(),
                                    user,
                                });
                            }
                            Err(_) => {
                                cookies.remove_private("auth-type");
                                cookies.remove_private("user-tokens");
                                cookies.remove_private("user-state");
                            }
                        }
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
fn index(account: Account) -> String {
    format!(
        "Hi, {} {} ({})! Your state is: {}, Email is: {}",
        account.user.given_name,
        account.user.family_name,
        account.user.id,
        account.state,
        account.id
    )
}

#[get("/", rank = 2)]
fn index_anonymous() -> &'static str {
    "Please login (/auth/google/login)"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, index_anonymous])
        .attach(auth::google::fairing())
}
