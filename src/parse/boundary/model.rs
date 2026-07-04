use serde::{Deserialize, Serialize};

use crate::parse::model::{BoundaryKind, CommandSeed, SourceDocument, SourceLine, TextSpan};

pub type ParseDocumentInput = SourceDocument;

/// Rich boundary metadata kind (candidate shape / termination style).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryMetadataKind {
    CommandSeedLine,
    InlineCommand,
    SameLinePayload,
    NextLineBody,
    IndentedBody,
    BracketedBody,
    FencedBody,
    HeadingSection,
    ListItem,
    NumberedListItem,
    BulletListItem,
    BlankLineTerminated,
    NextSeedTerminated,
    OutdentTerminated,
    HeadingTerminated,
    EndOfFileTerminated,
    Ambiguous,
    Unknown,
}

/// Legacy marker kind kept for existing tests and strategy names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryMarkerKind {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyDirection {
    InlineRight,
    Below,
    Above,
    Around,
    None,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyShapeHint {
    SingleLine,
    MultiLineBlock,
    IndentedBlock,
    BracketedBlock,
    MarkdownSection,
    ListItemBody,
    KeyValueBlock,
    FreeformProse,
    Empty,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryEvidence {
    pub kind: String,
    pub line: usize,
    pub column: usize,
    pub text_preview: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryCandidate {
    pub id: String,
    pub kind: BoundaryMarkerKind,
    pub metadata_kind: BoundaryMetadataKind,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: Option<usize>,
    pub end_column: Option<usize>,
    pub body_direction: BodyDirection,
    pub body_shape_hint: BodyShapeHint,
    pub confidence: f32,
    pub reason: String,
    pub evidence: Vec<BoundaryEvidence>,
    pub diagnostics: Vec<String>,
}

impl BoundaryCandidate {
    pub fn build(
        strategy: &str,
        line: &SourceLine,
        kind: BoundaryMarkerKind,
        metadata_kind: BoundaryMetadataKind,
        body_direction: BodyDirection,
        body_shape_hint: BodyShapeHint,
        confidence: f32,
        reason: impl Into<String>,
    ) -> Self {
        let column = line.text.find('@').unwrap_or(0);
        let preview: String = line.text.chars().take(120).collect();
        let reason = reason.into();
        Self {
            id: format!("{strategy}:L{}", line.number),
            kind,
            metadata_kind,
            start_line: line.number,
            start_column: column,
            end_line: None,
            end_column: None,
            body_direction,
            body_shape_hint,
            confidence,
            reason: reason.clone(),
            evidence: vec![BoundaryEvidence {
                kind: strategy.to_string(),
                line: line.number,
                column,
                text_preview: preview,
                reason,
            }],
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandBlock {
    pub seed: CommandSeed,
    pub body_lines: Vec<String>,
    pub span: TextSpan,
    pub boundary_kind: BoundaryKind,
    pub location: crate::parse::model::SourceLocation,
    pub shape_analysis: Option<crate::parse::shape::CommandShapeAnalysis>,
}
