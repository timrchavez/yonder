use envconfig::Envconfig;
use jsonwebtoken::{decode, errors::ErrorKind, DecodingKey, Validation};
use serde::Deserialize;

pub const BEARER: &str = "Bearer ";
pub const AUTHORIZATION: &str = "Authorization";

#[derive(Deserialize, Debug)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: i64,
}

#[derive(Debug, PartialEq)]
pub enum AuthenticationError {
    Missing,
    Decoding(String),
    Expired,
}

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "YONDER_JWT_SECRET")]
    key: String,
}

impl Claims {
    /// Create a `Claims` from a 'Bearer <token>' value
    pub fn from_authorization(value: &str) -> Result<Self, AuthenticationError> {
        let config = Config::init_from_env().unwrap();
        let access_token = value.strip_prefix(BEARER).map(str::trim);

        if access_token.is_none() {
            return Err(AuthenticationError::Missing);
        }

        let decoded_token = decode::<Claims>(
            access_token.unwrap(),
            &DecodingKey::from_secret(config.key.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| match e.kind() {
            ErrorKind::ExpiredSignature => AuthenticationError::Expired,
            _ => AuthenticationError::Decoding(e.to_string()),
        })?;

        Ok(decoded_token.claims)
    }
}
