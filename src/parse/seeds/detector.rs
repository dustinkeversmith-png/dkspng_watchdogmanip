use crate::parse::model::CommandSeed;
use crate::parse::passes::detect::detect_command_seeds;
use crate::parse::registry::CommandRegistry;
use crate::parse::seeds::model::{DetectedSeed, ParseDocumentInput, SeedKind};
use once_cell::sync::Lazy;
use regex::Regex;

pub trait SeedDetector {
    fn name(&self) -> &'static str;
    fn detect(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed>;
}

static INLINE_STATUS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\((?P<status>done|complete|deferred|deffered|building|adapting|blocked)\)")
        .unwrap()
});
static REFERENCE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)@(?:reference|ref)\b").unwrap());
static CURRENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)@current\b").unwrap());
static AT_COMMAND: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?x)^\s*(?P<chain>(?:@[A-Za-z][A-Za-z0-9_/-]*(?:\s+|$))+)(?P<payload>.*)$")
        .unwrap()
});
static CHAINED: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?x)^\s*@[A-Za-z][A-Za-z0-9_/-]*\s+@[A-Za-z]").unwrap());

pub struct AtCommandSeedDetector;

impl SeedDetector for AtCommandSeedDetector {
    fn name(&self) -> &'static str {
        "at_command"
    }

    fn detect(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed> {
        input
            .lines
            .iter()
            .filter_map(|line| {
                let text = line.text.trim_end();
                let caps = AT_COMMAND.captures(text)?;
                let chain = caps.name("chain")?.as_str().trim();
                Some(DetectedSeed {
                    kind: if CHAINED.is_match(text) {
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
                })
            })
            .collect()
    }
}

pub struct ChainedAtCommandSeedDetector;

impl SeedDetector for ChainedAtCommandSeedDetector {
    fn name(&self) -> &'static str {
        "chained_at_command"
    }

    fn detect(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed> {
        AtCommandSeedDetector
            .detect(input)
            .into_iter()
            .filter(|seed| seed.kind == SeedKind::ChainedCommand)
            .collect()
    }
}

pub struct InlineStatusSeedDetector;

impl SeedDetector for InlineStatusSeedDetector {
    fn name(&self) -> &'static str {
        "inline_status"
    }

    fn detect(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed> {
        input
            .lines
            .iter()
            .flat_map(|line| {
                INLINE_STATUS
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
                    })
            })
            .collect()
    }
}

pub struct ReferenceSeedDetector;

impl SeedDetector for ReferenceSeedDetector {
    fn name(&self) -> &'static str {
        "reference"
    }

    fn detect(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed> {
        input
            .lines
            .iter()
            .filter_map(|line| {
                if !REFERENCE.is_match(&line.text) {
                    return None;
                }
                Some(DetectedSeed {
                    kind: SeedKind::ReferenceMarker,
                    raw: line.text.clone(),
                    normalized_identity: "reference".to_string(),
                    line: line.number,
                    column: line.text.find('@').unwrap_or(0),
                    confidence: 0.85,
                })
            })
            .collect()
    }
}

pub struct CurrentSeedDetector;

impl SeedDetector for CurrentSeedDetector {
    fn name(&self) -> &'static str {
        "current"
    }

    fn detect(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed> {
        input
            .lines
            .iter()
            .filter_map(|line| {
                if !CURRENT.is_match(&line.text) {
                    return None;
                }
                Some(DetectedSeed {
                    kind: SeedKind::CurrentMarker,
                    raw: line.text.clone(),
                    normalized_identity: "current".to_string(),
                    line: line.number,
                    column: line.text.to_ascii_lowercase().find("@current").unwrap_or(0),
                    confidence: 0.9,
                })
            })
            .collect()
    }
}

pub fn default_detectors() -> Vec<Box<dyn SeedDetector>> {
    vec![
        Box::new(AtCommandSeedDetector),
        Box::new(ChainedAtCommandSeedDetector),
        Box::new(InlineStatusSeedDetector),
        Box::new(ReferenceSeedDetector),
        Box::new(CurrentSeedDetector),
    ]
}

pub fn detect_all_seeds(input: &ParseDocumentInput) -> Vec<DetectedSeed> {
    let mut seeds = Vec::new();
    for detector in default_detectors() {
        seeds.extend(detector.detect(input));
    }
    seeds.sort_by_key(|seed| (seed.line, seed.column));
    seeds
}

pub fn detect_command_seeds_for_pipeline(
    input: &ParseDocumentInput,
    registry: &CommandRegistry,
) -> Vec<CommandSeed> {
    detect_command_seeds(input, registry)
}

pub fn unknown_at_commands(
    input: &ParseDocumentInput,
    registry: &CommandRegistry,
) -> Vec<DetectedSeed> {
    AtCommandSeedDetector
        .detect(input)
        .into_iter()
        .filter(|seed| {
            let chain: Vec<String> = seed
                .normalized_identity
                .split_whitespace()
                .map(str::to_string)
                .collect();
            registry.lookup_chain(&chain).is_none()
                && !matches!(
                    registry.lookup_chain(&chain),
                    None if seed.normalized_identity.is_empty()
                )
        })
        .map(|mut seed| {
            seed.kind = SeedKind::UnknownAtCommand;
            seed.confidence = 0.55;
            seed
        })
        .collect()
}
