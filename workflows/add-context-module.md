# Workflow: Add context module behavior

Extend context building, resolution, or persistence.

## Sub-areas

| Area | Location | Purpose |
|------|----------|---------|
| Build | `src/context/build/` | Walk → context index, folding |
| Index | `src/context/index/` | `ContextIndex` tree |
| Resolver | `src/context/index/resolver.rs` | Path/id resolution |
| Database | `src/context/database/` | SQLite context store |
| Legacy | `src/context/fs_indexer.rs` | File-tree context (wrapped by build) |

## Add build behavior

1. Edit `src/context/build/config.rs` for new `ContextBuildConfig` fields.
2. Implement in `fs_builder.rs` or new file under `build/`.
3. Re-export from `src/context/build/mod.rs`.

Example — folding is in `build/folding.rs`:

- `attach_walked_files()` — map files to contexts
- `fold_small_contexts()` — `min_files_per_context`

## Add resolver behavior

1. Add method on `ContextResolver` in `src/context/index/resolver.rs`.
2. Return `ContextResolution { context_id, matched_by, root_path }`.
3. Test in `tests/context/context_resolution_test.rs`.

```bash
cargo test --test context context_resolver
```

## Add context database table

See [add-database-domain.md](add-database-domain.md) — use `src/context/database/migrations.rs`.

## Test orchestration

```text
walk fixture → build_context_index → ContextResolver queries → ContextStore inserts
```

Example test: `tests/context/parser_navigation_context_test.rs`

```bash
cargo test --test context parser_navigation
```

## Checklist

- [ ] Build uses `src/walk` (no duplicate tree walk)
- [ ] Resolver method + test
- [ ] Migrations if DB changed
- [ ] Glossary updated
