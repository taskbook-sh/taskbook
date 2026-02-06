use std::net::SocketAddr;

use axum::extract::{ConnectInfo, State};
use axum::Json;
use base64::Engine;
use chrono::{Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{hash_password, verify_password};
use crate::error::{Result, ServerError};
use crate::middleware::AuthUser;
use crate::router::AppState;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub username: String,
    pub email: String,
}

pub async fn register(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>> {
    // Rate limit check
    if !state.auth_rate_limiter.check(addr.ip()).await {
        tracing::warn!(ip = %addr.ip(), "register rate limited");
        return Err(ServerError::RateLimited);
    }

    validate_registration(&req)?;

    let password_hash = hash_password(&req.password)
        .map_err(|e| ServerError::Internal(format!("password hashing failed: {e}")))?;

    let user_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO users (username, email, password) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
            ServerError::UserAlreadyExists
        }
        _ => ServerError::Database(e),
    })?;

    let token = create_session(&state.pool, user_id, state.session_expiry_days).await?;

    tracing::info!(username = %req.username, "user registered");

    Ok(Json(RegisterResponse { token }))
}

pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    // Rate limit check
    if !state.auth_rate_limiter.check(addr.ip()).await {
        tracing::warn!(ip = %addr.ip(), "login rate limited");
        return Err(ServerError::RateLimited);
    }

    let user =
        sqlx::query_as::<_, (Uuid, String)>("SELECT id, password FROM users WHERE username = $1")
            .bind(&req.username)
            .fetch_optional(&state.pool)
            .await
            .map_err(ServerError::Database)?
            .ok_or(ServerError::InvalidCredentials)?;

    let (user_id, password_hash) = user;

    let valid = verify_password(&req.password, &password_hash)
        .map_err(|e| ServerError::Internal(format!("password verification failed: {e}")))?;

    if !valid {
        tracing::warn!(username = %req.username, "failed login attempt");
        return Err(ServerError::InvalidCredentials);
    }

    let token = create_session(&state.pool, user_id, state.session_expiry_days).await?;

    tracing::info!(username = %req.username, "user logged in");

    Ok(Json(LoginResponse { token }))
}

pub async fn logout(State(state): State<AppState>, auth: AuthUser) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(auth.user_id)
        .execute(&state.pool)
        .await
        .map_err(ServerError::Database)?;

    tracing::info!(user_id = %auth.user_id, "user logged out");

    Ok(())
}

pub async fn me(State(state): State<AppState>, auth: AuthUser) -> Result<Json<MeResponse>> {
    let user =
        sqlx::query_as::<_, (String, String)>("SELECT username, email FROM users WHERE id = $1")
            .bind(auth.user_id)
            .fetch_one(&state.pool)
            .await
            .map_err(ServerError::Database)?;

    Ok(Json(MeResponse {
        username: user.0,
        email: user.1,
    }))
}

/// Generate a cryptographically random 256-bit session token.
async fn create_session(pool: &PgPool, user_id: Uuid, expiry_days: i64) -> Result<String> {
    let mut token_bytes = [0u8; 32];
    rand::thread_rng().fill(&mut token_bytes);
    let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(token_bytes);
    let expires_at = Utc::now() + Duration::days(expiry_days);

    sqlx::query("INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&token)
        .bind(expires_at)
        .execute(pool)
        .await
        .map_err(ServerError::Database)?;

    Ok(token)
}

/// Validate registration input fields.
fn validate_registration(req: &RegisterRequest) -> Result<()> {
    if req.username.is_empty() || req.password.is_empty() || req.email.is_empty() {
        return Err(ServerError::Validation(
            "username, email, and password are required".to_string(),
        ));
    }

    if req.username.len() > 64 {
        return Err(ServerError::Validation(
            "username must be at most 64 characters".to_string(),
        ));
    }

    if !req
        .username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ServerError::Validation(
            "username must contain only alphanumeric characters, hyphens, or underscores"
                .to_string(),
        ));
    }

    if req.email.len() > 255 {
        return Err(ServerError::Validation(
            "email must be at most 255 characters".to_string(),
        ));
    }

    if !req.email.contains('@') || !req.email.contains('.') {
        return Err(ServerError::Validation(
            "email must be a valid email address".to_string(),
        ));
    }

    if req.password.len() < 8 {
        return Err(ServerError::Validation(
            "password must be at least 8 characters".to_string(),
        ));
    }

    if req.password.len() > 1024 {
        return Err(ServerError::Validation(
            "password must be at most 1024 characters".to_string(),
        ));
    }

    Ok(())
}
