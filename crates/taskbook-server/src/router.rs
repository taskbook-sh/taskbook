use axum::routing::{delete, get, post, put};
use axum::Router;
use sqlx::PgPool;

use crate::handlers::{health, items, user};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub session_expiry_days: i64,
}

pub fn build(pool: PgPool, session_expiry_days: i64) -> Router {
    let state = AppState {
        pool,
        session_expiry_days,
    };

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
        .with_state(state)
}
