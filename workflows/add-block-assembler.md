# Workflow: Add a block assembly strategy

Turn detected `CommandSeed` rows into `CommandBlock` bodies (content span + boundary kind).

## Files to touch

| File | Change |
|------|--------|
| `src/parse/boundary/solver.rs` | Implement `BlockAssemblyStrategy` |
| `src/parse/boundary/solver.rs` | Register in `BlockAssemblerRegistry::with_defaults()` (optional) |
| `tests/parse/parser_pipeline_detection_test.rs` | Test via `BoundarySolver` + `ParseContext` |

## Steps

### 1. Implement the strategy

```rust
use crate::parse::boundary::model::CommandBlock;
use crate::parse::model::CommandSeed;
use crate::parse::pipeline::ParseContext;

pub struct MyBlockAssemblyStrategy;

impl BlockAssemblyStrategy for MyBlockAssemblyStrategy {
    fn name(&self) -> &'static str {
        "my_block_assembler"
    }

    fn assemble_blocks(
        &self,
        ctx: &ParseContext,
        seeds: &[CommandSeed],
    ) -> Vec<CommandBlock> {
        // decide end-of-block per seed using ctx.document lines
        Vec::new()
    }
}
```

### 2. Register in BlockAssemblerRegistry

```rust
impl BlockAssemblerRegistry {
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(NextSeedBlockAssemblyStrategy));
        registry.register(Box::new(MyBlockAssemblyStrategy)); // new
        registry
    }
}
```

`BlockAssemblerRegistry::assemble_blocks` tries assemblers in order; the first non-empty result wins (or empty seeds).

### 3. Attach via BoundarySolver → pipeline

```rust
let mut assemblers = BlockAssemblerRegistry::with_defaults();
assemblers.register(Box::new(MyBlockAssemblyStrategy));

let mut boundary = BoundaryStrategyRegistry::with_defaults();
let solver = BoundarySolver::new(boundary, assemblers);

let pipeline = MacroPipeline::with_defaults(PipelineConfig::default())
    .with_boundary_solver(solver);
```

### 4. Test

```rust
let doc = SourceDocument::new("test.md", include_str!("../fixtures/..."));
let registry = CommandRegistry::default();
let ctx = ParseContext::new(&doc, &registry);
let seeds = CommandSeedDetector::with_defaults().detect(&ctx);
let blocks = BoundarySolver::with_defaults().assemble_blocks(&ctx, &seeds);
assert_eq!(blocks.len(), seeds.len());
```

```bash
cargo test --test parse pipeline_assembles_blocks
cargo test --test parse parser_pipeline_detection
```

## Existing assemblers

| Strategy | Rule |
|----------|------|
| `NextSeedBlockAssemblyStrategy` | Block runs until next seed, outdent, blank-before-heading, or EOF |

## Checklist

- [ ] Implements `BlockAssemblyStrategy` with `assemble_blocks(&ParseContext, &[CommandSeed])`
- [ ] Registered in `BlockAssemblerRegistry`
- [ ] Pipeline test verifies block count / body content
- [ ] Glossary updated

See also: [add-boundary-strategy.md](add-boundary-strategy.md), [pipeline-and-registries.md](pipeline-and-registries.md)
