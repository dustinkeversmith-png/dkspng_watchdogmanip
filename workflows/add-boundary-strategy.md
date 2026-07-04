# Workflow: Add a boundary strategy

Detect boundary **marker candidates** in inconsistently formatted docs (where blocks might start/end).

Boundary strategies feed analysis and `BoundarySolver::collect_boundary_candidates`. Block bodies are assembled separately by `BlockAssemblyStrategy` — see [add-block-assembler.md](add-block-assembler.md).

## Files to touch

| File | Change |
|------|--------|
| `src/parse/boundary/model.rs` | Add `BoundaryMarkerKind` variant if new type |
| `src/parse/boundary/strategies.rs` | Implement `BoundaryStrategy` |
| `src/parse/boundary/strategies.rs` | Register in `BoundaryStrategyRegistry::with_defaults()` |
| `tests/parse/parser_boundary_test.rs` | Per-strategy test on messy fixture |

## Steps

### 1. Add marker kind (optional)

```rust
// src/parse/boundary/model.rs
pub enum BoundaryMarkerKind {
    // ...
    MyBoundary,
}
```

### 2. Implement strategy

Keep regex/pattern logic inside the struct (not module-level statics):

```rust
pub struct MyBoundaryStrategy;

impl BoundaryStrategy for MyBoundaryStrategy {
    fn name(&self) -> &'static str {
        "my_boundary"
    }

    fn find_boundaries(&self, document: &ParseDocumentInput) -> Vec<BoundaryCandidate> {
        document.lines.iter().map(|line| BoundaryCandidate {
            kind: BoundaryMarkerKind::MyBoundary,
            start_line: line.number,
            end_line: Some(line.number),
            confidence: 0.7,
            reason: "description".to_string(),
        }).collect()
    }
}
```

### 3. Register in BoundaryStrategyRegistry

```rust
impl BoundaryStrategyRegistry {
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        // existing strategies...
        registry.register(Box::new(MyBoundaryStrategy));
        registry
    }
}
```

### 4. Use via BoundarySolver

```rust
let solver = BoundarySolver::with_defaults();
let candidates = solver.collect_boundary_candidates(&doc);
```

Or attach custom registry to the pipeline:

```rust
let mut strategies = BoundaryStrategyRegistry::with_defaults();
strategies.register(Box::new(MyBoundaryStrategy));
let solver = BoundarySolver::new(strategies, BlockAssemblerRegistry::with_defaults());
```

### 5. Test

```rust
#[test]
fn my_boundary_strategy_finds_markers() {
    let doc = SourceDocument::new("test.md", include_str!("../fixtures/..."));
    let candidates = MyBoundaryStrategy.find_boundaries(&doc);
    assert!(!candidates.is_empty());
}
```

```bash
cargo test --test parse my_boundary
cargo test --test parse parser_boundary
```

Writes per-strategy JSON to `target/test-logs/parser_boundary_test/messy_notes_<strategy>_results.json`.

## Existing strategies

| Strategy | `BoundaryMarkerKind` |
|----------|----------------------|
| `CommandSeedBoundaryStrategy` | `CommandStart` |
| `HeadingBoundaryStrategy` | `HeadingBoundary` |
| `BlankLineBoundaryStrategy` | `BlankLineBoundary` |
| `IndentationBoundaryStrategy` | `IndentationBoundary` |
| `InlineCommandBoundaryStrategy` | `InlineCommand` |

## Checklist

- [ ] Strategy implements `BoundaryStrategy`
- [ ] Registered in `BoundaryStrategyRegistry::with_defaults()` or custom registry on pipeline
- [ ] Test on messy fixture (not just clean markdown)
- [ ] Glossary updated

See also: [pipeline-and-registries.md](pipeline-and-registries.md)
