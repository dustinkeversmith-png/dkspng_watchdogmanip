# Macro OS Engines App

This package merges the previously separate Rust applications into **one Cargo application** while keeping each engine modularly separated.

## Engine modules

```txt
src/
  parse/       # registry-based macro parser (see docs/PARSE_README.md)
  context/     # context hierarchy, local queues/currents, aliases, symbols
  navigation/  # typed alias/navigation resolver and dry-run plans
  history/     # append-only history events, JSONL store, frequency/suggestion scoring
  watchdog/    # watch specs, rules, routines, simulated file events, action planner
  database/    # shared SQLite connection + migration helpers
  walk/        # tree walking (files only, no parse)
  test_logging/# cross-referenced JSON test output builder
```

The crate exposes a library named `macro_os_engines` and one binary named `macro-os`.

## Validation

```bash
cargo fmt
cargo test
```

### Running tests individually

See **[tests/GLOSSARY.md](tests/GLOSSARY.md)** for every integration target, subtest name, fixture path, and exact `cargo test` command.

Example:

```bash
cargo test --test parse                    # 17 parse subtests
cargo test --test parse parser_pipeline_detection
cargo test --test parse parser_detection
cargo test --test parse parser_boundary
cargo test --test context context_resolver
cargo test --test history suggestion_engine
```

Parse test JSON logs: `target/test-logs/<test_file_name>/` (see glossary).

### Extending modular parse/context/database

Start with **[workflows/pipeline-and-registries.md](workflows/pipeline-and-registries.md)** for the parse pipeline.

See **[workflows/README.md](workflows/README.md)** for checklists: registry commands, command seed strategies, seed detectors, boundary strategies, block assemblers, extractors, integration tests, database domains.

Parse architecture: **[docs/PARSE_README.md](docs/PARSE_README.md)** · Fixture map: **[docs/TEST_FIXTURE_COVERAGE.md](docs/TEST_FIXTURE_COVERAGE.md)**.

## CLI examples

### Parse engine

```bash
cargo run --bin macro-os -- parse examples/ambiguous_macros.txt --pretty
```

### Context engine

```bash
cargo run --bin macro-os -- context index examples/project_contexts.txt
cargo run --bin macro-os -- context tree examples/project_contexts.txt --root project
cargo run --bin macro-os -- context inspect examples/project_contexts.txt parser
```

### Navigation engine

```bash
cargo run --bin macro-os -- nav mock
cargo run --bin macro-os -- nav resolve parser --scope parser
cargo run --bin macro-os -- nav plan parser-workspace --scope project --action open
```

### History engine

```bash
cargo run --bin macro-os -- history print examples/mock_history_events.jsonl
cargo run --bin macro-os -- history stats examples/mock_history_events.jsonl --limit 10
cargo run --bin macro-os -- history suggest examples/mock_history_events.jsonl parser --context parser --workspace macro_processor
cargo run --bin macro-os -- history mock --out .macro/history.jsonl
```

### Watchdog engine

```bash
cargo run --bin macro-os -- watchdog validate examples/watch_spec.json
cargo run --bin macro-os -- watchdog list-rules examples/watch_spec.json
cargo run --bin macro-os -- watchdog simulate examples/watch_spec.json examples/file_events.jsonl
cargo run --bin macro-os -- watchdog simulate examples/watch_spec.json examples/file_events.jsonl --expand-routines
```

## Design note

This intentionally remains one application, not a Cargo workspace. The engines are separated by module boundaries, so later you can split any module back into a crate if it grows too large.


## Added Deep Fixture Tests

This package now includes integration-style tests for the requested engines:

```bash
cargo test context_file_tree_fixture_assigns_unique_context_layers_and_indexes_up_down
cargo test watchdog_filters_file_types_ignores_paths_and_expands_routines
cargo test watchdog_timer_event_runs_timely_routine_from_fixture
cargo test parser_deeply_nested_commands_are_inserted_into_database_and_searchable
cargo test history_log_tracks_file_navigation_explorer_locations_and_console_commands
```

New fixtures live under:

```text
tests/fixtures/deep_tree/
tests/fixtures/deep_nested_macros.md
tests/fixtures/watch_spec_file_types_and_timer.json
tests/fixtures/file_change_events.jsonl
tests/fixtures/history_navigation_commands.jsonl
```

See `docs/TEST_FIXTURE_COVERAGE.md` for details.
