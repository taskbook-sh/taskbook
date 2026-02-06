use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Encrypted item data transferred between client and server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedItemData {
    /// Base64-encoded ciphertext
    pub data: String,
    /// Base64-encoded 12-byte AES-GCM nonce
    pub nonce: String,
}

/// Response from GET /api/v1/items and GET /api/v1/items/archive
#[derive(Debug, Serialize, Deserialize)]
pub struct ItemsResponse {
    pub items: HashMap<String, EncryptedItemData>,
}

/// Request body for PUT /api/v1/items and PUT /api/v1/items/archive
#[derive(Debug, Serialize, Deserialize)]
pub struct PutItemsRequest {
    pub items: HashMap<String, EncryptedItemData>,
}

/// Request body for POST /api/v1/register
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Response from POST /api/v1/register
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub token: String,
}

/// Request body for POST /api/v1/login
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Response from POST /api/v1/login
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

/// Response from GET /api/v1/me
#[derive(Debug, Serialize, Deserialize)]
pub struct MeResponse {
    pub username: String,
    pub email: String,
}

/// Response from GET /api/v1/health
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}
