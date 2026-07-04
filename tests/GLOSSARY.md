# Test Glossary

How to run every test in this repo individually — by integration target, source file, and subtest name.

Run all tests from the repo root:

```bash
cd watchdogmanip
cargo test
```

Run with output:

```bash
cargo test -- --nocapture
```

List every test name without running:

```bash
cargo test -- --list
```

---

## How filtering works

| What you want | Command pattern |
|---------------|-------------------|
| One integration target | `cargo test --test <target>` |
| One subtest (name filter) | `cargo test --test <target> <substring>` |
| Library unit tests only | `cargo test --lib` |
| One library subtest | `cargo test --lib <substring>` |
| Exact subtest name | `cargo test --test <target> exact_test_name` |

Integration targets are the `tests/*.rs` entry files (`parse`, `context`, `walk`, etc.). Subtest source files live in subfolders (`tests/parse/`, `tests/context/`, …) and are wired in via `#[path = "..."]` in those entry files.

Nested modules in test files appear as `module_name::test_name` in `cargo test -- --list`.

---

## Integration targets at a glance

| Target | Entry file | Subtest files | Subtest count |
|--------|------------|---------------|---------------|
| `parse` | `tests/parse.rs` | `tests/parse/*.rs` | 17 |
| `context` | `tests/context.rs` | `tests/context/*.rs` | 5 |
| `database` | `tests/database.rs` | `tests/database/*.rs` | 8 |
| `history` | `tests/history.rs` | `tests/history/*.rs` | 3 |
| `walk` | `tests/walk.rs` | `tests/walk/*.rs` | 1 |
| `engine_fixture_tests` | `tests/engine_fixture_tests.rs` | (inline) | 6 |
| `integrated_engines_tests` | `tests/integrated_engines_tests.rs` | (inline) | 5 |
| lib | `src/lib.rs` + modules | `src/test_logging/builder.rs` | 1 |

**Total integration subtests:** 45 · **lib unit tests:** 1 · **Grand total:** 46

---

## `parse` — `tests/parse.rs`

Run the whole parse target:

```bash
cargo test --test parse
cargo test --test parse -- --nocapture
```

### Parse test log outputs

Several parse tests write JSON artifacts under `target/test-logs/<test_file_name>/` (fixed paths — no env vars). Run the test, then open the matching folder to inspect results.

| Test file | Log directory | Files written |
|-----------|---------------|---------------|
| `parser_boundary_test.rs` | `target/test-logs/parser_boundary_test/` | Per-strategy: `messy_notes_<strategy>_results.json` (`command_seed`, `inline_command`, `heading`, `blank_line`, `indentation`); merged: `messy_notes_strategy_comparison.json` |
| `parser_detection_test.rs` | `target/test-logs/parser_detection_test/` | `detects_at_commands.json`, `detects_chained_commands.json`, `nested_commands_seeds.json` |
| `parser_pipeline_detection_test.rs` | `target/test-logs/parser_pipeline_detection_test/` | `architecture_command_seeds.json`, `architecture_command_blocks.json`, `messy_notes_boundary_candidates.json`, `deep_nested_pipeline_summary.json` |
| `parse_database_test.rs` | `target/test-logs/parse_database_test/` | `stats.json`, `search_parser_hits.json`, `search_task_hits.json`, `table_dumps.json` |
| `real_path_tree_database_test.rs` | `target/test-logs/parser_real_path_test/` | Full `examples/parse_example/` walk → cross-ref JSON |

Shared messy-doc fixture: `tests/fixtures/example_docs/planner/docs/Scratch/messy_notes.txt` — used by boundary, detection, and pipeline marker tests.

---

### `tests/parse/parser_boundary_test.rs`

Boundary **marker** strategies on messy docs (`BoundaryStrategy` + `BoundarySolver`).

| Subtest | What it checks |
|---------|----------------|
| `boundary_strategies_detect_messy_doc_markers` | Each `BoundaryStrategy` run individually; merged `BoundarySolver::collect_boundary_candidates`; writes per-strategy + comparison JSON |

```bash
cargo test --test parse boundary_strategies_detect_messy_doc_markers
cargo test --test parse parser_boundary
```

