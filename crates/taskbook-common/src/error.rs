use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),
}

pub type CommonResult<T> = std::result::Result<T, CommonError>;
