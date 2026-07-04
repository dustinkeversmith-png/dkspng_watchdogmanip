# Workflow: Add a registry command

Add a new `@command` kind that the parser recognizes and extracts.

## Files to touch

| File | Change |
|------|--------|
| `src/parse/model/command.rs` | Add variant to `CommandKind` if new kind |
| `src/parse/registry/mod.rs` | Register `CommandSpec` in `CommandRegistry` impl `Default` |
| `tests/parse/parser_command_test.rs` | Fixture text + registry / pipeline test |

## Steps

### 1. Add or reuse `CommandKind`

```rust
// src/parse/model/command.rs
pub enum CommandKind {
    // ...
    MyNewKind,
    Unknown(String),
    Inferred(String),
}
```

Skip if an existing kind fits (e.g. `Reference`, `Task`).

### 2. Register the command spec

In `src/parse/registry/mod.rs` inside `impl Default for CommandRegistry`:

```rust
spec(
    MyNewKind,
    "mynew",                    // canonical name
    &["alias1", "alias2"],       // aliases
    &[],                          // optional_parameters
    &["Title", "Description"],    // optional_members
    UntilNextCommand,             // boundary hint
),
```

For a fully custom registry (no built-ins):

```rust
let mut registry = CommandRegistry::new();
registry.register(CommandSpec { /* ... */ });
```

Built-in specs load via `CommandRegistry::default()`.

`CommandSpec` fields:

- `optional_parameters` / `optional_members` — preferred; most params are optional
- `required_members` — only when truly required
- `accepted_layouts` — defaults to `Block` + `Inline`
- `allow_loose_body: true` — typical for messy docs

### 3. Verify command seed detection

`AtCommandLineSeedStrategy` picks up `@mynew` when the line matches the `@command` pattern. For special syntax, add a [command seed strategy](add-command-seed-strategy.md).

### 4. Add test

```rust
use macro_os_engines::parse::{
    CommandKind, CommandRegistry, CommandSeedDetector, MacroPipeline, ParseContext,
};
use macro_os_engines::parse::model::SourceDocument;

#[test]
fn registry_recognizes_mynew_command() {
    let output = MacroPipeline::default().parse("fixture.md", "@mynew Do the thing\nBody.");
    assert!(output.commands.iter().any(|c| matches!(c.kind, CommandKind::MyNewKind)));
}

// Or test detection only:
#[test]
fn registry_resolves_mynew_via_context() {
    let doc = SourceDocument::new("fixture.md", "@mynew Do the thing");
    let registry = CommandRegistry::default();
    let ctx = ParseContext::new(&doc, &registry);
    let seeds = CommandSeedDetector::with_defaults().detect(&ctx);
    assert!(seeds.iter().any(|s| matches!(s.canonical_kind, CommandKind::MyNewKind)));
}
```

### 5. Run

```bash
cargo test --test parse mynew
cargo test --test parse parser_command
cargo fmt
```

## Checklist

- [ ] `CommandKind` variant added (if needed)
- [ ] `CommandSpec` registered in `CommandRegistry::default()` or custom registry
- [ ] Fixture covers messy formatting variant
- [ ] Test passes via `MacroPipeline` or `ParseContext` + `CommandSeedDetector`
- [ ] Row added to `tests/GLOSSARY.md`

See also: [pipeline-and-registries.md](pipeline-and-registries.md)
