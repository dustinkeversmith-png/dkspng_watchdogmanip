use crate::context::build::config::ContextBuildConfig;
use crate::context::build::folding::{attach_walked_files, fold_small_contexts};
use crate::context::error::Result;
use crate::context::index::ContextIndex;
use crate::context::model::InheritancePolicy;
use crate::context::{build_contexts_from_file_tree, FileTreeContextOptions};
use crate::walk::{TreeWalker, TreeWalkerConfig};
use std::path::Path;

pub fn build_context_index(config: ContextBuildConfig) -> Result<ContextIndex> {
    let walker = TreeWalker::new(
        TreeWalkerConfig::new(&config.root)
            .recursive(true)
            .include_extensions(config.include_extensions.clone())
            .ignore_dirs(config.ignore_dirs.clone())
            .max_depth(config.max_depth.unwrap_or(usize::MAX)),
    );

    let walked = walker
        .walk()
        .map_err(|err| crate::context::error::ContextError::ParseError(err.to_string()))?;

    let root_context_id = slug_context_id(&config.root);
    let root_name = config
        .root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("root")
        .to_string();

    let mut index = build_contexts_from_file_tree(
        &config.root,
        FileTreeContextOptions {
            root_context_id: root_context_id.clone(),
            root_name,
            inheritance: InheritancePolicy::Ancestors,
            include: vec!["**".to_string()],
            exclude: config
                .ignore_dirs
                .iter()
                .map(|dir| format!("{dir}/**"))
                .collect(),
            create_context_for_root: true,
        },
    )?;

    attach_walked_files(&mut index, &walked.files)?;
    fold_small_contexts(&mut index, &walked.files, config.min_files_per_context)?;

    Ok(index)
}

fn slug_context_id(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            name.chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() {
                        c.to_ascii_lowercase()
                    } else {
                        '_'
                    }
                })
                .collect::<String>()
        })
        .filter(|slug| !slug.is_empty())
        .unwrap_or_else(|| "project".to_string())
}
