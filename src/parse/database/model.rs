use crate::parse::model::{CommandKind, ParsedCommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    pub file_path: Option<String>,
    pub start_line: Option<usize>,
    pub start_column: Option<usize>,
    pub end_line: Option<usize>,
    pub end_column: Option<usize>,
}

impl NewParsedCommandRecord {
    pub fn from_parsed(source_name: impl Into<String>, command: ParsedCommand) -> Self {
        let file_path = command
            .location
            .file_path
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned());
        Self {
            source_name: source_name.into(),
            command_id: command.id,
            kind: command.kind,
            raw_identity: command.raw_identity,
            title: command.title,
            description: command.description,
            content: command.content,
            members: command.members,
            parameters: command.parameters,
            tags: command.tags,
            references: command.references,
            statuses: command.statuses,
            source_trace: command.source_trace,
            file_path,
            start_line: Some(command.location.start_line),
            start_column: Some(command.location.start_column),
            end_line: command.location.end_line,
            end_column: command.location.end_column,
        }
    }
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
    pub file_path: Option<String>,
    pub start_line: Option<usize>,
    pub start_column: Option<usize>,
    pub end_line: Option<usize>,
    pub end_column: Option<usize>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTableDump {
    pub table_name: String,
    pub row_count: usize,
    pub rows: Vec<Value>,
}
