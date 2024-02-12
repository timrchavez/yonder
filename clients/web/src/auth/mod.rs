use std::str::FromStr;

use chrono::{Duration, Utc};
use envconfig::Envconfig;
use jsonwebtoken::{
    decode, decode_header, encode,
    errors::{Error, ErrorKind},
    jwk::{AlgorithmParameters, JwkSet},
    Algorithm, DecodingKey, EncodingKey, Header, Validation,
};

use rocket_oauth2::{OAuth2, TokenResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod google;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "YONDER_JWT_SECRET")]
    key: String,
    #[envconfig(from = "GOOGLE_OAUTH_AUDIENCE")]
    google_oauth_audience: String,
    #[envconfig(from = "GOOGLE_OAUTH_CERTS_ENDPOINT")]
    google_oauth_jwks_endpoint: String,
}

//const ACCESS_TOKEN_EXPIRY_WINDOW: i64 = 3600; // 6 minutes

#[derive(Debug, PartialEq)]
pub enum AuthenticationError {
    Decoding(String),
    Expired,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: i64,
}

#[derive(Deserialize)]
pub struct Token {}

pub trait OAuth2Token {
    fn access_token(&self) -> &str;

    fn access_token_valid(&self) -> bool {
        let config = Config::init_from_env().unwrap();
        match Claims::from_token_with_secret(
            self.access_token(),
            config.key.as_str(),
            &Validation::default(),
        ) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn claims(&self) -> Result<Claims, AuthenticationError> {
        let config: Config = Config::init_from_env().unwrap();
        Claims::from_token_with_secret(
            self.access_token(),
            config.key.as_str(),
            &Validation::default(),
        )
    }

    fn refresh_token(&self) -> &str;
}

#[async_trait]
pub trait Refreshable {
    async fn refresh_access_token(&mut self, oauth2: OAuth2<Token>);
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tokens {
    access_token: String,
    refresh_token: String,
}

impl OAuth2Token for Tokens {
    fn access_token(&self) -> &str {
        return self.access_token.as_str();
    }
    fn refresh_token(&self) -> &str {
        return self.refresh_token.as_str();
    }
}

impl Claims {
    fn from_token_with_secret(
        token: &str,
        secret: &str,
        validation: &Validation,
    ) -> Result<Self, AuthenticationError> {
        let token = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            validation,
        )
        .map_err(|e| match e.kind() {
            ErrorKind::ExpiredSignature => AuthenticationError::Expired,
            _ => AuthenticationError::Decoding(e.to_string()),
        })?;
        Ok(token.claims)
    }

    async fn from_token_with_jwks(
        token: &str,
        audience: &str,
        jwks: &JwkSet,
    ) -> Result<Self, AuthenticationError> {
        let header: Header = decode_header(token).unwrap();
        let kid: String = header.kid.unwrap().to_owned();
        if let Some(j) = jwks.find(kid.as_str()) {
            match &j.algorithm {
                AlgorithmParameters::RSA(rsa) => {
                    let decoding_key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e).unwrap();
                    let mut validation = Validation::new(
                        Algorithm::from_str(j.common.key_algorithm.unwrap().to_string().as_str())
                            .unwrap(),
                    );
                    validation.set_audience(&[audience]);
                    validation.validate_exp = false;
                    let decoded_token = decode::<Claims>(token, &decoding_key, &validation)
                        .map_err(|e| match e.kind() {
                            ErrorKind::ExpiredSignature => AuthenticationError::Expired,
                            _ => AuthenticationError::Decoding(e.to_string()),
                        })?;
                    return Ok(decoded_token.claims);
                }
                _ => {
                    return Err(AuthenticationError::Decoding(String::from(
                        "InvalidAlgorithm",
                    )))
                }
            }
        }
        Err(AuthenticationError::Decoding(String::from("JwkNotFound")))
    }
}

pub fn create_jwt(uid: String, email: String, exp: i64, secret: &str) -> Result<String, Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::seconds(exp.to_owned()))
        .expect("failed to create an expiration time")
        .timestamp();
    let claims = Claims {
        sub: uid.to_string(),
        email: email.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

async fn claims_from_response(
    response: &TokenResponse<Token>,
) -> Result<Claims, AuthenticationError> {
    let config: Config = Config::init_from_env().unwrap();
    let id_token: &str = response
        .as_value()
        .get("id_token")
        .and_then(Value::as_str)
        .unwrap();
    let response: String = reqwest::get(config.google_oauth_jwks_endpoint)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let jwks: JwkSet = serde_json::from_str(response.as_str()).unwrap();
    let audience = config.google_oauth_audience;
    Ok(
        Claims::from_token_with_jwks(id_token, audience.as_str(), &jwks)
            .await
            .unwrap(),
    )
}

// Implement FromRequest fairing, this will construct Tokens for the request
