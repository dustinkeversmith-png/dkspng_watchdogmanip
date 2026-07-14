# Inline Definition Commands — Macro Processor Starter Spec

This document combines the uploaded running glossary, behavior notes, structure sets, entity sets, and feature notes into a single starter specification for an inline-definition macro processor.

The goal is not only to preserve the commands, but to normalize them into a workable parsing model that can power:

- inline task and idea capture
- workspace and environment context assignment
- macro and clipboard command expansion
- queueing/backlog behavior
- automatic tagging and categorization
- project/workflow documentation updates
- cross-context analysis and control-panel views

---

## 1. Core Idea

The macro processor should scan plain text, Markdown, project notes, and file-adjacent documents for symbolic inline commands such as `@Task`, `@Idea`, `@in`, `@Alias`, `(done)`, and `@Enqueue`.

It should convert these into structured records that can be indexed, queued, linked, updated, and displayed in a control panel.

A useful first implementation should support three levels of detection:

1. **Explicit macro commands** — user-authored commands such as `@Task BuildParser ...`.
2. **Inline status indicators** — small local markers such as `(done)` and `(deferred)`.
3. **Loose auto-detection** — inferred references such as file paths, action verbs, object names, project names, saved-at text, and task-like statements.

---

## 2. Minimal Syntax Model

### 2.1 Command Forms

```text
@Command
@Command Value
@Command {name} {details}
@Command[key]
@Command @Modifier
@Command
[
  block content
]
@Command
{
  structured content
}
```

### 2.2 Inline Indicator Forms

```text
(done)
(deferred)
(adapting)
(building)
(complete)
(deprecated)
```

### 2.3 Recommended Parser Interpretation

| Form | Meaning | Example |
|---|---|---|
| `@Command` | standalone marker or object type | `@Idea` |
| `@Command Value` | command with short argument | `@Reference ./notes.md` |
| `@Command {a} {b}` | command with explicit slots | `@Task BuildParser {implement tokenizer}` |
| `@Command[key]` | command variant or indexed queue | `@Enqueue[urgent]` |
| `@Command @Modifier` | command plus behavior modifier | `@Macro @Clipboard` |
| `(status)` | local status applied to nearby object | `(done)` |

---

## 3. Canonical Object Model

Every parsed inline definition should normalize into a common record shape.

```json
{
  "id": "generated-stable-id",
  "type": "task | idea | project | workspace | alias | macro | queue_item | reference | status | group | behavior | policy | question_answer | thought | tutorial | prompt",
  "command": "@Task",
  "title": "optional title",
  "description": "optional description",
  "content": "body or nearby text",
  "status": "new | current | adapting | building | complete | deferred | deprecated",
  "context": {
    "environment": null,
    "workspace": null,
    "module": null,
    "file": null,
    "section": null,
    "relative_context": null
  },
  "tags": [],
  "categories": [],
  "aliases": [],
  "references": [],
  "relationships": [],
  "queue": null,
  "source": {
    "file_path": null,
    "line_start": null,
    "line_end": null,
    "detected_at": null
  }
}
```

---

## 4. Command Catalog

### 4.1 Environment and Workspace Commands

#### `@Environments`

Defines a container for root-level context, contextual bundles, and workspaces.

```text
@Environments
{
  @Root
  @Context
  @Workspaces
}
```

Suggested normalized type: `environment_set`.

Fields:

- `root`
- `contexts[]`
- `workspaces[]`

#### `@Root`

Root module, project, entity, or base context.

```text
@Root MainPlanner
```

#### `@Context`

A wrapper around a bundle of objects associated with an environment.

```text
@Context PlanningSystem
[
  Notes, tasks, macros, references
]
```

#### `@Workspaces`

Defines workspace collections.

```text
@Workspaces
{
  @Entity
  @Purpose
  Folders/Files/FileSections active
  @Workflows
}
```

#### `@Entity`

Defines what a workspace is for: task, group, module, project, etc.

```text
@Entity MacroProcessor
```

#### `@Purpose`

Explains why the workspace exists.

```text
@Purpose Build an inline definition parser and command processor.
```

#### `@Assignments`

Associates tasks or workspaces with a context.

```text
@Assignments
[
  Workspace: MacroProcessor
  Task: ProjectDataCollection
]
```

#### `@in`

Associates following content with an environment, workspace, module, folder, or context.

```text
@in
[
  Environments/Workspaces/Modules/etc
]
```

