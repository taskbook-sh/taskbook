use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid nonce length: expected {expected}, got {got}")]
    InvalidNonce { expected: usize, got: usize },

    #[error("Decryption failed: ciphertext authentication error")]
    DecryptionFailed,
}

pub type CommonResult<T> = std::result::Result<T, CommonError>;
