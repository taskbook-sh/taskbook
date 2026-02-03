mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod router;

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::config::ServerConfig;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = ServerConfig::load();
    let pool = db::create_pool(&config.database_url).await;

    // Run migrations
    sqlx::migrate!("src/migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    let app = router::build(pool, config.session_expiry_days);
    let addr = SocketAddr::from((config.host, config.port));

    tracing::info!("Starting taskbook server on {}", addr);

    let listener = TcpListener::bind(addr).await.expect("Failed to bind address");
    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
