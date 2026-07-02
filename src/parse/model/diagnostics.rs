use serde::{Deserialize, Serialize};

use super::TextSpan;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub span: Option<TextSpan>,
}

impl Diagnostic {
    pub fn info(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<TextSpan>,
    ) -> Self {
        Self {
            severity: Severity::Info,
            code: code.into(),
            message: message.into(),
            span,
        }
    }
    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<TextSpan>,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            code: code.into(),
            message: message.into(),
            span,
        }
    }
    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<TextSpan>,
    ) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            span,
        }
    }
}