Fixture: `tests/fixtures/example_docs/planner/docs/Scratch/messy_notes.txt`  
Logs: `target/test-logs/parser_boundary_test/messy_notes_*_results.json`

---

### `tests/parse/parser_detection_test.rs`

Marker seed detectors (`SeedDetectionStrategy` modules + `SeedDetector` registry).

| Subtest | What it checks |
|---------|----------------|
| `at_command::detects_explicit_at_commands_with_payload` | `AtCommandSeedDetector` on messy notes: `@Idea`, `@tutorial`, payload field |
| `chained_at_command::detects_chained_commands_only` | `ChainedAtCommandSeedDetector` on messy notes; writes JSON log (fixture has no `@A @B` chains — inspect output) |
| `inline_status::detects_parenthetical_status_markers` | `(building)`, `(deffered)` |
| `reference::detects_reference_markers` | `@Reference`, `@ref` |
| `current::detects_current_markers_in_prose` | Inline and line-start `@current` |
| `seed_detector_registry_merges_on_nested_fixture` | Full `SeedDetector::with_defaults()` merge |

```bash
cargo test --test parse parser_detection
cargo test --test parse detects_explicit_at_commands_with_payload
cargo test --test parse detects_chained_commands_only
cargo test --test parse detects_parenthetical_status_markers
cargo test --test parse detects_reference_markers
cargo test --test parse detects_current_markers_in_prose
cargo test --test parse seed_detector_registry_merges_on_nested_fixture
```

Fixtures:
- `tests/fixtures/example_docs/planner/docs/Scratch/messy_notes.txt` — at_command, chained_at_command
- Inline strings — inline_status, reference, current
- `tests/fixtures/example_docs/planner/example_docs/nested_commands.md` — registry merge test

Logs: `target/test-logs/parser_detection_test/`

---

### `tests/parse/parser_command_test.rs`

`CommandRegistry` + `CommandSeedDetector` + full `MacroPipeline` on inconsistent layouts.

| Subtest | What it checks |
|---------|----------------|
| `registry_resolves_aliases_and_multi_word_chains` | `CommandRegistry::default()` aliases via `ParseContext` |
| `custom_registry_registers_flexible_command_spec` | Custom `CommandRegistry::new()` + spec registration |
| `macropipeline_parses_inconsistent_command_layouts_without_panic` | End-to-end parse on moneyplan scratch doc |

```bash
cargo test --test parse parser_command
cargo test --test parse registry_resolves_aliases
cargo test --test parse custom_registry_registers
cargo test --test parse macropipeline_parses_inconsistent
```

Fixtures: inline text + `tests/fixtures/example_docs/moneyplan/selected/planning_scratch.txt`

---

### `tests/parse/parser_pipeline_detection_test.rs`

Pipeline-attached registries: `CommandSeedDetector`, `BoundarySolver`, count alignment.

| Subtest | What it checks |
|---------|----------------|
| `pipeline_detects_command_seeds_via_attached_detector` | `pipeline.command_seed_detector().detect(&ctx)` |
| `pipeline_assembles_blocks_via_attached_boundary_solver` | Seeds → blocks via attached solver |
| `pipeline_boundary_solver_collects_marker_candidates` | Marker candidates on messy notes |
| `full_pipeline_parse_matches_detection_and_assembly_counts` | Parse output count == seeds == blocks (inference off) |

```bash
cargo test --test parse parser_pipeline_detection
cargo test --test parse pipeline_detects_command_seeds
cargo test --test parse pipeline_assembles_blocks
cargo test --test parse pipeline_boundary_solver_collects
cargo test --test parse full_pipeline_parse_matches
```

Fixtures: `tests/fixtures/example_docs/planner/docs/ARCHITECTURE.md`, `messy_notes.txt`, `tests/fixtures/deep_nested_macros.md`

---

### `tests/parse/parser_hierarchy_test.rs`

| Subtest | What it checks |
|---------|----------------|
| `parser_hierarchy_assigns_heading_context_to_nested_commands` | `#` headings, numbered `@Task` lines, `heading_context` / `hierarchy_path`, hierarchy nodes for every command, no error diagnostics |

