use crate::parse::boundary::model::{BoundaryCandidate, BoundaryKind, ParseDocumentInput};
use crate::parse::model::CommandSeed;
use once_cell::sync::Lazy;
use regex::Regex;

pub trait BoundaryStrategy {
    fn name(&self) -> &'static str;
    fn find_boundaries(&self, document: &ParseDocumentInput) -> Vec<BoundaryCandidate>;
}

static AT_COMMAND: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?x)^\s*(?:@[A-Za-z][A-Za-z0-9_/-]*(?:\s+|$))+").unwrap());
static HASH_HEADING: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*#{1,6}\s+").unwrap());

pub struct CommandSeedBoundaryStrategy;

impl BoundaryStrategy for CommandSeedBoundaryStrategy {
    fn name(&self) -> &'static str {
        "command_seed"
    }

    fn find_boundaries(&self, document: &ParseDocumentInput) -> Vec<BoundaryCandidate> {
        document
            .lines
            .iter()
            .filter(|line| AT_COMMAND.is_match(&line.text))
            .map(|line| BoundaryCandidate {
                kind: BoundaryKind::CommandStart,
                start_line: line.number,
                end_line: None,
                confidence: 0.96,
                reason: "line starts with @command seed".to_string(),
            })
            .collect()
    }
}

pub struct HeadingBoundaryStrategy;

impl BoundaryStrategy for HeadingBoundaryStrategy {
    fn name(&self) -> &'static str {
        "heading"
    }

    fn find_boundaries(&self, document: &ParseDocumentInput) -> Vec<BoundaryCandidate> {
        document
            .lines
            .iter()
            .filter(|line| HASH_HEADING.is_match(&line.text))
            .map(|line| BoundaryCandidate {
                kind: BoundaryKind::HeadingBoundary,
                start_line: line.number,
                end_line: Some(line.number),
                confidence: 0.85,
                reason: "markdown heading".to_string(),
            })
            .collect()
    }
}

pub struct BlankLineBoundaryStrategy;

impl BoundaryStrategy for BlankLineBoundaryStrategy {
    fn name(&self) -> &'static str {
        "blank_line"
    }

    fn find_boundaries(&self, document: &ParseDocumentInput) -> Vec<BoundaryCandidate> {
        document
            .lines
            .iter()
            .filter(|line| line.text.trim().is_empty())
            .map(|line| BoundaryCandidate {
                kind: BoundaryKind::BlankLineBoundary,
                start_line: line.number,
                end_line: Some(line.number),
                confidence: 0.6,
                reason: "blank line".to_string(),
            })
            .collect()
    }
}

pub struct IndentationBoundaryStrategy;

impl BoundaryStrategy for IndentationBoundaryStrategy {
    fn name(&self) -> &'static str {
        "indentation"
    }

    fn find_boundaries(&self, document: &ParseDocumentInput) -> Vec<BoundaryCandidate> {
        let mut out = Vec::new();
        for (idx, line) in document.lines.iter().enumerate().skip(1) {
            let prev = &document.lines[idx - 1];
            if line.text.trim().is_empty() || prev.text.trim().is_empty() {
                continue;
            }
            if line.indent < prev.indent {
                out.push(BoundaryCandidate {
                    kind: BoundaryKind::IndentationBoundary,
                    start_line: line.number,
                    end_line: Some(line.number),
                    confidence: 0.7,
                    reason: format!("outdent from {} to {} spaces", prev.indent, line.indent),
                });
            }
        }
        out
    }
}

pub struct InlineCommandBoundaryStrategy;

impl BoundaryStrategy for InlineCommandBoundaryStrategy {
    fn name(&self) -> &'static str {
        "inline_command"
    }

    fn find_boundaries(&self, document: &ParseDocumentInput) -> Vec<BoundaryCandidate> {
        document
            .lines
            .iter()
            .filter(|line| {
                let trimmed = line.text.trim();
                trimmed.contains('@') && !trimmed.starts_with('@')
            })
            .map(|line| BoundaryCandidate {
                kind: BoundaryKind::InlineCommand,
                start_line: line.number,
                end_line: Some(line.number),
                confidence: 0.55,
                reason: "inline @command inside prose".to_string(),
            })
            .collect()
    }
}

pub fn default_strategies() -> Vec<Box<dyn BoundaryStrategy>> {
    vec![
        Box::new(CommandSeedBoundaryStrategy),
        Box::new(HeadingBoundaryStrategy),
        Box::new(BlankLineBoundaryStrategy),
        Box::new(IndentationBoundaryStrategy),
        Box::new(InlineCommandBoundaryStrategy),
    ]
}

// TODO: NonLinearBoundarySearch
// TODO: RelevanceBasedBoundarySearch
// TODO: BackwardContentAttachment

pub fn map_legacy_boundary(
    seed: &CommandSeed,
    next_seed_line: Option<usize>,
    doc_len: usize,
) -> BoundaryKind {
    if next_seed_line.is_none() && seed.start_line_index + 1 >= doc_len {
        BoundaryKind::CommandEnd
    } else if next_seed_line.is_some() {
        BoundaryKind::NextSeedBoundary
    } else {
        BoundaryKind::BlockCommand
    }
}
