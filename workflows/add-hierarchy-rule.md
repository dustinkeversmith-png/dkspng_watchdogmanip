# Workflow: Add hierarchy rule

Extend how commands inherit heading, list, or parent context inside a document.

## Files to touch

| File | Change |
|------|--------|
| `src/parse/hierarchy/model.rs` | `ParseHierarchyNode` fields |
| `src/parse/hierarchy/resolver.rs` | `resolve_hierarchy()` logic |
| `src/parse/model/command.rs` | `parent_id`, `heading_context`, etc. |
| `src/parse/model/output.rs` | `ParseOutput.hierarchy` |
| `tests/parse/parser_hierarchy_test.rs` | Hierarchy assertions |

## Steps

### 1. Decide metadata location

Commands carry:

- `parent_id`, `child_ids`, `hierarchy_path`
- `heading_context`, `list_context`

`ParseOutput.hierarchy` holds `Vec<ParseHierarchyNode>` for graph traversal.

### 2. Update resolver

Edit `resolve_hierarchy()` in `src/parse/hierarchy/resolver.rs`:

- Track heading stack when `CommandKind::Inferred("heading_section")`
- Track list context from numbered/bullet lines
- Assign `parent_id` from nearest heading or prior command

### 3. Pipeline hook

`MacroPipeline` already calls `resolve_hierarchy()` after extract in `src/parse/pipeline/mod.rs`. No change needed unless you add a separate pass.

### 4. Test

```rust
let output = MacroPipeline::default().parse("nested.md", include_str!("..."));
assert!(output.hierarchy.len() >= output.commands.len());
assert!(output.commands.iter().any(|c| !c.heading_context.is_empty()));
```

```bash
cargo test --test parse parser_hierarchy
```

## Checklist

- [ ] Resolver assigns new hierarchy metadata
- [ ] `ParseHierarchyNode` updated if graph shape changes
- [ ] Test uses nested headings + lists + scattered commands
- [ ] No panics on ambiguous docs (diagnostics OK)
- [ ] Glossary updated
