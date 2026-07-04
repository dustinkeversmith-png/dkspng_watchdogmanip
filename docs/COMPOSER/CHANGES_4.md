1. HierarchyDetectorRegistry → resolve_hierarchy()
src/parse/hierarchy/resolver.rs

resolve_hierarchy(doc, blocks, commands) now runs HierarchyDetectorRegistry::with_defaults().detect_all()
Signals advance heading/list stacks as commands are processed (markdown # headings work even when they are not commands)
Parent links use path prefix + heading context
ParseHierarchyNode includes signal_kinds (e.g. MarkdownHeading, NumberedList)
src/parse/pipeline/mod.rs

Passes &blocks into resolve_hierarchy
Test update: parser_hierarchy_test.rs asserts heading context comes from markdown signals and that hierarchy nodes record signal_kinds.

2. ParameterSpec / MemberSpec in registry
src/parse/registry/mod.rs

CommandSpec now uses:
parameters: Vec<ParameterSpec>
optional_members: Vec<MemberSpec>
required_members: Vec<MemberSpec>
Helpers: parameter(), member(), with auto-tags (title_candidate, navigation_target, etc.)
Backward-compat accessors: optional_parameters(), expected_parameters(), members_with_tag(), etc.
src/parse/passes/extract.rs

extract_block_with_registry() uses registry member tags to resolve title/description (e.g. members tagged title_candidate)
Pipeline passes Some(&command_registry) into extraction on every parse.

Usage
use macro_os_engines::parse::registry::{member, parameter, CommandSpec, ...};
registry.register(CommandSpec {
    parameters: vec![parameter("title", false, &["name"], &["title_candidate"])],
    optional_members: vec![member("Title", false, &["name"], &["title_candidate", "display"])],
    // ...
    ..other fields
});
let spec = registry.lookup_name("task").unwrap();
spec.members_with_tag("title_candidate"); // [MemberSpec { name: "Title", ... }]
