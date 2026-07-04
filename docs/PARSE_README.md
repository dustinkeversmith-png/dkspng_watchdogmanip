# Inline Macro Processor

Ambiguity-tolerant parser for notes mixing `@commands`, markdown structure, inline statuses, references, and loose prose.

## Run

```bash
cargo run --bin macro-os -- parse examples/ambiguous_macros.txt --pretty
cargo test --test parse
```

See **[tests/GLOSSARY.md](../tests/GLOSSARY.md)** for every parse subtest and log output path.

## Architecture (registry-based pipeline)

Parse is organized as attachable registries orchestrated by `MacroPipeline`. There are no free functions like `default_registry()` or `detect_command_seeds()` — detection always goes through detectors + `ParseContext`.

```text
src/parse/
  pipeline/            MacroPipeline, ParseContext, PipelineConfig
  registry/            CommandSpec, CommandRegistry (Default = built-in @commands)
  boundary/
    model.rs           BoundaryMetadataKind, BodyDirection, BodyShapeHint, BoundaryCandidate
    strategies.rs      BoundaryStrategy, BoundaryStrategyRegistry
    solver.rs          BlockAssemblyStrategy, BlockAssemblerRegistry, BoundarySolver
  shape/
    model.rs           CommandShapeKind, TitleCandidate, CommandShapeAnalysis
    detector.rs        CommandShapeDetector (runs after block assembly)
  seeds/
    command.rs         CommandSeedStrategy, CommandSeedStrategyRegistry, CommandSeedDetector
    classifier.rs      Classifier keyword seeds without `@`
    strategies/        SeedDetectionStrategy modules + SeedDetector (marker analysis)
  hierarchy/
    model.rs           HierarchySignal, CommandHierarchyLink, HierarchyDetector trait
    detector.rs        HierarchyDetectorRegistry
    strategies/        heading, numbered-list, bullet-list, indentation detectors
  passes/              extract, infer, normalize, validate
  database/            ParseCommandStore (SQLite), in-memory store, migrations
  parser.rs            Parser façade
```

### Data flow

```text
SourceDocument
    │
    ▼
ParseContext { document, command_registry }
    │
    ├─► CommandSeedDetector ──► Vec<CommandSeed>
    │       (CommandSeedStrategyRegistry)
    │
    ├─► optional infer_loose_objects (PipelineConfig)
    │
    ├─► BoundarySolver.assemble_blocks ──► Vec<CommandBlock>
    │       (BlockAssemblerRegistry)
    │
    ├─► CommandShapeDetector.analyze ──► shape_analysis on each block
    │
    └─► extract → normalize → validate → hierarchy → ParseOutput
```

Marker-only analysis (no registry command resolution):

```text
SeedDetector.detect_all(document) ──► Vec<DetectedSeed>
BoundarySolver.collect_boundary_candidates(document) ──► Vec<BoundaryCandidate>
```

## Quick API

```rust
use macro_os_engines::parse::MacroPipeline;

let output = MacroPipeline::default().parse("notes.md", input_text);

// With file path for structured location metadata:
use macro_os_engines::parse::SourceDocument;
use std::path::PathBuf;

let doc = SourceDocument::with_path("notes.md", Some(PathBuf::from("docs/notes.md")), input_text);
let output = MacroPipeline::default().parse_document(doc);
```

Custom registries:

```rust
use macro_os_engines::parse::{
    BoundarySolver, CommandRegistry, CommandSeedDetector, MacroPipeline, ParseContext,
    PipelineConfig,
};

let pipeline = MacroPipeline::with_defaults(PipelineConfig::default())
    .with_command_registry(CommandRegistry::default());

let doc = macro_os_engines::parse::SourceDocument::new("doc.md", text);
let ctx = ParseContext::new(&doc, pipeline.command_registry());
let seeds = pipeline.command_seed_detector().detect(&ctx);
```

## PipelineConfig

