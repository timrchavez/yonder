use rocket::serde::json::Value;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SingleApiResponse {
    pub result: Value,
}
