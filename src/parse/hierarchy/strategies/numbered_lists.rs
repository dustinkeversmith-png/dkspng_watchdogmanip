use once_cell::sync::Lazy;
use regex::Regex;

use crate::parse::boundary::CommandBlock;
use crate::parse::hierarchy::model::{HierarchySignal, HierarchySignalKind};
use crate::parse::hierarchy::HierarchyDetector;
use crate::parse::model::SourceDocument;

pub struct NumberedListHierarchyDetector;

impl NumberedListHierarchyDetector {
    fn list_re() -> &'static Regex {
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^\s*(?P<num>\d+)[.)]\s+(?P<body>.+)$").unwrap());
        &RE
    }
}

impl HierarchyDetector for NumberedListHierarchyDetector {
    fn name(&self) -> &'static str {
        "numbered_list"
    }

    fn detect(&self, document: &SourceDocument, _blocks: &[CommandBlock]) -> Vec<HierarchySignal> {
        let mut signals = Vec::new();
        let mut group_level = 0usize;
        let mut last_num = 0usize;
        let mut last_line = 0usize;

        for line in &document.lines {
            let Some(caps) = Self::list_re().captures(&line.text) else {
                continue;
            };
            let Some(num_str) = caps.name("num") else {
                continue;
            };
            let Some(num) = num_str.as_str().parse::<usize>().ok() else {
                continue;
            };
            if num == 1 && last_num > 1 && line.number.saturating_sub(last_line) > 1 {
                group_level = group_level.saturating_add(1);
            } else if num == 1 && last_num == 0 {
                group_level = 1;
            }
            last_num = num;
            last_line = line.number;
            let label = caps.name("body").map(|m| m.as_str().trim().to_string());
            signals.push(HierarchySignal {
                kind: HierarchySignalKind::NumberedList,
                line: line.number,
                level: group_level,
                label,
                raw: line.text.clone(),
                confidence: 0.82,
            });
        }
        signals
    }
}
