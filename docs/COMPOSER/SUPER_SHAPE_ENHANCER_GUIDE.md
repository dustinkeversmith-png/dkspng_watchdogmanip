# Composer Implementation Plan: Boundary Metadata, Loose Shape Parsing, Seed Classification, Hierarchy Detection, and Body Parsing Tests

You are working in the Rust repo:

```text
C:/Users/Cutie Magic 500/watchdogmanip
```

The parser is already organized around `MacroPipeline`, `ParseContext`, command registries, seed detectors, boundary solvers, extractors, hierarchy logic, and parse database support.

The next goal is to make the parser more robust for wild, inconsistent documentation by improving:

1. Boundary metadata collection.
2. Command/body/title/parameter shape identification.
3. Classifier-keyword seed detection.
4. Attachable hierarchy detectors.
5. Body parsing tests.
6. File path and precise file-location tracking on every parsed command/block.

Do not make the parser strict. Everything should remain loose. Remove or avoid concepts like `allow_loose_body`. Loose parsing should be the default. Instead, model uncertainty using optional fields, shape metadata, diagnostics, confidence, and candidate records.

---

# 0. Core Principle

The parser should not try to perfectly understand a document in one pass.

It should collect candidates:

```text
seeds
boundaries
blocks
body shapes
parameter shapes
title candidates
hierarchy candidates
extraction candidates
diagnostics
```

Then the parser can choose the best parse while still logging why it chose it.

This gives the system enough metadata to improve later without losing weird cases.

---

# 1. Add Rich Boundary Metadata

## Problem

Current boundary detection likely identifies command block starts/ends but does not collect enough information about the boundary shape. That metadata would help determine how to extract command bodies, titles, parameters, nested blocks, and tags.

## Goal

Every boundary candidate should describe:

```text
what kind of boundary it is
where it starts/ends
what evidence created it
how confident it is
what extraction style it implies
whether the body is above, below, inline, nested, list-bound, heading-bound, etc.
```

## Add or expand:

```text
src/parse/boundary/model.rs
```

Implement:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryKind {
    CommandSeedLine,
    InlineCommand,
    SameLinePayload,
    NextLineBody,
    IndentedBody,
    BracketedBody,
    FencedBody,
    HeadingSection,
    ListItem,
    NumberedListItem,
    BulletListItem,
    BlankLineTerminated,
    NextSeedTerminated,
    OutdentTerminated,
    HeadingTerminated,
    EndOfFileTerminated,
    Ambiguous,
    Unknown,
}
```

Add:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BodyDirection {
    InlineRight,
    Below,
    Above,
    Around,
    None,
    Unknown,
}
```

Add:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BodyShapeHint {
    SingleLine,
    MultiLineBlock,
    IndentedBlock,
    BracketedBlock,
    MarkdownSection,
    ListItemBody,
    KeyValueBlock,
    FreeformProse,
    Empty,
    Unknown,
}
```

Add:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryEvidence {
    pub kind: String,
    pub line: usize,
    pub column: usize,
    pub text_preview: String,
    pub reason: String,
}
```

Expand `BoundaryCandidate`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryCandidate {
    pub id: String,
    pub kind: BoundaryKind,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: Option<usize>,
    pub end_column: Option<usize>,
    pub body_direction: BodyDirection,
    pub body_shape_hint: BodyShapeHint,
    pub confidence: f32,
    pub evidence: Vec<BoundaryEvidence>,
    pub diagnostics: Vec<String>,
}
```

## Acceptance Criteria

* Boundary candidates are serializable to JSON.
* Existing boundary tests continue passing.
* Boundary logs include kind, body direction, body shape hint, confidence, and evidence.
* No command extraction should depend on a single hard-coded boundary interpretation.

---

# 2. Add General Shape Identifier for Command Bodies, Titles, Parameters, and Members

## Problem

Commands can appear in many forms:

```text
@Task Build parser
@Task:
    Build parser

@Task
Title: Build parser
Description: Fix boundary solver

@Project @Idea maybe use context DB

@current
[
    Finish parser body tests
]
```

The parser needs a general way to describe the “shape” of a command block before extracting exact fields.

## Add module:

```text
src/parse/shape/
    mod.rs
    model.rs
    detector.rs
    strategies.rs
