use serde::{Deserialize, Serialize};

use crate::parse::model::SourceDocument;

pub type ParseDocumentInput = SourceDocument;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedKind {
    ExplicitCommand,
    ChainedCommand,
    InlineStatus,
    ReferenceMarker,
    CurrentMarker,
    UnknownAtCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedSeed {
    pub kind: SeedKind,
    pub raw: String,
    pub normalized_identity: String,
    pub line: usize,
    pub column: usize,
    pub confidence: f32,
}
