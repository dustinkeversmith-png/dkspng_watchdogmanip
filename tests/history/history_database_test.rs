use macro_os_engines::history::{HistoryEventRecord, HistoryStore};

#[test]
fn history_database_persists_and_queries_events() {
    let store = HistoryStore::open_memory().expect("history store opens");

    let base = 1_700_000_000_000_i64;
    let events = vec![
        HistoryEventRecord {
            id: None,
            timestamp_unix_ms: base,
            event_type: "FileOpened".to_string(),
            source: "NavigationEngine".to_string(),
            target_kind: "file".to_string(),
            target_value: "./src/parse/parser.rs".to_string(),
            context_id: Some("parser".to_string()),
            workspace_id: Some("macro_processor".to_string()),
            metadata: Default::default(),
        },
        HistoryEventRecord {
            id: None,
            timestamp_unix_ms: base + 1,
            event_type: "FolderOpened".to_string(),
            source: "NavigationEngine".to_string(),
            target_kind: "folder".to_string(),
            target_value: "./docs".to_string(),
            context_id: Some("docs".to_string()),
            workspace_id: Some("macro_processor".to_string()),
            metadata: Default::default(),
        },
        HistoryEventRecord {
            id: None,
            timestamp_unix_ms: base + 2,
            event_type: "CommandExecuted".to_string(),
            source: "MacroConsole".to_string(),
            target_kind: "command".to_string(),
            target_value: "cargo test parse".to_string(),
            context_id: Some("parser".to_string()),
            workspace_id: Some("macro_processor".to_string()),
            metadata: Default::default(),
        },
    ];

    for event in &events {
        store.insert_event(event).expect("insert event");
    }

    let recent = store.recent_events(10).expect("recent events");
    assert_eq!(recent.len(), 3);

    let frequent = store.frequent_targets(5).expect("frequent targets");
    assert!(!frequent.is_empty());

    let parser_events = store
        .events_by_context("parser", 10)
        .expect("events by context");
    assert_eq!(parser_events.len(), 2);
}
