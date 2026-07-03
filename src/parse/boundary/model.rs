use serde::{Deserialize, Serialize};

use crate::parse::model::SourceDocument;

pub type ParseDocumentInput = SourceDocument;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryKind {
    CommandStart,
    CommandEnd,
    InlineCommand,
    BlockCommand,
    HeadingBoundary,
    IndentationBoundary,
    BlankLineBoundary,
    NextSeedBoundary,
    DelimiterBoundary,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryCandidate {
    pub kind: BoundaryKind,
    pub start_line: usize,
    pub end_line: Option<usize>,
    pub confidence: f32,
    pub reason: String,
}
