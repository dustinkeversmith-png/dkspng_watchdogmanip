use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunLog {
    pub run_id: String,
    pub test_name: String,
    pub started_at_unix_ms: u128,
    pub root_path: Option<PathBuf>,
    pub sections: Vec<TestLogSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestLogSection {
    pub name: String,
    pub summary: String,
    pub records: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkLogRecord {
    pub source_name: String,
    pub path: PathBuf,
    pub depth: usize,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub included: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkEfficacySummary {
    pub root_path: PathBuf,
    pub total_files_seen: usize,
    pub included_files: usize,
    pub skipped_files: usize,
    pub max_depth_seen: usize,
    pub extensions_seen: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationCapabilityLog {
    pub capability: String,
    pub available: bool,
    pub adapter: Option<String>,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseCommandLogRecord {
    pub source_name: String,
    pub file_path: PathBuf,
    pub command_id: String,
    pub kind: String,
    pub raw_identity: String,
    pub title: Option<String>,
    pub source_trace: String,
    pub content_preview: String,
    pub parameters: Vec<String>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub statuses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseFileLogRecord {
    pub source_name: String,
    pub file_path: PathBuf,
    pub command_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTableLog {
    pub table_name: String,
    pub row_count: usize,
    pub rows: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchLogRecord {
    pub query: String,
    pub hit_count: usize,
    pub hits: Vec<Value>,
}