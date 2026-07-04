use macro_os_engines::parse::boundary::BodyShapeHint;
use macro_os_engines::parse::shape::CommandShapeKind;
use macro_os_engines::parse::MacroPipeline;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

const LOG_DIR: &str = "target/test-logs/parser_body_parsing_test";

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

fn parse_case(name: &str, text: &str) -> macro_os_engines::parse::ParseOutput {
    MacroPipeline::default().parse(name, text)
}

#[test]
fn inline_body_captures_title_candidate() {
    let output = parse_case("inline.md", "@Task Build parser body tests");
    let cmd = &output.commands[0];
    assert!(!output.commands.is_empty());
    assert!(cmd.title.is_some() || cmd.content.contains("Build"));
    let shape = cmd.shape_analysis.as_ref().expect("shape analysis");
    assert!(shape.shape_kinds.iter().any(|k| {
        matches!(
            k,
            CommandShapeKind::InlineTitle | CommandShapeKind::InlineParameters
        )
    }));
}

#[test]
fn next_line_body_captures_content_below_command() {
    let output = parse_case("next_line.md", "@Task\nBuild parser body tests");
    let cmd = &output.commands[0];
    assert!(cmd.content.contains("Build") || cmd.title.as_deref() == Some("Build parser body tests"));
}

#[test]
fn key_value_body_extracts_members() {
    let text = "@Task\nTitle: Build parser body tests\nDescription: Validate extraction\nTags: parser body";
    let output = parse_case("kv.md", text);
    let cmd = &output.commands[0];
    assert!(cmd.members.contains_key("Title"));
    assert!(cmd.members.contains_key("Description"));
    let shape = cmd.shape_analysis.as_ref().unwrap();
    assert!(shape.shape_kinds.contains(&CommandShapeKind::KeyValueMembers));
    assert_eq!(shape.body_shape, BodyShapeHint::KeyValueBlock);
}

#[test]
fn bracketed_body_captures_block_content() {
    let text = "@current\n[\n    Fix body parser\n    Add hierarchy detector\n]";
    let output = parse_case("bracket.md", text);
    let cmd = &output.commands[0];
    assert!(cmd.content.contains("Fix body parser") || cmd.content.contains("hierarchy"));
    let shape = cmd.shape_analysis.as_ref().unwrap();
    assert!(shape.shape_kinds.contains(&CommandShapeKind::BracketedBody));
}

#[test]
fn mixed_body_allows_multiple_shapes_and_diagnostics() {
    let text = "@Project @Idea maybe parser should try shapes\nTitle: Better body parser\n- random note\n(done)";
    let output = parse_case("mixed.md", text);
    assert!(!output.commands.is_empty());
    let cmd = output
        .commands
        .iter()
        .find(|c| !c.statuses.is_empty() || c.title.is_some())
        .unwrap();
    let shape = cmd.shape_analysis.as_ref().expect("shape");
    assert!(!shape.shape_kinds.is_empty());
}

#[test]
fn body_parsing_logs_shapes_and_locations() {
    let text = "@Task Build parser\n@Task\nTitle: Second";
    let output = parse_case("log_case.md", text);
    write_json(
        PathBuf::from(LOG_DIR).join("body_parsing_output.json"),
        &json!({
            "source": output.source_name,
            "commands": output.commands,
            "diagnostics": output.diagnostics,
            "hierarchy": output.hierarchy,
        }),
    );
    assert!(output.commands.iter().all(|c| !c.source_trace.is_empty()));
    assert!(output.commands.iter().all(|c| c.location.start_line >= 1));
}
