use macro_os_engines::parse::model::SourceDocument;
use macro_os_engines::parse::registry::CommandRegistry;
use macro_os_engines::parse::boundary::BoundarySolver;
use macro_os_engines::parse::{MacroPipeline, ParseContext, PipelineConfig};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

const LOG_DIR: &str = "target/test-logs/parser_pipeline_detection_test";

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
fn pipeline_detects_command_seeds_via_attached_detector() {
    let text = include_str!("../fixtures/example_docs/planner/docs/ARCHITECTURE.md");
    let doc = SourceDocument::new("ARCHITECTURE.md", text);
    let registry = CommandRegistry::default();
    let ctx = ParseContext::new(&doc, &registry);

    let pipeline = MacroPipeline::with_defaults(PipelineConfig::default());
    let seeds = pipeline.command_seed_detector().detect(&ctx);

    assert!(!seeds.is_empty());
    write_json(
        PathBuf::from(LOG_DIR).join("architecture_command_seeds.json"),
        &serde_json::to_value(&seeds).expect("serialize seeds"),
    );
}

#[test]
fn pipeline_assembles_blocks_via_attached_boundary_solver() {
    let text = include_str!("../fixtures/example_docs/planner/docs/ARCHITECTURE.md");
    let doc = SourceDocument::new("ARCHITECTURE.md", text);
    let registry = CommandRegistry::default();
    let ctx = ParseContext::new(&doc, &registry);

    let pipeline = MacroPipeline::with_defaults(PipelineConfig::default());
    let seeds = pipeline.command_seed_detector().detect(&ctx);
    let blocks = pipeline.boundary_solver().assemble_blocks(&ctx, &seeds);

    assert!(!seeds.is_empty());
    assert_eq!(blocks.len(), seeds.len());

    write_json(
        PathBuf::from(LOG_DIR).join("architecture_command_blocks.json"),
        &json!({
            "seed_count": seeds.len(),
            "block_count": blocks.len(),
            "block_identities": blocks.iter().map(|b| b.seed.raw_identity.clone()).collect::<Vec<_>>(),
        }),
    );
}

#[test]
fn pipeline_boundary_solver_collects_marker_candidates() {
    let text = include_str!("../fixtures/example_docs/planner/docs/Scratch/messy_notes.txt");
    let doc = SourceDocument::new("messy_notes.txt", text);
    let solver = BoundarySolver::with_defaults();
    let candidates = solver.collect_boundary_candidates(&doc);

    assert!(!candidates.is_empty());
    write_json(
        PathBuf::from(LOG_DIR).join("messy_notes_boundary_candidates.json"),
        &serde_json::to_value(&candidates).expect("serialize candidates"),
    );
}

#[test]
fn full_pipeline_parse_matches_detection_and_assembly_counts() {
    let text = include_str!("../fixtures/deep_nested_macros.md");
    let config = PipelineConfig {
        enable_loose_inference: false,
        ..PipelineConfig::default()
    };
    let pipeline = MacroPipeline::with_defaults(config);
    let output = pipeline.parse("deep_nested_macros.md", text);

    let doc = SourceDocument::new("deep_nested_macros.md", text);
    let ctx = ParseContext::new(&doc, pipeline.command_registry());
    let seeds = pipeline.command_seed_detector().detect(&ctx);
    let blocks = pipeline.boundary_solver().assemble_blocks(&ctx, &seeds);

    assert!(!output.commands.is_empty());
    assert_eq!(output.commands.len(), blocks.len());
    assert_eq!(seeds.len(), blocks.len());

    write_json(
        PathBuf::from(LOG_DIR).join("deep_nested_pipeline_summary.json"),
        &json!({
            "command_count": output.commands.len(),
            "seed_count": seeds.len(),
            "block_count": blocks.len(),
        }),
    );
}
