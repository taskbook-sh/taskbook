use std::collections::HashMap;

use axum::extract::State;
use axum::Json;
use base64::Engine as _;
use serde::{Deserialize, Serialize};

use crate::error::{Result, ServerError};
use crate::middleware::AuthUser;
use crate::router::{AppState, SyncEvent};

#[derive(Deserialize, Serialize, Clone)]
pub struct EncryptedItemData {
    pub data: String,  // base64-encoded ciphertext
    pub nonce: String, // base64-encoded nonce
}

#[derive(Serialize)]
pub struct ItemsResponse {
    pub items: HashMap<String, EncryptedItemData>,
}

#[derive(Deserialize)]
pub struct PutItemsRequest {
    pub items: HashMap<String, EncryptedItemData>,
}

/// Convert raw database rows `(item_key, data_bytes, nonce_bytes)` into the
/// base64-encoded `EncryptedItemData` map returned to callers.
fn rows_to_encrypted_items(
    rows: Vec<(String, Vec<u8>, Vec<u8>)>,
) -> HashMap<String, EncryptedItemData> {
    rows.into_iter()
        .map(|(key, data, nonce)| {
            (
                key,
                EncryptedItemData {
                    data: base64::engine::general_purpose::STANDARD.encode(&data),
                    nonce: base64::engine::general_purpose::STANDARD.encode(&nonce),
                },
            )
        })
        .collect()
}

pub async fn get_items(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ItemsResponse>> {
    let rows = sqlx::query_as::<_, (String, Vec<u8>, Vec<u8>)>(
        "SELECT item_key, data, nonce FROM items WHERE user_id = $1 AND archived = false",
    )
    .bind(auth.user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(ServerError::Database)?;

    Ok(Json(ItemsResponse {
        items: rows_to_encrypted_items(rows),
    }))
}

pub async fn put_items(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<PutItemsRequest>,
) -> Result<()> {
    replace_items(&state.pool, auth.user_id, false, &req.items).await?;
    state
        .notifications
        .notify(auth.user_id, SyncEvent::DataChanged { archived: false });
    Ok(())
}

pub async fn get_archive(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ItemsResponse>> {
    let rows = sqlx::query_as::<_, (String, Vec<u8>, Vec<u8>)>(
        "SELECT item_key, data, nonce FROM items WHERE user_id = $1 AND archived = true",
    )
    .bind(auth.user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(ServerError::Database)?;

    Ok(Json(ItemsResponse {
        items: rows_to_encrypted_items(rows),
    }))
}

pub async fn put_archive(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<PutItemsRequest>,
) -> Result<()> {
    replace_items(&state.pool, auth.user_id, true, &req.items).await?;
    state
        .notifications
        .notify(auth.user_id, SyncEvent::DataChanged { archived: true });
    Ok(())
}

/// Maximum number of items a user can store per category (active or archived).
const MAX_ITEMS_PER_CATEGORY: usize = 10_000;

/// Replace all items for a user (active or archived) with the provided set.
async fn replace_items(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    archived: bool,
    items: &HashMap<String, EncryptedItemData>,
) -> Result<()> {
    if items.len() > MAX_ITEMS_PER_CATEGORY {
        return Err(ServerError::Validation(format!(
            "too many items: maximum is {MAX_ITEMS_PER_CATEGORY}, got {}",
            items.len()
        )));
    }

    // Validate individual item sizes
    for (key, item) in items {
        if key.len() > 64 {
            return Err(ServerError::Validation(
                "item key must be at most 64 characters".to_string(),
            ));
        }
        // Base64-decoded nonce should be 12 bytes (16 chars in base64)
        if item.nonce.len() > 24 {
            return Err(ServerError::Validation("invalid nonce size".to_string()));
        }
        // Limit individual item data to 1 MB (base64-encoded)
        if item.data.len() > 1_400_000 {
            return Err(ServerError::Validation("item data too large".to_string()));
        }
    }

    let mut tx = pool.begin().await.map_err(ServerError::Database)?;

    sqlx::query("DELETE FROM items WHERE user_id = $1 AND archived = $2")
        .bind(user_id)
        .bind(archived)
        .execute(&mut *tx)
        .await
        .map_err(ServerError::Database)?;

    for (key, item) in items {
        let data = base64::engine::general_purpose::STANDARD
            .decode(&item.data)
            .map_err(|e| ServerError::Validation(format!("invalid base64 data: {e}")))?;
        let nonce = base64::engine::general_purpose::STANDARD
            .decode(&item.nonce)
            .map_err(|e| ServerError::Validation(format!("invalid base64 nonce: {e}")))?;

        sqlx::query(
            "INSERT INTO items (user_id, item_key, data, nonce, archived) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(user_id)
        .bind(key)
        .bind(&data)
        .bind(&nonce)
        .bind(archived)
        .execute(&mut *tx)
        .await
        .map_err(ServerError::Database)?;
    }

    tx.commit().await.map_err(ServerError::Database)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_row(key: &str, data: &[u8], nonce: &[u8]) -> (String, Vec<u8>, Vec<u8>) {
        (key.to_string(), data.to_vec(), nonce.to_vec())
    }

    #[test]
    fn rows_to_encrypted_items_encodes_base64() {
        let data = b"ciphertext bytes";
        let nonce = b"nonce123nonce"; // 13 bytes
        let rows = vec![make_row("item-1", data, nonce)];

        let map = rows_to_encrypted_items(rows);

        assert_eq!(map.len(), 1);
        let item = map.get("item-1").expect("item-1 should be present");

        assert_eq!(
            item.data,
            base64::engine::general_purpose::STANDARD.encode(data)
        );
        assert_eq!(
            item.nonce,
            base64::engine::general_purpose::STANDARD.encode(nonce)
        );
    }

    #[test]
    fn rows_to_encrypted_items_empty_input() {
        let map = rows_to_encrypted_items(vec![]);
        assert!(map.is_empty());
    }

    #[test]
    fn rows_to_encrypted_items_multiple_rows() {
        let rows = vec![
            make_row("key-a", b"data-a", b"nonce-a"),
            make_row("key-b", b"data-b", b"nonce-b"),
        ];
        let map = rows_to_encrypted_items(rows);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("key-a"));
        assert!(map.contains_key("key-b"));
    }
}
