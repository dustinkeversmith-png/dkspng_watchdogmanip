Yes — this is exactly where you want a **structured test logging layer**, not random `println!`.

The goal should be:

```text
test run
  -> walk log
  -> navigation/capability availability log
  -> parse log
  -> database table dump log
  -> search/vectorization prep log
```

Use JSON/JSONL so a future “logging agent” can inspect results.

Below is a practical implementation layout.

---

# 1. Add a test logging module

Create:

```text
src/test_logging/
  mod.rs
  model.rs
  writer.rs
```

And export it in `src/lib.rs`:

```rust
pub mod test_logging;
```

---

## `/src/test_logging/mod.rs`

```rust
pub mod model;
pub mod writer;

pub use model::*;
pub use writer::*;
```

---

## `/src/test_logging/model.rs`

```rust
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
```

---

## `/src/test_logging/writer.rs`

```rust
use crate::test_logging::{TestLogSection, TestRunLog};
use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TestLogWriter {
    output_dir: PathBuf,
    run: TestRunLog,
}

impl TestLogWriter {
    pub fn new(
        output_dir: impl Into<PathBuf>,
        test_name: impl Into<String>,
        root_path: Option<PathBuf>,
    ) -> Result<Self> {
        let output_dir = output_dir.into();
        fs::create_dir_all(&output_dir)
            .with_context(|| format!("failed to create log dir {}", output_dir.display()))?;

        let started_at_unix_ms = now_unix_ms();
        let run_id = format!("{}_{}", sanitize_name(&test_name.into()), started_at_unix_ms);

        Ok(Self {
            output_dir,
            run: TestRunLog {
                run_id,
                test_name: "parse_real_path_logging".to_string(),
                started_at_unix_ms,
                root_path,
                sections: Vec::new(),
            },
        })
    }

    pub fn add_section<T: Serialize>(
        &mut self,
        name: impl Into<String>,
        summary: impl Into<String>,
        records: &[T],
    ) -> Result<()> {
        let records = records
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<Value>, _>>()?;

        self.run.sections.push(TestLogSection {
            name: name.into(),
            summary: summary.into(),
            records,
        });

        Ok(())
    }

    pub fn write_json(&self) -> Result<PathBuf> {
        let path = self.output_dir.join(format!("{}.json", self.run.run_id));
        let json = serde_json::to_string_pretty(&self.run)?;
        fs::write(&path, json)?;
        Ok(path)
    }

    pub fn write_section_jsonl<T: Serialize>(
        &self,
        section_name: &str,
        records: &[T],
    ) -> Result<PathBuf> {
        let path = self.output_dir.join(format!(
            "{}__{}.jsonl",
            self.run.run_id,
            sanitize_name(section_name)
        ));

        let mut body = String::new();

        for record in records {
            body.push_str(&serde_json::to_string(record)?);
            body.push('\n');
        }

        fs::write(&path, body)?;
        Ok(path)
    }
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
```

Tiny bug in `new`: `test_name.into()` is consumed. Use this fixed version instead:

```rust
pub fn new(
    output_dir: impl Into<PathBuf>,
    test_name: impl Into<String>,
    root_path: Option<PathBuf>,
) -> Result<Self> {
    let output_dir = output_dir.into();
    let test_name = test_name.into();

    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create log dir {}", output_dir.display()))?;

    let started_at_unix_ms = now_unix_ms();
    let run_id = format!("{}_{}", sanitize_name(&test_name), started_at_unix_ms);

    Ok(Self {
        output_dir,
        run: TestRunLog {
            run_id,
            test_name,
            started_at_unix_ms,
            root_path,
            sections: Vec::new(),
        },
    })
}
```

---

# 2. Add database table dump support

Add these to your `/src/database/model.rs`:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTableDump {
    pub table_name: String,
    pub row_count: usize,
    pub rows: Vec<Value>,
}
```

---

## Add dump methods to `/src/database/sqlite.rs`

Inside `impl CommandSqliteDatabase`:

```rust
use crate::database::model::DatabaseTableDump;
use serde_json::{json, Value};
```

Add:

```rust
pub fn dump_table(&self, table_name: &str, limit: usize) -> anyhow::Result<DatabaseTableDump> {
    let allowed_tables = [
        "sources",
        "parsed_commands",
        "command_parameters",
        "command_tags",
        "command_references",
        "command_statuses",
    ];

    if !allowed_tables.contains(&table_name) {
        anyhow::bail!("table dump refused for unknown or unsafe table: {table_name}");
    }

    let count_sql = format!("SELECT COUNT(*) FROM {table_name}");
    let row_count: usize = self.conn.query_row(&count_sql, [], |row| row.get(0))?;

    let sql = format!("SELECT * FROM {table_name} LIMIT {limit}");
    let mut stmt = self.conn.prepare(&sql)?;

    let column_names = stmt
        .column_names()
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();

    let rows = stmt.query_map([], |row| {
        let mut object = serde_json::Map::new();

        for (index, column_name) in column_names.iter().enumerate() {
            let value = sqlite_value_to_json(row, index)?;
            object.insert(column_name.clone(), value);
        }

        Ok(Value::Object(object))
    })?;

    let mut out = Vec::new();

    for row in rows {
        out.push(row?);
    }

    Ok(DatabaseTableDump {
        table_name: table_name.to_string(),
        row_count,
        rows: out,
    })
}

