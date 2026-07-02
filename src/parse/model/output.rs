use serde::{Deserialize, Serialize};

use super::{Diagnostic, ParsedCommand};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseOutput {
    pub source_name: String,
    pub commands: Vec<ParsedCommand>,
    pub diagnostics: Vec<Diagnostic>,
}

impl ParseOutput {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| matches!(d.severity, super::Severity::Error))
    }
}
