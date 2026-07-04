# Composer Implementation Plan: Context, Parser, Database, History, and Walk Refactor

You are working in the Rust codebase under:

```text
watchdogmanip/
```

The current system has modules for parsing macro-like inline commands, walking file trees, storing parsed commands in SQLite, logging test output, context/navigation indexing, history, and watchdog behavior.

The next refactor goal is to make the system more modular, testable, and tolerant of messy real-world documentation.

Do not implement a giant pipeline-first architecture. Keep each subsystem independently testable.

The intended module boundaries are:

```text
src/walk
    Only walks file trees.
    Does not parse.
    Does not store database records.
    Does not know about contexts unless explicitly used by context/build.

src/parse
    Parses text into parsed command/document structures.
    Does not walk the filesystem.
    Does not own persistent database storage.
    Does not infer context globally.
    Does not perform navigation.

src/context
    Builds and resolves context layers from file paths, folders, project identifiers, and parsed context-specific commands.
    Uses walk utilities where needed.
    Owns context index/resolution logic.

src/navigation
    Resolves aliases/targets against context information.
    Should be usable in tests without opening OS windows.

src/database
    Owns persistent storage.
    Should be split by domain: parse command storage, context storage, history storage, etc.
    SQLite is currently acceptable.

src/history
    Owns event history and suggestions.
    Should move away from JSON-only storage toward database-backed storage.

src/watchdog
    Owns observation/rules/routines, but does not directly mutate every other subsystem.
```

---

# Phase 0: Safety and Ground Rules

Before refactoring:

1. Run existing tests if possible:
    

```bash
cargo test
```

2. Do not delete existing public structs/functions unless replacing them with compatible names or updating all call sites.
    
3. Prefer additive modules first, then migrate tests.
    
4. Avoid hard-coding assumptions from example docs. The parser must support variable, messy, inconsistent documentation.
    
5. Use realistic test docs from the provided example docs archive. The archive contains multiple nested documentation/project structures, including `planner/` and `moneyplan/`, with markdown, scratch notes, project docs, and ignored folders such as `.git`.
    
6. Do not put parser orchestration into the walker. The walker only walks.
    
7. Do not put database writes into the parser. Tests should orchestrate:
    

```text
walk -> read file -> parse/macro pipeline -> insert command/context/history records -> query/search
```

---

# Phase 1: Remove Embedded Parser Module From `parser.rs`

## Goal

Get rid of any embedded sub-parser logic that makes `parser.rs` too large or too coupled.

The parser should become a small façade over modular components:

```text
src/parse/
    mod.rs
    model.rs
    parser.rs
    boundary/
        mod.rs
        model.rs
        solver.rs
        strategies.rs
    seeds/
        mod.rs
        model.rs
        detector.rs
        regex_seed.rs
        command_seed.rs
    extractors/
        mod.rs
        model.rs
        member_extractor.rs
        parameter_extractor.rs
        reference_extractor.rs
        status_extractor.rs
    hierarchy/
        mod.rs
        model.rs
        resolver.rs
    registry.rs
```

## Required Work

1. Move boundary-related logic out of `parser.rs` into:
    

```text
src/parse/boundary/
```

2. Move command seed detection out of hard-coded parser internals into:
    

```text
src/parse/seeds/
```

3. Move member/parameter/tag/reference/status extraction into:
    

```text
src/parse/extractors/
```

4. Keep inference out of command detection for now. Detection should produce candidates. Inference can be added later as a separate pass/module.
    
5. Preserve a simple parser API:
    

```rust
let parser = Parser::default();
let output = parser.parse(source_name, text);
```

6. Preserve `MacroPipeline` if it exists, but make it use the parser modules instead of embedding everything.
    

---

# Phase 2: Expand Boundary Resolution

## Goal

Boundary detection should support wildly formatted docs and command layouts.

Current docs may contain commands like:

```text
@Task Build database search
Some body

@Alias parser-core
target: ./src/parse/parser.rs
```

But future docs may contain inconsistent forms:

```text
@Task:
    Build this thing

some prose @current [ fix this later ]

@Project @Idea maybe this belongs to planner
```

## Required Types

Create:

