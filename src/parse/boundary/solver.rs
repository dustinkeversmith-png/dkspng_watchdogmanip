use crate::parse::boundary::model::{BoundaryCandidate, ParseDocumentInput};
use crate::parse::boundary::strategies::default_strategies;
use crate::parse::model::CommandSeed;
use crate::parse::passes::boundary::{solve_boundaries, CommandBlock};

pub fn collect_boundary_candidates(document: &ParseDocumentInput) -> Vec<BoundaryCandidate> {
    let mut candidates = Vec::new();
    for strategy in default_strategies() {
        candidates.extend(strategy.find_boundaries(document));
    }
    candidates.sort_by_key(|c| c.start_line);
    candidates
}

pub fn solve_command_blocks(
    document: &ParseDocumentInput,
    seeds: &[CommandSeed],
) -> Vec<CommandBlock> {
    solve_boundaries(document, seeds)
}
