use super::auth;
use super::db;
use rocket::http::ContentType;
use rocket::response;
use rocket::response::{Responder, Response};
use rocket::serde::json::json;
use rocket::serde::json::Value;
use rocket::Request;
use rocket_db_pools::Connection;

use rocket::http::Status;

#[derive(Debug)]
pub struct ApiResponse {
    json: Value,
    status: Status,
}

impl<'r> Responder<'r, 'static> for ApiResponse {
    fn respond_to(self, req: &Request) -> response::Result<'static> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

#[get("/groups")]
pub async fn groups_list(claims: auth::Claims, conn: Connection<db::UserDb>) -> ApiResponse {
    if claims.is_superuser {
        let groups: Value = db::get_groups(conn).await.unwrap();
        ApiResponse {
            json: groups,
            status: Status::Ok,
        }
    } else {
        ApiResponse {
            json: json!({"error": {"short": "NotFound", "long": "resource not found"}}),
            status: Status::NotFound,
        }
    }
}

#[post("/groups")]
fn groups_add(claims: auth::Claims) -> String {
    format!("User '{}' found!", claims.email)
    // fetch user information
}

#[delete("/groups")]
fn groups_remove(claims: auth::Claims) -> String {
    format!("User '{}' found!", claims.email)
    // fetch user information
}

#[get("/groups/<id>")]
fn group_show(claims: auth::Claims, id: &str) -> String {
    format!("User '{},{}' found!", claims.email, id.to_string())
    // fetch user information
}

#[get("/groups/<id>/members")]
fn group_members_list(claims: auth::Claims, id: &str) -> String {
    format!("User '{}' found!", claims.email)
    // fetch user information
}

#[post("/groups/<id>/members")]
fn group_member_add(claims: auth::Claims, id: &str) -> String {
    format!("User '{}' found!", claims.email)
    // fetch user information
}

#[delete("/groups/<id>/members")]
fn group_member_remove(claims: auth::Claims, id: &str) -> String {
    format!("User '{}' found!", claims.email)
    // fetch user information
}

#[get("/users")]
fn users_list(claims: auth::Claims) -> String {
    format!("User '{}' found!", claims.email)
}

#[post("/users")]
fn users_add(claims: auth::Claims) -> String {
    format!("User '{}' found!", claims.email)
}

#[get("/users/me")]
pub async fn user_show_me(claims: auth::Claims, conn: Connection<db::UserDb>) -> ApiResponse {
    let email: &str = claims.email.as_str();
    match db::get_user_by_email(conn, email).await {
        Ok(user) => {
            return ApiResponse {
                json: user,
                status: Status::Ok,
            }
        }
        Err(_) => {}
    };
    ApiResponse {
        json: json!({"error": {"short": "NotFound", "long": "resource not found"}}),
        status: Status::NotFound,
    }
}

#[get("/users/<id>")]
fn user_show(claims: auth::Claims, id: &str) -> String {
    format!("User '{}' found!", claims.email)
}

#[put("/users/<id>")]
fn user_update(claims: auth::Claims, id: &str) -> String {
    format!("User '{}' found!", claims.email)
}