```rust
pub enum BoundaryKind {
    CommandStart,
    CommandEnd,
    InlineCommand,
    BlockCommand,
    HeadingBoundary,
    IndentationBoundary,
    BlankLineBoundary,
    NextSeedBoundary,
    DelimiterBoundary,
    Unknown,
}
```

Create:

```rust
pub struct BoundaryCandidate {
    pub kind: BoundaryKind,
    pub start_line: usize,
    pub end_line: Option<usize>,
    pub confidence: f32,
    pub reason: String,
}
```

Create:

```rust
pub trait BoundaryStrategy {
    fn name(&self) -> &'static str;
    fn find_boundaries(&self, document: &ParseDocumentInput) -> Vec<BoundaryCandidate>;
}
```

## Required Strategy Modules

Implement starter strategies:

```text
CommandSeedBoundaryStrategy
HeadingBoundaryStrategy
BlankLineBoundaryStrategy
IndentationBoundaryStrategy
InlineCommandBoundaryStrategy
```

## Deferred

Do not implement non-linear boundary collection yet. Add TODO notes for later:

```text
NonLinearBoundarySearch
RelevanceBasedBoundarySearch
BackwardContentAttachment
```

---

# Phase 3: Add Seed Detection System

## Goal

Do not hard-code command detection in one parser function. Add a seed detector system.

## Required Types

```rust
pub enum SeedKind {
    ExplicitCommand,
    ChainedCommand,
    InlineStatus,
    ReferenceMarker,
    CurrentMarker,
    UnknownAtCommand,
}
```

```rust
pub struct CommandSeed {
    pub kind: SeedKind,
    pub raw: String,
    pub normalized_identity: String,
    pub line: usize,
    pub column: usize,
    pub confidence: f32,
}
```

```rust
pub trait SeedDetector {
    fn name(&self) -> &'static str;
    fn detect(&self, input: &ParseDocumentInput) -> Vec<CommandSeed>;
}
```

## Starter Detectors

Implement:

```text
AtCommandSeedDetector
ChainedAtCommandSeedDetector
InlineStatusSeedDetector
ReferenceSeedDetector
CurrentSeedDetector
```

Examples:

```text
@Task
@Project @Idea
(done)
(deffered)
@Reference
@current
```

---

# Phase 4: Make Extraction Modular

## Goal

Extraction should not assume every command expects parameters. Most parameters are optional and should be dynamically interpreted.

## Required Types

```rust
pub struct ExtractionContext {
    pub source_name: String,
    pub seed: CommandSeed,
    pub boundary: BoundaryCandidate,
    pub body: String,
}
```

```rust
pub trait CommandExtractor {
    fn name(&self) -> &'static str;
    fn extract(&self, context: &ExtractionContext) -> ExtractedCommandParts;
}
```

```rust
pub struct ExtractedCommandParts {
    pub title: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<String>,
    pub members: BTreeMap<String, serde_json::Value>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub statuses: Vec<String>,
}
```

## Starter Extractors

Implement:

```text
TitleExtractor
DescriptionExtractor
LooseParameterExtractor
KeyValueMemberExtractor
TagExtractor
ReferenceExtractor
StatusExtractor
```

## Important Registry Change

Update command registration so it no longer aggressively assumes required parameters.

Command specs should look like:

```rust
pub struct CommandSpec {
    pub identity: String,
    pub aliases: Vec<String>,
    pub optional_parameters: Vec<ParameterSpec>,
    pub optional_members: Vec<MemberSpec>,
    pub required_members: Vec<MemberSpec>,
    pub accepted_layouts: Vec<CommandLayoutKind>,
}
```

Most command specs should use optional members/parameters unless there is a strong reason not to.

---

# Phase 5: Add Document Hierarchy/Familial Parse Structure

## Goal

`ParseOutput` should not just be a flat list of commands. It should support hierarchical command relationships inside a document.

Examples:

```text
# Project A

@Project A

## Tasks

1. @Task Build parser
   @Reference ./src/parse/parser.rs

2. @Task Build context database
```

A command may belong to:

```text
document root
heading section
list item
parent command block
previous command context
```

## Required Model Additions

Add to `ParsedCommand`:

