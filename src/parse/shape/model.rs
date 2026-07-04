use serde::{Deserialize, Serialize};

use crate::parse::boundary::BodyShapeHint;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandShapeKind {
    InlineTitle,
    InlineParameters,
    KeyValueMembers,
    BracketedBody,
    IndentedBody,
    HeadingAttached,
    ListAttached,
    ProseOnly,
    EmptyBody,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TitleCandidateKind {
    InlineAfterCommand,
    TitleMember,
    FirstNonEmptyBodyLine,
    HeadingContext,
    ListItemText,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterShapeKind {
    None,
    SingleLooseParameter,
    MultipleLooseParameters,
    KeyValueParameter,
    BracketedArray,
    InlineQuotedString,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleCandidate {
    pub kind: TitleCandidateKind,
    pub text: String,
    pub line: usize,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandShapeAnalysis {
    pub command_id: Option<String>,
    pub shape_kinds: Vec<CommandShapeKind>,
    pub parameter_shape: ParameterShapeKind,
    pub body_shape: BodyShapeHint,
    pub title_candidates: Vec<TitleCandidate>,
    pub confidence: f32,
    pub diagnostics: Vec<String>,
}
