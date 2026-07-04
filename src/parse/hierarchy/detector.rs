use crate::parse::boundary::CommandBlock;
use crate::parse::hierarchy::model::HierarchySignal;
use crate::parse::model::SourceDocument;

use super::model::HierarchyDetector;
use super::strategies::{
    bullet_lists::BulletListHierarchyDetector, headings::MarkdownHeadingHierarchyDetector,
    indentation::IndentationHierarchyDetector, numbered_lists::NumberedListHierarchyDetector,
};

#[derive(Default)]
pub struct HierarchyDetectorRegistry {
    detectors: Vec<Box<dyn HierarchyDetector>>,
}

impl HierarchyDetectorRegistry {
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(MarkdownHeadingHierarchyDetector));
        registry.register(Box::new(NumberedListHierarchyDetector));
        registry.register(Box::new(BulletListHierarchyDetector));
        registry.register(Box::new(IndentationHierarchyDetector));
        registry
    }

    pub fn register(&mut self, detector: Box<dyn HierarchyDetector>) {
        self.detectors.push(detector);
    }

    pub fn detect_all(
        &self,
        document: &SourceDocument,
        blocks: &[CommandBlock],
    ) -> Vec<HierarchySignal> {
        let mut signals = Vec::new();
        for detector in &self.detectors {
            signals.extend(detector.detect(document, blocks));
        }
        signals.sort_by_key(|s| (s.line, s.level));
        signals
    }
}
