use macro_os_engines::history::stats::FrequencyIndex;
use macro_os_engines::history::{
    suggest_from_events, suggest_from_index, HistoryEventRecord, SuggestionRequest,
};

#[test]
fn suggestion_engine_scores_events_by_frequency_and_context() {
    let events = vec![
        HistoryEventRecord {
            id: None,
            timestamp_unix_ms: 1,
            event_type: "CommandExecuted".to_string(),
            source: "MacroConsole".to_string(),
            target_kind: "command".to_string(),
            target_value: "cargo test parse".to_string(),
            context_id: Some("parser".to_string()),
            workspace_id: Some("macro_processor".to_string()),
            metadata: Default::default(),
        },
        HistoryEventRecord {
            id: None,
            timestamp_unix_ms: 2,
            event_type: "CommandExecuted".to_string(),
            source: "MacroConsole".to_string(),
            target_kind: "command".to_string(),
            target_value: "cargo test parse".to_string(),
            context_id: Some("parser".to_string()),
            workspace_id: Some("macro_processor".to_string()),
            metadata: Default::default(),
        },
        HistoryEventRecord {
            id: None,
            timestamp_unix_ms: 3,
            event_type: "FileOpened".to_string(),
            source: "NavigationEngine".to_string(),
            target_kind: "file".to_string(),
            target_value: "./docs/ARCHITECTURE.md".to_string(),
            context_id: Some("docs".to_string()),
            workspace_id: None,
            metadata: Default::default(),
        },
    ];

    let results = suggest_from_events(
        &events,
        &SuggestionRequest {
            query: Some("cargo".to_string()),
            context_id: Some("parser".to_string()),
            workspace_id: None,
            limit: 5,
        },
    );

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].target_value, "cargo test parse");
    assert!(results[0].score >= 2.0);
}

#[test]
fn suggestion_engine_uses_frequency_index_with_context_weight() {
    let events = macro_os_engines::history::read_jsonl_events(
        "tests/fixtures/history_navigation_commands.jsonl",
    )
    .expect("fixture history");

    let index = FrequencyIndex::build(&events, 14);
    let results = suggest_from_index(
        &index,
        &SuggestionRequest {
            query: Some("cargo".to_string()),
            context_id: Some("parser".to_string()),
            workspace_id: Some("macro_processor".to_string()),
            limit: 3,
        },
    );

    assert!(!results.is_empty());
    assert!(results[0].score > 0.0);
}
