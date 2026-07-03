All six deferred items are implemented. cargo test passes (37 tests across the suite).

1. Context path resolver — src/context/index/resolver.rs
ContextResolver supports:

resolve_by_file_path / resolve_by_folder_path
resolve_by_identifier (context id or slugged name)
resolve_nearest_parent (walks up the path)
resolve_child_contexts, resolve_current_objectives, resolve_local_files
ContextIndex gained remove_context, attach_local_file, and ContextNode gained local_files / local_commands.

2. min_files_per_context folder folding — src/context/build/folding.rs
After walk + index build:

Files are attached to contexts by relative path
Contexts with fewer than min_files_per_context direct files are folded into their parent (files reparented)
Test: min_files_per_context_folds_small_scratch_folder — single-file docs/Scratch/ folds when min is 2.

3. Example docs fixture tree — tests/fixtures/example_docs/
planner/
  docs/ARCHITECTURE.md, overview.md
  docs/Scratch/messy_notes.txt
  example_docs/nested_commands.md
  context_commands.md (existing)
moneyplan/
  README.md
  budgets/q1_notes.txt
  selected/planning_scratch.txt
Includes @Context, @current, @Alias, @Reference, headings, lists, inline prose, unknown commands, and inconsistent formatting.

4. Database root split
src/database/connection.rs      — DatabaseConnection wrapper
src/database/migrations.rs      — apply_migrations, tables_present, missing_tables
src/parse/database/migrations.rs    — parse command schema + drop
src/context/database/migrations.rs — context schema
src/history/database/migrations.rs — history schema
Domain stores use migrations for create_schema() and expose health_check() + connection().

Test: tests/database/database_health_test.rs — per-domain schema health + drop/recreate.

5. Dedicated tests
File	Coverage
tests/parse/parser_boundary_test.rs
Boundary strategies, solver blocks, seed detection, messy moneyplan parse
tests/history/suggestion_engine_test.rs
suggest_from_events, suggest_from_index with context weighting
6. Extended context tests — tests/context/context_resolution_test.rs
Added resolver tests and folding test alongside existing build/resolution tests.

Run targeted tests:

cargo test --test context
cargo test --test parse parser_boundary
cargo test --test history suggestion_engine
cargo test --test database database_health