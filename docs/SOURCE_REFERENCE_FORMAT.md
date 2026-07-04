# Source reference format

How parsed commands point back to files/lines, and how real-path test JSON logs cross-reference entities.

## SourceLocation + source_trace

Every `ParsedCommand` carries:

| Field | Example |
|-------|---------|
| `location.source_name` | `planner/docs/Scratch/messy_notes.txt` |
| `location.file_path` | `examples/parse_example/planner/docs/Scratch/messy_notes.txt` |
| `location.start_line` | `4` |
| `location.end_line` | `6` (optional) |
| `source_trace` | `planner/docs/Scratch/messy_notes.txt:4-6` |

Use `MacroPipeline::parse_document` with `SourceDocument::with_path` so `file_path` is populated.

```rust
let doc = SourceDocument::with_path(relative_name, Some(full_path), text);
let output = MacroPipeline::default().parse_document(doc);
```

## Cross-ref test log IDs

Real-path test output (`target/test-logs/parser_real_path_test/`) uses stable refs:

| ID | Pattern |
|----|---------|
| `file_ref` | `file:{source_name}` |
| `parse_file_ref` | `parse_file:{source_name}` |
| `command_ref` | `command:{source_name}:{command_id}` |
| `db_ref` | `db:parsed_commands:{row_id}` |

Each command record includes `location`, `source_trace`, and derived `source_span`.

## Real-path walk test

Walks the **full** `examples/parse_example/` tree (all nested projects).

```bash
cargo test --test parse logged_real_path_walk_macropipeline_database_efficacy -- --nocapture
```

Performance notes (475 md/txt files → ~8s on a typical dev machine):

- Parallel parse (`PARSE_TEST_THREADS`, default = CPU count)
- `enable_loose_inference: false` for walk throughput
- SQLite batch transaction + WAL pragmas for inserts
- Only `.md` / `.txt` (not `.rs` source files)
- Skips files over 1 MiB

Optional env:

| Variable | Default | Purpose |
|----------|---------|---------|
| `PARSE_TEST_THREADS` | CPU count | Parallel parse workers |
| `PARSE_TEST_LOG_DIR` | `target/test-logs/parser_real_path_test` | JSON log output dir |

See also [docs/PARSE_README.md](PARSE_README.md) and [tests/GLOSSARY.md](../tests/GLOSSARY.md).