pub fn dump_core_tables(&self, limit_per_table: usize) -> anyhow::Result<Vec<DatabaseTableDump>> {
    let tables = [
        "sources",
        "parsed_commands",
        "command_parameters",
        "command_tags",
        "command_references",
        "command_statuses",
    ];

    tables
        .iter()
        .map(|table| self.dump_table(table, limit_per_table))
        .collect()
}
```

Add this helper near the bottom:

```rust
fn sqlite_value_to_json(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<serde_json::Value> {
    use rusqlite::types::ValueRef;

    match row.get_ref(index)? {
        ValueRef::Null => Ok(serde_json::Value::Null),
        ValueRef::Integer(value) => Ok(serde_json::json!(value)),
        ValueRef::Real(value) => Ok(serde_json::json!(value)),
        ValueRef::Text(value) => {
            let text = String::from_utf8_lossy(value).to_string();

            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&text) {
                Ok(json_value)
            } else {
                Ok(serde_json::json!(text))
            }
        }
        ValueRef::Blob(value) => Ok(serde_json::json!({
            "blob_len": value.len()
        })),
    }
}
```

This gives you viewable JSON snapshots of each table.

---

# 3. Add navigation/capability logging test stubs

Since the test should not require actually opening Explorer every time, log availability separately from execution.

Create a simple adapter/capability probe in the test for now:

```rust
fn navigation_capability_probe() -> Vec<NavigationCapabilityLog> {
    vec![
        NavigationCapabilityLog {
            capability: "explorer_navigation".to_string(),
            available: cfg!(target_os = "windows"),
            adapter: if cfg!(target_os = "windows") {
                Some("windows_explorer_adapter".to_string())
            } else {
                None
            },
            details: if cfg!(target_os = "windows") {
                "Explorer navigation should be available through Windows process spawning."
                    .to_string()
            } else {
                "Explorer navigation is Windows-specific; use generic folder opener adapter here."
                    .to_string()
            },
        },
        NavigationCapabilityLog {
            capability: "process_management".to_string(),
            available: true,
            adapter: Some("std_process_command".to_string()),
            details: "std::process::Command is available for dry-run process plan tests."
                .to_string(),
        },
        NavigationCapabilityLog {
            capability: "folder_open_plan".to_string(),
            available: true,
            adapter: Some("navigation_engine_dry_run".to_string()),
            details: "Navigation engine can produce open-folder plans without executing them."
                .to_string(),
        },
    ]
}
```

Later, wire this to your actual capability engine. For now, this gives you the exact log shape you want.

---

# 4. Add enhanced real-path logging test

Update or add:

```text
tests/parse/logged_real_path_tree_database_test.rs
```

And include it from `/tests/parse.rs`:

```rust
#[path = "parse/logged_real_path_tree_database_test.rs"]
mod logged_real_path_tree_database_test;
```

---

## `/tests/parse/logged_real_path_tree_database_test.rs`

```rust
use macro_os_engines::database::{
    new_record_from_parsed_command, CommandSearchOptions, CommandSqliteDatabase,
};
use macro_os_engines::parse::Parser;
use macro_os_engines::test_logging::{
    DatabaseTableLog, NavigationCapabilityLog, ParseCommandLogRecord, ParseFileLogRecord,
    SearchLogRecord, TestLogWriter, WalkEfficacySummary, WalkLogRecord,
};
use macro_os_engines::walk::{TreeWalker, TreeWalkerConfig};
use serde_json::json;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use tempfile::tempdir;

