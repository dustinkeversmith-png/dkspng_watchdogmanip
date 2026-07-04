use once_cell::sync::Lazy;
use regex::Regex;

use crate::parse::model::{CommandKind, CommandSeed, TextSpan};
use crate::parse::pipeline::ParseContext;
use crate::parse::seeds::classifier::ClassifierCommandSeedStrategy;

pub trait CommandSeedStrategy {
    fn name(&self) -> &'static str;
    fn detect(&self, ctx: &ParseContext) -> Vec<CommandSeed>;
}

#[derive(Default)]
pub struct CommandSeedStrategyRegistry {
    strategies: Vec<Box<dyn CommandSeedStrategy>>,
}

impl CommandSeedStrategyRegistry {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn register(&mut self, strategy: Box<dyn CommandSeedStrategy>) {
        self.strategies.push(strategy);
    }

    pub fn detect(&self, ctx: &ParseContext) -> Vec<CommandSeed> {
        let mut seeds = Vec::new();
        for strategy in &self.strategies {
            seeds.extend(strategy.detect(ctx));
        }
        seeds.sort_by_key(|seed| seed.start_line_index);
        seeds
    }
}

impl CommandSeedStrategyRegistry {
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(AtCommandLineSeedStrategy));
        registry.register(Box::new(HeadingCommandSeedStrategy));
        registry.register(Box::new(ClassifierCommandSeedStrategy::with_defaults()));
        registry
    }
}

#[derive(Default)]
pub struct CommandSeedDetector {
    strategies: CommandSeedStrategyRegistry,
}

impl CommandSeedDetector {
    pub fn new(strategies: CommandSeedStrategyRegistry) -> Self {
        Self { strategies }
    }

    pub fn with_defaults() -> Self {
        Self::new(CommandSeedStrategyRegistry::with_defaults())
    }

    pub fn strategy_registry(&self) -> &CommandSeedStrategyRegistry {
        &self.strategies
    }

    pub fn strategy_registry_mut(&mut self) -> &mut CommandSeedStrategyRegistry {
        &mut self.strategies
    }

    pub fn detect(&self, ctx: &ParseContext) -> Vec<CommandSeed> {
        self.strategies.detect(ctx)
    }
}

pub struct AtCommandLineSeedStrategy;

impl AtCommandLineSeedStrategy {
    fn line_pattern() -> &'static Regex {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(?x)^\s*(?:\d+[.)]\s*)?(?P<chain>(?:@[A-Za-z][A-Za-z0-9_/-]*(?:\s+|$))+)(?P<payload>.*)$",
            )
            .unwrap()
        });
        &RE
    }
}

impl CommandSeedStrategy for AtCommandLineSeedStrategy {
    fn name(&self) -> &'static str {
        "at_command_line"
    }

    fn detect(&self, ctx: &ParseContext) -> Vec<CommandSeed> {
        let doc = ctx.document;
        let registry = ctx.command_registry;
        let mut seeds = Vec::new();
        for (line_idx, line) in doc.lines.iter().enumerate() {
            let text = line.text.trim_end();
            if text.trim().is_empty() {
                continue;
            }

            let Some(caps) = Self::line_pattern().captures(text) else {
                continue;
            };

            let chain_raw = caps.name("chain").unwrap().as_str().trim();
            let mut chain: Vec<String> = chain_raw
                .split_whitespace()
                .filter(|p| p.starts_with('@'))
                .map(|p| p.trim_start_matches('@').to_string())
                .collect();

            let payload = caps
                .name("payload")
                .map(|m| m.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            if chain.len() == 1 {
                if let Some(first_word) = payload.split_whitespace().next() {
                    let possible_two = vec![chain[0].clone(), first_word.to_string()];
                    if registry.lookup_chain(&possible_two).is_some() {
                        chain = possible_two;
                    }
                }
            }

            let spec = registry.lookup_chain(&chain);
            let kind = spec
                .as_ref()
                .map(|s| s.kind.clone())
                .unwrap_or_else(|| CommandKind::Unknown(chain.join(" ")));
            let confidence = if spec.is_some() { 0.96 } else { 0.55 };

            seeds.push(CommandSeed {
                raw_identity: chain_raw.to_string(),
                chain,
                canonical_kind: kind,
                inline_payload: payload,
                start_line_index: line_idx,
                span: TextSpan::new(line.start, line.end, line.number, line.number),
                confidence,
            });
        }
        seeds
    }
}

pub struct HeadingCommandSeedStrategy;

impl HeadingCommandSeedStrategy {
    fn heading_pattern() -> &'static Regex {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*#{1,6}\s+(?P<title>.+)$").unwrap());
        &RE
    }
}

impl CommandSeedStrategy for HeadingCommandSeedStrategy {
    fn name(&self) -> &'static str {
        "heading_command"
    }

    fn detect(&self, ctx: &ParseContext) -> Vec<CommandSeed> {
        let doc = ctx.document;
        let mut seeds = Vec::new();
        for (line_idx, line) in doc.lines.iter().enumerate() {
            let text = line.text.trim_end();
            if text.trim().is_empty() {
                continue;
            }

            let Some(caps) = Self::heading_pattern().captures(text) else {
                continue;
            };

            let title = caps.name("title").unwrap().as_str().trim().to_string();
            seeds.push(CommandSeed {
                raw_identity: "#".to_string(),
                chain: vec!["heading".to_string()],
                canonical_kind: CommandKind::Inferred("heading_section".to_string()),
                inline_payload: title,
                start_line_index: line_idx,
                span: TextSpan::new(line.start, line.end, line.number, line.number),
                confidence: 0.45,
            });
        }
        seeds
    }
}
