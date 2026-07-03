use macro_os_engines::database::{new_record_from_parsed_command, CommandSearchOptions};
use macro_os_engines::parse::{CommandKind, MacroPipeline, ParseCommandStore};

#[test]
fn parse_database_insert_fetch_search_and_dump() {
    let input = include_str!("../fixtures/deep_nested_macros.md");
    let output = MacroPipeline::default().parse("deep_nested_macros.md", input);

    let temp = tempfile::tempdir().unwrap();
    let db = ParseCommandStore::open(temp.path().join("parse.sqlite")).expect("db opens");

    for command in output.commands {
        let record = new_record_from_parsed_command("deep_nested_macros.md", command);
        db.insert_command(&record).expect("insert command");
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

    let dumps = db.dump_core_tables(50).expect("dump tables");
    assert!(dumps
        .iter()
        .any(|table| table.table_name == "parsed_commands"));
}