#[test]
fn logged_real_path_walk_parse_database_efficacy() {
    let root = match env::var("PARSE_TEST_ROOT") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Skipping test. Set PARSE_TEST_ROOT to a real folder path to enable it.");
            return;
        }
    };

    let log_dir = env::var("PARSE_TEST_LOG_DIR").unwrap_or_else(|_| "target/test-logs/parse".to_string());

    let mut log_writer = TestLogWriter::new(
        &log_dir,
        "logged_real_path_walk_parse_database_efficacy",
        Some(root.clone().into()),
    )
    .expect("test log writer should initialize");

    let walker = TreeWalker::new(
        TreeWalkerConfig::new(&root)
            .recursive(true)
            .include_extensions(["md", "txt", "rs"])
            .ignore_dirs(["target", ".git", "node_modules", "dist", "build"]),
    );

    let walked = walker.walk().expect("tree walker should walk real path");

    let walk_records: Vec<WalkLogRecord> = walked
        .files
        .iter()
        .map(|file| WalkLogRecord {
            source_name: file.source_name.clone(),
            path: file.path.clone(),
            depth: file.depth,
            extension: file.extension.clone(),
            size_bytes: file.size_bytes,
            included: true,
            reason: "matched include extension and ignore rules".to_string(),
        })
        .collect();

    let extensions_seen = walked
        .files
        .iter()
        .filter_map(|file| file.extension.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let walk_summary = WalkEfficacySummary {
        root_path: walked.root.clone(),
        total_files_seen: walked.files.len(),
        included_files: walked.files.len(),
        skipped_files: 0,
        max_depth_seen: walked.files.iter().map(|file| file.depth).max().unwrap_or(0),
        extensions_seen,
    };

    log_writer
        .add_section(
            "walk_summary",
            "High-level filesystem walk efficacy summary.",
            &[walk_summary],
        )
        .unwrap();

    log_writer
        .add_section(
            "walk_files",
            "Every file included by the walker.",
            &walk_records,
        )
        .unwrap();

    let capability_records = navigation_capability_probe();

    log_writer
        .add_section(
            "navigation_capability_probe",
            "Dry-run capability probe for navigation/process management availability.",
            &capability_records,
        )
        .unwrap();

    let parser = Parser::default();

    let temp = tempdir().unwrap();
    let db_path = temp.path().join("parsed_commands.sqlite");
    let db = CommandSqliteDatabase::open(&db_path).expect("sqlite database should open");

    let mut parse_file_records = Vec::new();
    let mut parse_command_records = Vec::new();
    let mut inserted_ids = Vec::new();

    for file in &walked.files {
        let Ok(text) = fs::read_to_string(&file.path) else {
            continue;
        };

        let output = parser.parse(file.source_name.clone(), text);

        parse_file_records.push(ParseFileLogRecord {
            source_name: file.source_name.clone(),
            file_path: file.path.clone(),
            command_count: output.commands.len(),
            warning_count: output.warnings.len(),
            error_count: output.errors.len(),
        });

        for parsed_command in output.commands {
            parse_command_records.push(ParseCommandLogRecord {
                source_name: file.source_name.clone(),
                file_path: file.path.clone(),
                command_id: parsed_command.id.clone(),
                kind: format!("{:?}", parsed_command.kind),
                raw_identity: parsed_command.raw_identity.clone(),
                title: parsed_command.title.clone(),
                source_trace: parsed_command.source_trace.clone(),
                content_preview: parsed_command
                    .content
                    .lines()
                    .find(|line| !line.trim().is_empty())
                    .unwrap_or_default()
                    .chars()
                    .take(160)
                    .collect(),
                parameters: parsed_command.parameters.clone(),
                tags: parsed_command.tags.clone(),
                references: parsed_command.references.clone(),
                statuses: parsed_command.statuses.clone(),
            });

            let record = new_record_from_parsed_command(file.source_name.clone(), parsed_command);

            let inserted_id = db
                .insert_command(&record)
                .expect("individual command should insert");

            inserted_ids.push(inserted_id);
        }
    }

    log_writer
        .add_section(
            "parse_files",
            "Per-file parser output counts and file references.",
            &parse_file_records,
        )
        .unwrap();

    log_writer
        .add_section(
            "parse_commands",
            "Every parsed command with source path and trace location.",
            &parse_command_records,
        )
        .unwrap();

    let table_dumps = db
        .dump_core_tables(100)
        .expect("database table dumps should work");

    let table_logs: Vec<DatabaseTableLog> = table_dumps
        .into_iter()
        .map(|dump| DatabaseTableLog {
            table_name: dump.table_name,
            row_count: dump.row_count,
            rows: dump.rows,
        })
        .collect();

    log_writer
        .add_section(
            "database_tables",
            "JSON snapshots of database tables for search/vectorization planning.",
            &table_logs,
        )
        .unwrap();

    let stats = db.stats().expect("database stats should load");

    let parser_hits = db
        .search(CommandSearchOptions {
            query: Some("parser".to_string()),
            limit: Some(25),
            ..Default::default()
        })
        .expect("parser search should run");

    let database_hits = db
        .search(CommandSearchOptions {
            query: Some("database".to_string()),
            limit: Some(25),
            ..Default::default()
        })
        .expect("database search should run");

    let search_records = vec![
        SearchLogRecord {
            query: "parser".to_string(),
            hit_count: parser_hits.len(),
            hits: parser_hits.into_iter().map(|hit| json!(hit)).collect(),
        },
        SearchLogRecord {
            query: "database".to_string(),
            hit_count: database_hits.len(),
            hits: database_hits.into_iter().map(|hit| json!(hit)).collect(),
        },
    ];

    log_writer
        .add_section(
            "database_searches",
            "Example search results for future search/vectorization improvement tests.",
            &search_records,
        )
        .unwrap();

    let log_path = log_writer.write_json().expect("test log should write");

    println!("test log written to: {}", log_path.display());
    println!("temporary sqlite db path: {}", db_path.display());
    println!("walked files: {}", walked.files.len());
    println!("parsed commands: {}", parse_command_records.len());
    println!("inserted commands: {}", inserted_ids.len());
    println!("database stats: {stats:#?}");

    assert_eq!(stats.command_count, inserted_ids.len() as i64);
}