```rust
pub parent_id: Option<String>,
pub child_ids: Vec<String>,
pub hierarchy_path: Vec<String>,
pub heading_context: Vec<String>,
pub list_context: Option<String>,
```

Or if modifying `ParsedCommand` is too disruptive, add a separate structure:

```rust
pub struct ParseHierarchyNode {
    pub command_id: String,
    pub parent_id: Option<String>,
    pub child_ids: Vec<String>,
    pub hierarchy_path: Vec<String>,
}
```

And add to `ParseOutput`:

```rust
pub hierarchy: Vec<ParseHierarchyNode>,
```

If `ParseOutput` currently only has:

```rust
source_name
commands
diagnostics
```

preserve those fields and add hierarchy as optional/additive.

## Required Test

Create a parser test with deeply nested markdown, headings, lists, and scattered commands.

Test that:

1. Commands are parsed.
    
2. Commands have source traces.
    
3. Commands under headings inherit heading context.
    
4. Nested/list commands receive parent or hierarchy information where possible.
    
5. Unknown ambiguous cases produce diagnostics, not panics.
    

---

# Phase 6: Context Database and Context Resolution

## Goal

Context should have its own database and tests. It should be possible to resolve context from:

```text
file path
folder path
project identifier
context name
nearest parent context
local context command
```

## Proposed Files

```text
src/context/
    mod.rs
    model.rs
    build/
        mod.rs
        config.rs
        fs_builder.rs
    index/
        mod.rs
        model.rs
        resolver.rs
    database/
        mod.rs
        model.rs
        sqlite.rs
```

## Rename

Current `fs_indexer` should be renamed or wrapped as:

```text
src/context/build/
```

because it is not merely indexing; it is building context layers.

## Build Config

Create:

```rust
pub struct ContextBuildConfig {
    pub root: PathBuf,
    pub include_extensions: Vec<String>,
    pub ignore_dirs: Vec<String>,
    pub min_files_per_context: usize,
    pub max_depth: Option<usize>,
    pub create_context_for_every_folder: bool,
    pub parse_context_commands: bool,
}
```

Rules:

1. Use `src/walk` for file discovery.
    
2. Do not duplicate tree walking logic inside context.
    
3. If `min_files_per_context` is not met, fold that folder into parent context.
    
4. If `parse_context_commands` is true, parse docs looking for context-specific commands.
    

## Context Model

```rust
pub struct ContextNode {
    pub id: String,
    pub name: String,
    pub root_path: PathBuf,
    pub parent_id: Option<String>,
    pub child_ids: Vec<String>,
    pub local_files: Vec<PathBuf>,
    pub local_commands: Vec<String>,
    pub metadata: BTreeMap<String, serde_json::Value>,
}
```

## Context Database Tables

Create context-specific SQLite tables under `src/context/database`:

```text
contexts
context_edges
context_files
context_commands
context_aliases
context_currents
```

Do not put all context tables into the parse command database module.

## Required Context Tests

Create:

```text
tests/context.rs
tests/context/context_resolution_test.rs
```

Test cases:

1. Build context layers from a real or fixture file tree.
    
2. Resolve context by file path.
    
3. Resolve context by folder path.
    
4. Resolve nearest parent context.
    
5. Resolve child contexts.
    
6. Resolve local `@current`.
    
7. Resolve `@prev` or previous context-dependent references if currently supported.
    
8. Ensure ignored folders are skipped.
    
9. Ensure small folders can be folded according to `min_files_per_context`.
    

---

# Phase 7: Parser + Navigation Context Integration Test

## Goal

Create an integration test that uses parser output and navigation/context indexing together.

Test fixture should include context-specific commands such as:

```text
@Context Planner
@current
[
    Fix parser boundary resolution
]

@Alias parser-core
target: ./src/parse/parser.rs

@Reference ./docs/ARCHITECTURE.md
```

## Required Test

Create:

```text
tests/context/parser_navigation_context_test.rs
```

Test flow:

```text
walk fixture docs
parse each file with MacroPipeline or Parser, depending current branch
insert parsed commands into parse database one by one
build context index from walked files
insert context records into context database
resolve context for a specific file path
retrieve local current commands
resolve alias/reference through navigation context
assert expected context details
```

