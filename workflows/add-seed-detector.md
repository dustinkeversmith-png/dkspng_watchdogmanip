# Workflow: Add a seed detector

Detect inline **markers** in raw text (`DetectedSeed`) for analysis and logging.

For pipeline command seeds (`CommandSeed`), use [add-command-seed-strategy.md](add-command-seed-strategy.md) instead.

## Files to touch

| File | Change |
|------|--------|
| `src/parse/seeds/model.rs` | Add `SeedKind` variant if new category |
| `src/parse/seeds/strategies/<name>.rs` | Implement `SeedDetectionStrategy` |
| `src/parse/seeds/strategies/mod.rs` | Export + register in `SeedDetectionStrategyRegistry::with_defaults()` |
| `tests/parse/parser_detection_test.rs` | Submodule test per strategy |

## Steps

### 1. Define seed kind (optional)

```rust
// src/parse/seeds/model.rs
pub enum SeedKind {
    // ...
    MyMarker,
}
```

### 2. Implement detector module

```rust
// src/parse/seeds/strategies/my_marker.rs
use super::SeedDetectionStrategy;
use crate::parse::seeds::model::{DetectedSeed, ParseDocumentInput, SeedKind};

pub struct MyMarkerSeedDetector;

impl SeedDetectionStrategy for MyMarkerSeedDetector {
    fn name(&self) -> &'static str {
        "my_marker"
    }

    fn detect(&self, input: &ParseDocumentInput) -> Vec<DetectedSeed> {
        input.lines.iter().filter_map(|line| {
            // return DetectedSeed { kind, raw, normalized_identity, line, column, confidence, payload }
            None
        }).collect()
    }
}
```

Keep regex inside the struct (see `AtCommandSeedDetector` for the pattern).

### 3. Register in SeedDetectionStrategyRegistry

```rust
// src/parse/seeds/strategies/mod.rs
pub mod my_marker;
pub use my_marker::MyMarkerSeedDetector;

impl SeedDetectionStrategyRegistry {
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        // existing...
        registry.register(Box::new(MyMarkerSeedDetector));
        registry
    }
}
```

### 4. Run detection via SeedDetector

```rust
let markers = SeedDetector::with_defaults().detect_all(&doc);
```

Custom registry:

```rust
let mut registry = SeedDetectionStrategyRegistry::with_defaults();
registry.register(Box::new(MyMarkerSeedDetector));
let detector = SeedDetector::new(registry);
```

### 5. Test

Add a submodule in `tests/parse/parser_detection_test.rs`:

```rust
mod my_marker {
    use super::*;

    #[test]
    fn detects_my_marker() {
        let doc = SourceDocument::new("test.md", "some @mymarker text");
        let seeds = MyMarkerSeedDetector.detect(&doc);
        assert!(seeds.iter().any(|s| s.kind == SeedKind::MyMarker));
    }
}
```

```bash
cargo test --test parse detects_my_marker
cargo test --test parse parser_detection
```

## Existing detectors

| Detector | Finds |
|----------|-------|
| `AtCommandSeedDetector` | `@Task`, `@Alias`, payload on same line |
| `ChainedAtCommandSeedDetector` | `@Project @Idea` chains |
| `InlineStatusSeedDetector` | `(done)`, `(deferred)` |
| `ReferenceSeedDetector` | `@Reference`, `@ref` |
| `CurrentSeedDetector` | `@current` |

## Shared fixture: messy_notes.txt

`tests/fixtures/example_docs/planner/docs/Scratch/messy_notes.txt` is the standard messy-doc fixture for:

- `AtCommandSeedDetector` — asserts `idea`, `tutorial`, non-empty `payload`
- `ChainedAtCommandSeedDetector` — JSON log only (no `@A @B` chains in file today)
- `BoundaryStrategy` tests — see `parser_boundary_test.rs`

When adding assertions, match identities present in the fixture (not `@Task` unless you add it to the file).

## Checklist

- [ ] `SeedDetectionStrategy` implemented in `src/parse/seeds/strategies/`
- [ ] Registered in `SeedDetectionStrategyRegistry::with_defaults()`
- [ ] Test submodule in `parser_detection_test.rs`
- [ ] Glossary updated

See also: [pipeline-and-registries.md](pipeline-and-registries.md)
