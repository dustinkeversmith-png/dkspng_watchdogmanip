use macro_os_engines::parse::model::SourceDocument;
use macro_os_engines::parse::registry::{
    member, parameter, CommandBodyPolicy, CommandLayoutKind, CommandRegistry, CommandSpec,
};
use macro_os_engines::parse::seeds::CommandSeedDetector;
use macro_os_engines::parse::shape::CommandShapeKind;
use macro_os_engines::parse::{CommandKind, MacroPipeline, ParseContext};
use macro_os_engines::parse::model::BoundaryKind;

#[test]
fn registry_resolves_aliases_and_multi_word_chains() {
    let registry = CommandRegistry::default();
    let detector = CommandSeedDetector::with_defaults();

    assert!(registry.lookup_name("todo").is_some());
    assert!(registry.lookup_name("@Task").is_some());
    assert!(registry.lookup_chain(&["task".into()]).is_some());

    let task_spec = registry.lookup_name("task").unwrap();
    assert!(task_spec.members_with_tag("title_candidate").iter().any(|m| m.name == "Title"));

    let doc = SourceDocument::new("chains.md", "@deferred Idea fold later\n@Task Build resolver");
    let ctx = ParseContext::new(&doc, &registry);
    let seeds = detector.detect(&ctx);

    assert!(seeds.iter().any(|s| matches!(s.canonical_kind, CommandKind::Deferred)));
    assert!(seeds.iter().any(|s| matches!(s.canonical_kind, CommandKind::Task)));
}

#[test]
fn custom_registry_registers_flexible_command_spec() {
    let mut registry = CommandRegistry::new();
    registry.register(CommandSpec {
        kind: CommandKind::Inferred("planner_note".into()),
        canonical: "planner note".into(),
        aliases: vec!["scratch".into(), "note".into()],
        parameters: vec![parameter("title", false, &[], &["title_candidate"])],
        optional_members: vec![member("Body", false, &[], &["body"])],
        required_members: vec![],
        accepted_layouts: vec![CommandLayoutKind::Inline, CommandLayoutKind::Prose],
        accepted_shapes: vec![CommandShapeKind::ProseOnly, CommandShapeKind::Mixed],
        body_policy: CommandBodyPolicy::CaptureIfPresent,
        boundary: BoundaryKind::UntilNextCommand,
    });

    let doc = SourceDocument::new("custom.md", "@scratch quick planner note");
    let ctx = ParseContext::new(&doc, &registry);
    let seeds = CommandSeedDetector::with_defaults().detect(&ctx);

    assert_eq!(seeds.len(), 1);
    assert!(matches!(
        seeds[0].canonical_kind,
        CommandKind::Inferred(ref name) if name == "planner_note"
    ));
}

#[test]
fn macropipeline_parses_inconsistent_command_layouts_without_panic() {
    let text = include_str!("../fixtures/example_docs/moneyplan/selected/planning_scratch.txt");
    let output = MacroPipeline::default().parse("planning_scratch.txt", text);

    assert!(!output.commands.is_empty());
    assert!(output
        .diagnostics
        .iter()
        .all(|d| !matches!(d.severity, macro_os_engines::parse::Severity::Error)));
}
