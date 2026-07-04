# Workflows

Step-by-step guides for extending the modular engine layout.

Each workflow is a checklist you can follow when adding new behavior without coupling subsystems.

## Index

| Workflow | When to use |
|----------|-------------|
| [pipeline-and-registries.md](pipeline-and-registries.md) | **Start here** — `MacroPipeline`, `ParseContext`, registries, attaching detectors |
| [add-registry-command.md](add-registry-command.md) | New `@command` kind in `CommandRegistry` |
| [add-command-seed-strategy.md](add-command-seed-strategy.md) | New pipeline seed strategy → `CommandSeed` |
| [add-seed-detector.md](add-seed-detector.md) | New marker detector → `DetectedSeed` (analysis) |
| [add-boundary-strategy.md](add-boundary-strategy.md) | New boundary marker candidate strategy |
| [add-block-assembler.md](add-block-assembler.md) | New seed → `CommandBlock` assembly rule |
| [add-extractor.md](add-extractor.md) | New field extraction from command body text |
| [add-hierarchy-rule.md](add-hierarchy-rule.md) | New parent/heading/list hierarchy behavior |
| [add-integration-test.md](add-integration-test.md) | New test under `tests/` |
| [add-context-module.md](add-context-module.md) | Context build, resolver, or database extension |
| [add-database-domain.md](add-database-domain.md) | New SQLite domain tables or migrations |

## Module map (where things live)

```text
src/walk/              Tree walking only — no parse, no DB
src/parse/
  pipeline/            MacroPipeline, ParseContext, PipelineConfig
  registry/            CommandSpec + CommandRegistry (Default = built-ins)
  seeds/
    command.rs         CommandSeedStrategy, CommandSeedDetector, CommandSeedStrategyRegistry
    strategies/        SeedDetectionStrategy modules + SeedDetector
  boundary/
    strategies.rs      BoundaryStrategy + BoundaryStrategyRegistry
    solver.rs          BlockAssemblyStrategy, BlockAssemblerRegistry, BoundarySolver
  extractors/          Title, tags, references, parameters, …
  hierarchy/           Document parent/heading context
  parser.rs            Parser façade
  database/            Parse command SQLite + in-memory store
  passes/              extract, infer, normalize, validate

src/context/
  build/               ContextBuildConfig, folding, walk integration
  index/               ContextIndex + ContextResolver
  database/            Context SQLite store

src/database/          Shared connection + migrations helpers
src/history/
  database/            History SQLite store
  suggestions/         Scoring (separate from storage)
src/navigation/        Alias/target resolution
src/watchdog/          Rules, routines, planner
src/test_logging/      Cross-referenced test output builder
```

## Test-first rule

Every workflow ends with:

1. Add or extend a focused test under `tests/`.
2. Run the targeted command from [tests/GLOSSARY.md](../tests/GLOSSARY.md).
3. Run `cargo fmt` and `cargo test`.

## Orchestration rule

Tests orchestrate; components do not call each other across boundaries:

```text
walk → read file → parse (MacroPipeline) → insert (one-by-one) → query/search → log output
```

Inside parse, the pipeline owns registries:

```text
ParseContext → CommandSeedDetector → BoundarySolver → extract → hierarchy
```

Do not put parse logic in walk, DB writes in parse, or filesystem walks inside extractors.
