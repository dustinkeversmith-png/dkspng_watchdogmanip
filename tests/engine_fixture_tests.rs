use macro_os_engines::{context, history, parse, watchdog};
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn context_file_tree_fixture_assigns_unique_context_layers_and_indexes_up_down() {
    let root = std::path::PathBuf::from("tests/fixtures/deep_tree");
    let index = context::build_contexts_from_file_tree(&root, context::FileTreeContextOptions {
        root_context_id: "project".to_string(),
        root_name: "Deep Tree Project".to_string(),
        inheritance: context::InheritancePolicy::Ancestors,
        include: vec!["**".to_string()],
        exclude: vec!["target/**".to_string(), ".git/**".to_string(), "node_modules/**".to_string()],
        create_context_for_root: true,
    }).unwrap();

    assert!(index.contexts.contains_key("project"));
    assert!(index.contexts.contains_key("project_src"));
    assert!(index.contexts.contains_key("project_src_parser"));
    assert!(index.contexts.contains_key("project_docs_guides"));
    assert!(!index.contexts.contains_key("project_target"));
    assert!(!index.contexts.contains_key("project_node_modules"));

    let parser_up = index.ancestor_ids("project_src_parser").unwrap();
    assert_eq!(parser_up, vec!["project_src_parser", "project_src", "project"]);

    let project_down = index.descendant_ids("project").unwrap();
    assert!(project_down.contains(&"project_src_parser".to_string()));
    assert!(project_down.contains(&"project_docs_guides".to_string()));

    let direct_children = index.direct_child_ids("project").unwrap();
    assert!(direct_children.contains(&"project_src".to_string()));
    assert!(direct_children.contains(&"project_docs".to_string()));

    let local_contexts = index.local_context_ids();
    assert!(local_contexts.contains(&"project".to_string()));
    assert!(local_contexts.contains(&"project_src_context".to_string()));
    assert_eq!(index.context("project_src_parser").unwrap().metadata.get("local_context"), Some(&"true".to_string()));

    let tree = index.tree_from("project").unwrap();
    assert_eq!(tree.id, "project");
    assert!(!tree.children.is_empty());
}

#[test]
fn watchdog_filters_file_types_ignores_paths_and_expands_routines() {
    let spec = watchdog::read_watch_spec("tests/fixtures/watch_spec_file_types_and_timer.json").unwrap();
    let events = watchdog::read_file_events_jsonl("tests/fixtures/file_change_events.jsonl").unwrap();

    let planned = watchdog::WatchdogPlanner::plan(&spec, &events).unwrap();
    assert!(planned.iter().any(|a| a.event_id == "evt_rs_1" && a.rule_id == "rust_file_modified"));
    assert!(planned.iter().any(|a| a.event_id == "evt_md_1" && a.rule_id == "markdown_file_modified"));
    assert!(!planned.iter().any(|a| a.event_id == "evt_ignored_target"));
    assert!(!planned.iter().any(|a| a.event_id == "evt_ignored_tmp"));

    let expanded = watchdog::WatchdogPlanner::expand_routine_actions(&spec, &planned);
    assert!(expanded.iter().any(|a| matches!(&a.action, watchdog::WatchAction::RefreshAliases { context_id } if context_id.as_deref() == Some("parser"))));
    assert!(expanded.iter().any(|a| matches!(&a.action, watchdog::WatchAction::EmitHistoryEvent { event_type } if event_type == "ContextUpdated")));
}

#[test]
fn watchdog_timer_event_runs_timely_routine_from_fixture() {
    let spec = watchdog::read_watch_spec("tests/fixtures/watch_spec_file_types_and_timer.json").unwrap();
    let events = watchdog::read_file_events_jsonl("tests/fixtures/file_change_events.jsonl").unwrap();
    let planned = watchdog::WatchdogPlanner::plan(&spec, &events).unwrap();
    let expanded = watchdog::WatchdogPlanner::expand_routine_actions(&spec, &planned);

    assert!(planned.iter().any(|a| a.event_id == "evt_timer_due" && a.rule_id == "timer_refresh"));
    assert!(expanded.iter().any(|a| a.event_id == "evt_timer_due" && matches!(&a.action, watchdog::WatchAction::RunCommand { command, .. } if command == "cargo test")));
    assert!(expanded.iter().any(|a| a.event_id == "evt_timer_due" && matches!(&a.action, watchdog::WatchAction::ReindexContext { context_id } if context_id == "project")));
    assert!(expanded.iter().any(|a| a.event_id == "evt_timer_due" && matches!(&a.action, watchdog::WatchAction::EmitHistoryEvent { event_type } if event_type == "RoutineRan")));
}

