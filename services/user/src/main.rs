#[macro_use]
extern crate rocket;

mod api;
mod auth;
mod db;

use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::request;
use rocket::request::Outcome;
use rocket_db_pools::Database;

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for auth::Claims {
    type Error = auth::AuthenticationError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one(auth::AUTHORIZATION) {
            None => Outcome::Error((Status::Forbidden, auth::AuthenticationError::Missing)),
            Some(value) => match auth::Claims::from_authorization(value) {
                Err(e) => Outcome::Error((Status::Forbidden, e)),
                Ok(claims) => Outcome::Success(claims),
            },
        }
    }
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    let migrations = AdHoc::try_on_ignite("database migrations", db::run_migrations);
    rocket::build()
        .attach(db::UserDb::init())
        .attach(migrations)
        .mount("/", routes![api::user_show_me, api::groups_list,])
}
