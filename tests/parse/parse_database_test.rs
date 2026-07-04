use macro_os_engines::parse::database::{CommandSearchOptions, ParseCommandStore};
use macro_os_engines::parse::{CommandKind, MacroPipeline};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

const LOG_DIR: &str = "target/test-logs/parse_database_test";

fn write_json(path: PathBuf, value: &serde_json::Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create log dir");
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(value).expect("serialize json"),
    )
    .expect("write log file");
}

#[test]
fn parse_database_insert_fetch_search_and_dump() {
    let input = include_str!("../fixtures/deep_nested_macros.md");
    let output = MacroPipeline::default().parse("deep_nested_macros.md", input);

    let temp = tempfile::tempdir().unwrap();
    let db = ParseCommandStore::open(temp.path().join("parse.sqlite")).expect("db opens");

    let mut inserted_ids = Vec::new();
    for command in output.commands {
        let id = db
            .insert_parsed_command("deep_nested_macros.md", command)
            .expect("insert command");
        inserted_ids.push(id);
    }

    let stats = db.stats().expect("stats");
    assert!(stats.command_count >= 8);

    let parser_hits = db
        .search(CommandSearchOptions {
            query: Some("parser".to_string()),
            limit: Some(10),
            ..Default::default()
        })
        .expect("search parser");
    assert!(!parser_hits.is_empty());

    let task_hits = db
        .search(CommandSearchOptions {
            kind: Some(CommandKind::Task),
            limit: Some(10),
            ..Default::default()
        })
        .expect("search task kind");
    assert!(!task_hits.is_empty());

    let dump_limit = inserted_ids.len().max(50);
    let dumps = db.dump_core_tables(dump_limit).expect("dump tables");
    assert!(dumps
        .iter()
        .any(|table| table.table_name == "parsed_commands"));

    let log_dir = PathBuf::from(LOG_DIR);
    write_json(
        log_dir.join("stats.json"),
        &json!({
            "command_count": stats.command_count,
            "source_count": stats.source_count,
            "tag_count": stats.tag_count,
            "reference_count": stats.reference_count,
            "inserted_ids": inserted_ids,
        }),
    );
    write_json(
        log_dir.join("search_parser_hits.json"),
        &serde_json::to_value(&parser_hits).expect("parser hits"),
    );
    write_json(
        log_dir.join("search_task_hits.json"),
        &serde_json::to_value(&task_hits).expect("task hits"),
    );
    write_json(
        log_dir.join("table_dumps.json"),
        &serde_json::to_value(&dumps).expect("table dumps"),
    );
}
