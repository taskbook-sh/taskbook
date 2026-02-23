use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit, Nonce};

use crate::error::CommonError;
use crate::StorageItem;

/// An encrypted item with its ciphertext and nonce.
pub struct EncryptedItem {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Generate a new random 256-bit encryption key.
pub fn generate_key() -> [u8; 32] {
    let key = Aes256Gcm::generate_key(OsRng);
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&key);
    bytes
}

/// Encrypt a `StorageItem` using AES-256-GCM.
///
/// The item is serialized to JSON, then encrypted with a random 12-byte nonce.
/// The nonce is returned alongside the ciphertext so it can be stored for decryption.
pub fn encrypt_item(key: &[u8; 32], item: &StorageItem) -> Result<EncryptedItem, CommonError> {
    let plaintext = serde_json::to_vec(item).map_err(CommonError::Json)?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_ref())
        .map_err(|_| CommonError::DecryptionFailed)?;

    Ok(EncryptedItem {
        data: ciphertext,
        nonce: nonce.to_vec(),
    })
}

/// Decrypt an `EncryptedItem` back into a `StorageItem` using AES-256-GCM.
pub fn decrypt_item(key: &[u8; 32], encrypted: &EncryptedItem) -> Result<StorageItem, CommonError> {
    if encrypted.nonce.len() != 12 {
        return Err(CommonError::InvalidNonce {
            expected: 12,
            got: encrypted.nonce.len(),
        });
    }

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&encrypted.nonce);
    let plaintext = cipher
        .decrypt(nonce, encrypted.data.as_ref())
        .map_err(|_| CommonError::DecryptionFailed)?;

    let item: StorageItem = serde_json::from_slice(&plaintext).map_err(CommonError::Json)?;
    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Note, Task};

    #[test]
    fn test_generate_key_is_32_bytes() {
        let key = generate_key();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_generate_key_is_random() {
        let key1 = generate_key();
        let key2 = generate_key();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_encrypt_decrypt_task_roundtrip() {
        let key = generate_key();
        let task = Task::new(1, "Test task".to_string(), vec!["My Board".to_string()], 1);
        let item = StorageItem::Task(task);

        let encrypted = encrypt_item(&key, &item).unwrap();
        assert!(!encrypted.data.is_empty());
        assert_eq!(encrypted.nonce.len(), 12);

        let decrypted = decrypt_item(&key, &encrypted).unwrap();
        assert_eq!(decrypted.description(), item.description());
        assert_eq!(decrypted.id(), item.id());
        assert!(decrypted.is_task());
    }

    #[test]
    fn test_encrypt_decrypt_note_roundtrip() {
        let key = generate_key();
        let note = Note::new(42, "Test note".to_string(), vec!["Notes".to_string()]);
        let item = StorageItem::Note(note);

        let encrypted = encrypt_item(&key, &item).unwrap();
        let decrypted = decrypt_item(&key, &encrypted).unwrap();

        assert_eq!(decrypted.description(), "Test note");
        assert_eq!(decrypted.id(), 42);
        assert!(!decrypted.is_task());
    }

    #[test]
    fn test_wrong_key_fails_decryption() {
        let key1 = generate_key();
        let key2 = generate_key();
        let task = Task::new(1, "Secret".to_string(), vec!["default".to_string()], 1);
        let item = StorageItem::Task(task);

        let encrypted = encrypt_item(&key1, &item).unwrap();
        let result = decrypt_item(&key2, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_nonce_length() {
        let key = generate_key();
        let encrypted = EncryptedItem {
            data: vec![1, 2, 3],
            nonce: vec![1, 2, 3], // wrong length, should be 12
        };
        let result = decrypt_item(&key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = generate_key();
        let task = Task::new(1, "Test".to_string(), vec!["default".to_string()], 1);
        let item = StorageItem::Task(task);

        let mut encrypted = encrypt_item(&key, &item).unwrap();
        // Tamper with the ciphertext
        if let Some(byte) = encrypted.data.first_mut() {
            *byte ^= 0xFF;
        }
        let result = decrypt_item(&key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_each_encryption_produces_unique_nonce() {
        let key = generate_key();
        let task = Task::new(1, "Test".to_string(), vec!["default".to_string()], 1);
        let item = StorageItem::Task(task);

        let enc1 = encrypt_item(&key, &item).unwrap();
        let enc2 = encrypt_item(&key, &item).unwrap();

        // Nonces should differ (random)
        assert_ne!(enc1.nonce, enc2.nonce);
        // Ciphertext should differ (due to different nonces)
        assert_ne!(enc1.data, enc2.data);
    }
}
