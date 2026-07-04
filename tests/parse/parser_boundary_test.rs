use macro_os_engines::parse::boundary::{
    BlankLineBoundaryStrategy, BoundaryMarkerKind, BoundarySolver, BoundaryStrategy,
    CommandSeedBoundaryStrategy, HeadingBoundaryStrategy, InlineCommandBoundaryStrategy,
    IndentationBoundaryStrategy,
};
use macro_os_engines::parse::model::SourceDocument;
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
fn boundary_strategies_detect_messy_doc_markers() {
    let text = include_str!("../fixtures/example_docs/planner/docs/Scratch/messy_notes.txt");
    let doc = SourceDocument::new("messy_notes.txt", text);
    let solver = BoundarySolver::with_defaults();

    let strategy_results = [
        ("command_seed", CommandSeedBoundaryStrategy.find_boundaries(&doc)),
        ("inline_command", InlineCommandBoundaryStrategy.find_boundaries(&doc)),
        ("heading", HeadingBoundaryStrategy.find_boundaries(&doc)),
        ("blank_line", BlankLineBoundaryStrategy.find_boundaries(&doc)),
        ("indentation", IndentationBoundaryStrategy.find_boundaries(&doc)),
    ];

    // 1. Loop through individual strategy results and save them
    for (name, results) in &strategy_results {
        // Construct filename like: "messy_notes_command_seed_results.json"
        let filename = format!("messy_notes_{}_results.json", name);
        let path = PathBuf::from(LOG_DIR).join(filename);
        
        // Write the individual results to a file
        // Write all of the different results
        write_json(
            path,
            &json!({ "results": results }),
        );
    }


        

    let command_boundaries = &strategy_results[0].1;
    let inline_boundaries = &strategy_results[1].1;
    let heading_boundaries = &strategy_results[2].1;
    let all = solver.collect_boundary_candidates(&doc);

    assert!(
        command_boundaries
            .iter()
            .any(|b| b.kind == BoundaryMarkerKind::CommandStart),
        "expected command start boundaries"
    );
    assert!(
        inline_boundaries
            .iter()
            .any(|b| b.kind == BoundaryMarkerKind::InlineCommand),
        "expected inline @current boundary"
    );
    assert!(
        heading_boundaries
            .iter()
            .any(|b| b.kind == BoundaryMarkerKind::HeadingBoundary),
        "expected markdown heading boundary"
    );
    assert!(!all.is_empty());

    let comparison: Vec<_> = strategy_results
        .iter()
        .map(|(name, candidates)| {
            json!({
                "strategy": name,
                "count": candidates.len(),
                "kinds": candidates.iter().map(|c| format!("{:?}", c.kind)).collect::<Vec<_>>(),
            })
        })
        .collect();

    

    write_json(
        PathBuf::from(LOG_DIR).join("messy_notes_strategy_comparison.json"),
        &json!({ "strategies": comparison, "merged_count": all.len() }),
    );
}