```

## Add types:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandShapeKind {
    InlineTitle,
    InlineParameters,
    KeyValueMembers,
    BracketedBody,
    IndentedBody,
    HeadingAttached,
    ListAttached,
    ProseOnly,
    EmptyBody,
    Mixed,
    Unknown,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TitleCandidateKind {
    InlineAfterCommand,
    TitleMember,
    FirstNonEmptyBodyLine,
    HeadingContext,
    ListItemText,
    Unknown,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterShapeKind {
    None,
    SingleLooseParameter,
    MultipleLooseParameters,
    KeyValueParameter,
    BracketedArray,
    InlineQuotedString,
    Unknown,
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleCandidate {
    pub kind: TitleCandidateKind,
    pub text: String,
    pub line: usize,
    pub confidence: f32,
    pub reason: String,
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandShapeAnalysis {
    pub command_id: Option<String>,
    pub shape_kinds: Vec<CommandShapeKind>,
    pub parameter_shape: ParameterShapeKind,
    pub body_shape: BodyShapeHint,
    pub title_candidates: Vec<TitleCandidate>,
    pub confidence: f32,
    pub diagnostics: Vec<String>,
}
```

## Implementation Notes

Create a `CommandShapeDetector` that runs after block assembly but before final extraction.

It should inspect:

```text
raw seed line
inline text after command identity
body lines
key-value lines
bracket markers
indentation
heading context
list context
boundary metadata
```

## Important Rule

Do not make shape detection strict.

If multiple shapes seem possible, keep multiple `shape_kinds`.

Example:

```text
@Task Build parser
Title: Parser Rewrite
```

This can have:

```text
InlineTitle
KeyValueMembers
Mixed
```

The extractor can later prefer the `Title:` member over inline title, or log ambiguity.

---

# 3. Remove `allow_loose_body` Concept

## Problem

The parser should not need `allow_loose_body`. Loose should be the default behavior.

## Required Refactor

Search for:

```text
allow_loose_body
allow_loose
strict_body
```

Remove or deprecate `allow_loose_body`.

Replace with:

```rust
pub struct CommandSpec {
    pub identity: String,
    pub aliases: Vec<String>,
    pub optional_parameters: Vec<ParameterSpec>,
    pub optional_members: Vec<MemberSpec>,
    pub required_members: Vec<MemberSpec>,
    pub tags: Vec<String>,
    pub accepted_shapes: Vec<CommandShapeKind>,
}
```

Everything should allow body capture unless a command explicitly says it is marker-only.

Add:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandBodyPolicy {
    CaptureIfPresent,
    MarkerOnly,
    PreferInline,
    PreferBlock,
}
```

Default:

```rust
CommandBodyPolicy::CaptureIfPresent
```

## Acceptance Criteria

* Most commands accept optional parameters/members.
* Parser does not reject commands just because they do not match an expected strict body layout.
* Unknown or weird body layouts create diagnostics, not failures.

---

# 4. Add Optional Tags to Members and Parameters

## Problem

You want optional “tags” or metadata on members/parameters rather than hard-coded expected layouts.

## Add to registry model:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub name: String,
    pub required: bool,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
    pub shape_hints: Vec<ParameterShapeKind>,
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberSpec {
    pub name: String,
    pub required: bool,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
    pub value_shape_hints: Vec<String>,
}
```

Examples:

```rust
ParameterSpec {
    name: "title".to_string(),
    required: false,
    aliases: vec!["name".to_string(), "label".to_string()],
    tags: vec!["title_candidate".to_string(), "display".to_string()],
    shape_hints: vec![ParameterShapeKind::SingleLooseParameter],
}
```

```rust
MemberSpec {
    name: "target".to_string(),
    required: false,
    aliases: vec!["path".to_string(), "file".to_string()],
    tags: vec!["navigation_target".to_string(), "reference".to_string()],
    value_shape_hints: vec!["path".to_string()],
}
```

---

# 5. Expand Parser Seed Command Detection With Classifier Keywords

## Problem

Not all command-like things will start with explicit `@`.

You want to detect chained commands and command types based on glossary/classifier keywords.

Examples:

```text
Project Idea: build a context database
Deferred Idea maybe move this later
Task - fix boundary body extraction
current objective: repair parser logs
Reference ./src/parse/parser.rs
```

## Add seed classifier model:

