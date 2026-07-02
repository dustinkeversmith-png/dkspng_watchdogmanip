use macro_os_engines::{context, history, navigation, parse, watchdog};
use history::adapters::{HistoryAdapter, MockHistoryAdapter};

#[test]
fn context_index_parses_alias_current_and_queue() {
    let input = "@Context project . inheritance=ancestors\n@Context parser ./src/parser parent=project inheritance=ancestors\n@Alias parser ./src/parser/mod.rs context=project type=file\n@current Finish docs context=parser\n@Queue Fix alias parser context=parser priority=high\n";
    let index = context::build_index_from_document(input, context::ParseConfig::default()).unwrap();
    assert!(index.contexts.contains_key("parser"));
    assert_eq!(index.aliases_visible_from("parser").unwrap().len(), 1);
    assert_eq!(index.contexts.get("parser").unwrap().currents.len(), 1);
    assert_eq!(index.contexts.get("parser").unwrap().queues.len(), 1);
}

#[test]
fn navigation_local_scope_alias_wins_before_parent() {
    let index = navigation::mock_navigation_index();
    let resolver = navigation::NavigationResolver::new(&index);
    let targets = resolver.resolve("parser", "parser").unwrap();
    let json = serde_json::to_string(&targets[0]).unwrap();
    assert!(json.contains("src/parser/mod.rs"));
}

#[test]
fn parse_pipeline_handles_explicit_task_and_status() {
    let input = "@Task Build parser (done)\nDescription: boundary solver first\n";
    let out = parse::MacroPipeline::default().parse("test", input);
    assert_eq!(out.commands.len(), 1);
    assert!(matches!(out.commands[0].kind, parse::CommandKind::Task));
    assert!(out.commands[0].statuses.contains(&"done".to_string()));
}

#[test]
fn history_mock_adapter_builds_suggestions() {
    let mut adapter = MockHistoryAdapter::default();
    let events = adapter.collect().unwrap();
    let index = history::FrequencyIndex::build(&events, 14);
    let suggestions = history::suggest(&index, &history::SuggestionQuery {
        text: Some("parser".into()),
        context_id: Some("parser".into()),
        workspace_id: Some("macro_processor".into()),
        limit: 5,
    });
    assert!(!suggestions.is_empty());
}

#[test]
fn watchdog_fixture_plans_actions() {
    let spec = watchdog::read_watch_spec("examples/watch_spec.json").unwrap();
    let events = watchdog::read_file_events_jsonl("examples/file_events.jsonl").unwrap();
    let planned = watchdog::WatchdogPlanner::plan(&spec, &events).unwrap();
    assert!(planned.iter().any(|a| a.rule_id == "rule_rs_modified_refresh_parser"));
}
