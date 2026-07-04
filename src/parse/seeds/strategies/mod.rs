use crate::parse::registry::CommandRegistry;
use crate::parse::seeds::model::{DetectedSeed, ParseDocumentInput, SeedKind};

pub trait SeedDetectionStrategy {
    fn name(&self) -> &'static str;
    fn detect(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed>;
}

pub mod at_command;
pub mod chained_at_command;
pub mod current;
pub mod inline_status;
pub mod reference;

pub use at_command::AtCommandSeedDetector;
pub use chained_at_command::ChainedAtCommandSeedDetector;
pub use current::CurrentSeedDetector;
pub use inline_status::InlineStatusSeedDetector;
pub use reference::ReferenceSeedDetector;

#[derive(Default)]
pub struct SeedDetectionStrategyRegistry {
    strategies: Vec<Box<dyn SeedDetectionStrategy>>,
}

impl SeedDetectionStrategyRegistry {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn register(&mut self, strategy: Box<dyn SeedDetectionStrategy>) {
        self.strategies.push(strategy);
    }

    pub fn detect_all(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed> {
        let mut seeds = Vec::new();
        for strategy in &self.strategies {
            seeds.extend(strategy.detect(input));
        }
        seeds.sort_by_key(|seed| (seed.line, seed.column));
        seeds
    }

    pub fn unknown_at_commands(
        &self,
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
                registry.lookup_chain(&chain).is_none() && !seed.normalized_identity.is_empty()
            })
            .map(|mut seed| {
                seed.kind = SeedKind::UnknownAtCommand;
                seed.confidence = 0.55;
                seed
            })
            .collect()
    }
}

impl SeedDetectionStrategyRegistry {
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(AtCommandSeedDetector));
        registry.register(Box::new(ChainedAtCommandSeedDetector));
        registry.register(Box::new(InlineStatusSeedDetector));
        registry.register(Box::new(ReferenceSeedDetector));
        registry.register(Box::new(CurrentSeedDetector));
        registry
    }
}

#[derive(Default)]
pub struct SeedDetector {
    strategies: SeedDetectionStrategyRegistry,
}

impl SeedDetector {
    pub fn new(strategies: SeedDetectionStrategyRegistry) -> Self {
        Self { strategies }
    }

    pub fn with_defaults() -> Self {
        Self::new(SeedDetectionStrategyRegistry::with_defaults())
    }

    pub fn strategy_registry(&self) -> &SeedDetectionStrategyRegistry {
        &self.strategies
    }

    pub fn strategy_registry_mut(&mut self) -> &mut SeedDetectionStrategyRegistry {
        &mut self.strategies
    }

    pub fn detect_all(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed> {
        self.strategies.detect_all(input)
    }
}
