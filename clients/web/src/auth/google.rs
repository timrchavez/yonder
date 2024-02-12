use crate::auth::Token;

use anyhow::Error;
use envconfig::Envconfig;
use rocket::fairing::{AdHoc, Fairing};
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::request;
use rocket::response::{Debug, Redirect};
use rocket_oauth2::{OAuth2, TokenResponse};
use serde::{Deserialize, Serialize};

use super::{claims_from_response, create_jwt, Claims, Config, OAuth2Token, Refreshable, Tokens};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GoogleTokens {
    access_token: String,
    refresh_token: String,
}

impl OAuth2Token for GoogleTokens {
    fn access_token(&self) -> &str {
        return self.access_token.as_str();
    }
    fn refresh_token(&self) -> &str {
        return self.refresh_token.as_str();
    }
}

#[async_trait]
impl Refreshable for GoogleTokens {
    async fn refresh_access_token(&mut self, oauth2: OAuth2<Token>) {
        let config = Config::init_from_env().unwrap();
        let response: TokenResponse<Token> =
            oauth2.refresh(self.refresh_token.as_str()).await.unwrap();

        let mut new_refresh_token = String::from(self.refresh_token.as_str()).clone();
        if response.refresh_token().is_some() {
            new_refresh_token = response.refresh_token().unwrap().to_string();
        }

        let claims: Claims = claims_from_response(&response).await.unwrap();

        self.access_token =
            create_jwt(claims.sub, claims.email, claims.exp, config.key.as_str()).unwrap();
        self.refresh_token = String::from(new_refresh_token);
    }
}

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for GoogleTokens {
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
        if let (Some(auth_type), Some(serialized_tokens)) = (
            cookies.get_private("auth-type"),
            cookies.get_private("user-tokens"),
        ) {
            if auth_type.value() == "google" {
                let tokens_result = request
                    .local_cache_async(async {
                        let mut tokens: GoogleTokens =
                            serde_json::from_str(serialized_tokens.value()).unwrap();
                        tokens.refresh_access_token(oauth2).await;
                        cookies.add_private(
                            Cookie::build(("user-tokens", serde_json::to_string(&tokens).unwrap()))
                                .same_site(SameSite::Lax)
                                .build(),
                        );
                        tokens
                    })
                    .await;
                if tokens_result.access_token_valid() {
                    return request::Outcome::Success(tokens_result.clone());
                }
            }
        }
        request::Outcome::Forward(Status::Unauthorized)
    }
}

#[get("/auth/google/login")]
pub fn google_login(oauth2: OAuth2<Token>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2.get_redirect(cookies, &["email", "profile"]).unwrap()
}

#[get("/auth/google/callback")]
pub async fn google_callback(
    response: TokenResponse<Token>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Debug<Error>> {
    cookies.add_private(
        Cookie::build(("user-state", "logged_out"))
            .same_site(SameSite::Lax)
            .build(),
    );
    let config: Config = Config::init_from_env().unwrap();
    let claims: Claims = claims_from_response(&response).await.unwrap();
    let tokens: Tokens = Tokens {
        access_token: create_jwt(claims.sub, claims.email, claims.exp, config.key.as_str())
            .unwrap(),
        refresh_token: response.refresh_token().unwrap().to_string(),
    };
    cookies.add_private(
        Cookie::build(("auth-type", "google"))
            .same_site(SameSite::Lax)
            .build(),
    );
    cookies.add_private(
        Cookie::build(("user-tokens", serde_json::to_string(&tokens).unwrap()))
            .same_site(SameSite::Lax)
            .build(),
    );
    cookies.add_private(
        Cookie::build(("user-state", "logged_in"))
            .same_site(SameSite::Lax)
            .build(),
    );
    Ok(Redirect::to("/"))
}

pub fn fairing() -> impl Fairing {
    AdHoc::on_ignite("Google OAuth2", |rocket| async {
        rocket
            .mount("/", rocket::routes![google_login, google_callback])
            .attach(OAuth2::<Token>::fairing("google"))
    })
}