| Field | Default | Effect |
|-------|---------|--------|
| `enable_loose_inference` | `true` | Infer task/idea/path seeds from uncovered prose lines |
| `preserve_unknown_commands` | `true` | Keep `CommandKind::Unknown` seeds in output |

When comparing seed/block counts in tests, set `enable_loose_inference: false`.

## Database

Parse commands persist via `ParseCommandStore` in `src/parse/database/`:

```rust
use macro_os_engines::parse::database::ParseCommandStore;

let db = ParseCommandStore::open("parse.sqlite")?;
let id = db.insert_parsed_command("file.md", parsed_command)?;
```

Shared SQLite helpers live in `src/database/` (connection, migrations). Parse-specific schema lives in `src/parse/database/migrations.rs`.

## Extending

| Goal | Workflow |
|------|----------|
| Pipeline overview | [workflows/pipeline-and-registries.md](../workflows/pipeline-and-registries.md) |
| New `@command` | [workflows/add-registry-command.md](../workflows/add-registry-command.md) |
| New pipeline seed | [workflows/add-command-seed-strategy.md](../workflows/add-command-seed-strategy.md) |
| New marker detector | [workflows/add-seed-detector.md](../workflows/add-seed-detector.md) |
| New boundary marker | [workflows/add-boundary-strategy.md](../workflows/add-boundary-strategy.md) |
| New block assembler | [workflows/add-block-assembler.md](../workflows/add-block-assembler.md) |
| New integration test | [workflows/add-integration-test.md](../workflows/add-integration-test.md) |

## Loose parsing model

The parser collects candidates (seeds, boundaries, shapes, title candidates, hierarchy signals) with confidence and diagnostics instead of strict rejection. `CommandBodyPolicy::CaptureIfPresent` is the default — bodies are captured when present; weird layouts produce diagnostics, not failures.

## Parse test files

| File | Focus |
|------|-------|
| `parser_boundary_test.rs` | `BoundaryStrategy` per-strategy + merged candidates |
| `parser_boundary_metadata_test.rs` | Rich boundary metadata JSON (kind, direction, evidence) |
| `parser_detection_test.rs` | `SeedDetectionStrategy` modules + `SeedDetector` |
| `parser_classifier_seed_test.rs` | Classifier keyword seeds without `@` |
| `parser_command_test.rs` | `CommandRegistry` + `CommandSeedDetector` + full pipeline |
| `parser_pipeline_detection_test.rs` | Attached detector/solver on pipeline |
| `parser_body_parsing_test.rs` | Inline, next-line, key-value, bracketed, mixed bodies + shapes |
| `parser_hierarchy_test.rs` | Heading/list hierarchy metadata on parsed commands |
| `parser_hierarchy_detector_test.rs` | Attachable hierarchy detectors (heading, list, indent) |
| `parser_location_tracking_test.rs` | `SourceLocation`, file path, DB location columns |
| `parse_database_test.rs` | Pipeline → SQLite insert/search/dump |
| `real_path_tree_database_test.rs` | Walk → parse → DB → cross-ref logs (env root) |

Log outputs: `target/test-logs/<test_file_name>/` (see glossary).

## Features

- Explicit commands: `@Task`, `@Idea`, `@Project`, `@Reference`, etc.
- Classifier keywords without `@`: `Task:`, `Project Idea:`, `Reference ./path`
- Command chains: `@Project @Idea`, numbered-list prefixes (`1. @Task`)
- Shape analysis: inline title, key-value members, bracketed body, mixed shapes
- Boundary metadata: body direction, shape hints, evidence, confidence
- Structured source location on every command (`SourceLocation` + `source_trace`)
- Attachable hierarchy detectors (markdown headings, numbered/bullet lists, indentation)
- Registry aliases: `@todo` → `@Task`, `@deferred` / `@deffered`
- Loose prose inference for ambiguous task/idea/path-like lines
- Modular boundary marker strategies + next-seed block assembly
- Member extraction from `Key: Value` lines
- Tags, references, inline statuses `(done)`, `(building)`
- Unknown command recovery
- JSON test logs for strategy comparison
