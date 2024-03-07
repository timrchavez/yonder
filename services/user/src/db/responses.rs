use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Group {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Pagination {
    pub total_pages: i32,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub id: Uuid,
    pub given_name: String,
    pub family_name: String,
    pub is_superuser: bool,
    pub groups: Option<Vec<Uuid>>,
}
