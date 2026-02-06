use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::HeaderMap;
use chrono::Utc;
use uuid::Uuid;

use crate::error::ServerError;
use crate::router::AppState;

/// Extracted from the Authorization header after middleware validation.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ServerError;

    fn from_request_parts<'a, 'b, 'c>(
        parts: &'a mut Parts,
        state: &'b AppState,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'c>,
    >
    where
        'a: 'c,
        'b: 'c,
        Self: 'c,
    {
        Box::pin(async move {
            let headers = &parts.headers;
            let token = extract_bearer_token(headers).ok_or(ServerError::Unauthorized)?;

            let session = sqlx::query_as::<_, (Uuid,)>(
                "SELECT user_id FROM sessions WHERE token = $1 AND expires_at > $2",
            )
            .bind(&token)
            .bind(Utc::now())
            .fetch_optional(&state.pool)
            .await
            .map_err(ServerError::Database)?
            .ok_or(ServerError::Unauthorized)?;

            Ok(AuthUser { user_id: session.0 })
        })
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get("authorization")?.to_str().ok()?;
    value.strip_prefix("Bearer ").map(|token| token.to_string())
}
