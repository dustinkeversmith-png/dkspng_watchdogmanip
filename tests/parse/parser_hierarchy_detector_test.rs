use macro_os_engines::parse::boundary::CommandBlock;
use macro_os_engines::parse::hierarchy::{
    HierarchyDetector, HierarchyDetectorRegistry, HierarchySignalKind,
    MarkdownHeadingHierarchyDetector, NumberedListHierarchyDetector,
};
use macro_os_engines::parse::model::SourceDocument;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

const LOG_DIR: &str = "target/test-logs/parser_hierarchy_test";

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

fn empty_blocks() -> Vec<CommandBlock> {
    Vec::new()
}

#[test]
fn heading_hierarchy_detector_finds_markdown_headings() {
    let text = "# Top\n## Sub\n### Deep";
    let doc = SourceDocument::new("headings.md", text);
    let detector = MarkdownHeadingHierarchyDetector;
    let signals = detector.detect(&doc, &empty_blocks());

    assert_eq!(signals.len(), 3);
    assert!(signals
        .iter()
        .all(|s| s.kind == HierarchySignalKind::MarkdownHeading));
    assert_eq!(signals[0].level, 1);
    assert_eq!(signals[1].level, 2);
}

#[test]
fn numbered_list_detector_handles_restart_groups() {
    let text = "1. First\n2. Second\n\n1. New group\n2. Another";
    let doc = SourceDocument::new("numbered.md", text);
    let detector = NumberedListHierarchyDetector;
    let signals = detector.detect(&doc, &empty_blocks());

    assert_eq!(signals.len(), 4);
    assert!(signals
        .iter()
        .all(|s| s.kind == HierarchySignalKind::NumberedList));
    let restart = signals
        .iter()
        .find(|s| s.label.as_deref() == Some("New group"))
        .unwrap();
    assert!(restart.level >= signals[0].level);
}

#[test]
fn bullet_list_and_indentation_detectors_emit_signals() {
    let text = "- bullet one\n  - nested\n    indented body";
    let doc = SourceDocument::new("lists.md", text);
    let registry = HierarchyDetectorRegistry::with_defaults();
    let signals = registry.detect_all(&doc, &empty_blocks());

    assert!(signals
        .iter()
        .any(|s| s.kind == HierarchySignalKind::BulletList));
    assert!(signals
        .iter()
        .any(|s| s.kind == HierarchySignalKind::Indentation));

    write_json(
        PathBuf::from(LOG_DIR).join("hierarchy_detector_signals.json"),
        &json!({ "source": doc.source_name, "signals": signals }),
    );
}

#[test]
fn registry_collects_signals_from_all_detectors() {
    let text = "# Section\n1. Item\n- bullet";
    let doc = SourceDocument::new("mixed.md", text);
    let registry = HierarchyDetectorRegistry::with_defaults();
    let signals = registry.detect_all(&doc, &empty_blocks());

    let kinds: Vec<_> = signals.iter().map(|s| &s.kind).collect();
    assert!(kinds.contains(&&HierarchySignalKind::MarkdownHeading));
    assert!(kinds.contains(&&HierarchySignalKind::NumberedList));
    assert!(kinds.contains(&&HierarchySignalKind::BulletList));
}
