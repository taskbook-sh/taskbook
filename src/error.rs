use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskbookError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid item ID: {0}")]
    InvalidId(u64),

    #[error("Invalid custom directory: {0}")]
    InvalidDirectory(String),

    #[error("Missing taskbook-dir flag value")]
    MissingTaskbookDirValue,

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("No items to copy")]
    NoItemsToCopy,

    #[error("TUI error: {0}")]
    Tui(String),
}

pub type Result<T> = std::result::Result<T, TaskbookError>;
