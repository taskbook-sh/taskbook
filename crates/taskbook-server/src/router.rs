use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::http::HeaderValue;
use axum::routing::{delete, get, post, put};
use axum::Router;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use uuid::Uuid;

use crate::handlers::{events, health, items, user};
use crate::rate_limit::RateLimiter;

/// Event broadcast to connected SSE clients when data changes.
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// Items or archive were updated.
    DataChanged { archived: bool },
}

/// Per-user broadcast hub for real-time sync notifications.
#[derive(Clone, Default)]
pub struct NotificationHub {
    senders: Arc<RwLock<HashMap<Uuid, broadcast::Sender<SyncEvent>>>>,
}

impl NotificationHub {
    /// Subscribe to notifications for the given user.
    /// Creates a new broadcast channel if one doesn't exist yet.
    pub fn subscribe(&self, user_id: Uuid) -> broadcast::Receiver<SyncEvent> {
        let mut map = self.senders.write().unwrap();
        let sender = map
            .entry(user_id)
            .or_insert_with(|| broadcast::channel(64).0);
        sender.subscribe()
    }

    /// Send a notification to all connected clients for the given user.
    pub fn notify(&self, user_id: Uuid, event: SyncEvent) {
        let map = self.senders.read().unwrap();
        if let Some(sender) = map.get(&user_id) {
            // Ignore send errors — they just mean no receivers are connected.
            let _ = sender.send(event);
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub session_expiry_days: i64,
    pub auth_rate_limiter: RateLimiter,
    pub notifications: NotificationHub,
}

pub fn build(pool: PgPool, session_expiry_days: i64, cors_origins: &[String]) -> Router {
    // 10 auth requests per IP per 60 seconds
    let auth_rate_limiter = RateLimiter::new(10, 60);

    let state = AppState {
        pool,
        session_expiry_days,
        auth_rate_limiter,
        notifications: NotificationHub::default(),
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
        .route("/api/v1/events", get(events::events))
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
        // No TB_CORS_ORIGINS configured. Use http://localhost as the default
        // so that local browser-based development works out of the box.
        // For any deployed or production browser client, set TB_CORS_ORIGINS
        // explicitly (e.g. TB_CORS_ORIGINS=https://app.example.com).
        cors.allow_origin(AllowOrigin::exact(HeaderValue::from_static(
            "http://localhost",
        )))
    } else {
        let parsed: Vec<HeaderValue> = origins.iter().filter_map(|o| o.parse().ok()).collect();
        cors.allow_origin(AllowOrigin::list(parsed))
    }
}
