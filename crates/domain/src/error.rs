use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Scan error: {0}")]
    Scan(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}
