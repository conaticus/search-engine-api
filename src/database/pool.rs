use std::env;

use sqlx::{postgres::PgPool, Pool, Postgres};

pub async fn init_pool() -> Result<Pool<Postgres>, sqlx::Error> {
    let host = env::var("DB_HOST").expect("Failed to get DB_HOST variable");
    let user = env::var("DB_USER").expect("Failed to get DB_USER variable");
    let name = env::var("DB_NAME").expect("Failed to get DB_NAME variable");
    let password = env::var("DB_PASSWORD").expect("Failed to get DB_PASSWORD variable");

    let pool =
        PgPool::connect(format!("postgres://{}:{}@{}/{}", user, password, host, name).as_str())
            .await?;

    Ok(pool)
}