Do not make the walker parse. The test orchestrates the parts.

---

# Phase 8: Database Modularization

## Goal

Split database by domain.

Current `src/database` should be used as shared database utilities or high-level database root, but each domain should own its tables and record model.

Recommended structure:

```text
src/database/
    mod.rs
    health.rs
    connection.rs
    migrations.rs

src/parse/database/
    mod.rs
    model.rs
    sqlite.rs

src/context/database/
    mod.rs
    model.rs
    sqlite.rs

src/history/database/
    mod.rs
    model.rs
    sqlite.rs
```

If this is too much for one pass, start by adding domain-specific modules while preserving current `src/database` exports.

## Required Changes

1. Keep parse command tables separate from context tables.
    
2. Add schema creation methods per domain:
    

```rust
ParseCommandStore::create_schema()
ContextStore::create_schema()
HistoryStore::create_schema()
```

3. Add database health checks that can verify each domain independently.
    
4. Preserve one-by-one insertion:
    

```rust
insert_command(record)
insert_context(record)
insert_history_event(record)
```

Avoid returning to `insert_parse_output`.

---

# Phase 9: History Database Migration

## Goal

Move history from JSON store toward SQLite-backed event storage.

## Proposed Files

```text
src/history/
    mod.rs
    model.rs
    database/
        mod.rs
        model.rs
        sqlite.rs
    suggestions/
        mod.rs
        model.rs
        scorer.rs
```

## History Tables

```text
history_events
history_targets
history_context_links
history_metadata
```

## History Event Model

```rust
pub struct HistoryEventRecord {
    pub id: Option<i64>,
    pub timestamp_unix_ms: i64,
    pub event_type: String,
    pub source: String,
    pub target_kind: String,
    pub target_value: String,
    pub context_id: Option<String>,
    pub workspace_id: Option<String>,
    pub metadata: BTreeMap<String, serde_json::Value>,
}
```

## Required Tests

Create:

```text
tests/history.rs
tests/history/history_database_test.rs
```

Test:

1. Insert file navigation event.
    
2. Insert explorer folder event.
    
3. Insert console command event.
    
4. Insert workspace/context event.
    
5. Query recent events.
    
6. Query frequent targets.
    
7. Query events by context.
    
8. Verify event rows are persisted in SQLite.
    

---

# Phase 10: Move Suggestion Engine Out of History Core

## Goal

Suggestion logic should not live directly in history storage.

Move to:

```text
src/history/suggestions/
```

## Required Types

```rust
pub struct SuggestionRequest {
    pub query: Option<String>,
    pub context_id: Option<String>,
    pub workspace_id: Option<String>,
    pub limit: usize,
}
```

```rust
pub struct SuggestionResult {
    pub target_kind: String,
    pub target_value: String,
    pub score: f64,
    pub reasons: Vec<String>,
}
```

## Scoring

Start simple:

```text
score =
    frequency_weight
  + recency_weight
  + context_match_weight
```

Test with mock events inserted into SQLite.

---

# Phase 11: Parse Temporary Database

## Goal

Parse should have its own database folder for temporary storage and exploration.

Create:

```text
src/parse/database/
    mod.rs
    model.rs
    sqlite.rs
```

Use this for parsed command exploration, search, and later vectorization prep.

Keep current `CommandSqliteDatabase` if already working, but move or re-export it from `parse/database`.

## Required Tests

Create:

```text
tests/parse/parse_database_test.rs
```

Test:

1. Insert parsed command one by one.
    
2. Fetch inserted command.
    
3. Search by query.
    
4. Search by kind.
    
5. Search by tag.
    
6. Search by reference.
    
7. Dump tables to JSON for inspection.
    

---

# Phase 12: Improve Logging for Real-Path Test

## Goal

Keep test logging useful but not annoying.

Do not require `add_section`.

The real-path parse/database test should write a single JSON file with:

```text
walk summary
walk files
parse file summaries
parse command records
database stats
database table dumps
search outputs
```

Do not log capability records right now.

Use direct JSON writing:

```rust
let log = json!({
    "run": {},
    "walk": {},
    "parse": {},
    "database": {},
    "searches": []
});
```

