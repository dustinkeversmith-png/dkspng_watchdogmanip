# Workflow: Add an integration test

Add a new test under `tests/` following the modular orchestration pattern.

## Choose a domain entry file

| Domain | Entry | Subfolder |
|--------|-------|-----------|
| parse | `tests/parse.rs` | `tests/parse/` |
| context | `tests/context.rs` | `tests/context/` |
| database | `tests/database.rs` | `tests/database/` |
| history | `tests/history.rs` | `tests/history/` |
| walk | `tests/walk.rs` | `tests/walk/` |
| cross-engine | `tests/engine_fixture_tests.rs` | inline |
| smoke | `tests/integrated_engines_tests.rs` | inline |

Prefer subfolder + `#[path]` for domain-specific tests.

## Steps

### 1. Create test file

Parse tests should use the registry pipeline — not removed free functions:

```rust
// tests/parse/my_feature_test.rs
use macro_os_engines::parse::{MacroPipeline, ParseContext, CommandRegistry};
use macro_os_engines::parse::model::SourceDocument;

#[test]
fn my_feature_does_expected_thing() {
    let output = MacroPipeline::default().parse("fixture.md", "@Task Do thing");
    assert!(!output.commands.is_empty());
}

// Or test detection in isolation:
#[test]
fn my_feature_detects_via_context() {
    let doc = SourceDocument::new("fixture.md", "@Task Do thing");
    let registry = CommandRegistry::default();
    let ctx = ParseContext::new(&doc, &registry);
    let pipeline = MacroPipeline::default();
    let seeds = pipeline.command_seed_detector().detect(&ctx);
    assert!(!seeds.is_empty());
}
```

### 2. Wire entry file

```rust
// tests/parse.rs
#[path = "parse/my_feature_test.rs"]
mod my_feature_test;
```

### 3. Add fixture (if needed)

Place under `tests/fixtures/` or `tests/fixtures/example_docs/`. Reuse `messy_notes.txt` for strategy comparison tests when appropriate.

Keep subsets small; no `.git`, `node_modules`, or huge generated trees.

### 4. Optional JSON log output

Match existing parse tests — fixed directory named after the test file:

```rust
const LOG_DIR: &str = "target/test-logs/my_feature_test";

fn write_json(path: std::path::PathBuf, value: &serde_json::Value) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create log dir");
    }
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}
```

Do not use env vars for standard parse unit tests. Reserve env vars for optional real-path tests only.

### 5. Run individually

```bash
cargo test --test parse my_feature_does_expected_thing
cargo test --test parse my_feature
```

### 6. Document in glossary

Add file, subtest name, fixture, log path, and run command to `tests/GLOSSARY.md`.

### 7. Full suite

```bash
cargo fmt
cargo test
```

## Orchestration template (multi-phase test)

```rust
#[test]
fn walk_parse_database_efficacy() {
    // 1. walk
    let walked = TreeWalker::new(config).walk().unwrap();
    let pipeline = MacroPipeline::default();
    let db = ParseCommandStore::open(temp.path().join("parse.sqlite")).unwrap();
    // 2. read + parse (per file)
    for file in &walked.files {
        let text = fs::read_to_string(&file.path).unwrap();
        let output = pipeline.parse(&file.source_name, text);
        // 3. insert one-by-one
        for cmd in output.commands {
            db.insert_parsed_command(&file.source_name, cmd).unwrap();
        }
    }
    // 4. query
    let hits = db.search(options).unwrap();
    assert!(!hits.is_empty());
}
```

Do not merge phases into a single mega-component.

## Real-path / env-var tests

Only for optional tests that walk a user-specific directory:

```rust
let root = std::env::var("PARSE_TEST_ROOT").unwrap_or_else(|_| "...".into());
if !PathBuf::from(&root).exists() {
    eprintln!("skipping: root missing");
    return;
}
```

## Parse test file guide

| If testing… | Prefer file |
|-------------|-------------|
| Boundary marker strategies | `parser_boundary_test.rs` |
| Seed marker detectors | `parser_detection_test.rs` |
| Command registry / specs | `parser_command_test.rs` |
| Pipeline-attached detector/solver | `parser_pipeline_detection_test.rs` |
| Hierarchy metadata | `parser_hierarchy_test.rs` |
| SQLite round-trip | `parse_database_test.rs` |

See [pipeline-and-registries.md](pipeline-and-registries.md).

## Checklist

- [ ] Test file created
- [ ] Entry `#[path]` added
- [ ] Fixture added if needed
- [ ] Uses `MacroPipeline` / `ParseContext` / registries (not legacy free functions)
- [ ] Individual `cargo test --test ...` passes
- [ ] `tests/GLOSSARY.md` updated
- [ ] Log dir documented if JSON output added
