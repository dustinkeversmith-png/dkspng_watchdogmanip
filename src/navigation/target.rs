use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenMode {
    Default,
    Editor,
    Explorer,
    Terminal,
    Browser,
    Preview,
}

impl Default for OpenMode {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileTarget {
    pub path: PathBuf,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub marker: Option<String>,
    #[serde(default)]
    pub open_mode: OpenMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderTarget {
    pub path: PathBuf,
    #[serde(default)]
    pub open_mode: OpenMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolTarget {
    pub name: String,
    pub kind: Option<String>,
    pub source_path: Option<PathBuf>,
    pub line: Option<u32>,
    pub scope_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextTarget {
    pub context_id: String,
    pub root_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceTarget {
    pub workspace_id: String,
    pub folders: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandTarget {
    pub command_name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchTarget {
    pub query: String,
    pub scope: Option<String>,
    pub filters: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NavigationTarget {
    File(FileTarget),
    Folder(FolderTarget),
    Symbol(SymbolTarget),
    Context(ContextTarget),
    Workspace(WorkspaceTarget),
    Command(CommandTarget),
    Search(SearchTarget),
    Multi { targets: Vec<NavigationTarget> },
}

impl NavigationTarget {
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::File(FileTarget { path: path.into(), line: None, column: None, marker: None, open_mode: OpenMode::Default })
    }

    pub fn folder(path: impl Into<PathBuf>) -> Self {
        Self::Folder(FolderTarget { path: path.into(), open_mode: OpenMode::Explorer })
    }
}