```bash
cargo test --test parse parser_hierarchy_assigns_heading_context_to_nested_commands
cargo test --test parse parser_hierarchy
```

Uses inline markdown fixture (headings + numbered list tasks) via `MacroPipeline::default()`.

---

### `tests/parse/parse_database_test.rs`

| Subtest | What it checks |
|---------|----------------|
| `parse_database_insert_fetch_search_and_dump` | `MacroPipeline` → `ParseCommandStore` insert, FTS search, table dump |

```bash
cargo test --test parse parse_database_insert_fetch_search_and_dump
cargo test --test parse parse_database
```

Fixture: `tests/fixtures/deep_nested_macros.md`  
Log: `target/test-logs/parse_database_test/`

---

### `tests/parse/real_path_tree_database_test.rs`

| Subtest | What it checks |
|---------|----------------|
| `logged_real_path_walk_macropipeline_database_efficacy` | Full `examples/parse_example/` walk → parallel parse → batch DB → cross-ref JSON with `SourceLocation` |

```bash
cargo test --test parse logged_real_path_walk_macropipeline_database_efficacy -- --nocapture
cargo test --test parse real_path
```

Walk root: **`examples/parse_example/`** (full nested tree). See [docs/SOURCE_REFERENCE_FORMAT.md](../docs/SOURCE_REFERENCE_FORMAT.md).

| Variable | Default | Purpose |
|----------|---------|---------|
| `PARSE_TEST_THREADS` | CPU count | Parallel parse worker count |
| `PARSE_TEST_LOG_DIR` | `target/test-logs/parser_real_path_test` | Cross-ref JSON output directory |

---

## `context` — `tests/context.rs`

Run the whole context target:

```bash
cargo test --test context
cargo test --test context -- --nocapture
```

### `tests/context/context_resolution_test.rs`

| Subtest | What it checks |
|---------|----------------|
| `context_resolution_from_fixture_tree` | Layered context index from deep_tree fixture |
| `context_build_config_uses_walk_module` | `ContextBuildConfig` + walk-based build |
| `context_resolver_resolves_file_folder_and_identifier` | `ContextResolver` file/folder/id/nearest-parent |
| `min_files_per_context_folds_small_scratch_folder` | Small-folder folding into parent context |

```bash
cargo test --test context context_resolution_from_fixture_tree
cargo test --test context context_build_config_uses_walk_module
cargo test --test context context_resolver_resolves_file_folder_and_identifier
cargo test --test context min_files_per_context_folds_small_scratch_folder
cargo test --test context context_resolution
```

Fixtures: `tests/fixtures/deep_tree/`, `tests/fixtures/example_docs/planner/`

### `tests/context/parser_navigation_context_test.rs`

| Subtest | What it checks |
|---------|----------------|
| `parser_navigation_context_integration_from_fixture` | Walk → parse → parse DB → context build → context DB |

```bash
cargo test --test context parser_navigation_context_integration_from_fixture
cargo test --test context parser_navigation
```

Fixture: `tests/fixtures/example_docs/planner/`

---

## `database` — `tests/database.rs`

Run the whole database target:

```bash
cargo test --test database
cargo test --test database -- --nocapture
```

### `tests/database/database_online_test.rs`

| Subtest | What it checks |
|---------|----------------|
| `sqlite_database_is_online_in_memory` | In-memory DB health |
| `sqlite_database_is_online_from_file` | File-backed DB health |
| `sqlite_database_insert_get_and_search_round_trip` | Insert, fetch, FTS search round-trip |
| `optional_real_database_path_is_online` | Optional env-path DB (skips if unset) |

```bash
cargo test --test database sqlite_database_is_online_in_memory
cargo test --test database sqlite_database_is_online_from_file
cargo test --test database sqlite_database_insert_get_and_search_round_trip
cargo test --test database optional_real_database_path_is_online
cargo test --test database database_online
```

### `tests/database/database_health_test.rs`

