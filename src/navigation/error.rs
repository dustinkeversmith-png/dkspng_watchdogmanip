use thiserror::Error;

pub type Result<T> = std::result::Result<T, NavigationError>;

#[derive(Debug, Error)]
pub enum NavigationError {
    #[error("scope not found: {0}")]
    ScopeNotFound(String),

    #[error("alias not found: {0}")]
    AliasNotFound(String),

    #[error("ambiguous alias '{query}' in scope '{scope}': {candidates:?}")]
    AmbiguousAlias { query: String, scope: String, candidates: Vec<String> },

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
