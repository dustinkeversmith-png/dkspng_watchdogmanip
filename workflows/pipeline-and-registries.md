# Workflow: Pipeline, registries, and ParseContext

How the parse pipeline wires command specs, seed detection, boundary markers, and block assembly through attachable registries.

## Architecture overview

```text
ParseContext { document, command_registry }
        │
        ├─► CommandSeedDetector
        │       └─ CommandSeedStrategyRegistry
        │             ├─ AtCommandLineSeedStrategy
        │             └─ HeadingCommandSeedStrategy
        │
        ├─► BoundarySolver
        │       ├─ BoundaryStrategyRegistry   (marker candidates)
        │       └─ BlockAssemblerRegistry     (seed → CommandBlock)
        │
        └─► passes: extract → normalize → validate → hierarchy
```

Analysis-only marker detection (not pipeline command seeds) uses a separate path:

```text
SeedDetector
  └─ SeedDetectionStrategyRegistry
        ├─ AtCommandSeedDetector
        ├─ ChainedAtCommandSeedDetector
        ├─ InlineStatusSeedDetector
        ├─ ReferenceSeedDetector
        └─ CurrentSeedDetector
```

## Core types

| Type | Location | Role |
|------|----------|------|
| `ParseContext` | `src/parse/pipeline/context.rs` | Document + command registry shared by detectors and assemblers |
| `CommandRegistry` | `src/parse/registry/mod.rs` | `@command` specs; use `CommandRegistry::default()` for built-ins |
| `CommandSeedDetector` | `src/parse/seeds/command.rs` | Runs command seed strategies → `Vec<CommandSeed>` |
| `CommandSeedStrategyRegistry` | `src/parse/seeds/command.rs` | Registered `CommandSeedStrategy` implementations |
| `SeedDetector` | `src/parse/seeds/strategies/mod.rs` | Runs marker seed strategies → `Vec<DetectedSeed>` |
| `BoundarySolver` | `src/parse/boundary/solver.rs` | Boundary marker collection + block assembly |
| `BoundaryStrategyRegistry` | `src/parse/boundary/strategies.rs` | Registered `BoundaryStrategy` implementations |
| `BlockAssemblerRegistry` | `src/parse/boundary/solver.rs` | Registered `BlockAssemblyStrategy` implementations |
| `MacroPipeline` | `src/parse/pipeline/mod.rs` | Owns all registries; orchestrates parse |

There is no free `detect_command_seeds()` or `default_registry()` function. Detection always goes through a detector + `ParseContext`.

## Default pipeline

```rust
use macro_os_engines::parse::{MacroPipeline, PipelineConfig};

let output = MacroPipeline::default().parse("notes.md", input_text);
```

`MacroPipeline::default()` builds:

- `CommandRegistry::default()` — all built-in command specs
- `CommandSeedDetector::with_defaults()`
- `BoundarySolver::with_defaults()`
- `PipelineConfig::default()` — loose inference on, preserve unknown commands

## Custom pipeline (swap registries)

```rust
use macro_os_engines::parse::{
    BoundarySolver, CommandRegistry, CommandSeedDetector, CommandSeedStrategyRegistry,
    MacroPipeline, ParseContext, PipelineConfig,
};
use macro_os_engines::parse::model::SourceDocument;

// 1. Command registry — empty for fully custom, or default for built-ins
let command_registry = CommandRegistry::default();

// 2. Command seed strategies — start from defaults, add your own
let mut seed_strategies = CommandSeedStrategyRegistry::with_defaults();
// seed_strategies.register(Box::new(MyCommandSeedStrategy));

let command_seed_detector = CommandSeedDetector::new(seed_strategies);

// 3. Boundary solver — defaults include marker strategies + next-seed assembler
let boundary_solver = BoundarySolver::with_defaults();

// 4. Build pipeline
let pipeline = MacroPipeline::new(
    command_registry,
    command_seed_detector,
    boundary_solver,
    PipelineConfig::default(),
);

let output = pipeline.parse("doc.md", text);
```

Or start from defaults and override one piece:

```rust
let pipeline = MacroPipeline::with_defaults(PipelineConfig::default())
    .with_command_registry(my_registry)
    .with_command_seed_detector(my_detector);
```

## Using ParseContext directly (tests / tooling)

```rust
let doc = SourceDocument::new("fixture.md", text);
let registry = CommandRegistry::default();
let ctx = ParseContext::new(&doc, &registry);

let seeds = CommandSeedDetector::with_defaults().detect(&ctx);
let blocks = BoundarySolver::with_defaults().assemble_blocks(&ctx, &seeds);
let candidates = BoundarySolver::with_defaults().collect_boundary_candidates(&doc);
```

Marker analysis (no registry required for most strategies):

```rust
let markers = SeedDetector::with_defaults().detect_all(&doc);
```

## PipelineConfig

```rust
pub struct PipelineConfig {
    pub enable_loose_inference: bool,   // infer task/idea seeds from prose
    pub preserve_unknown_commands: bool, // keep CommandKind::Unknown seeds
}
```

When comparing seed/block counts in tests, disable loose inference so detector output matches parse output:

```rust
let config = PipelineConfig {
    enable_loose_inference: false,
    ..PipelineConfig::default()
};
let pipeline = MacroPipeline::with_defaults(config);
```

## Adding new behavior (which workflow?)

| Goal | Workflow |
|------|----------|
| New `@command` kind | [add-registry-command.md](add-registry-command.md) |
| New pipeline command seed (`CommandSeed`) | [add-command-seed-strategy.md](add-command-seed-strategy.md) |
| New inline marker (`DetectedSeed`) | [add-seed-detector.md](add-seed-detector.md) |
| New boundary marker candidate | [add-boundary-strategy.md](add-boundary-strategy.md) |
| New block assembly rule | [add-block-assembler.md](add-block-assembler.md) |
| New integration test | [add-integration-test.md](add-integration-test.md) |

## Test targets for the pipeline system

```bash
cargo test --test parse parser_pipeline_detection
cargo test --test parse parser_detection
cargo test --test parse parser_command
cargo test --test parse parser_boundary
```

See [tests/GLOSSARY.md](../tests/GLOSSARY.md) for every subtest name and log output paths.

## Test log directories

Parse tests that emit JSON use fixed paths under `target/test-logs/` (no env vars):

| Directory | Test file | Notable outputs |
|-----------|-----------|-----------------|
| `parser_boundary_test/` | `parser_boundary_test.rs` | `messy_notes_<strategy>_results.json`, `messy_notes_strategy_comparison.json` |
| `parser_detection_test/` | `parser_detection_test.rs` | `detects_at_commands.json`, `detects_chained_commands.json`, `nested_commands_seeds.json` |
| `parser_pipeline_detection_test/` | `parser_pipeline_detection_test.rs` | `architecture_command_seeds.json`, `deep_nested_pipeline_summary.json` |
| `parse_database_test/` | `parse_database_test.rs` | `stats.json`, `table_dumps.json` |

Shared fixture for strategy comparison: `tests/fixtures/example_docs/planner/docs/Scratch/messy_notes.txt`.

## Checklist

- [ ] New strategy implements the correct trait (`CommandSeedStrategy`, `SeedDetectionStrategy`, `BoundaryStrategy`, or `BlockAssemblyStrategy`)
- [ ] Strategy registered in the matching `*Registry::with_defaults()` or a custom registry passed to the pipeline
- [ ] Pipeline test uses `ParseContext` + attached detector/solver (not free functions)
- [ ] Focused test added under `tests/parse/`
- [ ] Row added to `tests/GLOSSARY.md`
- [ ] `cargo test --test parse` passes
