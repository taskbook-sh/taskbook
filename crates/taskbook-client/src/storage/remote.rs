use std::collections::HashMap;

use base64::Engine;
use taskbook_common::encryption::{decrypt_item, encrypt_item, EncryptedItem};
use taskbook_common::StorageItem;

use super::StorageBackend;
use crate::api_client::{ApiClient, EncryptedItemData};
use crate::credentials::Credentials;
use crate::error::{Result, TaskbookError};

/// Remote storage backend that communicates with a taskbook server.
/// All data is encrypted client-side before being sent to the server.
pub struct RemoteStorage {
    client: ApiClient,
    encryption_key: [u8; 32],
}

impl RemoteStorage {
    pub fn new(server_url: &str) -> Result<Self> {
        let creds = Credentials::load()?.ok_or_else(|| {
            TaskbookError::Auth("not logged in — run `tb register` or `tb login` first".to_string())
        })?;

        let encryption_key = creds.encryption_key_bytes()?;
        let client = ApiClient::new(server_url, Some(&creds.token));

        Ok(Self {
            client,
            encryption_key,
        })
    }

    fn decrypt_items(
        &self,
        encrypted: &HashMap<String, EncryptedItemData>,
    ) -> Result<HashMap<String, StorageItem>> {
        let engine = base64::engine::general_purpose::STANDARD;
        let mut result = HashMap::new();

        for (key, item_data) in encrypted {
            let data = engine
                .decode(&item_data.data)
                .map_err(|e| TaskbookError::General(format!("invalid base64 data: {e}")))?;
            let nonce = engine
                .decode(&item_data.nonce)
                .map_err(|e| TaskbookError::General(format!("invalid base64 nonce: {e}")))?;

            let encrypted_item = EncryptedItem { data, nonce };
            let item = decrypt_item(&self.encryption_key, &encrypted_item)
                .map_err(|e| TaskbookError::General(format!("decryption failed: {e}")))?;

            result.insert(key.clone(), item);
        }

        Ok(result)
    }

    fn encrypt_items(
        &self,
        items: &HashMap<String, StorageItem>,
    ) -> Result<HashMap<String, EncryptedItemData>> {
        let engine = base64::engine::general_purpose::STANDARD;
        let mut result = HashMap::new();

        for (key, item) in items {
            let encrypted = encrypt_item(&self.encryption_key, item)
                .map_err(|e| TaskbookError::General(format!("encryption failed: {e}")))?;

            result.insert(
                key.clone(),
                EncryptedItemData {
                    data: engine.encode(&encrypted.data),
                    nonce: engine.encode(&encrypted.nonce),
                },
            );
        }

        Ok(result)
    }
}

impl StorageBackend for RemoteStorage {
    fn get(&self) -> Result<HashMap<String, StorageItem>> {
        let encrypted = self.client.get_items()?;
        self.decrypt_items(&encrypted)
    }

    fn get_archive(&self) -> Result<HashMap<String, StorageItem>> {
        let encrypted = self.client.get_archive()?;
        self.decrypt_items(&encrypted)
    }

    fn set(&self, data: &HashMap<String, StorageItem>) -> Result<()> {
        let encrypted = self.encrypt_items(data)?;
        self.client.put_items(&encrypted)
    }

    fn set_archive(&self, data: &HashMap<String, StorageItem>) -> Result<()> {
        let encrypted = self.encrypt_items(data)?;
        self.client.put_archive(&encrypted)
    }
}
