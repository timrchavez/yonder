use super::auth;
use super::db;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response;
use rocket::response::{Responder, Response};
use rocket::serde::json::json;
use rocket::serde::json::Value;
use rocket::Request;
use rocket_db_pools::Connection;

const DEFAULT_PAGE: i32 = 1;
const DEFAULT_PER_PAGE: i32 = 10;

#[derive(Debug)]
pub struct ApiResponse {
    json: Value,
    status: Status,
}

impl<'r> Responder<'r, 'static> for ApiResponse {
    fn respond_to(self, req: &Request) -> response::Result<'static> {
        // check if response is paginated
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

#[get("/groups?<page>&<per_page>")]
pub async fn groups_list(
    claims: auth::Claims,
    user_conn: Connection<db::UserDb>,
    group_conn: Connection<db::UserDb>,
    count_conn: Connection<db::UserDb>,
    page: Option<i32>,
    per_page: Option<i32>,
) -> ApiResponse {
    let email: &str = claims.email.as_str();
    match db::get_user_by_email(user_conn, email).await {
        Ok(j) => {
            let user: db::responses::User = serde_json::from_value(j).unwrap();
            if user.is_superuser {
                let mut resolved_page = page.unwrap_or(DEFAULT_PAGE);
                let mut resolved_per_page = per_page.unwrap_or(DEFAULT_PER_PAGE);
                let mut total_pages: i32 = 1;
                match db::get_group_count(count_conn).await {
                    Ok(j) => {
                        let pagination: db::responses::Pagination =
                            serde_json::from_value(j).unwrap();
                        total_pages = pagination.total_pages;
                    }
                    Err(_) => {}
                }
                if resolved_page < 1 {
                    resolved_page = 1;
                } else if resolved_page > total_pages {
                    resolved_page = total_pages
                }
                if resolved_per_page < 10 {
                    resolved_per_page = 10
                } else if resolved_per_page > 100 {
                    resolved_per_page = 100;
                }
                match db::get_groups(group_conn, resolved_page, resolved_per_page, total_pages)
                    .await
                {
                    Ok(groups) => {
                        return ApiResponse {
                            json: groups,
                            status: Status::Ok,
                        };
                    }
                    Err(_) => {}
                }
            };
        }
        Err(_) => {}
    }
    ApiResponse {
        json: json!({"error": {"short": "NotFound", "long": "resource not found"}}),
        status: Status::NotFound,
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
pub async fn me_show(claims: auth::Claims, conn: Connection<db::UserDb>) -> ApiResponse {
    let email: &str = claims.email.as_str();
    match db::get_user_by_email(conn, email).await {
        Ok(user) => {
            return ApiResponse {
                json: user,
                status: Status::Ok,
            };
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
