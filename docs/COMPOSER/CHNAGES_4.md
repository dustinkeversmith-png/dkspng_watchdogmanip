Test glossary — tests/GLOSSARY.md
Reference for running every test on its own:

8 integration targets (parse, context, database, history, walk, engine_fixture_tests, integrated_engines_tests, lib)
36 subtests with file paths, descriptions, and exact commands
Filter patterns (cargo test --test parse boundary_strategies_detect_messy_doc_markers)
Env vars for the real-path test (PARSE_TEST_ROOT, PARSE_TEST_LOG_DIR)
Fixture index mapping paths → tests
Steps for adding new subtests
Quick example:

cargo test --test parse parser_boundary
cargo test --test context min_files_per_context_folds_small_scratch_folder
cargo test --test history suggestion_engine_scores_events_by_frequency_and_context
Workflows folder — workflows/
File	Purpose
README.md
Index + module map + orchestration rules
add-registry-command.md
New @command in registry
add-seed-detector.md
New seed detector
add-extractor.md
New body field extractor
add-boundary-strategy.md
New boundary strategy
add-hierarchy-rule.md
Heading/list/parent hierarchy
add-integration-test.md
New test under tests/
add-context-module.md
Context build/resolver/DB
add-database-domain.md
SQLite migrations per domain
Each workflow lists files to touch, code snippets, test commands, and a checklist.

The root README.md now links to both. When you add a test or module, update tests/GLOSSARY.md using the checklist in workflows/add-integration-test.md.