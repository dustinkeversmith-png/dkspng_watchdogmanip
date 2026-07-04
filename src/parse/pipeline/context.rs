use crate::parse::model::SourceDocument;
use crate::parse::registry::CommandRegistry;

/// Shared inputs for seed detection, boundary search, and block assembly.
#[derive(Debug, Clone, Copy)]
pub struct ParseContext<'a> {
    pub document: &'a SourceDocument,
    pub command_registry: &'a CommandRegistry,
}

impl<'a> ParseContext<'a> {
    pub fn new(document: &'a SourceDocument, command_registry: &'a CommandRegistry) -> Self {
        Self {
            document,
            command_registry,
        }
    }
}
