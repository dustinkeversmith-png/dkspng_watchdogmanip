use macro_os_engines::walk::{TreeWalker, TreeWalkerConfig};
use std::collections::BTreeSet;

#[test]
fn tree_walker_only_collects_files_without_parsing() {
    let root = "tests/fixtures/deep_tree";
    let walker = TreeWalker::new(
        TreeWalkerConfig::new(root)
            .recursive(true)
            .include_extensions(["md", "rs", "txt"])
            .ignore_dirs(["target", ".git", "node_modules"]),
    );

    let output = walker.walk().expect("walk should succeed");
    assert!(!output.files.is_empty());

    let extensions: BTreeSet<_> = output
        .files
        .iter()
        .filter_map(|file| file.extension.clone())
        .collect();
    assert!(extensions.contains("md") || extensions.contains("rs"));

    assert!(
        output
            .files
            .iter()
            .all(|file| !file.path.to_string_lossy().contains("node_modules")),
        "ignored directories should not appear in walked files"
    );
}