| Subtest | What it checks |
|---------|----------------|
| `parse_command_domain_schema_is_independently_healthy` | Parse SQLite migrations + tables |
| `context_domain_schema_is_independently_healthy` | Context SQLite migrations + tables |
| `history_domain_schema_is_independently_healthy` | History SQLite migrations + tables |
| `parse_migrations_can_drop_and_recreate_schema` | Parse schema drop + recreate |

```bash
cargo test --test database parse_command_domain_schema_is_independently_healthy
cargo test --test database context_domain_schema_is_independently_healthy
cargo test --test database history_domain_schema_is_independently_healthy
cargo test --test database parse_migrations_can_drop_and_recreate_schema
cargo test --test database database_health
```

---

## `history` — `tests/history.rs`

Run the whole history target:

```bash
cargo test --test history
cargo test --test history -- --nocapture
```

### `tests/history/history_database_test.rs`

| Subtest | What it checks |
|---------|----------------|
| `history_database_persists_and_queries_events` | SQLite insert, recent, frequent, by-context queries |

```bash
cargo test --test history history_database_persists_and_queries_events
cargo test --test history history_database
```

### `tests/history/suggestion_engine_test.rs`

| Subtest | What it checks |
|---------|----------------|
| `suggestion_engine_scores_events_by_frequency_and_context` | `suggest_from_events` scoring |
| `suggestion_engine_uses_frequency_index_with_context_weight` | `suggest_from_index` + fixture JSONL |

```bash
cargo test --test history suggestion_engine_scores_events_by_frequency_and_context
cargo test --test history suggestion_engine_uses_frequency_index_with_context_weight
cargo test --test history suggestion_engine
```

Fixture: `tests/fixtures/history_navigation_commands.jsonl`

---

## `walk` — `tests/walk.rs`

### `tests/walk/tree_walker_test.rs`

| Subtest | What it checks |
|---------|----------------|
| `tree_walker_only_collects_files_without_parsing` | Walker returns files only; ignores node_modules/target |

```bash
cargo test --test walk
cargo test --test walk tree_walker_only_collects_files_without_parsing
```

Fixture: `tests/fixtures/deep_tree/`

---

## `engine_fixture_tests` — `tests/engine_fixture_tests.rs`

Cross-engine fixture tests (inline, no subfolder).

| Subtest | What it checks |
|---------|----------------|
| `context_file_tree_fixture_assigns_unique_context_layers_and_indexes_up_down` | Context tree layers, ancestors, descendants |
| `watchdog_filters_file_types_ignores_paths_and_expands_routines` | Watchdog rule matching + routine expansion |
| `watchdog_timer_event_runs_timely_routine_from_fixture` | Timer trigger + routine actions |
| `parser_deeply_nested_commands_are_inserted_into_database_and_searchable` | Parse + in-memory ParseDatabase search |
| `history_log_tracks_file_navigation_explorer_locations_and_console_commands` | JSONL history + frequency index |
| `history_can_append_live_style_events_with_metadata` | Live-style append + metadata round-trip |

```bash
cargo test --test engine_fixture_tests
cargo test --test engine_fixture_tests context_file_tree_fixture
cargo test --test engine_fixture_tests watchdog_filters
cargo test --test engine_fixture_tests watchdog_timer
cargo test --test engine_fixture_tests parser_deeply_nested
cargo test --test engine_fixture_tests history_log_tracks
cargo test --test engine_fixture_tests history_can_append
```

Fixtures: `tests/fixtures/deep_tree/`, `tests/fixtures/deep_nested_macros.md`, `tests/fixtures/watch_spec_file_types_and_timer.json`, `tests/fixtures/file_change_events.jsonl`, `tests/fixtures/history_navigation_commands.jsonl`

---

## `integrated_engines_tests` — `tests/integrated_engines_tests.rs`

Smaller cross-engine smoke tests (inline).

| Subtest | What it checks |
|---------|----------------|
| `context_index_parses_alias_current_and_queue` | Context parser: alias, @current, queue |
| `navigation_local_scope_alias_wins_before_parent` | Navigation alias resolution priority |
| `parse_pipeline_handles_explicit_task_and_status` | Pipeline task + inline status |
| `history_mock_adapter_builds_suggestions` | Mock history adapter suggestions |
| `watchdog_fixture_plans_actions` | Watchdog planner on fixture events |

