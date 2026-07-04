use macro_os_engines::parse::model::SourceDocument;
use macro_os_engines::parse::seeds::{
    ClassifierCommandSeedStrategy, CommandSeedDetector, CommandSeedStrategy,
};
use macro_os_engines::parse::registry::CommandRegistry;
use macro_os_engines::parse::pipeline::ParseContext;
use macro_os_engines::parse::CommandKind;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

const LOG_DIR: &str = "target/test-logs/parser_detection_test";

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

#[test]
fn classifier_detects_keyword_lines_without_at() {
    let text = r#"Task: do thing
Project Idea: do chained thing
Deferred Idea maybe later
current objective: do thing
Reference ./src/file.rs
"#;
    let doc = SourceDocument::new("classifier.md", text);
    let registry = CommandRegistry::default();
    let ctx = ParseContext::new(&doc, &registry);
    let strategy = ClassifierCommandSeedStrategy::with_defaults();
    let seeds = strategy.detect(&ctx);

    assert!(seeds.iter().any(|s| matches!(s.canonical_kind, CommandKind::Task)));
    assert!(seeds.iter().any(|s| matches!(s.canonical_kind, CommandKind::Project)));
    assert!(seeds.iter().any(|s| matches!(s.canonical_kind, CommandKind::Deferred)));
    assert!(seeds.iter().any(|s| matches!(s.canonical_kind, CommandKind::Current)));
    assert!(seeds.iter().any(|s| matches!(s.canonical_kind, CommandKind::Reference)));
    assert!(
        seeds
            .iter()
            .any(|s| s.chain.len() >= 2 && s.chain.contains(&"idea".to_string()))
    );
    assert!(seeds.iter().all(|s| s.confidence < 0.95));

    write_json(
        PathBuf::from(LOG_DIR).join("classifier_keyword_seeds.json"),
        &json!({ "source": doc.source_name, "seeds": seeds }),
    );
}

#[test]
fn explicit_at_commands_have_higher_confidence_than_classifier() {
    let text = "@Task explicit\nTask: classifier";
    let doc = SourceDocument::new("confidence.md", text);
    let registry = CommandRegistry::default();
    let ctx = ParseContext::new(&doc, &registry);
    let seeds = CommandSeedDetector::with_defaults().detect(&ctx);

    let explicit = seeds
        .iter()
        .find(|s| s.raw_identity.starts_with('@'))
        .expect("explicit @ seed");
    let classifier = seeds
        .iter()
        .find(|s| s.raw_identity.starts_with('~'))
        .expect("classifier seed");

    assert!(explicit.confidence > classifier.confidence);
}