Recommended behavior: push a context frame until the end of the block or section.

---

### 4.2 Objective, Task, and Status Commands

#### `@Task`

Outlines a task.

```text
@Task {task_name} {task_details}
```

Example:

```text
@Task ProjectDataCollection
For each project
  Mark down explicit capabilities
  Mark down explicit workflows
```

Recommended fields:

- `title`
- `description`
- `steps[]`
- `context`
- `status`

#### `@Complete`

Marks a nearby task/object as complete.

```text
@Complete
```

Equivalent inline indicator:

```text
(done)
```

#### `@Adapting`

Marks a task as being adapted from an existing definition or prototype.

```text
@Adapting
```

#### `@Building`

Marks a task or feature as currently being built, usually prototype/alpha stage.

```text
@Building
```

#### `@Deferred`

Objective/entity is deferred.

```text
@Deferred
Title
Description:
Content:
  Examples?
RelativeContext?
```

Equivalent inline indicator:

```text
(deferred)
```

#### `@Progressive`

A deferred-like object that is more likely to become a main feature.

```text
@Progressive
Title
Description
Content
```

#### `@Goals`

Creates a goal, optionally with automatic associated context.

```text
@Goals
[
  Build a macro processor that detects inline commands.
]
```

---

### 4.3 Idea, Thought, Project, and Prompt Commands

#### `@Idea`

Floating objective or idea, possibly without context.

```text
@Idea
Title
Description
Content
RelativeContext?
```

#### `@Thought`

Captures reasoning, loose notes, or design thoughts.

```text
@Thought
Title
Description
Content
RelativeContext?
```

#### `@Project`

Defines a project-level object containing ideas and other tags.

```text
@Project
@Ideas
Title
Description
Content
```

#### `@Tutorial`

Defines a tutorial with shortcuts and commands.

```text
@Tutorial
Title
Description
Content
@Shortcuts
@Commands
```

#### `@Prompt`

Stores a reusable prompt.

```text
@Prompt
Title
Description
Content
```

#### `@Q/A`

Captures questions and answers.

```text
@Q/A
[
- Itemized question
Answer text
]
```

---

### 4.4 Organization and Discovery Commands

#### `@Tags`

Tags files, workspaces, folders, tasks, or objects for search.

```text
@Tags
[
  macro
  parser
  workspace
]
```

#### `@Categories`

Categorizes objects for organization and search.

```text
@Categories
[
  Planning
  MacroProcessor
]
```

#### `@Alias`

Sets up quick aliases or commands for switching/opening documentation or workspaces.

```text
@Alias
Name: macro-start
Target: File/Folder/Path/Environment/Position/etc
```

Rule: workspaces with the same alias can open as a group.

#### `@Reference`

References a filepath or external object.

```text
@Reference ./docs/macro_processor.md
```

#### `@before`

A contextual back-reference to the preceding nearby item, type, or context.

```text
@before
```

Suggested use: attach the next command or note to the previous detected object.

#### `@current`

Represents the current objective of the parent project/document.

```text
@current
[
  objective
]
```

---

### 4.5 Macro, Clipboard, Queue, and Algorithm Commands

#### `@Macro`

Defines a reusable macro.

```text
@Macro
Title
Content
```

#### `@Macro @Clipboard`

Defines a macro meant to run against or from clipboard content.

```text
@Macro @Clipboard
Title: Convert Selection To Task
Input: clipboard text
Output: @Task block
```

Recommended fields:

- `title`
- `input_source`
- `transform`
- `output_format`
- `tags[]`

#### `@Enqueue`

Queues an object quickly.

```text
@Enqueue
```

#### `@Enqueue[x]`

Queues an object into a named or indexed queue.

```text
@Enqueue[urgent]
@Enqueue[research]
@Enqueue[random-objectives]
```

#### `@Algodocutize`

Creates a routine or algorithm from bulk formatted or Markdown text.

```text
@Algodocutize
Input: markdown notes
Output: algorithm + command list + task breakdown
```

---

### 4.6 Deprecation and Lifecycle Commands

#### `@Deprecated`

Marks an object, command, project, file, or workflow as deprecated.

```text
@Deprecated
Reason: superseded by @Macro @Clipboard pipeline
```

Inline equivalent:

```text
(deprecated)
```

---

### 4.7 Group and Structure Commands

#### `@Groups`

Defines collections of environments, workspaces, and modules.

