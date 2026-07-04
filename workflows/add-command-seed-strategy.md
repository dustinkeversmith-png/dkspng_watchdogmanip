# Workflow: Add a command seed strategy

Produce `CommandSeed` values for the parse pipeline (turns lines into commands via registry lookup).

Distinct from [add-seed-detector.md](add-seed-detector.md), which produces `DetectedSeed` markers for analysis only.

## Files to touch

| File | Change |
|------|--------|
| `src/parse/seeds/command.rs` | Implement `CommandSeedStrategy` |
| `src/parse/seeds/command.rs` | Register in `CommandSeedStrategyRegistry::with_defaults()` (optional) |
| `tests/parse/parser_command_test.rs` or `parser_pipeline_detection_test.rs` | Test via `ParseContext` + `CommandSeedDetector` |

## Steps

### 1. Implement the strategy

Each strategy owns its matching logic (regex inside the struct, not module-level statics):

```rust
// src/parse/seeds/command.rs (or src/parse/seeds/strategies/my_seed.rs)
use crate::parse::model::{CommandKind, CommandSeed, TextSpan};
use crate::parse::pipeline::ParseContext;

pub struct MyCommandSeedStrategy;

impl CommandSeedStrategy for MyCommandSeedStrategy {
    fn name(&self) -> &'static str {
        "my_command_seed"
    }

    fn detect(&self, ctx: &ParseContext) -> Vec<CommandSeed> {
        let doc = ctx.document;
        let registry = ctx.command_registry;
        // scan doc.lines, use registry.lookup_chain(...), build CommandSeed list
        Vec::new()
    }
}
```

### 2. Register in the strategy registry

Either add to defaults:

```rust
impl CommandSeedStrategyRegistry {
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(AtCommandLineSeedStrategy));
        registry.register(Box::new(HeadingCommandSeedStrategy));
        registry.register(Box::new(MyCommandSeedStrategy)); // new
        registry
    }
}
```

Or register only on a custom pipeline:

```rust
let mut strategies = CommandSeedStrategyRegistry::with_defaults();
strategies.register(Box::new(MyCommandSeedStrategy));
let detector = CommandSeedDetector::new(strategies);
```

### 3. Attach to the pipeline

```rust
let pipeline = MacroPipeline::with_defaults(PipelineConfig::default())
    .with_command_seed_detector(CommandSeedDetector::new(strategies));
```

### 4. Test

```rust
use macro_os_engines::parse::{
    CommandRegistry, CommandSeedDetector, ParseContext,
};
use macro_os_engines::parse::model::SourceDocument;

#[test]
fn my_command_seed_strategy_finds_commands() {
    let doc = SourceDocument::new("test.md", "@MyCmd body");
    let registry = CommandRegistry::default();
    let ctx = ParseContext::new(&doc, &registry);

    let mut strategies = CommandSeedStrategyRegistry::with_defaults();
    strategies.register(Box::new(MyCommandSeedStrategy));
    let seeds = CommandSeedDetector::new(strategies).detect(&ctx);

    assert!(!seeds.is_empty());
}
```

```bash
cargo test --test parse my_command_seed
cargo test --test parse parser_pipeline_detection
```

## Existing command seed strategies

| Strategy | Produces |
|----------|----------|
| `AtCommandLineSeedStrategy` | `@Task`, chained `@Project @Idea`, numbered-list prefixes, payload split |
| `HeadingCommandSeedStrategy` | `# heading` → inferred heading section seed |

## Checklist

- [ ] Implements `CommandSeedStrategy` with `detect(&ParseContext)`
- [ ] Registered in `CommandSeedStrategyRegistry`
- [ ] Test uses `CommandSeedDetector` + `ParseContext` (not a free function)
- [ ] Glossary updated

See also: [pipeline-and-registries.md](pipeline-and-registries.md)
