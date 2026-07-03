use crate::context::error::Result;
use crate::context::index::ContextIndex;
use crate::context::model::{ContextNode, CurrentObjective};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextResolution {
    pub context_id: String,
    pub matched_by: String,
    pub root_path: PathBuf,
}

pub struct ContextResolver<'a> {
    index: &'a ContextIndex,
    project_root: Option<PathBuf>,
}

impl<'a> ContextResolver<'a> {
    pub fn new(index: &'a ContextIndex) -> Self {
        Self {
            index,
            project_root: None,
        }
    }

    pub fn with_project_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.project_root = Some(root.into());
        self
    }

    pub fn resolve_by_file_path(&self, path: impl AsRef<Path>) -> Option<ContextResolution> {
        self.resolve_by_path(path.as_ref(), "file_path")
    }

    pub fn resolve_by_folder_path(&self, path: impl AsRef<Path>) -> Option<ContextResolution> {
        let path = path.as_ref();
        let folder = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };
        self.resolve_by_path(folder, "folder_path")
    }

    pub fn resolve_by_identifier(&self, identifier: &str) -> Option<ContextResolution> {
        let needle = slug(identifier);
        if let Ok(node) = self.index.context(identifier) {
            return Some(ContextResolution {
                context_id: node.id.clone(),
                matched_by: "context_id".to_string(),
                root_path: node.root_path.clone(),
            });
        }

        self.index
            .contexts
            .values()
            .find(|ctx| slug(&ctx.name) == needle || slug(&ctx.id) == needle)
            .map(|ctx| ContextResolution {
                context_id: ctx.id.clone(),
                matched_by: "project_identifier".to_string(),
                root_path: ctx.root_path.clone(),
            })
    }

    pub fn resolve_nearest_parent(&self, path: impl AsRef<Path>) -> Option<ContextResolution> {
        let mut cursor = Some(normalize_path(path.as_ref()));

        while let Some(current) = cursor {
            if let Some(resolution) = self.resolve_by_path(&current, "nearest_parent") {
                return Some(resolution);
            }
            cursor = current.parent().map(normalize_path);
        }

        None
    }

    pub fn resolve_child_contexts(&self, context_id: &str) -> Result<Vec<String>> {
        self.index.direct_child_ids(context_id)
    }

    pub fn resolve_current_objectives(&self, context_id: &str) -> Result<Vec<CurrentObjective>> {
        let mut objectives = Vec::new();
        for id in self.index.context_lookup_order(context_id)? {
            objectives.extend(self.index.context(&id)?.currents.clone());
        }
        Ok(objectives)
    }

    pub fn resolve_local_files(&self, context_id: &str) -> Result<Vec<PathBuf>> {
        Ok(self.index.context(context_id)?.local_files.clone())
    }

    pub fn resolve_context_node(&self, context_id: &str) -> Result<&ContextNode> {
        self.index.context(context_id)
    }

    fn resolve_by_path(&self, path: &Path, matched_by: &str) -> Option<ContextResolution> {
        let normalized = normalize_path(path);
        let mut best: Option<(usize, ContextResolution)> = None;

        for ctx in self.index.contexts.values() {
            let ctx_root = normalize_path(&ctx.root_path);
            if normalized.starts_with(&ctx_root) || path_matches_relative(&normalized, ctx) {
                let score = ctx_root.components().count();
                let candidate = ContextResolution {
                    context_id: ctx.id.clone(),
                    matched_by: matched_by.to_string(),
                    root_path: ctx.root_path.clone(),
                };
                if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                    best = Some((score, candidate));
                }
            }
        }

        best.map(|(_, resolution)| resolution)
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    PathBuf::from(
        path.to_string_lossy()
            .replace('\\', "/")
            .trim_end_matches('/')
            .to_string(),
    )
}

fn path_matches_relative(normalized: &Path, ctx: &ContextNode) -> bool {
    let Some(relative_path) = ctx.metadata.get("relative_path") else {
        return false;
    };
    let relative = relative_path.replace('\\', "/");
    let normalized_str = normalized.to_string_lossy().replace('\\', "/");
    normalized_str.ends_with(&relative) || normalized_str.contains(&format!("/{relative}"))
}

fn slug(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}
