use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Credentials for server authentication and encryption.
/// Stored at ~/.taskbook/credentials.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub server_url: String,
    pub token: String,
    pub encryption_key: String, // base64-encoded 32-byte key
}

impl Credentials {
    fn credentials_path() -> PathBuf {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".taskbook")
            .join("credentials.json")
    }

    /// Load credentials from disk. Returns None if the file doesn't exist.
    pub fn load() -> Result<Option<Self>> {
        let path = Self::credentials_path();
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path)?;
        let creds: Credentials = serde_json::from_str(&content)?;
        Ok(Some(creds))
    }

    /// Save credentials to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::credentials_path();
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(())
    }

    /// Delete the credentials file.
    pub fn delete() -> Result<()> {
        let path = Self::credentials_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Decode the encryption key from base64.
    pub fn encryption_key_bytes(&self) -> Result<[u8; 32]> {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&self.encryption_key)
            .map_err(|e| crate::error::TaskbookError::General(format!("invalid encryption key: {e}")))?;
        if bytes.len() != 32 {
            return Err(crate::error::TaskbookError::General(format!(
                "encryption key must be 32 bytes, got {}",
                bytes.len()
            )));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(key)
    }
}
