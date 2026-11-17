use aes_gcm::aead::{Aead, Generate};
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};

use crate::error::CommonError;
use crate::StorageItem;

/// AES-GCM nonce length in bytes.
const NONCE_LEN: usize = 12;

/// An encrypted item with its ciphertext and nonce.
pub struct EncryptedItem {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Generate a new random 256-bit encryption key.
pub fn generate_key() -> [u8; 32] {
    Key::<Aes256Gcm>::generate().into()
}

/// Encrypt a `StorageItem` using AES-256-GCM.
///
/// The item is serialized to JSON, then encrypted with a random 12-byte nonce.
/// The nonce is returned alongside the ciphertext so it can be stored for decryption.
pub fn encrypt_item(key: &[u8; 32], item: &StorageItem) -> Result<EncryptedItem, CommonError> {
    let plaintext = serde_json::to_vec(item).map_err(CommonError::Json)?;
    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from(*key));
    let nonce = Nonce::generate();
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
    let nonce_bytes: [u8; NONCE_LEN] =
        encrypted
            .nonce
            .as_slice()
            .try_into()
            .map_err(|_| CommonError::InvalidNonce {
                expected: NONCE_LEN,
                got: encrypted.nonce.len(),
            })?;

    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from(*key));
    let nonce = Nonce::from(nonce_bytes);
    let plaintext = cipher
        .decrypt(&nonce, encrypted.data.as_ref())
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

    /// Ciphertext produced by aes-gcm 0.10 (key = [7; 32], nonce = [9; 12]).
    /// Guards against a dependency upgrade silently breaking existing user data.
    #[test]
    fn test_decrypts_ciphertext_from_previous_aes_gcm_version() {
        let encrypted = EncryptedItem {
            data: vec![
                92, 167, 219, 253, 218, 210, 251, 86, 140, 64, 148, 92, 135, 158, 198, 253, 238,
                146, 46, 156, 90, 206, 189, 161, 1, 142, 118, 151, 245, 70, 49, 72, 231, 199, 203,
                225, 189, 38, 67, 125, 26, 192, 209, 119, 41, 68, 193, 238, 77, 99, 174, 160, 34,
                162, 153, 124, 85, 95, 49, 167, 102, 58, 88, 209, 5, 12, 220, 111, 250, 77, 206,
                153, 190, 129, 165, 7, 218, 197, 166, 145, 120, 154, 168, 100, 194, 119, 176, 212,
                18, 209, 116, 188, 125, 235, 104, 5, 98, 18, 173, 179, 167, 50, 255, 108, 0, 237,
                181, 97, 27, 180, 56, 118, 217, 124, 104, 28, 191, 175, 168, 137, 59, 161, 186,
                198, 111, 100, 47, 151, 242, 252, 108, 73, 77, 241, 118, 179, 49, 68, 142, 162, 25,
                151, 34, 128, 64, 170, 107, 140, 236, 37, 250, 46, 5, 58, 131, 58, 43, 209, 228,
                205, 84, 41,
            ],
            nonce: vec![9u8; 12],
        };

        let decrypted = decrypt_item(&[7u8; 32], &encrypted).unwrap();

        assert_eq!(decrypted.id(), 7);
        assert_eq!(decrypted.description(), "legacy note");
        assert!(!decrypted.is_task());
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