```text
@Groups
{
  Title
  Environments
  Workspaces
  Modules
}
```

#### `# ObjectiveQueue`

A context, group, workflow, or associated queue containing objectives queued for completion.

```text
# ObjectiveQueue
- @Task BuildTokenizer
- @Idea Workspace Control Panel
```

---

## 5. Inline Indicators

Inline indicators are small parenthetical markers that apply to the nearest relevant object.

| Indicator | Canonical status | Meaning |
|---|---|---|
| `(done)` | `complete` | Nearby task/object is complete. |
| `(deffered)` / `(deferred)` | `deferred` | Nearby objective is deferred. |
| `(deprecated)` | `deprecated` | Nearby object should not be used. |
| `(building)` | `building` | Prototype/alpha in progress. |
| `(adapting)` | `adapting` | Being adapted from an existing design. |

Parser rule: tolerate misspellings like `(deffered)` and normalize to `deferred`.

---

## 6. Auto-Detection Features

The macro processor should not rely only on explicit `@` commands. It should also detect likely objects from plain text.

### 6.1 Saveable Side Knowledge

Detect phrases that imply metadata:

- `Saved At`
- file paths
- folder paths
- URLs
- task-like lines
- project names
- workspace names

Example:

```text
Saved At: ./docs/parser-notes.md
```

Becomes:

```json
{
  "type": "reference",
  "command": "auto:file_path",
  "content": "./docs/parser-notes.md"
}
```

### 6.2 Action Words / Verbs

Detect action words such as:

- revisit
- compose
- build
- research
- update
- refactor
- test
- document
- expand

Example:

```text
Revisit the workspace alias behavior.
```

Could become a suggested task:

```text
@Task RevisitWorkspaceAliasBehavior
Revisit the workspace alias behavior.
```

### 6.3 Object References / Names

Detect references to:

- projects
- web articles
- ideas
- folders
- files
- modules
- environments
- workspaces
- tasks

### 6.4 Loose Associative Map

Build adaptive links between detected objects.

Example links:

```text
Workspace -> Project
Task -> File
Idea -> Module
Alias -> WorkspaceGroup
Reference -> DocumentationSection
```

---

## 7. Queueing Behavior

### 7.1 New Objective Queue

When the processor detects a loose idea, task, deferred feature, or brainstorm item without a clear project context, it should place it into a default objective queue.

Suggested queue names:

- `new-objectives`
- `random-objectives`
- `deferred-objectives`
- `research-objectives`
- `implementation-suggestions`

### 7.2 Queueing Policy

When automatically separating or organizing ideas, deferments, and tasks, use behavior config rules to determine where they go.

```json
{
  "queue_policy": {
    "unscoped_idea": "random-objectives",
    "unscoped_task": "new-objectives",
    "deferred_item": "deferred-objectives",
    "research_item": "research-objectives",
    "implementation_suggestion": "implementation-suggestions"
  }
}
```

### 7.3 `@Enqueue` Resolution

| Input | Queue result |
|---|---|
| `@Enqueue` | default queue for object type |
| `@Enqueue[urgent]` | `urgent` queue |
| `@Enqueue[research]` | `research-objectives` |
| `@Deferred` + `@Enqueue` | `deferred-objectives` |

---

## 8. Automatic Tagging and Categorization

The processor should recursively inspect the current context position to infer tags and categories.

Signals:

- current folder path
- file name
- nearest heading
- nearest `@in` context
- explicit `@Tags`
- explicit `@Categories`
- repeated object names
- project/module aliases

Example:

```text
# Macro Processor
@Idea
Add clipboard expansion.
```

Inferred tags:

```json
["macro-processor", "clipboard", "idea"]
```

---

## 9. Documentation Policy

The macro processor should support documentation update rules.

### 9.1 Documentation Update Interval

Policy for how often a watcher should run documentation update rules.

Example:

```json
{
  "documentation_update_interval": "on_save | hourly | daily | manual"
}
```

### 9.2 Documentation Rules

Suggested rules:

- document all workflows
- expand built features into documentation
- keep command catalog current
- update task status summaries
- regenerate project capability lists
- preserve original source references

### 9.3 Hierarchy Format

Documentation should adhere to a stable hierarchy:

```text
Project
  Environment
    Workspace
      Module
        Task
        Idea
        Reference
        Workflow
```

---

## 10. Update Policy and Linked Associations

### 10.1 Update Frequency

Defines how often a watchdog scans for updates and responds.