Write it to:

```text
target/test-logs/parse/
```

Controlled by env var:

```text
PARSE_TEST_LOG_DIR
```

Controlled root path:

```text
PARSE_TEST_ROOT
```

---

# Phase 13: Fixture Expansion

## Goal

Add realistic test fixtures from the example docs archive.

Create:

```text
tests/fixtures/example_docs/
```

Place a small curated subset from the uploaded example docs archive, not the entire archive.

Recommended subsets:

```text
planner/docs/
planner/docs/Scratch/
planner/example_docs/
moneyplan/selected docs only
```

Do not include `.git`, `.expo`, `node_modules`, or huge generated folders.

## Fixture Requirements

Add docs with:

1. Context-specific commands.
    
2. `@current`.
    
3. `@prev` or previous/context dependent references if supported.
    
4. `@Alias`.
    
5. `@Reference`.
    
6. Nested headings.
    
7. Numbered lists.
    
8. Commands in messy prose.
    
9. Unknown commands.
    
10. Inconsistent command formatting.
    

---

# Phase 14: Test Matrix

Add or update these tests:

```text
tests/walk.rs
tests/parse.rs
tests/context.rs
tests/database.rs
tests/history.rs
```

Suggested test files:

```text
tests/walk/tree_walker_test.rs
tests/parse/parser_boundary_test.rs
tests/parse/parser_hierarchy_test.rs
tests/parse/parse_database_test.rs
tests/parse/real_path_tree_database_test.rs
tests/context/context_build_test.rs
tests/context/context_resolution_test.rs
tests/context/parser_navigation_context_test.rs
tests/history/history_database_test.rs
tests/history/suggestion_engine_test.rs
tests/database/database_health_test.rs
```

---

# Phase 15: Acceptance Criteria

This refactor is successful when:

1. `src/walk` has no dependency on parser/database/context.
    
2. Parser boundary logic is outside `parser.rs`.
    
3. Seed detection is modular.
    
4. Extraction is modular.
    
5. Registry no longer assumes most parameters are required.
    
6. `ParseOutput` can represent or link to document hierarchy.
    
7. Context builder uses the walk utility.
    
8. Contexts can be resolved by file path/folder path/project identifier.
    
9. Context has its own database tables.
    
10. Parse command storage has its own database module.
    
11. History uses SQLite for tests.
    
12. Suggestion engine is moved out of core history storage.
    
13. Real-path tests write JSON logs with walk/parse/database/search data.
    
14. No capability logging is present in the parse/database test.
    
15. Tests use realistic messy docs.
    

---

# Immediate Implementation Order

Do this in the following order:

## Step 1

Clean up the current broken/warning test state.

- Remove capability logging.
    
- Remove `add_section` references.
    
- Use direct JSON writing.
    
- Use `output.diagnostics` instead of `output.warnings`/`output.errors`.
    

## Step 2

Refactor walk module.

- Ensure `TreeWalker` only returns walked files.
    
- No parse logic in walk.
    

## Step 3

Modularize parse internals.

- Add `boundary`, `seeds`, `extractors`.
    
- Move logic out of `parser.rs`.
    

## Step 4

Add parse hierarchy model.

- Add hierarchy metadata to commands or `ParseOutput`.
    

## Step 5

Move parse database into `src/parse/database`.

- Keep one-by-one `insert_command`.
    

## Step 6

Add context build module.

- Rename/wrap `fs_indexer` as `context/build`.
    
- Use walk utility.
    

## Step 7

Add context database.

- Add context tables and tests.
    

## Step 8

Add parser + context + navigation integration test.

## Step 9

Move history to SQLite.

## Step 10

Move suggestions into `history/suggestions`.

---

# Notes for Composer

Be conservative. Prefer small compiling steps. After each phase, run:

```bash
cargo fmt
cargo test
```

If a full test run is too slow, run targeted tests:

```bash
cargo test --test parse -- --nocapture
cargo test --test context -- --nocapture
cargo test --test database -- --nocapture
cargo test --test history -- --nocapture
```

Do not reintroduce large pipeline coupling. The macro pipeline may exist, but individual parser, walker, database, context, history, and suggestion components must remain independently testable.