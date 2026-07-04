use once_cell::sync::Lazy;
use regex::Regex;

use crate::parse::boundary::CommandBlock;
use crate::parse::hierarchy::model::{HierarchySignal, HierarchySignalKind};
use crate::parse::hierarchy::HierarchyDetector;
use crate::parse::model::SourceDocument;

pub struct MarkdownHeadingHierarchyDetector;

impl MarkdownHeadingHierarchyDetector {
    fn heading_re() -> &'static Regex {
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^(?P<hashes>#{1,6})\s+(?P<title>.+)$").unwrap());
        &RE
    }
}

impl HierarchyDetector for MarkdownHeadingHierarchyDetector {
    fn name(&self) -> &'static str {
        "markdown_heading"
    }

    fn detect(&self, document: &SourceDocument, _blocks: &[CommandBlock]) -> Vec<HierarchySignal> {
        document
            .lines
            .iter()
            .filter_map(|line| {
                let caps = Self::heading_re().captures(line.text.trim())?;
                Some(HierarchySignal {
                    kind: HierarchySignalKind::MarkdownHeading,
                    line: line.number,
                    level: caps.name("hashes")?.as_str().len(),
                    label: Some(caps.name("title")?.as_str().trim().to_string()),
                    raw: line.text.clone(),
                    confidence: 0.9,
                })
            })
            .collect()
    }
}