```json
{
  "watchdog_update_frequency": "manual | on_save | every_15_minutes | hourly | daily"
}
```

### 10.2 Cascading References

When a context value changes, associated items should be checked for updates.

Example:

```text
Workspace renamed -> aliases update -> references update -> control panel refreshes
```

### 10.3 Linked Associations

An edge or association between objects with a relationship type.

```json
{
  "from": "task:BuildParser",
  "to": "file:parser.ts",
  "relationship": "implemented_in",
  "confidence": 0.82
}
```

---

## 11. Control Panel Concept

The processor should support a relative control panel that can open or display symbol items based on the current place in documents, folders, workspaces, or environments.

Views:

- current objective
- nearby tasks
- current workspace
- unresolved ideas
- deferred objectives
- aliases
- references
- linked files
- workflow order
- queue contents

---

## 12. Analysis Features

### 12.1 Cross-Context Knowledge Analysis

Explore how cross-context or project knowledge analysis works and what it can do.

Potential analysis tasks:

- detect redundant projects
- suggest refactors
- find related modules
- identify missing documentation
- detect similar tasks across workspaces
- recommend adapters or input/output mechanisms

### 12.2 Automatic Task / Project / Idea Organization

A smart pipeline for accumulating:

- ideas
- tasks
- project suggestions
- implementation suggestions
- brainstorm fragments
- deferred features
- known project files

Pipeline:

```text
raw notes
  -> detect explicit commands
  -> detect inferred tasks/ideas
  -> assign context
  -> tag/categorize
  -> link references
  -> queue unresolved objectives
  -> produce documentation updates
```

### 12.3 Suggestions and Expansion

The system can generate:

- implementation suggestions
- research tasks
- development tasks
- test setup ideas
- architecture expansion notes
- prototype plans

---

## 13. Recommended First Parser Pass

### Pass 1 — Lexical Scan

Find:

- `@Command`
- `@Command[key]`
- parenthetical statuses
- headings
- fenced blocks
- file paths
- URLs

### Pass 2 — Block Capture

Attach block bodies to commands.

Rules:

- `{ ... }` is a structured object block.
- `[ ... ]` is a list/content block.
- indented lines after a command belong to that command until the next heading/command.
- headings create context frames.

### Pass 3 — Context Resolution

Resolve:

- `@in`
- `@before`
- nearest heading
- nearest project/workspace/module
- file path and section source

### Pass 4 — Object Normalization

Convert all commands into canonical records.

### Pass 5 — Linking and Queueing

Create:

- references
- aliases
- tags
- categories
- queue memberships
- relationship edges

### Pass 6 — Output

Possible outputs:

- `records.json`
- `queues.json`
- `relationships.json`
- `macro_index.md`
- `control_panel.md`

---

## 14. Minimal Grammar Sketch

```ebnf
Document        = { Block } ;
Block           = Heading | CommandBlock | TextBlock | StatusLine ;
Heading         = "#"+ Text Newline ;
CommandBlock    = CommandLine [BodyBlock] ;
CommandLine     = CommandName [Variant] {Argument} Newline ;
CommandName     = "@" Identifier ;
Variant         = "[" Identifier "]" ;
Argument        = BracedArgument | PlainTextArgument ;
BracedArgument  = "{" Text "}" ;
BodyBlock       = CurlyBlock | SquareBlock | IndentedBlock ;
CurlyBlock      = "{" {Block | Text} "}" ;
SquareBlock     = "[" {Block | Text} "]" ;
IndentedBlock   = { IndentedLine } ;
StatusLine      = "(" StatusWord ")" ;
StatusWord      = "done" | "deferred" | "deffered" | "building" | "adapting" | "deprecated" | "complete" ;
```

---

## 15. Starter Command Registry

