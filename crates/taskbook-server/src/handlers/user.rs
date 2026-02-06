use axum::extract::State;
use axum::Json;
use chrono::{Duration, Utc};
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
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>> {
    if req.username.is_empty() || req.password.is_empty() || req.email.is_empty() {
        return Err(ServerError::Validation(
            "username, email, and password are required".to_string(),
        ));
    }

    if req.password.len() < 8 {
        return Err(ServerError::Validation(
            "password must be at least 8 characters".to_string(),
        ));
    }

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

    Ok(Json(RegisterResponse { token }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
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
        return Err(ServerError::InvalidCredentials);
    }

    let token = create_session(&state.pool, user_id, state.session_expiry_days).await?;

    Ok(Json(LoginResponse { token }))
}

pub async fn logout(State(state): State<AppState>, auth: AuthUser) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(auth.user_id)
        .execute(&state.pool)
        .await
        .map_err(ServerError::Database)?;

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

async fn create_session(pool: &PgPool, user_id: Uuid, expiry_days: i64) -> Result<String> {
    let token = Uuid::new_v4().to_string();
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
