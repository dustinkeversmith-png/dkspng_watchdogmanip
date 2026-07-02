use thiserror::Error;

pub type Result<T> = std::result::Result<T, ContextError>;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("context not found: {0}")]
    ContextNotFound(String),

    #[error("parent context not found: {child} -> {parent}")]
    ParentNotFound { child: String, parent: String },

    #[error("duplicate context id: {0}")]
    DuplicateContext(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("glob error: {0}")]
    Glob(#[from] globset::Error),
}