```json
{
  "commands": {
    "@Environments": { "type": "environment_set", "block": true },
    "@Root": { "type": "environment_root" },
    "@Context": { "type": "context" },
    "@Workspaces": { "type": "workspace_set", "block": true },
    "@Entity": { "type": "entity" },
    "@Purpose": { "type": "purpose" },
    "@Assignments": { "type": "assignment_set", "block": true },
    "@Task": { "type": "task", "status_default": "new" },
    "@Complete": { "type": "status", "status": "complete" },
    "@Adapting": { "type": "status", "status": "adapting" },
    "@Building": { "type": "status", "status": "building" },
    "@Deferred": { "type": "objective", "status": "deferred" },
    "@Progressive": { "type": "objective", "status": "progressive" },
    "@Goals": { "type": "goal", "block": true },
    "@Idea": { "type": "idea" },
    "@Thought": { "type": "thought" },
    "@Project": { "type": "project" },
    "@Tutorial": { "type": "tutorial" },
    "@Prompt": { "type": "prompt" },
    "@Q/A": { "type": "question_answer", "block": true },
    "@Tags": { "type": "tags", "block": true },
    "@Categories": { "type": "categories", "block": true },
    "@Alias": { "type": "alias" },
    "@Reference": { "type": "reference" },
    "@before": { "type": "relative_reference", "direction": "previous" },
    "@current": { "type": "current_objective", "block": true },
    "@Macro": { "type": "macro" },
    "@Clipboard": { "type": "modifier", "modifier": "clipboard" },
    "@Enqueue": { "type": "queue_action" },
    "@Algodocutize": { "type": "algorithmize_document" },
    "@Deprecated": { "type": "status", "status": "deprecated" },
    "@Groups": { "type": "group_set", "block": true }
  },
  "indicators": {
    "done": "complete",
    "complete": "complete",
    "deferred": "deferred",
    "deffered": "deferred",
    "building": "building",
    "adapting": "adapting",
    "deprecated": "deprecated"
  }
}
```

---

## 16. Practical Example

### Input

```text
@in
[
  MacroProcessor/Parser
]

@Task BuildInlineCommandParser
Detect @Task, @Idea, @Alias, @Reference and inline status indicators.

@Tags
[
  macro
  parser
  inline-definition
]

@Enqueue[implementation]

(done)
```

### Normalized Output

```json
{
  "type": "task",
  "command": "@Task",
  "title": "BuildInlineCommandParser",
  "content": "Detect @Task, @Idea, @Alias, @Reference and inline status indicators.",
  "status": "complete",
  "context": {
    "workspace": "MacroProcessor",
    "module": "Parser"
  },
  "tags": ["macro", "parser", "inline-definition"],
  "queue": "implementation"
}
```

---

## 17. Recommended MVP Implementation Scope

For the first macro processor, implement only:

1. Command tokenizer for `@Command`, `@Command[x]`, and `(status)`.
2. Block parser for bracket, brace, and indented bodies.
3. Command registry mapping into canonical record types.
4. Context stack using headings and `@in`.
5. Status attachment to nearest prior object.
6. Queueing using `@Enqueue` and `@Enqueue[x]`.
7. Tag/category attachment to nearest prior object or active context.
8. Reference detection for file paths and URLs.
9. Output to `records.json` and `macro_index.md`.

Avoid for MVP:

- complex AI inference
- cross-project graph analysis
- live filesystem watchers
- automatic refactor suggestions
- control-panel UI

Those should come after the parser produces stable records.

---

## 18. Original Source Notes

The following source files were combined and normalized into this spec.


### RunningGlossary.txt

```text
# Running Glossary | PlanSystemResearch

# Relative Control Panel




# Inline Indicators
(done) - Marks a near by objective/task/module as done.
(deffered) - Marks a near by objective as deferred.

# Inline Commands
@in
Environment names/folder or module locations meant to associate the below subset with this kind of context
[
	Envs/Workspaces/Modules etc
]

@Reference
filepath

@before
contextual back reference for item types or context info based on the surrounding context

@Q/A 
[
- Itemized question
Answers
]

@current
[
	objective
]
Represents a current objective of the parent project/document that I am in
```

### RunningGlossary1.txt

```text
@Environments
{
	@Root :- Possibly a root module/entity or some kind of tied context
	@Context :- A wrapping around a bundle of objects associated with this environment
	@Workspaces
}

@Workspaces
{
	@Entity :- A task, groups, modules etc defiend what the workspace is for
	@Purpose :- Possibly a purpose for the workspace why this workspace what for
	Folders/Files/FileSections active
	@Workflows

}

	@Assignments :-
		Workspaces
		Tasks Etc
		

# Status Commands
@Adapting
Possibly marking a task as currently being adapterd but has some kind of predefinition
@Building
Currently building the prototype or alpha for something thats not done

# Objectives and Status

@Task {task_name} {task_details} - Outlines a task
@Complete :- Like done marks something complete
```

### RunningGlossary2.txt