fn navigation_capability_probe() -> Vec<NavigationCapabilityLog> {
    vec![
        NavigationCapabilityLog {
            capability: "explorer_navigation".to_string(),
            available: cfg!(target_os = "windows"),
            adapter: if cfg!(target_os = "windows") {
                Some("windows_explorer_adapter".to_string())
            } else {
                None
            },
            details: if cfg!(target_os = "windows") {
                "Explorer navigation should be available through Windows process spawning."
                    .to_string()
            } else {
                "Explorer navigation is Windows-specific; use generic folder opener adapter here."
                    .to_string()
            },
        },
        NavigationCapabilityLog {
            capability: "process_management".to_string(),
            available: true,
            adapter: Some("std_process_command".to_string()),
            details: "std::process::Command is available for dry-run process plan tests."
                .to_string(),
        },
        NavigationCapabilityLog {
            capability: "folder_open_plan".to_string(),
            available: true,
            adapter: Some("navigation_engine_dry_run".to_string()),
            details: "Navigation engine can produce open-folder plans without executing them."
                .to_string(),
        },
    ]
}
```

---

# 5. Run with log output path

PowerShell:

```powershell
$env:PARSE_TEST_ROOT="C:\Users\YourName\projects\your-project"
$env:PARSE_TEST_LOG_DIR="target\test-logs\parse"
cargo test --test parse logged_real_path_walk_parse_database_efficacy -- --nocapture
```

Bash:

```bash
PARSE_TEST_ROOT="./your-project" \
PARSE_TEST_LOG_DIR="target/test-logs/parse" \
cargo test --test parse logged_real_path_walk_parse_database_efficacy -- --nocapture
```

Output will look like:

```text
target/test-logs/parse/logged_real_path_walk_parse_database_efficacy_1780000000000.json
```

---

# 6. Suggested JSON log shape

The final JSON will be structured like:

```json
{
  "run_id": "logged_real_path_walk_parse_database_efficacy_1780000000000",
  "test_name": "logged_real_path_walk_parse_database_efficacy",
  "started_at_unix_ms": 1780000000000,
  "root_path": "C:/projects/my-project",
  "sections": [
    {
      "name": "walk_summary",
      "summary": "High-level filesystem walk efficacy summary.",
      "records": []
    },
    {
      "name": "walk_files",
      "summary": "Every file included by the walker.",
      "records": []
    },
    {
      "name": "navigation_capability_probe",
      "summary": "Dry-run capability probe for navigation/process management availability.",
      "records": []
    },
    {
      "name": "parse_files",
      "summary": "Per-file parser output counts and file references.",
      "records": []
    },
    {
      "name": "parse_commands",
      "summary": "Every parsed command with source path and trace location.",
      "records": []
    },
    {
      "name": "database_tables",
      "summary": "JSON snapshots of database tables for search/vectorization planning.",
      "records": []
    },
    {
      "name": "database_searches",
      "summary": "Example search results for future search/vectorization improvement tests.",
      "records": []
    }
  ]
}
```

That is good for an evaluator agent because it can inspect sections independently.

---

# 7. One important improvement to the walker

Right now, the walker only logs included files. If you want true walk efficacy, you should also log skipped files and skipped directories.

Upgrade `WalkedFile` later into:

```rust
pub enum WalkDecisionKind {
    IncludedFile,
    SkippedFile,
    SkippedDirectory,
}
```

And emit:

```rust
pub struct WalkDecision {
    pub path: PathBuf,
    pub source_name: String,
    pub kind: WalkDecisionKind,
    pub reason: String,
    pub depth: usize,
}
```

Then your logs can answer:

```text
Why was this file included?
Why was this folder ignored?
Did target/ really get skipped?
Did node_modules/ really get skipped?
How many parseable files were missed?
```

For now, the logging test above gets you a usable first pass.