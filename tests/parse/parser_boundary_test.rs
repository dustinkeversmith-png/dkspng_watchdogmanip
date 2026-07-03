use macro_os_engines::parse::boundary::{
    collect_boundary_candidates, solve_command_blocks, BoundaryKind, BoundaryStrategy,
    CommandSeedBoundaryStrategy, HeadingBoundaryStrategy, InlineCommandBoundaryStrategy,
};
use macro_os_engines::parse::model::SourceDocument;
use macro_os_engines::parse::seeds::{detect_all_seeds, detect_command_seeds_for_pipeline};
use macro_os_engines::parse::{default_registry, MacroPipeline};

#[test]
fn boundary_strategies_detect_messy_doc_markers() {
    let text = include_str!("../fixtures/example_docs/planner/docs/Scratch/messy_notes.txt");
    let doc = SourceDocument::new("messy_notes.txt", text);

    let command_boundaries = CommandSeedBoundaryStrategy.find_boundaries(&doc);
    let inline_boundaries = InlineCommandBoundaryStrategy.find_boundaries(&doc);
    let all = collect_boundary_candidates(&doc);

    assert!(
        command_boundaries
            .iter()
            .any(|b| b.kind == BoundaryKind::CommandStart),
        "expected command start boundaries"
    );
    assert!(
        inline_boundaries
            .iter()
            .any(|b| b.kind == BoundaryKind::InlineCommand),
        "expected inline @current boundary"
    );
    assert!(!all.is_empty());
    assert!(HeadingBoundaryStrategy.find_boundaries(&doc).is_empty());
}

#[test]
fn boundary_solver_produces_blocks_for_fixture_commands() {
    let text = include_str!("../fixtures/example_docs/planner/docs/ARCHITECTURE.md");
    let doc = SourceDocument::new("ARCHITECTURE.md", text);
    let registry = default_registry();
    let seeds = detect_command_seeds_for_pipeline(&doc, &registry);
    let blocks = solve_command_blocks(&doc, &seeds);

    assert!(!seeds.is_empty());
    assert_eq!(blocks.len(), seeds.len());
}

#[test]
fn seed_detector_finds_current_reference_and_status_markers() {
    let text = include_str!("../fixtures/example_docs/planner/example_docs/nested_commands.md");
    let doc = SourceDocument::new("nested_commands.md", text);
    let seeds = detect_all_seeds(&doc);

    assert!(seeds
        .iter()
        .any(|s| s.normalized_identity.contains("current")));
    assert!(seeds
        .iter()
        .any(|s| s.normalized_identity.contains("reference") || s.raw.contains("Reference")));
}

#[test]
fn macropipeline_parses_inconsistent_command_layouts_without_panic() {
    let text = include_str!("../fixtures/example_docs/moneyplan/selected/planning_scratch.txt");
    let output = MacroPipeline::default().parse("planning_scratch.txt", text);

    assert!(!output.commands.is_empty());
    assert!(output
        .diagnostics
        .iter()
        .all(|d| { !matches!(d.severity, macro_os_engines::parse::Severity::Error) }));
}
