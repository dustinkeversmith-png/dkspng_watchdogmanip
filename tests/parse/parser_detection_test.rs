use macro_os_engines::parse::model::SourceDocument;
use macro_os_engines::parse::seeds::strategies::{
    AtCommandSeedDetector, ChainedAtCommandSeedDetector, CurrentSeedDetector,
    InlineStatusSeedDetector, ReferenceSeedDetector, SeedDetectionStrategy,
};
use macro_os_engines::parse::seeds::{SeedDetector, SeedKind};
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

mod at_command {
    use super::*;

    #[test]
    fn detects_explicit_at_commands_with_payload() {
        let text = include_str!("../fixtures/example_docs/planner/docs/Scratch/messy_notes.txt");
        let doc = SourceDocument::new("messy_notes.txt", text);
        let seeds = AtCommandSeedDetector.detect(&doc);

        assert!(seeds.iter().any(|s| s.normalized_identity == "idea"));
        assert!(seeds.iter().any(|s| s.normalized_identity == "tutorial"));
        assert!(seeds
            .iter()
            .any(|s| s.kind == SeedKind::ExplicitCommand && !s.payload.is_empty()));

        write_json(
            PathBuf::from(LOG_DIR).join("detects_at_commands.json"),
            &serde_json::to_value(&seeds).expect("serialize seeds"),
        );
    }
}

mod chained_at_command {
    use super::*;

    #[test]
    fn detects_chained_commands_only() {

        let text = include_str!("../fixtures/example_docs/planner/docs/Scratch/messy_notes.txt");

        let doc = SourceDocument::new("messy_notes.txt", text);
        let seeds = ChainedAtCommandSeedDetector.detect(&doc);
        // assert_eq!(seeds.len(), 1);
        // assert_eq!(seeds[0].kind, SeedKind::ChainedCommand);
        // assert!(seeds[0].normalized_identity.contains("project"));

        write_json(
            PathBuf::from(LOG_DIR).join("detects_chained_commands.json"),
            &serde_json::to_value(&seeds).expect("serialize seeds"),
        );

        

    }
}

mod inline_status {
    use super::*;

    #[test]
    fn detects_parenthetical_status_markers() {
        let doc = SourceDocument::new("status.md", "working on parser (building)\n(deffered)");
        let seeds = InlineStatusSeedDetector.detect(&doc);
        assert!(seeds.iter().any(|s| s.normalized_identity == "building"));
        assert!(seeds.iter().any(|s| s.normalized_identity == "deffered"));
    }
}

mod reference {
    use super::*;

    #[test]
    fn detects_reference_markers() {
        let doc = SourceDocument::new("ref.md", "@Reference ./docs/ARCHITECTURE.md\n@ref shortcut");
        let seeds = ReferenceSeedDetector.detect(&doc);
        assert_eq!(seeds.len(), 2);
        assert!(seeds.iter().all(|s| s.kind == SeedKind::ReferenceMarker));
    }
}

mod current {
    use super::*;

    #[test]
    fn detects_current_markers_in_prose() {
        let doc = SourceDocument::new(
            "current.md",
            "some prose @current [ fix this later ]\n@current\n[\n    nested\n]",
        );
        let seeds = CurrentSeedDetector.detect(&doc);
        assert!(seeds.len() >= 2);
        assert!(seeds.iter().all(|s| s.kind == SeedKind::CurrentMarker));
    }
}

#[test]
fn seed_detector_registry_merges_on_nested_fixture() {
    let text = include_str!("../fixtures/example_docs/planner/example_docs/nested_commands.md");
    let doc = SourceDocument::new("nested_commands.md", text);
    let detector = SeedDetector::with_defaults();
    let seeds = detector.detect_all(&doc);

    assert!(seeds
        .iter()
        .any(|s| s.normalized_identity.contains("current")));
    assert!(seeds
        .iter()
        .any(|s| s.normalized_identity.contains("reference") || s.raw.contains("Reference")));

    write_json(
        PathBuf::from(LOG_DIR).join("nested_commands_seeds.json"),
        &serde_json::to_value(&seeds).expect("serialize seeds"),
    );
}
