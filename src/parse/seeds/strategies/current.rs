use once_cell::sync::Lazy;
use regex::Regex;

use super::SeedDetectionStrategy;
use crate::parse::seeds::model::{DetectedSeed, ParseDocumentInput, SeedKind};

pub struct CurrentSeedDetector;

impl CurrentSeedDetector {
    fn current_pattern() -> &'static Regex {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)@current\b").unwrap());
        &RE
    }
}

impl SeedDetectionStrategy for CurrentSeedDetector {
    fn name(&self) -> &'static str {
        "current"
    }

    fn detect(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed> {
        input
            .lines
            .iter()
            .filter_map(|line| {
                if !Self::current_pattern().is_match(&line.text) {
                    return None;
                }
                Some(DetectedSeed {
                    kind: SeedKind::CurrentMarker,
                    raw: line.text.clone(),
                    normalized_identity: "current".to_string(),
                    line: line.number,
                    column: line.text.to_ascii_lowercase().find("@current").unwrap_or(0),
                    confidence: 0.9,
                    payload: String::new(),
                })
            })
            .collect()
    }
}
