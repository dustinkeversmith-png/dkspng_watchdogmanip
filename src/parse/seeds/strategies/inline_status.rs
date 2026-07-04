use once_cell::sync::Lazy;
use regex::Regex;

use super::SeedDetectionStrategy;
use crate::parse::seeds::model::{DetectedSeed, ParseDocumentInput, SeedKind};

pub struct InlineStatusSeedDetector;

impl InlineStatusSeedDetector {
    fn status_pattern() -> &'static Regex {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\((?P<status>done|complete|deferred|deffered|building|adapting|blocked)\)")
                .unwrap()
        });
        &RE
    }
}

impl SeedDetectionStrategy for InlineStatusSeedDetector {
    fn name(&self) -> &'static str {
        "inline_status"
    }

    fn detect(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed> {
        input
            .lines
            .iter()
            .flat_map(|line| {
                Self::status_pattern()
                    .captures_iter(&line.text)
                    .map(|caps| DetectedSeed {
                        kind: SeedKind::InlineStatus,
                        raw: caps.name("status").unwrap().as_str().to_string(),
                        normalized_identity: caps
                            .name("status")
                            .unwrap()
                            .as_str()
                            .to_ascii_lowercase(),
                        line: line.number,
                        column: caps.get(0).map(|m| m.start()).unwrap_or(0),
                        confidence: 0.8,
                        payload: String::new(),
                    })
            })
            .collect()
    }
}
