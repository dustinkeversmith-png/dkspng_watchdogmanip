use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeWalkerConfig {
    pub root: PathBuf,
    pub recursive: bool,
    pub include_extensions: Vec<String>,
    pub ignore_dirs: Vec<String>,
    pub ignore_file_names: Vec<String>,
    pub max_depth: Option<usize>,
}

impl TreeWalkerConfig {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            recursive: true,
            include_extensions: vec![],
            ignore_dirs: vec![
                ".git".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
            ],
            ignore_file_names: vec![],
            max_depth: None,
        }
    }

    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    pub fn include_extensions(mut self, extensions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.include_extensions = extensions
            .into_iter()
            .map(|ext| ext.into().trim_start_matches('.').to_ascii_lowercase())
            .collect();
        self
    }

    pub fn ignore_dirs(mut self, dirs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.ignore_dirs = dirs.into_iter().map(Into::into).collect();
        self
    }

    pub fn ignore_file_names(mut self, names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.ignore_file_names = names.into_iter().map(Into::into).collect();
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkedFile {
    pub path: PathBuf,
    pub source_name: String,
    pub depth: usize,
    pub extension: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkOutput {
    pub root: PathBuf,
    pub files: Vec<WalkedFile>,
}

#[derive(Debug, Clone)]
pub struct TreeWalker {
    config: TreeWalkerConfig,
}

impl TreeWalker {
    pub fn new(config: TreeWalkerConfig) -> Self {
        Self { config }
    }

    pub fn walk(&self) -> Result<WalkOutput> {
        let root = self.config.root.clone();
        let mut files = Vec::new();

        self.collect_files(&root, 0, &mut files)
            .with_context(|| format!("failed walking tree at {}", root.display()))?;

        files.sort_by(|a, b| a.source_name.cmp(&b.source_name));

        Ok(WalkOutput { root, files })
    }

    fn collect_files(&self, dir: &Path, depth: usize, out: &mut Vec<WalkedFile>) -> Result<()> {
        if let Some(max_depth) = self.config.max_depth {
            if depth > max_depth {
                return Ok(());
            }
        }

        if should_ignore_dir(dir, &self.config.ignore_dirs) {
            return Ok(());
        }

        for entry in fs::read_dir(dir)
            .with_context(|| format!("failed reading directory {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if self.config.recursive {
                    self.collect_files(&path, depth + 1, out)?;
                }
                continue;
            }

            if path.is_file() && self.should_include_file(&path) {
                let metadata = fs::metadata(&path)?;
                out.push(WalkedFile {
                    source_name: source_name_for(&self.config.root, &path),
                    depth: relative_depth(&self.config.root, &path),
                    extension: path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.to_ascii_lowercase()),
                    size_bytes: metadata.len(),
                    path,
                });
            }
        }

        Ok(())
    }

    fn should_include_file(&self, path: &Path) -> bool {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };

        if self
            .config
            .ignore_file_names
            .iter()
            .any(|ignored| ignored == file_name)
        {
            return false;
        }

        if self.config.include_extensions.is_empty() {
            return true;
        }

        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            return false;
        };

        let extension = extension.to_ascii_lowercase();

        self.config
            .include_extensions
            .iter()
            .any(|allowed| allowed == &extension)
    }
}

fn should_ignore_dir(path: &Path, ignore_dirs: &[String]) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    ignore_dirs.iter().any(|ignored| ignored == name)
}

fn source_name_for(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn relative_depth(root: &Path, path: &Path) -> usize {
    path.parent()
        .and_then(|parent| parent.strip_prefix(root).ok())
        .map(|relative| relative.components().count())
        .unwrap_or(0)
}