mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod rate_limit;
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

    let config = match ServerConfig::load() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("configuration error: {e}");
            std::process::exit(1);
        }
    };

    let pool = match db::create_pool(&config.database_url).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("failed to connect to database: {e}");
            std::process::exit(1);
        }
    };

    // Run migrations
    if let Err(e) = sqlx::migrate!("src/migrations").run(&pool).await {
        tracing::error!("failed to run database migrations: {e}");
        std::process::exit(1);
    }

    let app = router::build(pool, config.session_expiry_days, &config.cors_origins);
    let addr = SocketAddr::from((config.host, config.port));

    tracing::info!("starting taskbook server on {}", addr);

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("failed to bind address {addr}: {e}");
            std::process::exit(1);
        }
    };

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap_or_else(|e| {
        tracing::error!("server error: {e}");
        std::process::exit(1);
    });

    tracing::info!("server shut down gracefully");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("received Ctrl+C, shutting down"),
        _ = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}
