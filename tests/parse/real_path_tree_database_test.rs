use macro_os_engines::parse::database::{CommandSearchOptions, ParseCommandStore};
use macro_os_engines::parse::model::{ParsedCommand, SourceDocument};
use macro_os_engines::parse::{MacroPipeline, PipelineConfig};
use macro_os_engines::test_logging::{
    DatabaseTableLog, ParseCommandLogRecord, ParseFileLogRecord, SearchLogRecord,
    TestOutputBuilder, TestOutputWriter, TestRunInfo, WalkEfficacySummary, WalkLogRecord,
};
use macro_os_engines::walk::{TreeWalker, TreeWalkerConfig};
use serde_json::json;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tempfile::tempdir;

const LOG_DIR: &str = "target/test-logs/parser_real_path_test";
const MAX_FILE_BYTES: u64 = 1024 * 1024;

fn parse_example_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("parse_example")
}

fn fast_pipeline() -> MacroPipeline {
    MacroPipeline::with_defaults(PipelineConfig {
        enable_loose_inference: false,
        preserve_unknown_commands: true,
    })
}

struct ParsedFileOutcome {
    source_name: String,
    file_path: PathBuf,
    commands: Vec<ParsedCommand>,
    warning_count: usize,
    error_count: usize,
}

#[test]
fn logged_real_path_walk_macropipeline_database_efficacy() {
    let root_path = parse_example_root();
    assert!(
        root_path.exists(),
        "examples/parse_example should exist at {}",
        root_path.display()
    );

    let log_dir = env::var("PARSE_TEST_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(LOG_DIR));

    let walker = TreeWalker::new(
        TreeWalkerConfig::new(&root_path)
            .recursive(true)
            .include_extensions(["md", "txt"])
            .ignore_dirs([
                "target",
                ".git",
                "node_modules",
                "dist",
                "build",
                "src-tauri",
            ]),
    );

    let walked = walker
        .walk()
        .expect("tree walker should walk parse example root");

    let parseable: Vec<_> = walked
        .files
        .iter()
        .filter(|file| file.size_bytes <= MAX_FILE_BYTES)
        .filter(|file| !file.source_name.eq_ignore_ascii_case("README.md"))
        .cloned()
        .collect();

    assert!(
        !parseable.is_empty(),
        "expected md/txt files under examples/parse_example"
    );

    let walk_started = Instant::now();
    let walk_records: Vec<WalkLogRecord> = walked
        .files
        .iter()
        .map(|file| WalkLogRecord {
            source_name: file.source_name.clone(),
            path: file.path.clone(),
            depth: file.depth,
            extension: file.extension.clone(),
            size_bytes: file.size_bytes,
            included: parseable.iter().any(|f| f.path == file.path),
            reason: if file.size_bytes > MAX_FILE_BYTES {
                format!("skipped: file exceeds {MAX_FILE_BYTES} byte cap")
            } else {
                "matched include extension and ignore rules".to_string()
            },
        })
        .collect();

    let extensions_seen = parseable
        .iter()
        .filter_map(|file| file.extension.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let walk_summary = WalkEfficacySummary {
        root_path: walked.root.clone(),
        total_files_seen: walked.files.len(),
        included_files: parseable.len(),
        skipped_files: walked.files.len().saturating_sub(parseable.len()),
        max_depth_seen: parseable
            .iter()
            .map(|file| file.depth)
            .max()
            .unwrap_or(0),
        extensions_seen,
    };

    let parse_started = Instant::now();
    let outcomes = Arc::new(Mutex::new(Vec::<ParsedFileOutcome>::new()));
    let thread_count = env::var("PARSE_TEST_THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4));

    std::thread::scope(|scope| {
        for thread_idx in 0..thread_count {
            let files = &parseable;
            let outcomes = Arc::clone(&outcomes);
            scope.spawn(move || {
                let pipeline = fast_pipeline();
                for (idx, file) in files.iter().enumerate() {
                    if idx % thread_count != thread_idx {
                        continue;
                    }
                    let Ok(text) = fs::read_to_string(&file.path) else {
                        continue;
                    };
                    let doc = SourceDocument::with_path(
                        file.source_name.clone(),
                        Some(file.path.clone()),
                        text,
                    );
                    let output = pipeline.parse_document(doc);
                    let error_count = output
                        .diagnostics
                        .iter()
                        .filter(|d| {
                            matches!(d.severity, macro_os_engines::parse::Severity::Error)
                        })
                        .count();
                    outcomes
                        .lock()
                        .expect("parse outcomes lock")
                        .push(ParsedFileOutcome {
                            source_name: file.source_name.clone(),
                            file_path: file.path.clone(),
                            commands: output.commands,
                            warning_count: output.diagnostics.len(),
                            error_count,
                        });
                }
            });
        }
    });

    let mut parsed_files = std::mem::take(&mut *outcomes.lock().expect("parse outcomes lock"));
    parsed_files.sort_by(|a, b| a.source_name.cmp(&b.source_name));
    let parse_elapsed = parse_started.elapsed();

    let temp = tempdir().unwrap();
    let db_path = temp.path().join("parsed_commands.sqlite");
    let db = ParseCommandStore::open(&db_path).expect("sqlite database should open");
    db.configure_bulk_load()
        .expect("bulk load pragmas should apply");

    let mut parse_file_records = Vec::with_capacity(parsed_files.len());
    let mut parse_command_records = Vec::new();
    let mut inserted_ids = Vec::new();

    let db_started = Instant::now();
    db.begin_batch().expect("batch begin");
    for file in &parsed_files {
        parse_file_records.push(ParseFileLogRecord {
            source_name: file.source_name.clone(),
            file_path: file.file_path.clone(),
            command_count: file.commands.len(),
            warning_count: file.warning_count,
            error_count: file.error_count,
        });

        for parsed_command in &file.commands {
            parse_command_records.push(ParseCommandLogRecord {
                source_name: file.source_name.clone(),
                file_path: file.file_path.clone(),
                command_id: parsed_command.id.clone(),
                kind: format!("{:?}", parsed_command.kind),
                raw_identity: parsed_command.raw_identity.clone(),
                title: parsed_command.title.clone(),
                source_trace: parsed_command.source_trace.clone(),
                location: parsed_command.location.clone(),
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

            let inserted_id = db
                .insert_parsed_command(file.source_name.clone(), parsed_command.clone())
                .expect("individual command should insert");
            inserted_ids.push(inserted_id);
        }
    }
    db.commit_batch().expect("batch commit");
    let db_elapsed = db_started.elapsed();

    assert!(
        !parse_command_records.is_empty(),
        "expected parse_example tree to yield parsed commands"
    );
    assert!(
        parse_command_records
            .iter()
            .all(|record| record.location.file_path.is_some()),
        "every command should carry structured file_path in SourceLocation"
    );
    assert!(
        parse_command_records
            .iter()
            .all(|record| record.source_trace.contains(':')),
        "source_trace should use source_name:line reference format"
    );

    let table_dumps = db
        .dump_core_tables(25)
        .expect("database table dumps should work");

    let table_logs: Vec<DatabaseTableLog> = table_dumps
        .into_iter()
        .map(|dump| DatabaseTableLog {
            table_name: dump.table_name,
            row_count: dump.row_count,
            rows: dump.rows,
        })
        .collect();

    let stats = db.stats().expect("database stats should load");

    let parser_hits = db
        .search(CommandSearchOptions {
            query: Some("parser".to_string()),
            limit: Some(25),
            ..Default::default()
        })
        .expect("parser search should run");

    let reference_hits = db
        .search(CommandSearchOptions {
            query: Some("reference".to_string()),
            limit: Some(25),
            ..Default::default()
        })
        .expect("reference search should run");

    let search_records = vec![
        SearchLogRecord {
            query: "parser".to_string(),
            hit_count: parser_hits.len(),
            hits: parser_hits.into_iter().map(|hit| json!(hit)).collect(),
        },
        SearchLogRecord {
            query: "reference".to_string(),
            hit_count: reference_hits.len(),
            hits: reference_hits.into_iter().map(|hit| json!(hit)).collect(),
        },
    ];

    let mut output = TestOutputBuilder::new(TestRunInfo {
        run_ref: String::new(),
        test_name: "logged_real_path_walk_macropipeline_database_efficacy".to_string(),
        started_at_unix_ms: now_unix_ms(),
        root_path: Some(root_path.clone()),
        temporary_sqlite_db_path: Some(db_path.clone()),
        log_kind: "parse_example_walk_parse_database_efficacy".to_string(),
    });

    output.add_walk_summary(walk_summary);
    output.add_walk_records(walk_records);
    output.add_parse_file_records(parse_file_records);
    output.add_parse_command_records(parse_command_records);
    output.add_inserted_command_count(inserted_ids.len());
    output.add_database_stats(stats);
    output.add_database_tables(table_logs);
    output.add_search_records(search_records);

    let document = output.build();

    let writer = TestOutputWriter::new(
        &log_dir,
        "logged_real_path_walk_macropipeline_database_efficacy",
    )
    .expect("output writer should initialize");

    let log_path = writer
        .write_json(&document)
        .expect("test output should write");

    println!("test log written to: {}", log_path.display());
    println!("parse example root: {}", root_path.display());
    println!("walked files (total / parsed): {} / {}", walked.files.len(), parsed_files.len());
    println!("parse threads: {thread_count}");
    println!("parse phase: {:.2?}", parse_elapsed);
    println!("database insert phase: {:.2?}", db_elapsed);
    println!("total since walk: {:.2?}", walk_started.elapsed());
    println!(
        "pipeline parsed commands: {}",
        document.summary.parsed_command_count
    );
    println!(
        "inserted commands: {}",
        document.summary.inserted_command_count
    );

    assert_eq!(
        document.sections.database.stats.command_count,
        inserted_ids.len() as i64
    );
    assert!(
        document
            .sections
            .parse
            .commands
            .iter()
            .all(|cmd| cmd.source_span.is_some()),
        "cross-ref output should resolve source_span from SourceLocation"
    );
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
