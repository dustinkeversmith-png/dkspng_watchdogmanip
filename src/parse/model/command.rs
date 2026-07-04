use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::TextSpan;
use super::SourceLocation;
use crate::parse::shape::CommandShapeAnalysis;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryKind {
    SameLine,
    UntilNextCommand,
    UntilBlankLine,
    UntilOutdent,
    UntilHeading,
    EndOfDocument,
    LooseObject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandKind {
    Task,
    Idea,
    Deferred,
    Progressive,
    Thought,
    Project,
    Tutorial,
    Prompt,
    Tags,
    Alias,
    Categories,
    Goals,
    Algodocutize,
    Deprecated,
    MacroClipboard,
    Enqueue,
    Groups,
    ObjectiveQueue,
    Reference,
    Before,
    QA,
    Current,
    In,
    Complete,
    Building,
    Adapting,
    Unknown(String),
    Inferred(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSeed {
    pub raw_identity: String,
    pub chain: Vec<String>,
    pub canonical_kind: CommandKind,
    pub inline_payload: String,
    pub start_line_index: usize,
    pub span: TextSpan,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCommand {
    pub id: String,
    pub kind: CommandKind,
    pub raw_identity: String,
    pub aliases_seen: Vec<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: String,
    pub parameters: Vec<String>,
    pub members: BTreeMap<String, serde_json::Value>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub statuses: Vec<String>,
    pub inferred: bool,
    pub confidence: f32,
    pub confidence_reason: String,
    pub boundary_kind: BoundaryKind,
    pub span: TextSpan,
    pub source_trace: String,
    pub location: SourceLocation,
    #[serde(default)]
    pub shape_analysis: Option<CommandShapeAnalysis>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub child_ids: Vec<String>,
    #[serde(default)]
    pub hierarchy_path: Vec<String>,
    #[serde(default)]
    pub heading_context: Vec<String>,
    #[serde(default)]
    pub list_context: Option<String>,
}
