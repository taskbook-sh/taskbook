use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication required")]
    Unauthorized,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ServerError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            ServerError::Unauthorized => (StatusCode::UNAUTHORIZED, "Authentication required"),
            ServerError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            ServerError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            ServerError::Validation(msg) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": msg })),
                )
                    .into_response();
            }
            ServerError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ServerError>;
