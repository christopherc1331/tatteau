#[cfg(feature = "ssr")]
use sqlx::postgres::PgPoolOptions;
#[cfg(feature = "ssr")]
use sqlx::{PgPool, Pool, Postgres};
#[cfg(feature = "ssr")]
use std::sync::OnceLock;

#[cfg(feature = "ssr")]
static DB_POOL: OnceLock<PgPool> = OnceLock::new();

#[cfg(feature = "ssr")]
pub async fn init_pool() -> Result<(), sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost/tatteau".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    DB_POOL.set(pool).map_err(|_| {
        sqlx::Error::Configuration(
            "Database pool already initialized".to_string().into(),
        )
    })?;

    Ok(())
}

#[cfg(feature = "ssr")]
pub fn get_pool() -> &'static Pool<Postgres> {
    DB_POOL.get().expect("Database pool not initialized. Call init_pool() first.")
}
