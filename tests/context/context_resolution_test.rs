use macro_os_engines::context::{
    build_context_index, build_contexts_from_file_tree, ContextBuildConfig, ContextResolver,
    FileTreeContextOptions,
};
use std::path::PathBuf;

#[test]
fn context_resolution_from_fixture_tree() {
    let root = std::path::PathBuf::from("tests/fixtures/deep_tree");
    let index = build_contexts_from_file_tree(
        &root,
        FileTreeContextOptions {
            root_context_id: "project".to_string(),
            root_name: "Deep Tree Project".to_string(),
            ..Default::default()
        },
    )
    .expect("context index should build");

    assert!(index.context("project").is_ok());
    assert!(index.context("project_src_parser").is_ok());
    assert!(!index.contexts.contains_key("project_target"));

    let ancestors = index.ancestor_ids("project_src_parser").unwrap();
    assert!(ancestors.contains(&"project".to_string()));

    let descendants = index.descendant_ids("project").unwrap();
    assert!(descendants.contains(&"project_docs_guides".to_string()));
}

#[test]
fn context_build_config_uses_walk_module() {
    let config = ContextBuildConfig {
        root: std::path::PathBuf::from("tests/fixtures/deep_tree"),
        include_extensions: vec!["md".to_string(), "rs".to_string(), "txt".to_string()],
        ignore_dirs: vec![
            "target".to_string(),
            ".git".to_string(),
            "node_modules".to_string(),
        ],
        ..Default::default()
    };

    let index = build_context_index(config).expect("build context");
    assert!(index.context_count_or_local() > 0);
}

#[test]
fn context_resolver_resolves_file_folder_and_identifier() {
    let root = PathBuf::from("tests/fixtures/deep_tree");
    let index =
        build_contexts_from_file_tree(&root, FileTreeContextOptions::default()).expect("index");

    let resolver = ContextResolver::new(&index).with_project_root(&root);

    let file = root.join("src/parser/boundary.rs");
    let file_resolution = resolver
        .resolve_by_file_path(&file)
        .expect("file path should resolve");
    assert!(file_resolution.context_id.contains("parser"));

    let folder_resolution = resolver
        .resolve_by_folder_path(root.join("docs/guides"))
        .expect("folder should resolve");
    assert!(folder_resolution.context_id.contains("docs"));

    let id_resolution = resolver
        .resolve_by_identifier("project")
        .expect("project id");
    assert_eq!(id_resolution.context_id, "project");

    let nearest = resolver
        .resolve_nearest_parent(&file)
        .expect("nearest parent");
    assert!(!nearest.context_id.is_empty());
}

#[test]
fn min_files_per_context_folds_small_scratch_folder() {
    let config = ContextBuildConfig {
        root: PathBuf::from("tests/fixtures/example_docs/planner"),
        include_extensions: vec!["md".to_string(), "txt".to_string()],
        min_files_per_context: 2,
        ..Default::default()
    };

    let index = build_context_index(config).expect("build planner context");

    assert!(
        !index.contexts.keys().any(|id| id.contains("scratch")),
        "single-file Scratch folder should fold into parent"
    );

    let docs_context = index
        .contexts
        .iter()
        .find(|(_, ctx)| {
            ctx.metadata
                .get("relative_path")
                .map(|path| path.contains("docs"))
                .unwrap_or(false)
                && !ctx
                    .metadata
                    .get("relative_path")
                    .unwrap()
                    .contains("Scratch")
        })
        .map(|(id, _)| id.clone());
    assert!(docs_context.is_some());

    let resolver = ContextResolver::new(&index);
    let scratch_file =
        PathBuf::from("tests/fixtures/example_docs/planner/docs/Scratch/messy_notes.txt");
    let resolved = resolver
        .resolve_by_file_path(&scratch_file)
        .expect("folded file should still resolve to parent docs context");
    assert!(!resolved.context_id.contains("scratch"));
}

trait ContextIndexExt {
    fn context_count_or_local(&self) -> usize;
}

impl ContextIndexExt for macro_os_engines::context::ContextIndex {
    fn context_count_or_local(&self) -> usize {
        self.local_context_ids().len()
    }
}
