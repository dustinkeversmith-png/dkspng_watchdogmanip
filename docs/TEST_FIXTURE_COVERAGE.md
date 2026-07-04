# Test Fixture Coverage

Integration fixtures and what they exercise in the unified `macro-os` application.

## Context

- Fixture tree: `tests/fixtures/deep_tree/`
- Builds a context layer for each included nested directory.
- Ignores `target/**`, `.git/**`, and `node_modules/**`.
- Tests upward lookup with `ancestor_ids`.
- Tests downward lookup with `descendant_ids` and `direct_child_ids`.
- Marks generated folder contexts with `metadata.local_context = true`.

**Tests:** `tests/context/context_resolution_test.rs`, `engine_fixture_tests` context test, `integrated_engines_tests`

## Watchdog

- Watch spec: `tests/fixtures/watch_spec_file_types_and_timer.json`
- File events: `tests/fixtures/file_change_events.jsonl`
- Tests include filters for `*.rs` and `*.md`.
- Tests ignore filters for build/vendor/temp paths.
- Tests routine expansion for file-change triggered routines.
- Tests timer-triggered routine execution planning.

**Tests:** `engine_fixture_tests` watchdog tests, `integrated_engines_tests`

## Parser — fixtures

| Fixture | Contents | Used by |
|---------|----------|---------|
| `tests/fixtures/deep_nested_macros.md` | Deeply nested mixed markdown/commands | `parse_database_test`, `parser_pipeline_detection_test`, `engine_fixture_tests` |
| `tests/fixtures/example_docs/planner/docs/Scratch/messy_notes.txt` | Inline `@current`, `@Idea`, `@tutorial`, `@deferred Idea`, `# heading` | `parser_boundary_test`, `parser_detection_test`, `parser_pipeline_detection_test` |
| `tests/fixtures/example_docs/planner/docs/ARCHITECTURE.md` | `@Context`, `@Alias`, `@Task` blocks | `parser_pipeline_detection_test` |
| `tests/fixtures/example_docs/planner/example_docs/nested_commands.md` | `@current`, `@Reference`, nested paths | `parser_detection_test` |
| `tests/fixtures/example_docs/moneyplan/selected/planning_scratch.txt` | Inconsistent command layouts | `parser_command_test` |
| `tests/fixtures/example_docs/planner/` (tree) | Full planner example docs | context, real-path, navigation |

## Parser — test files

| Test file | What it validates |
|-----------|-------------------|
| `parser_boundary_test.rs` | Five `BoundaryStrategy` implementations on messy notes; per-strategy JSON logs |
| `parser_boundary_metadata_test.rs` | `BoundaryMetadataKind`, `BodyDirection`, evidence, confidence in JSON logs |
| `parser_detection_test.rs` | Five `SeedDetectionStrategy` modules + `SeedDetector` registry merge |
| `parser_classifier_seed_test.rs` | Classifier keyword lines (`Task:`, `Project Idea:`, `Reference ./path`) |
| `parser_command_test.rs` | `CommandRegistry::default()`, custom registry, `MacroPipeline` on scratch doc |
| `parser_pipeline_detection_test.rs` | `MacroPipeline` attached `CommandSeedDetector` + `BoundarySolver` |
| `parser_body_parsing_test.rs` | Body shapes: inline, next-line, key-value, bracketed, mixed |
| `parser_hierarchy_test.rs` | Heading + numbered-list `@Task` hierarchy metadata |
| `parser_hierarchy_detector_test.rs` | Attachable hierarchy detectors, numbered list restart groups |
| `parser_location_tracking_test.rs` | `SourceLocation`, file path, DB location columns |
| `parse_database_test.rs` | `MacroPipeline` → `ParseCommandStore` insert, FTS search, table dumps |
| `real_path_tree_database_test.rs` | Real directory walk → parse → DB (optional `PARSE_TEST_ROOT`) |

**Log outputs:** `target/test-logs/<test_file_name>/` — see [tests/GLOSSARY.md](../tests/GLOSSARY.md).

**Architecture docs:** [docs/PARSE_README.md](PARSE_README.md), [workflows/pipeline-and-registries.md](../workflows/pipeline-and-registries.md).

## History

- Fixture: `tests/fixtures/history_navigation_commands.jsonl`
- Captures file navigation, Explorer-style folder locations, console commands, editor files, and focused windows.
- Tests JSONL append/read round trip.
- Tests frequency stats and context-aware suggestions.

**Tests:** `tests/history/suggestion_engine_test.rs`, `engine_fixture_tests`

## Database domains

Independent schema health for parse, context, and history SQLite stores:

**Tests:** `tests/database/database_health_test.rs`, `tests/database/database_online_test.rs`
