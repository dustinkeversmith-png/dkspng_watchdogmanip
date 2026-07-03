Implemented the refactor plan in incremental, compiling steps. All tests pass (cargo test).

Summary of changes
Step 1 — Test cleanup
Fixed watchdog timer matching: Timer/Startup/Manual triggers bypass file-path include filters
Fixed context fs_indexer: skips target, .git, node_modules, etc. by directory name
Real-path test now uses PARSE_TEST_ROOT and PARSE_TEST_LOG_DIR (default: target/test-logs/parse), skips gracefully if root missing
Uses output.diagnostics for error counts; no capability logging
Steps 2–4 — Parse modularization
New modules (legacy passes/ still works; pipeline delegates through them):

src/parse/
  boundary/     model, strategies, solver
  seeds/        model, detector (At/Chained/Status/Reference/Current)
  extractors/   model, extractors (Title, Description, Parameters, etc.)
  hierarchy/    model, resolver
  parser.rs     Parser façade over MacroPipeline
CommandSpec now has optional_parameters, optional_members, required_members, accepted_layouts (backward-compatible expected_* aliases kept)
ParseOutput adds hierarchy: Vec<ParseHierarchyNode>
ParsedCommand adds parent_id, child_ids, hierarchy_path, heading_context, list_context
List-item commands like 1. @Task ... now parse correctly
Step 5 — Parse database
src/parse/database/
  mod.rs, model.rs, sqlite.rs, memory.rs
ParseCommandStore re-exports CommandSqliteDatabase
In-memory ParseDatabase moved to memory.rs
Steps 6–7 — Context
src/context/build/       ContextBuildConfig, build_context_index (uses walk)
src/context/database/    ContextStore + SQLite tables
fs_indexer preserved; build/ wraps it with walk-based discovery
Steps 8–10 — History
src/history/database/    HistoryStore + SQLite tables
src/history/suggestions/ moved from suggest.rs (backward-compatible re-export)
Test matrix added
tests/walk.rs
tests/context.rs
tests/history.rs
tests/parse.rs (expanded)
tests/parse/parser_hierarchy_test.rs
tests/parse/parse_database_test.rs
tests/context/context_resolution_test.rs
tests/context/parser_navigation_context_test.rs
tests/history/history_database_test.rs
tests/fixtures/example_docs/planner/context_commands.md
Test orchestration preserved
Tests still follow: walk → read → parse → insert one-by-one → query/search — no parser in walker, no DB writes in parser.

Deferred (TODOs left in code / not fully built yet)
Non-linear boundary search strategies
Full context path resolver (context/index/resolver.rs)
min_files_per_context folder folding
Larger example_docs archive subset (planner/moneyplan trees)
Full database root split (connection.rs, migrations.rs per domain)
Dedicated suggestion_engine_test.rs and parser_boundary_test.rs
Run targeted tests
cargo test --test parse -- --nocapture
cargo test --test context -- --nocapture
cargo test --test history -- --nocapture
cargo test --test walk -- --nocapture
Real-path log output:

set PARSE_TEST_ROOT=C:\path\to\docs
set PARSE_TEST_LOG_DIR=target\test-logs\parse
cargo test --test parse logged_real_path -- --nocapture