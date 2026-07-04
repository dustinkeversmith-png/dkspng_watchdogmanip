use super::{AtCommandSeedDetector, SeedDetectionStrategy};
use crate::parse::seeds::model::{DetectedSeed, ParseDocumentInput, SeedKind};

pub struct ChainedAtCommandSeedDetector;

impl SeedDetectionStrategy for ChainedAtCommandSeedDetector {
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
