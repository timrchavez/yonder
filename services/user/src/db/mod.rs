pub mod requests;
pub mod responses;

use std::cmp;

use rocket::fairing;
use rocket::serde::json::json;
use rocket::serde::json::Value;
use rocket::Build;
use rocket::Rocket;
use rocket_db_pools::sqlx;
use rocket_db_pools::Connection;
use rocket_db_pools::Database;
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;

pub type DBResult<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(Database)]
#[database("userdb")]
pub struct UserDb(PgPool);

pub async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match UserDb::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("./migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to run database migrations: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct PaginatedGroupsQueryResult {
    result: Vec<responses::Group>,
    prev_page: i32,
    next_page: i32,
    last_page: i32,
}

pub async fn get_groups(
    mut conn: Connection<UserDb>,
    page: i32,
    per_page: i32,
    total_pages: i32,
) -> DBResult<Value> {
    let query = sqlx::query_as!(
        responses::Group,
        r#"
        SELECT id, name FROM public."UserGroup" LIMIT $1 OFFSET $2;
        "#,
        i64::from(per_page),
        i64::from((page - 1) * per_page)
    )
    .fetch_all(&mut **conn)
    .await?;

    Ok(json!(PaginatedGroupsQueryResult {
        result: query,
        prev_page: cmp::max(1, page - 1),
        next_page: cmp::min(total_pages, page),
        last_page: total_pages,
    }))
}

pub async fn get_group_count(mut conn: Connection<UserDb>) -> DBResult<Value> {
    let groups_count = sqlx::query_as::<_, responses::Pagination>(
        r#"
        SELECT COUNT(*) as total_pages FROM public."UserGroup";
        "#,
    )
    .fetch_one(&mut **conn)
    .await?;
    Ok(json!(groups_count))
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct UserQueryResult {
    result: responses::User,
}

pub async fn get_user_by_email(mut conn: Connection<UserDb>, email: &str) -> DBResult<Value> {
    let query = sqlx::query_as!(
        responses::User,
        r#"
        SELECT user_id AS id, u.given_name AS given_name, u.family_name AS family_name, u.is_superuser AS is_superuser, u.groups AS groups
        FROM public."Account" a
        LEFT JOIN (
            SELECT u.id as user_id, u.given_name, u.family_name, u.is_superuser, array_remove(array_agg(g.id), NULL) AS groups
            FROM public."User" u
            LEFT OUTER JOIN public."UserGroup" g ON u.id = g.user_id
            GROUP BY u.id
        ) u USING (user_id) WHERE a.email = $1
        "#,
        email
    ).fetch_one(&mut **conn).await?;
    Ok(json!(UserQueryResult { result: query }))
}
