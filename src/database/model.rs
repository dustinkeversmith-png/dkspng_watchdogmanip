use crate::parse::model::CommandKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewParsedCommandRecord {
    pub source_name: String,
    pub command_id: String,
    pub kind: CommandKind,
    pub raw_identity: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: String,
    pub members: BTreeMap<String, serde_json::Value>,
    pub parameters: Vec<String>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub statuses: Vec<String>,
    pub source_trace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredParsedCommandRecord {
    pub id: i64,
    pub source_name: String,
    pub command_id: String,
    pub kind: CommandKind,
    pub raw_identity: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: String,
    pub members: BTreeMap<String, serde_json::Value>,
    pub parameters: Vec<String>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub statuses: Vec<String>,
    pub source_trace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSearchHit {
    pub id: i64,
    pub source_name: String,
    pub command_id: String,
    pub kind: CommandKind,
    pub raw_identity: String,
    pub title: Option<String>,
    pub content_preview: String,
    pub score: i64,
}

#[derive(Debug, Clone, Default)]
pub struct CommandSearchOptions {
    pub query: Option<String>,
    pub kind: Option<CommandKind>,
    pub tag: Option<String>,
    pub reference: Option<String>,
    pub source_name: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub source_count: i64,
    pub command_count: i64,
    pub tag_count: i64,
    pub reference_count: i64,
}



use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTableDump {
    pub table_name: String,
    pub row_count: usize,
    pub rows: Vec<Value>,
}