#[test]
fn parser_deeply_nested_commands_are_inserted_into_database_and_searchable() {
    let input = fs::read_to_string("tests/fixtures/deep_nested_macros.md").unwrap();
    let output = parse::MacroPipeline::default().parse("deep_nested_macros.md", input);
    assert!(output.commands.len() >= 8, "expected nested command and inferred heading/prose captures, got {}", output.commands.len());
    assert!(output.commands.iter().any(|c| matches!(c.kind, parse::CommandKind::Project)));
    assert!(output.commands.iter().any(|c| matches!(c.kind, parse::CommandKind::Task)));
    assert!(output.commands.iter().any(|c| matches!(c.kind, parse::CommandKind::Alias)));
    assert!(output.commands.iter().any(|c| matches!(c.kind, parse::CommandKind::QA)));

    let mut db = parse::ParseDatabase::new();
    db.insert_output(output);
    assert!(db.command_count() >= 8);

    let parser_hits = db.search("parser");
    assert!(!parser_hits.is_empty());
    assert!(parser_hits.iter().any(|hit| hit.source_name == "deep_nested_macros.md"));

    let boundary_hits = db.search("boundary");
    assert!(boundary_hits.iter().any(|hit| hit.title.as_deref().unwrap_or_default().to_ascii_lowercase().contains("boundary") || hit.content_preview.to_ascii_lowercase().contains("boundary")));

    let tasks = db.search_by_kind(&parse::CommandKind::Task);
    assert!(tasks.len() >= 2);
}

#[test]
fn history_log_tracks_file_navigation_explorer_locations_and_console_commands() {
    let events = history::read_jsonl_events("tests/fixtures/history_navigation_commands.jsonl").unwrap();
    assert_eq!(events.len(), 6);
    assert!(events.iter().any(|e| matches!(e.target, history::HistoryTarget::File { .. })));
    assert!(events.iter().any(|e| matches!(e.target, history::HistoryTarget::Folder { .. })));
    assert!(events.iter().any(|e| matches!(e.target, history::HistoryTarget::Command { .. })));
    assert!(events.iter().any(|e| matches!(e.target, history::HistoryTarget::Window { .. })));

    let dir = tempdir().unwrap();
    let log_path = dir.path().join("history.jsonl");
    let store = history::JsonlEventStore::new(&log_path);
    store.append_many(&events).unwrap();
    let round_tripped = store.read_all().unwrap();
    assert_eq!(round_tripped.len(), events.len());

    let index = history::FrequencyIndex::build(&round_tripped, 14);
    let top = index.top(3);
    assert!(top.iter().any(|stat| stat.label == "cargo test parser_database_search" && stat.total_count == 2));

    let suggestions = history::suggest(&index, &history::SuggestionQuery {
        text: Some("cargo".to_string()),
        context_id: Some("parser".to_string()),
        workspace_id: Some("macro_processor".to_string()),
        limit: 5,
    });
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0].label, "cargo test parser_database_search");
}

#[test]
fn history_can_append_live_style_events_with_metadata() {
    let dir = tempdir().unwrap();
    let log_path = dir.path().join("live_history.jsonl");
    let store = history::JsonlEventStore::new(&log_path);

    let mut metadata = BTreeMap::new();
    metadata.insert("opened_by".to_string(), "Explorer".to_string());
    let event = history::HistoryEvent {
        id: "live_001".to_string(),
        timestamp: chrono::DateTime::parse_from_rfc3339("2026-06-30T19:00:00Z").unwrap().with_timezone(&chrono::Utc),
        source: history::EventSource::NavigationEngine,
        actor: history::Actor::User,
        event_type: history::HistoryEventType::FolderOpened,
        target: history::HistoryTarget::Folder { path: "./docs".into() },
        context_id: Some("docs".to_string()),
        workspace_id: Some("macro_processor".to_string()),
        metadata,
    };

    store.append(&event).unwrap();
    let loaded = store.read_all().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].metadata.get("opened_by"), Some(&"Explorer".to_string()));
}
