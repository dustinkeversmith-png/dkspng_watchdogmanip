use crate::context::error::Result;
use crate::context::index::ContextIndex;
use crate::context::model::{ContextNode, ContextRules, InheritancePolicy};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs;
use std::path::{Path, PathBuf};

/// Options for turning a nested folder tree into a layered context hierarchy.
///
/// Each included directory becomes one ContextNode. The resulting context id is
/// generated from the relative path, for example:
/// - project
/// - project_src
/// - project_src_parser
#[derive(Debug, Clone)]
pub struct FileTreeContextOptions {
    pub root_context_id: String,
    pub root_name: String,
    pub inheritance: InheritancePolicy,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub create_context_for_root: bool,
}

impl Default for FileTreeContextOptions {
    fn default() -> Self {
        Self {
            root_context_id: "project".to_string(),
            root_name: "project".to_string(),
            inheritance: InheritancePolicy::Ancestors,
            include: vec!["**".to_string()],
            exclude: vec!["target/**".to_string(), ".git/**".to_string(), "node_modules/**".to_string()],
            create_context_for_root: true,
        }
    }
}

/// Build a ContextIndex by recursively walking a folder tree.
pub fn build_contexts_from_file_tree(root: impl AsRef<Path>, options: FileTreeContextOptions) -> Result<ContextIndex> {
    let root = root.as_ref().to_path_buf();
    let include = build_globset(&options.include)?;
    let exclude = build_globset(&options.exclude)?;
    let mut index = ContextIndex::new();

    if options.create_context_for_root {
        let mut root_ctx = ContextNode::new(&options.root_context_id, &options.root_name, &root);
        root_ctx.inheritance = options.inheritance.clone();
        root_ctx.rules = ContextRules { include: options.include.clone(), exclude: options.exclude.clone() };
        root_ctx.metadata.insert("local_context".to_string(), "true".to_string());
        root_ctx.metadata.insert("generated_from_file_tree".to_string(), "true".to_string());
        index.upsert_context(root_ctx);
    }

    walk_dir(&root, &root, &options.root_context_id, &options, &include, &exclude, &mut index)?;
    Ok(index)
}

fn walk_dir(
    root: &Path,
    current: &Path,
    parent_context_id: &str,
    options: &FileTreeContextOptions,
    include: &Option<GlobSet>,
    exclude: &Option<GlobSet>,
    index: &mut ContextIndex,
) -> Result<()> {
    let mut entries: Vec<PathBuf> = fs::read_dir(current)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    entries.sort();

    for path in entries {
        if !path.is_dir() {
            continue;
        }
        let rel = path.strip_prefix(root).unwrap_or(&path);
        if !path_allowed(rel, include, exclude) {
            continue;
        }

        let context_id = context_id_for_relative(&options.root_context_id, rel);
        let mut ctx = ContextNode::new(&context_id, rel.file_name().and_then(|s| s.to_str()).unwrap_or(&context_id), &path)
            .with_parent(parent_context_id.to_string());
        ctx.inheritance = options.inheritance.clone();
        ctx.rules = ContextRules { include: options.include.clone(), exclude: options.exclude.clone() };
        ctx.metadata.insert("local_context".to_string(), "true".to_string());
        ctx.metadata.insert("relative_path".to_string(), rel.to_string_lossy().to_string());
        index.upsert_context(ctx);

        walk_dir(root, &path, &context_id, options, include, exclude, index)?;
    }
    Ok(())
}

fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(Some(builder.build()?))
}

fn path_allowed(relative: &Path, include: &Option<GlobSet>, exclude: &Option<GlobSet>) -> bool {
    if let Some(exclude) = exclude {
        if exclude.is_match(relative) {
            return false;
        }
    }
    if let Some(include) = include {
        return include.is_match(relative);
    }
    true
}

fn context_id_for_relative(root_context_id: &str, rel: &Path) -> String {
    let suffix = rel
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .map(slug)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    if suffix.is_empty() {
        root_context_id.to_string()
    } else {
        format!("{}_{}", root_context_id, suffix)
    }
}

fn slug(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}