```bash
cargo test --test integrated_engines_tests
cargo test --test integrated_engines_tests context_index_parses
cargo test --test integrated_engines_tests navigation_local_scope
cargo test --test integrated_engines_tests parse_pipeline_handles
cargo test --test integrated_engines_tests history_mock_adapter
cargo test --test integrated_engines_tests watchdog_fixture_plans
```

---

## Library unit tests — `src/`

| Module | Subtest | What it checks |
|--------|---------|----------------|
| `src/test_logging/builder.rs` | `builder_assigns_cross_references` | TestOutputBuilder refs, indexes, links |

```bash
cargo test --lib
cargo test --lib builder_assigns_cross_references
cargo test --lib test_logging
```

---

## Fixture index

| Path | Used by |
|------|---------|
| `tests/fixtures/deep_tree/` | walk, context, engine_fixture, integrated |
| `tests/fixtures/deep_nested_macros.md` | parse_database, parser_pipeline_detection, engine_fixture |
| `tests/fixtures/example_docs/planner/docs/Scratch/messy_notes.txt` | parser_boundary, parser_detection, parser_pipeline_detection |
| `tests/fixtures/example_docs/planner/docs/ARCHITECTURE.md` | parser_pipeline_detection |
| `tests/fixtures/example_docs/planner/example_docs/nested_commands.md` | parser_detection |
| `tests/fixtures/example_docs/planner/` | context resolution, parser_navigation, planner tree |
| `tests/fixtures/example_docs/moneyplan/` | parser_command |
| `tests/fixtures/watch_spec_file_types_and_timer.json` | engine_fixture watchdog tests |
| `tests/fixtures/file_change_events.jsonl` | engine_fixture watchdog tests |
| `tests/fixtures/history_navigation_commands.jsonl` | history suggestion, engine_fixture |

---

## Adding a new subtest

1. Create `tests/<domain>/<name>_test.rs` or add a `#[test] fn ...` to an existing file.
2. Wire it in the domain entry file, e.g. `tests/parse.rs`:

   ```rust
   #[path = "parse/my_new_test.rs"]
   mod my_new_test;
   ```

3. Run it:

   ```bash
   cargo test --test parse my_new_test
   ```

4. Add a row to this glossary.

See `workflows/add-integration-test.md` and `workflows/pipeline-and-registries.md` for parse pipeline tests.

---

## Quick reference cheat sheet

```bash
# By domain
cargo test --test parse
cargo test --test context
cargo test --test database
cargo test --test history
cargo test --test walk
cargo test --test engine_fixture_tests
cargo test --test integrated_engines_tests
cargo test --lib

# Parse — by test file
cargo test --test parse parser_boundary
cargo test --test parse parser_detection
cargo test --test parse parser_command
cargo test --test parse parser_pipeline_detection
cargo test --test parse parser_hierarchy
cargo test --test parse parse_database
cargo test --test parse real_path
cargo test --test parse logged_real_path

# Context
cargo test --test context context_resolution
cargo test --test context parser_navigation

# Database
cargo test --test database database_online
cargo test --test database database_health

# History
cargo test --test history history_database
cargo test --test history suggestion_engine
```

---

## Workflow docs (extending parse)

| Doc | Topic |
|-----|-------|
| [workflows/pipeline-and-registries.md](../workflows/pipeline-and-registries.md) | `MacroPipeline`, `ParseContext`, attaching registries |
| [workflows/add-registry-command.md](../workflows/add-registry-command.md) | New `@command` in `CommandRegistry` |
| [workflows/add-command-seed-strategy.md](../workflows/add-command-seed-strategy.md) | New `CommandSeedStrategy` |
| [workflows/add-seed-detector.md](../workflows/add-seed-detector.md) | New `SeedDetectionStrategy` |
| [workflows/add-boundary-strategy.md](../workflows/add-boundary-strategy.md) | New `BoundaryStrategy` |
| [workflows/add-block-assembler.md](../workflows/add-block-assembler.md) | New `BlockAssemblyStrategy` |
