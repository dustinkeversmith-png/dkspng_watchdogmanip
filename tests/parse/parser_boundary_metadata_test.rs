use macro_os_engines::parse::boundary::{
    BodyDirection, BodyShapeHint, BoundaryMetadataKind, BoundarySolver, CommandSeedBoundaryStrategy,
};
use macro_os_engines::parse::model::SourceDocument;
use macro_os_engines::parse::BoundaryStrategy;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

const LOG_DIR: &str = "target/test-logs/parser_boundary_test";

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
fn boundary_candidates_include_rich_metadata() {
    let text = "@Task Build parser\n\n# Section\n\ninline @current [item]";
    let doc = SourceDocument::new("boundary_meta.md", text);
    let solver = BoundarySolver::with_defaults();
    let candidates = solver.collect_boundary_candidates(&doc);

    assert!(!candidates.is_empty());
    assert!(candidates
        .iter()
        .any(|c| c.metadata_kind == BoundaryMetadataKind::CommandSeedLine
            || c.metadata_kind == BoundaryMetadataKind::SameLinePayload));
    assert!(candidates
        .iter()
        .any(|c| c.body_direction != BodyDirection::Unknown));
    assert!(candidates
        .iter()
        .any(|c| c.body_shape_hint != BodyShapeHint::Unknown));
    assert!(candidates.iter().all(|c| !c.evidence.is_empty()));
    assert!(candidates.iter().all(|c| c.confidence > 0.0));

    write_json(
        PathBuf::from(LOG_DIR).join("boundary_metadata_candidates.json"),
        &json!({
            "source": doc.source_name,
            "boundaries": candidates,
        }),
    );
}

#[test]
fn command_seed_strategy_serializes_evidence_to_json() {
    let text = "@Task inline title\n@Idea\nbody line";
    let doc = SourceDocument::new("seed_meta.md", text);
    let strategy = CommandSeedBoundaryStrategy;
    let results = strategy.find_boundaries(&doc);

    let serialized = serde_json::to_string(&results).expect("serialize boundaries");
    assert!(serialized.contains("metadata_kind"));
    assert!(serialized.contains("body_direction"));
    assert!(serialized.contains("evidence"));
}