```text
src/parse/seeds/classifier.rs
```

Types:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedClassifierKind {
    ExplicitAtCommand,
    ChainedAtCommand,
    ClassifierKeyword,
    StatusKeyword,
    ParameterKeyword,
    GlossaryIdentifier,
    ReferenceLike,
    Unknown,
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedClassifierMatch {
    pub kind: SeedClassifierKind,
    pub keyword: String,
    pub normalized_identity: Option<String>,
    pub line: usize,
    pub column: usize,
    pub confidence: f32,
    pub reason: String,
}
```

## Add classifier registry:

```rust
pub struct SeedClassifierRegistry {
    pub command_keywords: Vec<ClassifierKeywordSpec>,
    pub status_keywords: Vec<ClassifierKeywordSpec>,
    pub parameter_keywords: Vec<ClassifierKeywordSpec>,
    pub glossary_keywords: Vec<ClassifierKeywordSpec>,
}
```

```rust
pub struct ClassifierKeywordSpec {
    pub keyword: String,
    pub maps_to_identity: Option<String>,
    pub kind: SeedClassifierKind,
    pub aliases: Vec<String>,
    pub case_sensitive: bool,
}
```

## Built-In Classifier Keywords

Start with:

```text
task -> @Task
todo -> @Task
idea -> @Idea
project -> @Project
prompt -> @Prompt
tutorial -> @Tutorial
deferred -> @Deferred
current -> @current
reference -> @Reference
alias -> @Alias
goal -> @Goals
done -> status
complete -> status
building -> status
adapting -> status
```

## Chained Command Detection Without Explicit `@`

Detect:

```text
Project Idea
Deferred Idea
Current Task
Parser Task
```

But do not over-convert everything.

Only produce a candidate if:

```text
keyword appears near line start
keyword matches registry/glossary
line has command-like separator or shape
confidence is above threshold
```

Examples of command-like separators:

```text
:
-
—
[
{
```

## Acceptance Criteria

* Explicit `@` detection still works.
* Chained `@Project @Idea` still works.
* Non-`@` classifier keyword detection creates seeds with lower confidence.
* Classifier keyword seeds include reason and confidence.
* Tests can compare explicit vs classifier-based detection.

---

# 6. Add Attachable Hierarchy Detectors

## Problem

Hierarchy should be modular. Different docs use headings, numbered lists, bullet points, indentation, or mixed structures. The hierarchy detector should assign parent IDs/topology where possible.

## Add module:

```text
src/parse/hierarchy/
    mod.rs
    model.rs
    detector.rs
    strategies/
        headings.rs
        numbered_lists.rs
        bullet_lists.rs
        indentation.rs
```

## Add types:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HierarchySignalKind {
    MarkdownHeading,
    NumberedList,
    BulletList,
    Indentation,
    CommandNesting,
    BoundaryContainment,
    Unknown,
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchySignal {
    pub kind: HierarchySignalKind,
    pub line: usize,
    pub level: usize,
    pub label: Option<String>,
    pub raw: String,
    pub confidence: f32,
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHierarchyLink {
    pub command_id: String,
    pub parent_id: Option<String>,
    pub child_ids: Vec<String>,
    pub hierarchy_path: Vec<String>,
    pub signal_kinds: Vec<HierarchySignalKind>,
}
```

## Add trait:

```rust
pub trait HierarchyDetector {
    fn name(&self) -> &'static str;
    fn detect(&self, document: &SourceDocument, blocks: &[CommandBlock]) -> Vec<HierarchySignal>;
}
```

## Add resolver:

```rust
pub struct HierarchyResolver {
    detectors: Vec<Box<dyn HierarchyDetector>>,
}
```

The resolver should:

1. Collect signals from all detectors.
2. Sort commands by source location.
3. Assign parent links based on nearest compatible hierarchy signal.
4. Assign hierarchy paths.
5. Preserve ambiguity in diagnostics if multiple parents are plausible.

## Required Detectors

Implement starter detectors:

```text
MarkdownHeadingHierarchyDetector
NumberedListHierarchyDetector
BulletListHierarchyDetector
IndentationHierarchyDetector
```

## Numbered List Details

Support lists that restart:

```text
1. First
2. Second

1. New group
2. Another item
```

The detector should not assume every `1.` belongs to the same parent. Use heading/blank-line proximity when possible.

---

# 7. Add Body Parsing Tests

## Goal

Add a test specifically for body parsing behavior.

Create:

```text
tests/parse/parser_body_parsing_test.rs
```

Register it in:

```text
tests/parse.rs
```

## Test Fixtures

Use inline strings and existing messy docs.

Include these cases:

### Inline body

```text
@Task Build parser body tests
```

Expected:

```text
title candidate = Build parser body tests
body shape = SingleLine or InlineParameters
```

### Next-line body

```text
@Task
Build parser body tests
```

Expected:

```text
body captured below command
title candidate may be first body line
```

### Key-value body

```text
@Task
Title: Build parser body tests
Description: Validate extraction shapes
Tags: parser body
```

Expected:

```text
members include Title, Description, Tags
title candidate from Title member
```

### Bracketed body

```text
@current
[
    Fix body parser
    Add hierarchy detector
]
```

Expected:

```text
body shape = BracketedBlock
content includes both lines
```

### Nested/list body

```text
1. @Task Build numbered item parser
   Details:
      This is inside the numbered list.
```

Expected:

```text
hierarchy/list context exists
body includes indented details
```

### Weird mixed body

```text
@Project @Idea maybe parser should try shapes
Title: Better body parser
- random note
(done)
```

Expected:

```text
multiple shape kinds
status detected
title candidate selected
diagnostics allowed
```

## Assertions

Assert:

```text
commands parsed
content captured
title candidates available
shape analysis exists
source trace exists
file path/source location exists
diagnostics do not panic
```

If shape analysis is not yet on `ParsedCommand`, add it.

---

# 8. Add File Location Tracking to Every Command/Block

## Problem

Commands need file path/location in addition to current source trace.

Current `source_trace` may be something like:

```text
docs/parser.md:4
```

That is useful, but add structured fields.

## Add to models:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub source_name: String,
    pub file_path: Option<PathBuf>,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: Option<usize>,
    pub end_column: Option<usize>,
}
```

Add to `ParsedCommand` or parallel metadata:

```rust
pub location: SourceLocation,
```

Add to `CommandBlock`:

```rust
pub location: SourceLocation,
```

## API Change

`MacroPipeline::parse(source_name, text)` does not know full file path. Keep it working.

Add optional richer API:

```rust
pub fn parse_with_path(
    &self,
    source_name: impl Into<String>,
    file_path: impl Into<PathBuf>,
    input: impl AsRef<str>,
) -> ParseOutput
```

Or:

```rust
pub struct SourceDocument {
    pub source_name: String,
    pub file_path: Option<PathBuf>,
    pub text: String,
}
```

Preferred approach:

```rust
let doc = SourceDocument::with_path(source_name, file.path.clone(), text);
let output = pipeline.parse_document(doc);
```

Keep the existing parse call as a convenience wrapper.

## Update Tests

Update real-path parse/database test to call the richer parse API where possible.

Then database records should include:

```text
source_name
file_path
start_line
start_column
end_line
end_column
source_trace
```

If database schema does not yet support this, add columns:

```sql
file_path TEXT
start_line INTEGER
start_column INTEGER
end_line INTEGER
end_column INTEGER
```

---

# 9. Add Boundary Metadata and Shape Logs

## Goal

Existing JSON test logs should include boundary and shape diagnostics.

Update boundary/body tests to write JSON logs to:

```text
target/test-logs/parser_body_parsing_test/
target/test-logs/parser_boundary_test/
```

Log:

```text
document source
seeds
boundary candidates
assembled blocks
shape analyses
parsed commands
hierarchy links
diagnostics
```

Do not use the old annoying `add_section` writer if it has been removed. Write one direct JSON object.

Example:

```rust
let log = json!({
    "source": source_name,
    "seeds": seeds,
    "boundaries": boundaries,
    "blocks": blocks,
    "shapes": shapes,
    "commands": output.commands,
    "diagnostics": output.diagnostics,
});
```

---

# 10. Update Parse README and Fixture Coverage

Update docs:

```text
docs/PARSE_README.md
docs/TEST_FIXTURE_COVERAGE.md
```

Add sections for:

```text
Boundary metadata
Shape detection
Classifier keyword detection
Hierarchy detectors
Body parsing tests
File location tracking
```

Keep docs accurate with actual module names.

---

# 11. Test Matrix to Add or Update

Create/update:

```text
tests/parse/parser_body_parsing_test.rs
tests/parse/parser_boundary_metadata_test.rs
tests/parse/parser_classifier_seed_test.rs
tests/parse/parser_hierarchy_detector_test.rs
tests/parse/parser_location_tracking_test.rs
```

## Tests

### `parser_boundary_metadata_test.rs`

Validates:

```text
BoundaryKind
BodyDirection
BodyShapeHint
BoundaryEvidence
confidence
candidate JSON log
```

### `parser_classifier_seed_test.rs`

Validates:

```text
Task: do thing
Project Idea: do thing
Deferred Idea maybe later
current objective: do thing
Reference ./src/file.rs
```

### `parser_hierarchy_detector_test.rs`

Validates:

```text
heading hierarchy
numbered list hierarchy
numbered list restart
bullet list hierarchy
indentation hierarchy
parent/child links
```

### `parser_body_parsing_test.rs`

Validates:

```text
inline body
next-line body
key-value body
bracketed body
list body
mixed body
title candidates
parameter shape
body shape
```

### `parser_location_tracking_test.rs`

Validates:

```text
source_name
file_path
start_line
start_column
end_line
end_column
source_trace
```

---

# 12. Implementation Order

Do this in small compiling passes.

## Pass 1: Models Only

Add models:

```text
BoundaryKind
BodyDirection
BodyShapeHint
BoundaryEvidence
CommandShapeKind
TitleCandidate
ParameterShapeKind
CommandShapeAnalysis
SeedClassifierKind
SeedClassifierMatch
HierarchySignal
CommandHierarchyLink
SourceLocation
```

Run:

```bash
cargo test
```

## Pass 2: Boundary Metadata

Expand boundary candidates and strategies.

Update existing boundary tests to account for the new fields.

Run:

```bash
cargo test --test parse parser_boundary
```

## Pass 3: Shape Detection

Add `src/parse/shape`.

Integrate shape detector after block assembly and before extraction.

Run:

```bash
cargo test --test parse parser_body_parsing
```

## Pass 4: Classifier Seed Detection

Add classifier registry and classifier strategy.

Keep confidence lower than explicit `@` commands.

Run:

```bash
cargo test --test parse parser_classifier_seed
```

## Pass 5: Hierarchy Detectors

Add attachable hierarchy detector registry.

Implement heading/list/indentation detectors.

Run:

```bash
cargo test --test parse parser_hierarchy
```

## Pass 6: Source Location Tracking

Add `SourceLocation`.

Add `SourceDocument::with_path`.

Add `MacroPipeline::parse_document`.

Update real path tests.

Run:

```bash
cargo test --test parse parser_location_tracking
cargo test --test parse real_path_tree_database
```

## Pass 7: Database Location Columns

Add location fields to parse command database.

Update insert/get/search/dump tests.

Run:

```bash
cargo test --test database
cargo test --test parse parse_database
```

## Pass 8: Docs and Fixture Coverage

Update docs and fixture coverage.

Run full suite:

```bash
cargo fmt
cargo test
```

---

# 13. Acceptance Criteria

The work is complete when:

1. Boundary candidates include kind, body direction, body shape hint, evidence, confidence, and diagnostics.
2. Command blocks have shape analysis.
3. Title candidates are collected instead of guessed from only one rule.
4. Parameters/members are optional by default.
5. `allow_loose_body` is gone or unused.
6. Seed detection can detect classifier keywords without `@`.
7. Chained command detection works with and without `@` where confidence allows.
8. Hierarchy detectors are attachable.
9. Numbered list restarts do not break hierarchy.
10. Body parsing tests cover inline, next-line, key-value, bracketed, list, and mixed bodies.
11. Every command/block has structured source location.
12. Real-path logs include file path/location data.
13. Parser tests still pass on messy example docs.
14. No capability logging is added to parse/database tests.
15. Parser remains loose and diagnostic-driven, not strict/failing.

---

# 14. Important Non-Goals

Do not implement non-linear semantic body collection yet.

Do not implement vector search yet.

Do not make the walker parse.

Do not make the database parse.

Do not make classifier detection too aggressive.

Do not remove `MacroPipeline`; improve the internals and keep direct modules testable.

Do not add OS navigation/capability probing into parser tests.