```text
@Idea
A floating objective or idea possibly without context
  Title
  Description
  Content
  RelativeContext?
  
 @Deferred
 A objective or entity marked as deferred 
	Title
	Description:
	Content:
		Examples?
	RelativeContext?
	
@Progressive
A marker designating the entity as progressive as in like deferred but more likely to be implemented as a main feature
	Title
	Description
	Content:

@Thought
	Title
	Description
	Content:
	RelativeContext?
	
@Project
	@Ideas,... Other tags
	Title
	Description
	Content
	

@Tutorial
	Title
	Description
	Content
	@Shortcuts
	@Commands
	
@Prompt
	Title
	Description
	Content
```

### RunningGlossary3.txt

```text
@Tags
Used to tag files/workspaces folders for searching
	Tags[]

@Alias
Used to set up quick aliases or commands for easily switching opening up documentations
Workspaces with the same alias will open up with the same group
	File/Folder/Path/Environment/Position/Etc

@Categories
Used to set categories for different things useful for searching
	Categories[]
	
@Goals
Used to create some goals possibly with a automatic associated context

@Algodocutize
Used to create a routine or algorithm from like a bulk formatted or markdown text

@Deprecated
Used to denote something as deprecated

@Macro @Clipboard
Used to define a macro possibly with tags like
	Title
	
	
@Enqueue | @Enqueue[x]
Used to enque something quickly
```

### StructureSet.txt

```text
# StructureSet

# ObjectiveQueue
A context, group, workflow, etc associated queue which contains objectives that have been queued up for completion
```

### TaskSet.txt

```text
@Task ProjectDataCollection
For each project
	Mark down explicit capabilities
	Mark down explicit workflows
```

### AnalysisFeatures.txt

```text
# Analysis Features
Explore how cross context or project knowledge analysis is, what it can do.
```

### BehaviorSet.txt

```text
# Behavior | Queing Config for New Objectives
Backlogging or Queing Behavior for New Objectives

# Context

When automatically seperating or orangizing ideas, deferrments, etc describe a config which enforces some type of behavior

# Behavior

How newly created objects are queued or categorized into the New or Random Objective Queue
```

### BehaviorSet1.txt

```text
BehaviorSet1

@ Research automatic capabilites/workflows
# Automatic Tagging/Categorization
Automatic tagging and categorization possibly for each looking distinctinly recursively down from a context position for tagged or categorized things



# Symbolic Objects for Ordering in Workflows
```

### EntitySet.txt

```text
@Groups
{
	Title
	Environments
	Workspaces
	Modules
}
```

### RunningFeatures.txt

```text
# Inline Auto Detection
## Detect keywords into saveable side knowledge estimates
Like
	Saved At
	or Like File Path
	Like A Task
	
## Detect Action Words or Verbs
Revisit
Compose


## Detect Object References/Names

### Loose Associative Map
@possibly adaptive to filling context to auto populate aliases or making associations or links
Links :- @Generics (like workspace)

projects,
web articles
ideas

# Able to open up a control panel view of symbols items or tasks based on like current place in my documents or so/environement or associated environment

## Environments or objects associated with folder namespaces or workspaces

# Update Policy
## Update Frequency
Frequency at which the watchdog tries to look for updates and respond.
## Cascading References
### A tracking topological backtracking association for change of context values, so that updates can cascade

## Linked Associations
### A edge or association between objects with some kind of relationship
For checking if needs update etc
```

### RunningFeatures1.txt

```text
# Automatic Task/Project/Idea Organization
A smart pipeline for accumulating ideas, tasks, project idea suggestions, implementation suggestions based on ideas, or brainstorms, or known project files, deferred features

## Ideas, Brainstorms, Dream Journals

### Analysis
A analysis algorithm for detecting seperated tasks in accumulated ideas/brainstorming/projects/feature descriptions etc a part of context

### Suggestions
A development suggestion or task system for automating building/research or development

### Expansion
expanding projects or architecture, setting up prototypes

It would be cool to get a like suggested implementation or test set up, or
```

### RunningFeatures2.txt

```text
# Global Analysis
Explore analysis of multi project contexts, detecting redundancies, suggesting refactors, developing input output mechanisms or adapters for help

# Documentation Policy

## Documentation Update Interval
Policy for how often the watch dog should run the documentation update rules etc

## Documentation Rules
Associated rules for mainting documentation, such as document all work flows, or expand of built features into some doc with this format

## Hierarchy Format
Some associated format that the documentation must adhere too
```
