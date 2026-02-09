use std::time::Duration;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

/// Create a PostgreSQL connection pool with resilience settings.
///
/// Disables `extra_float_digits` startup parameter for PgBouncer compatibility.
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let connect_options: PgConnectOptions = database_url
        .parse::<PgConnectOptions>()?
        .extra_float_digits(None);

    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .connect_with(connect_options)
        .await
}
