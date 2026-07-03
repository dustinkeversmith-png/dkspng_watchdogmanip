use macro_os_engines::database::{
    new_record_from_parsed_command, CommandSearchOptions, CommandSqliteDatabase,
};
use macro_os_engines::parse::MacroPipeline;
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
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::tempdir;

#[test]
fn logged_real_path_walk_macropipeline_database_efficacy() {
    let root = env::var("PARSE_TEST_ROOT")
        .unwrap_or_else(|_| "C:\\Users\\Cutie Magic 500\\Desktop\\desktop_temp_docs".to_string());
    let root_path = PathBuf::from(&root);
    if !root_path.exists() {
        eprintln!("skipping real-path test: PARSE_TEST_ROOT does not exist ({root})");
        return;
    }

    let log_dir = env::var("PARSE_TEST_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target/test-logs/parse"));

    let walker = TreeWalker::new(
        TreeWalkerConfig::new(&root_path)
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
        max_depth_seen: walked
            .files
            .iter()
            .map(|file| file.depth)
            .max()
            .unwrap_or(0),
        extensions_seen,
    };

    let pipeline = MacroPipeline::default();

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

        let output = pipeline.parse(file.source_name.clone(), text);

        let error_count = output
            .diagnostics
            .iter()
            .filter(|d| matches!(d.severity, macro_os_engines::parse::Severity::Error))
            .count();

        parse_file_records.push(ParseFileLogRecord {
            source_name: file.source_name.clone(),
            file_path: file.path.clone(),
            command_count: output.commands.len(),
            warning_count: output.diagnostics.len(),
            error_count,
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

    let dump_limit = inserted_ids.len().max(100);
    let table_dumps = db
        .dump_core_tables(dump_limit)
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

    let mut output = TestOutputBuilder::new(TestRunInfo {
        run_ref: String::new(),
        test_name: "logged_real_path_walk_macropipeline_database_efficacy".to_string(),
        started_at_unix_ms: now_unix_ms(),
        root_path: Some(root_path.clone()),
        temporary_sqlite_db_path: Some(db_path.clone()),
        log_kind: "real_path_walk_parse_database_efficacy".to_string(),
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
    println!("temporary sqlite db path: {}", db_path.display());
    println!("walked files: {}", document.summary.file_count);
    println!(
        "pipeline parsed commands: {}",
        document.summary.parsed_command_count
    );
    println!(
        "inserted commands: {}",
        document.summary.inserted_command_count
    );
    println!("database stats: {:#?}", document.sections.database.stats);
    println!("diagnostics: {}", document.summary.diagnostic_count);

    assert_eq!(
        document.sections.database.stats.command_count,
        inserted_ids.len() as i64
    );
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
