# Workflow: Add an extractor

Extract a new field (or member type) from parsed command body text.

## Files to touch

| File | Change |
|------|--------|
| `src/parse/extractors/extractors.rs` | New `CommandExtractor` impl |
| `src/parse/extractors/mod.rs` | Export if split to own file |
| `src/parse/extractors/model.rs` | Extend `ExtractedCommandParts` if new field |
| `src/parse/passes/extract.rs` | Legacy path (optional sync) |

## Steps

### 1. Extend parts model (if needed)

```rust
// src/parse/extractors/model.rs
pub struct ExtractedCommandParts {
    // existing fields...
    pub my_field: Vec<String>,  // add with #[derive(Default)]
}
```

Update `merge_parts()` in `extractors.rs` to merge the new field.

### 2. Implement extractor

```rust
pub struct MyFieldExtractor;

impl CommandExtractor for MyFieldExtractor {
    fn name(&self) -> &'static str {
        "my_field"
    }

    fn extract(&self, context: &ExtractionContext) -> ExtractedCommandParts {
        // read context.body, context.seed
        ExtractedCommandParts {
            my_field: vec![],
            ..Default::default()
        }
    }
}
```

### 3. Register in `default_extractors()`

```rust
vec![
    // ...
    Box::new(MyFieldExtractor),
]
```

### 4. Map to `ParsedCommand` (when wired to pipeline)

Today the pipeline uses `src/parse/passes/extract.rs`. Either:

- Call modular extractors from `extract_block()` eventually, or
- Mirror logic in `passes/extract.rs` until migration completes

Prefer one source of truth; don't duplicate regexes long-term.

### 5. Test

Use `MacroPipeline::default().parse(...)` and assert the field on `ParsedCommand` or `members` map.

```bash
cargo test --test parse my_field
```

## Design notes

- Most parameters are **optional** — extractors should not assume every command has them
- Use `ExtractionContext.body` for block text, `context.seed.raw` for inline payload
- Key-value lines (`Title: foo`) → `KeyValueMemberExtractor` pattern

## Checklist

- [ ] Extractor implements `CommandExtractor`
- [ ] Registered in `default_extractors()`
- [ ] `merge_parts` updated
- [ ] Test with optional + missing field cases
- [ ] Glossary updated
