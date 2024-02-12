use rocket::serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, FromRow, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Group {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, FromRow, Debug)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub id: i64,
    pub first_name: String,
    pub family_name: String,
    pub is_superuser: bool,
    pub groups: Vec<String>,
}
