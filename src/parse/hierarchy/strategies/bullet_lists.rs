use crate::parse::boundary::CommandBlock;
use crate::parse::hierarchy::model::{HierarchySignal, HierarchySignalKind};
use crate::parse::hierarchy::HierarchyDetector;
use crate::parse::model::SourceDocument;

pub struct BulletListHierarchyDetector;

impl HierarchyDetector for BulletListHierarchyDetector {
    fn name(&self) -> &'static str {
        "bullet_list"
    }

    fn detect(&self, document: &SourceDocument, _blocks: &[CommandBlock]) -> Vec<HierarchySignal> {
        document
            .lines
            .iter()
            .filter_map(|line| {
                let trimmed = line.text.trim_start();
                if !trimmed.starts_with("- ") && !trimmed.starts_with("* ") {
                    return None;
                }
                Some(HierarchySignal {
                    kind: HierarchySignalKind::BulletList,
                    line: line.number,
                    level: (line.indent / 2).max(1),
                    label: Some(trimmed.trim_start_matches(['-', '*']).trim().to_string()),
                    raw: line.text.clone(),
                    confidence: 0.78,
                })
            })
            .collect()
    }
}
