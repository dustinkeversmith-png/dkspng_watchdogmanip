use macro_os_engines::parse::database::ParseCommandStore;
use macro_os_engines::parse::model::SourceDocument;
use macro_os_engines::parse::MacroPipeline;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn parsed_commands_include_structured_source_location() {
    let file_path = PathBuf::from("tests/fixtures/example_docs/planner/docs/Scratch/messy_notes.txt");
    let text = "@Task Location tracking test\nTitle: With path";
    let doc = SourceDocument::with_path("messy_notes.txt", Some(file_path.clone()), text);
    let output = MacroPipeline::default().parse_document(doc);

    let cmd = output
        .commands
        .iter()
        .find(|c| c.title.as_deref() == Some("With path") || c.content.contains("Location"))
        .expect("parsed command with location");

    assert_eq!(cmd.location.source_name, "messy_notes.txt");
    assert_eq!(cmd.location.file_path.as_deref(), Some(file_path.as_path()));
    assert!(cmd.location.start_line >= 1);
    assert!(!cmd.source_trace.is_empty());
    assert!(cmd.source_trace.contains("messy_notes.txt"));
}

#[test]
fn database_stores_file_path_and_line_columns() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("location.sqlite");
    let store = ParseCommandStore::open(&db_path).expect("open db");

    let file_path = PathBuf::from("docs/example.md");
    let doc = SourceDocument::with_path(
        "example.md",
        Some(file_path.clone()),
        "@Task DB location test",
    );
    let output = MacroPipeline::default().parse_document(doc);
    let cmd = &output.commands[0];

    let id = store
        .insert_parsed_command("example.md", cmd.clone())
        .expect("insert");
    let stored = store.get_command(id).expect("get").expect("row");

    assert_eq!(stored.file_path.as_deref(), Some("docs/example.md"));
    assert_eq!(stored.start_line, Some(cmd.location.start_line));
    assert!(!stored.source_trace.is_empty());
}
