# Architecture

`navigation_engine` is the target resolution side of the system.

## Core modules

- `target.rs` — typed file/folder/symbol/context/workspace/command/search targets.
- `alias.rs` — alias and symbol definitions scoped to a context id.
- `index.rs` — scope graph plus alias/symbol storage.
- `resolver.rs` — deterministic resolution and dry-run planning.
- `mock.rs` — test fixture data.
- `main.rs` — CLI for mock/resolve/plan.

## Resolution order

1. current scope aliases
2. parent scope aliases
3. ancestor aliases
4. global aliases
5. visible symbols
6. fallback search target

## Platform openers

This app intentionally creates dry-run plans only. Later, add a platform adapter trait for VS Code, Explorer, terminal, browser, etc.
