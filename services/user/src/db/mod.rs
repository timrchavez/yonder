pub mod requests;
pub mod responses;

use rocket::fairing;
use rocket::serde::json::json;
use rocket::serde::json::Value;
use rocket::Build;
use rocket::Rocket;
use rocket_db_pools::sqlx;
use rocket_db_pools::Connection;
use rocket_db_pools::Database;
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

pub async fn get_groups(mut conn: Connection<UserDb>) -> DBResult<Value> {
    //let mut connection = pool.acquire().await.unwrap();
    let groups = sqlx::query_as::<_, responses::Group>(
        r#"
        SELECT id, name FROM UserGroup;
        "#,
    )
    .fetch_all(&mut **conn)
    .await?;
    Ok(json!(groups))
}

pub async fn get_user_by_email(mut conn: Connection<UserDb>, email: &str) -> DBResult<Value> {
    //let mut connection = pool.acquire().await.unwrap();
    let user = sqlx::query_as::<_, responses::User>(
        r#"
        SELECT user_id AS id, u.first_name AS first_name, u.family_name AS family_name, u.is_superuser AS is_superuser, u.groups AS groups
        FROM public."Account" a
        LEFT JOIN (
            SELECT u.id as user_id, u.first_name, u.family_name, u.is_superuser, array_remove(array_agg(g.id), NULL) AS groups
            FROM public."User" u
            LEFT OUTER JOIN public."UserGroup" g ON u.id = g.user_id
            GROUP BY u.id
        ) u USING (user_id) WHERE a.email = ?
        "#,
    )
    .bind(email)
    .fetch_one(&mut **conn)
    .await?;
    Ok(json!(user))
}
