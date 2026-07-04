use serde::{Deserialize, Serialize};

use crate::parse::boundary::CommandBlock;
use crate::parse::model::SourceDocument;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HierarchySignalKind {
    MarkdownHeading,
    NumberedList,
    BulletList,
    Indentation,
    CommandNesting,
    BoundaryContainment,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchySignal {
    pub kind: HierarchySignalKind,
    pub line: usize,
    pub level: usize,
    pub label: Option<String>,
    pub raw: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHierarchyLink {
    pub command_id: String,
    pub parent_id: Option<String>,
    pub child_ids: Vec<String>,
    pub hierarchy_path: Vec<String>,
    pub signal_kinds: Vec<HierarchySignalKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParseHierarchyNode {
    pub command_id: String,
    pub parent_id: Option<String>,
    pub child_ids: Vec<String>,
    pub hierarchy_path: Vec<String>,
    #[serde(default)]
    pub signal_kinds: Vec<HierarchySignalKind>,
}

pub trait HierarchyDetector {
    fn name(&self) -> &'static str;
    fn detect(&self, document: &SourceDocument, blocks: &[CommandBlock]) -> Vec<HierarchySignal>;
}
