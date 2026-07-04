use crate::parse::database::model::DatabaseStats;
use crate::parse::model::SourceLocation;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub type OutputRef = String;
pub type FileRef = String;
pub type ParseFileRef = String;
pub type CommandRef = String;
pub type DbRef = String;
pub type SearchRef = String;
pub type SearchHitRef = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOutputSchema {
    pub name: String,
    pub version: String,
    pub format: String,
}

impl Default for TestOutputSchema {
    fn default() -> Self {
        Self {
            name: "macro_os_test_output".to_string(),
            version: "0.1.0".to_string(),
            format: "cross_referenced_test_dump".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunInfo {
    pub run_ref: String,
    pub test_name: String,
    pub started_at_unix_ms: u128,
    pub root_path: Option<PathBuf>,
    pub temporary_sqlite_db_path: Option<PathBuf>,
    pub log_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOutputDocument {
    pub schema: TestOutputSchema,
    pub run: TestRunInfo,
    pub sections: TestOutputSections,
    pub indexes: TestOutputIndexes,
    pub links: Vec<TestOutputLink>,
    pub diagnostics: Vec<TestOutputDiagnostic>,
    pub summary: TestOutputSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOutputSections {
    pub walk: CrossRefWalkSection,
    pub parse: CrossRefParseSection,
    pub database: CrossRefDatabaseSection,
    pub searches: Vec<CrossRefSearchLogRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRefWalkSection {
    pub summary: WalkEfficacySummary,
    pub files: Vec<CrossRefWalkRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRefParseSection {
    pub files: Vec<CrossRefParseFileRecord>,
    pub commands: Vec<CrossRefParseCommandRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRefDatabaseSection {
    pub stats: DatabaseStats,
    pub tables: Vec<CrossRefDatabaseTableLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestOutputIndexes {
    pub files_by_source_name: BTreeMap<String, FileRef>,
    pub parse_files_by_file_ref: BTreeMap<FileRef, ParseFileRef>,
    pub commands_by_file_ref: BTreeMap<FileRef, Vec<CommandRef>>,
    pub commands_by_source_name: BTreeMap<String, Vec<CommandRef>>,
    pub commands_by_db_ref: BTreeMap<DbRef, CommandRef>,
    pub db_refs_by_command_ref: BTreeMap<CommandRef, Vec<DbRef>>,
    pub search_hits_by_command_ref: BTreeMap<CommandRef, Vec<SearchHitRef>>,
    pub database_rows_by_table: BTreeMap<String, Vec<DbRef>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOutputLink {
    pub from: OutputRef,
    pub to: OutputRef,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOutputDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub refs: Vec<OutputRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestOutputSummary {
    pub file_count: usize,
    pub parse_file_count: usize,
    pub parsed_command_count: usize,
    pub inserted_command_count: usize,
    pub database_command_count: i64,
    pub search_count: usize,
    pub search_hit_count: usize,
    pub diagnostic_count: usize,
    pub unresolved_reference_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRefWalkRecord {
    pub file_ref: FileRef,
    pub source_name: String,
    pub path: PathBuf,
    pub depth: usize,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub included: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRefParseFileRecord {
    pub parse_file_ref: ParseFileRef,
    pub file_ref: FileRef,
    pub source_name: String,
    pub file_path: PathBuf,
    pub command_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub command_refs: Vec<CommandRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRefParseCommandRecord {
    pub command_ref: CommandRef,
    pub file_ref: FileRef,
    pub parse_file_ref: ParseFileRef,
    pub source_name: String,
    pub file_path: PathBuf,
    pub command_id: String,
    pub kind: String,
    pub raw_identity: String,
    pub title: Option<String>,
    pub source_trace: String,
    pub location: SourceLocation,
    pub source_span: Option<SourceSpan>,
    pub content_preview: String,
    pub parameters: Vec<String>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub statuses: Vec<String>,
    pub db_refs: Vec<DbRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRefDatabaseTableLog {
    pub table_ref: String,
    pub table_name: String,
    pub row_count: usize,
    pub rows: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRefSearchLogRecord {
    pub search_ref: SearchRef,
    pub query: String,
    pub hit_count: usize,
    pub hits: Vec<CrossRefSearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRefSearchHit {
    pub search_hit_ref: SearchHitRef,
    pub command_ref: Option<CommandRef>,
    pub db_ref: Option<DbRef>,
    pub hit: Value,
}

// Input records collected by the test harness before cross-referencing.

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
    pub location: SourceLocation,
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
