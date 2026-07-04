use crate::parse::boundary::CommandBlock;
use crate::parse::hierarchy::model::{HierarchySignal, HierarchySignalKind};
use crate::parse::hierarchy::HierarchyDetector;
use crate::parse::model::SourceDocument;

pub struct IndentationHierarchyDetector;

impl HierarchyDetector for IndentationHierarchyDetector {
    fn name(&self) -> &'static str {
        "indentation"
    }

    fn detect(&self, document: &SourceDocument, _blocks: &[CommandBlock]) -> Vec<HierarchySignal> {
        let mut signals = Vec::new();
        for (idx, line) in document.lines.iter().enumerate().skip(1) {
            let prev = &document.lines[idx - 1];
            if line.text.trim().is_empty() || prev.text.trim().is_empty() {
                continue;
            }
            if line.indent > prev.indent {
                signals.push(HierarchySignal {
                    kind: HierarchySignalKind::Indentation,
                    line: line.number,
                    level: (line.indent / 2).max(1),
                    label: None,
                    raw: line.text.clone(),
                    confidence: 0.7,
                });
            }
        }
        signals
    }
}
