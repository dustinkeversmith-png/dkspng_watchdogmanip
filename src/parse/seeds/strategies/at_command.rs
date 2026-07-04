use once_cell::sync::Lazy;
use regex::Regex;

use super::SeedDetectionStrategy;
use crate::parse::seeds::model::{DetectedSeed, ParseDocumentInput, SeedKind};

pub struct AtCommandSeedDetector;

impl AtCommandSeedDetector {
    fn line_pattern() -> &'static Regex {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(?x)^\s*(?:\d+[.)]\s*)?(?P<chain>(?:@[A-Za-z][A-Za-z0-9_/-]*(?:\s+|$))+)(?P<payload>.*)$",
            )
            .unwrap()
        });
        &RE
    }

    fn chained_pattern() -> &'static Regex {
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?x)^\s*@[A-Za-z][A-Za-z0-9_/-]*\s+@[A-Za-z]").unwrap());
        &RE
    }
}

impl SeedDetectionStrategy for AtCommandSeedDetector {
    fn name(&self) -> &'static str {
        "at_command"
    }

    fn detect(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed> {
        input
            .lines
            .iter()
            .filter_map(|line| {
                let text = line.text.trim_end();
                let caps = Self::line_pattern().captures(text)?;
                let chain = caps.name("chain")?.as_str().trim();
                Some(DetectedSeed {
                    kind: if Self::chained_pattern().is_match(text) {
                        SeedKind::ChainedCommand
                    } else {
                        SeedKind::ExplicitCommand
                    },
                    raw: chain.to_string(),
                    normalized_identity: chain
                        .split_whitespace()
                        .map(|p| p.trim_start_matches('@').to_ascii_lowercase())
                        .collect::<Vec<_>>()
                        .join(" "),
                    line: line.number,
                    column: line.text.find('@').unwrap_or(0),
                    confidence: 0.96,
                    payload: caps
                        .name("payload")
                        .map(|m| m.as_str().trim().to_string())
                        .unwrap_or_default(),
                })
            })
            .collect()
    }
}
