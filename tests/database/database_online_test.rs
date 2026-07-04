use macro_os_engines::database::{
    CommandSearchOptions, CommandSqliteDatabase, NewParsedCommandRecord,
};
use macro_os_engines::parse::CommandKind;
use std::collections::BTreeMap;
use std::env;
use tempfile::tempdir;

#[test]
fn sqlite_database_is_online_in_memory() {
    let db = CommandSqliteDatabase::open_memory().expect("memory database should open");

    let health = db.health_check().expect("health check should run");

    assert!(health.online);
    assert!(
        health.required_tables_present,
        "missing tables: {:?}",
        health.missing_tables
    );
    assert!(health.missing_tables.is_empty());
    assert!(db.is_online());
}

#[test]
fn sqlite_database_is_online_from_file() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("parsed_commands.sqlite");

    let db = CommandSqliteDatabase::open(&db_path).expect("file database should open");

    let health = db.health_check().expect("health check should run");

    assert!(health.online);
    assert!(
        health.required_tables_present,
        "missing tables: {:?}",
        health.missing_tables
    );
    assert!(db_path.exists());
}

#[test]
fn sqlite_database_insert_get_and_search_round_trip() {
    let db = CommandSqliteDatabase::open_memory().expect("memory database should open");

    let record = NewParsedCommandRecord {
        source_name: "docs/parser.md".to_string(),
        command_id: "cmd_0001".to_string(),
        kind: CommandKind::Task,
        raw_identity: "@Task".to_string(),
        title: Some("Boundary Solver".to_string()),
        description: Some("Test parser boundary command storage.".to_string()),
        content: "The parser should detect where commands start and end.".to_string(),
        members: BTreeMap::new(),
        parameters: vec!["Boundary Solver".to_string()],
        tags: vec!["parser".to_string(), "boundary".to_string()],
        references: vec!["./src/parse/parser.rs".to_string()],
        statuses: vec!["building".to_string()],
        source_trace: "docs/parser.md:4".to_string(),
        file_path: Some("docs/parser.md".to_string()),
        start_line: Some(4),
        start_column: Some(0),
        end_line: Some(6),
        end_column: None,
    };

    let inserted_id = db
        .insert_command(&record)
        .expect("command should insert one by one");

    let stored = db
        .get_command(inserted_id)
        .expect("get command query should run")
        .expect("inserted command should exist");

    assert_eq!(stored.id, inserted_id);
    assert_eq!(stored.source_name, "docs/parser.md");
    assert_eq!(stored.command_id, "cmd_0001");
    assert_eq!(stored.kind, CommandKind::Task);
    assert_eq!(stored.title.as_deref(), Some("Boundary Solver"));
    assert!(stored.tags.contains(&"parser".to_string()));
    assert!(stored
        .references
        .contains(&"./src/parse/parser.rs".to_string()));

    let hits = db
        .search(CommandSearchOptions {
            query: Some("boundary".to_string()),
            limit: Some(10),
            ..Default::default()
        })
        .expect("search should run");

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, inserted_id);

    let task_hits = db
        .search(CommandSearchOptions {
            kind: Some(CommandKind::Task),
            limit: Some(10),
            ..Default::default()
        })
        .expect("kind search should run");

    assert_eq!(task_hits.len(), 1);

    let tag_hits = db
        .search(CommandSearchOptions {
            tag: Some("parser".to_string()),
            limit: Some(10),
            ..Default::default()
        })
        .expect("tag search should run");

    assert_eq!(tag_hits.len(), 1);

    let reference_hits = db
        .search(CommandSearchOptions {
            reference: Some("./src/parse/parser.rs".to_string()),
            limit: Some(10),
            ..Default::default()
        })
        .expect("reference search should run");

    assert_eq!(reference_hits.len(), 1);
}

#[test]
fn optional_real_database_path_is_online() {
    let db_path = match env::var("PARSE_DB_PATH") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Skipping real database test. Set PARSE_DB_PATH to enable it.");
            return;
        }
    };

    let db = CommandSqliteDatabase::open_existing(&db_path)
        .expect("real database path should open or initialize");

    let health = db.health_check().expect("health check should run");

    println!("database path: {db_path}");
    println!("health: {health:#?}");

    assert!(health.online);
    assert!(
        health.required_tables_present,
        "missing tables: {:?}",
        health.missing_tables
    );
}
