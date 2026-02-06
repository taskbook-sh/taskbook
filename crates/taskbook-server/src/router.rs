use axum::http::HeaderValue;
use axum::routing::{delete, get, post, put};
use axum::Router;
use sqlx::PgPool;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;

use crate::handlers::{health, items, user};
use crate::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub session_expiry_days: i64,
    pub auth_rate_limiter: RateLimiter,
}

pub fn build(pool: PgPool, session_expiry_days: i64, cors_origins: &[String]) -> Router {
    // 10 auth requests per IP per 60 seconds
    let auth_rate_limiter = RateLimiter::new(10, 60);

    let state = AppState {
        pool,
        session_expiry_days,
        auth_rate_limiter,
    };

    let cors = build_cors_layer(cors_origins);

    Router::new()
        .route("/api/v1/health", get(health::health))
        .route("/api/v1/register", post(user::register))
        .route("/api/v1/login", post(user::login))
        .route("/api/v1/logout", delete(user::logout))
        .route("/api/v1/me", get(user::me))
        .route("/api/v1/items", get(items::get_items))
        .route("/api/v1/items", put(items::put_items))
        .route("/api/v1/items/archive", get(items::get_archive))
        .route("/api/v1/items/archive", put(items::put_archive))
        // 10 MB body limit for item uploads
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        .layer(cors)
        .with_state(state)
}

fn build_cors_layer(origins: &[String]) -> CorsLayer {
    let cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    if origins.is_empty() {
        // No origins configured: reject cross-origin requests
        cors.allow_origin(AllowOrigin::exact(HeaderValue::from_static(
            "https://localhost",
        )))
    } else {
        let parsed: Vec<HeaderValue> = origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        cors.allow_origin(AllowOrigin::list(parsed))
    }
}
