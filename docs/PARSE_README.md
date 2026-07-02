# Inline Macro Processor, Rust Starter

A modular Rust implementation of an expansive inline definition macro parser.

It parses mixed notes like:

```text
@Project @Idea maybe build a queue resolver
@Task fix boundary solver (done)
@Reference ./src/pipeline/mod.rs #rust
Need to revisit alias resolution?
```

into JSON records with command kind, title, content, members, tags, references, statuses, confidence, source trace, and diagnostics.

## Run

```bash
cargo run --bin macro-parse -- fixtures/ambiguous_mixed_input.txt --pretty
```

## Test

```bash
cargo test
```

## Main modules

```text
src/model      parsed object types, diagnostics, spans
src/registry   command specs and aliases
src/passes     detection, inference, boundaries, extraction, normalization, validation
src/pipeline   orchestrates the full parse
src/main.rs    CLI entry point
```

## Important features

- Explicit commands: `@Task`, `@Idea`, `@Project`, `@Prompt`, `@Reference`, etc.
- Command chains: `@Project @Idea`, `@Macro @Clipboard`.
- Loose prose inference for ambiguous task/idea/path-like lines.
- Boundary solver for command bodies.
- Member extraction from `Key: Value` lines.
- Tags, references, and inline statuses such as `(done)` and `(deffered)`.
- Unknown command recovery.
- JSON output and source tracing.
