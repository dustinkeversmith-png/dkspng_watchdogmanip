# Test Fixture Coverage

This update adds deeper integration fixtures and tests for the unified `macro-os` application.

## Context

- Fixture tree: `tests/fixtures/deep_tree/`
- Builds a context layer for each included nested directory.
- Ignores `target/**`, `.git/**`, and `node_modules/**`.
- Tests upward lookup with `ancestor_ids`.
- Tests downward lookup with `descendant_ids` and `direct_child_ids`.
- Marks generated folder contexts with `metadata.local_context = true`.

## Watchdog

- Watch spec: `tests/fixtures/watch_spec_file_types_and_timer.json`
- File events: `tests/fixtures/file_change_events.jsonl`
- Tests include filters for `*.rs` and `*.md`.
- Tests ignore filters for build/vendor/temp paths.
- Tests routine expansion for file-change triggered routines.
- Tests timer-triggered routine execution planning.

## Parser

- Fixture: `tests/fixtures/deep_nested_macros.md`
- Runs the macro parser over a deeply nested mixed Markdown/prose/command document.
- Inserts parse output into `ParseDatabase`.
- Tests search by text and by command kind.

## History

- Fixture: `tests/fixtures/history_navigation_commands.jsonl`
- Captures file navigation, Explorer-style folder locations, console commands, editor files, and focused windows.
- Tests JSONL append/read round trip.
- Tests frequency stats and context-aware suggestions.
