use macro_os_engines::parse::MacroPipeline;

#[test]
fn parser_hierarchy_assigns_heading_context_to_nested_commands() {
    let input = r#"# Project A

@Project A

## Tasks

1. @Task Build parser
   @Reference ./src/parse/parser.rs

2. @Task Build context database
"#;

    let output = MacroPipeline::default().parse("hierarchy_fixture.md", input);

    assert!(
        !output.commands.is_empty(),
        "expected commands to be parsed"
    );
    assert!(
        !output.hierarchy.is_empty(),
        "expected hierarchy nodes to be populated"
    );

    let task = output
        .commands
        .iter()
        .find(|cmd| matches!(cmd.kind, macro_os_engines::parse::CommandKind::Task))
        .expect("expected at least one @Task");

    assert!(task.source_trace.contains("lines"));
    assert!(
        !task.heading_context.is_empty() || !task.hierarchy_path.is_empty(),
        "nested task should inherit heading context"
    );

    for cmd in &output.commands {
        assert!(
            output
                .hierarchy
                .iter()
                .any(|node| node.command_id == cmd.id),
            "every command should have a hierarchy node"
        );
    }

    assert!(
        output
            .diagnostics
            .iter()
            .all(|d| { !matches!(d.severity, macro_os_engines::parse::Severity::Error) }),
        "hierarchy parse should not produce error diagnostics"
    );
}
