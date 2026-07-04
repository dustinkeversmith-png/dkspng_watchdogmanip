use crate::parse::boundary::model::{
    BodyDirection, BodyShapeHint, BoundaryCandidate, BoundaryMarkerKind, BoundaryMetadataKind,
    ParseDocumentInput,
};
use once_cell::sync::Lazy;
use regex::Regex;

pub trait BoundaryStrategy {
    fn name(&self) -> &'static str;
    fn find_boundaries(&self, document: &ParseDocumentInput) -> Vec<BoundaryCandidate>;
}

pub struct CommandSeedBoundaryStrategy;

impl CommandSeedBoundaryStrategy {
    fn at_command_prefix() -> &'static Regex {
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?x)^\s*(?:@[A-Za-z][A-Za-z0-9_/-]*(?:\s+|$))+").unwrap());
        &RE
    }
}

impl BoundaryStrategy for CommandSeedBoundaryStrategy {
    fn name(&self) -> &'static str {
        "command_seed"
    }

    fn find_boundaries(&self, document: &ParseDocumentInput) -> Vec<BoundaryCandidate> {
        document
            .lines
            .iter()
            .filter(|line| Self::at_command_prefix().is_match(&line.text))
            .map(|line| {
                let has_payload =
                    line.text.trim().contains(' ') && !line.text.trim().ends_with('@');
                let (meta, direction, shape) = if has_payload {
                    (
                        BoundaryMetadataKind::SameLinePayload,
                        BodyDirection::InlineRight,
                        BodyShapeHint::SingleLine,
                    )
                } else {
                    (
                        BoundaryMetadataKind::CommandSeedLine,
                        BodyDirection::Below,
                        BodyShapeHint::MultiLineBlock,
                    )
                };
                BoundaryCandidate::build(
                    self.name(),
                    line,
                    BoundaryMarkerKind::CommandStart,
                    meta,
                    direction,
                    shape,
                    0.96,
                    "line starts with @command seed",
                )
            })
            .collect()
    }
}

pub struct HeadingBoundaryStrategy;

impl HeadingBoundaryStrategy {
    fn heading_prefix() -> &'static Regex {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*#{1,6}\s+").unwrap());
        &RE
    }
}

impl BoundaryStrategy for HeadingBoundaryStrategy {
    fn name(&self) -> &'static str {
        "heading"
    }

    fn find_boundaries(&self, document: &ParseDocumentInput) -> Vec<BoundaryCandidate> {
        document
            .lines
            .iter()
            .filter(|line| Self::heading_prefix().is_match(&line.text))
            .map(|line| {
                BoundaryCandidate::build(
                    self.name(),
                    line,
                    BoundaryMarkerKind::HeadingBoundary,
                    BoundaryMetadataKind::HeadingSection,
                    BodyDirection::Below,
                    BodyShapeHint::MarkdownSection,
                    0.85,
                    "markdown heading",
                )
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
            .map(|line| {
                BoundaryCandidate::build(
                    self.name(),
                    line,
                    BoundaryMarkerKind::BlankLineBoundary,
                    BoundaryMetadataKind::BlankLineTerminated,
                    BodyDirection::None,
                    BodyShapeHint::Empty,
                    0.6,
                    "blank line",
                )
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
                out.push(BoundaryCandidate::build(
                    self.name(),
                    line,
                    BoundaryMarkerKind::IndentationBoundary,
                    BoundaryMetadataKind::OutdentTerminated,
                    BodyDirection::Below,
                    BodyShapeHint::IndentedBlock,
                    0.7,
                    format!("outdent from {} to {} spaces", prev.indent, line.indent),
                ));
            }
        }
        out
    }
}

pub struct InlineCommandBoundaryStrategy;

impl InlineCommandBoundaryStrategy {
    fn bracket_body() -> &'static Regex {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[\s*[^\]]").unwrap());
        &RE
    }
}

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
            .map(|line| {
                let shape = if Self::bracket_body().is_match(&line.text) {
                    BodyShapeHint::BracketedBlock
                } else {
                    BodyShapeHint::FreeformProse
                };
                BoundaryCandidate::build(
                    self.name(),
                    line,
                    BoundaryMarkerKind::InlineCommand,
                    BoundaryMetadataKind::InlineCommand,
                    BodyDirection::Around,
                    shape,
                    0.55,
                    "inline @command inside prose",
                )
            })
            .collect()
    }
}

#[derive(Default)]
pub struct BoundaryStrategyRegistry {
    strategies: Vec<Box<dyn BoundaryStrategy>>,
}

impl BoundaryStrategyRegistry {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn register(&mut self, strategy: Box<dyn BoundaryStrategy>) {
        self.strategies.push(strategy);
    }

    pub fn collect_candidates(&self, document: &ParseDocumentInput) -> Vec<BoundaryCandidate> {
        let mut candidates = Vec::new();
        for strategy in &self.strategies {
            candidates.extend(strategy.find_boundaries(document));
        }
        candidates.sort_by_key(|c| c.start_line);
        candidates
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(CommandSeedBoundaryStrategy));
        registry.register(Box::new(HeadingBoundaryStrategy));
        registry.register(Box::new(BlankLineBoundaryStrategy));
        registry.register(Box::new(IndentationBoundaryStrategy));
        registry.register(Box::new(InlineCommandBoundaryStrategy));
        registry
    }
}

// TODO: NonLinearBoundarySearch
// TODO: RelevanceBasedBoundarySearch
// TODO: BackwardContentAttachment
