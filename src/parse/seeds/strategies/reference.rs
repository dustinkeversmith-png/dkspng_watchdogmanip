use once_cell::sync::Lazy;
use regex::Regex;

use super::SeedDetectionStrategy;
use crate::parse::seeds::model::{DetectedSeed, ParseDocumentInput, SeedKind};

pub struct ReferenceSeedDetector;

impl ReferenceSeedDetector {
    fn reference_pattern() -> &'static Regex {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)@(?:reference|ref)\b").unwrap());
        &RE
    }
}

impl SeedDetectionStrategy for ReferenceSeedDetector {
    fn name(&self) -> &'static str {
        "reference"
    }

    fn detect(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed> {
        input
            .lines
            .iter()
            .filter_map(|line| {
                if !Self::reference_pattern().is_match(&line.text) {
                    return None;
                }
                Some(DetectedSeed {
                    kind: SeedKind::ReferenceMarker,
                    raw: line.text.clone(),
                    normalized_identity: "reference".to_string(),
                    line: line.number,
                    column: line.text.find('@').unwrap_or(0),
                    confidence: 0.85,
                    payload: String::new(),
                })
            })
            .collect()
    }
}
