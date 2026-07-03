use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InheritancePolicy {
    None,
    ParentOnly,
    Ancestors,
    ProjectRoot,
    WorkspaceRoot,
    ExplicitImportsOnly,
}

impl Default for InheritancePolicy {
    fn default() -> Self {
        Self::Ancestors
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextRules {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

impl Default for ContextRules {
    fn default() -> Self {
        Self {
            include: vec!["**/*".to_string()],
            exclude: vec![
                "target/**".to_string(),
                ".git/**".to_string(),
                "node_modules/**".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AliasTargetRef {
    File {
        path: PathBuf,
        line: Option<u32>,
        column: Option<u32>,
        marker: Option<String>,
    },
    Folder {
        path: PathBuf,
    },
    Symbol {
        name: String,
        kind: Option<String>,
    },
    Context {
        context_id: String,
    },
    Workspace {
        workspace_id: String,
    },
    Command {
        command_name: String,
        args: Vec<String>,
    },
    Search {
        query: String,
        scope: Option<String>,
    },
    Multi {
        targets: Vec<AliasTargetRef>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AliasRecord {
    pub name: String,
    pub target: AliasTargetRef,
    pub description: Option<String>,
    pub source_path: Option<PathBuf>,
    pub metadata: BTreeMap<String, String>,
}

impl AliasRecord {
    pub fn new(name: impl Into<String>, target: AliasTargetRef) -> Self {
        Self {
            name: name.into(),
            target,
            description: None,
            source_path: None,
            metadata: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentObjective {
    pub title: String,
    pub details: Option<String>,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueItem {
    pub title: String,
    pub details: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolRecord {
    pub name: String,
    pub kind: String,
    pub source_path: Option<PathBuf>,
    pub line: Option<u32>,
    pub context_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextNode {
    pub id: String,
    pub name: String,
    pub root_path: PathBuf,
    pub parent_id: Option<String>,
    pub child_ids: Vec<String>,
    #[serde(default)]
    pub rules: ContextRules,
    #[serde(default)]
    pub inheritance: InheritancePolicy,
    #[serde(default)]
    pub imports: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<AliasRecord>,
    #[serde(default)]
    pub currents: Vec<CurrentObjective>,
    #[serde(default)]
    pub queues: Vec<QueueItem>,
    #[serde(default)]
    pub symbols: Vec<SymbolRecord>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
    #[serde(default)]
    pub local_files: Vec<PathBuf>,
    #[serde(default)]
    pub local_commands: Vec<String>,
}

impl ContextNode {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        root_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            root_path: root_path.into(),
            parent_id: None,
            child_ids: Vec::new(),
            rules: ContextRules::default(),
            inheritance: InheritancePolicy::default(),
            imports: Vec::new(),
            aliases: Vec::new(),
            currents: Vec::new(),
            queues: Vec::new(),
            symbols: Vec::new(),
            metadata: BTreeMap::new(),
            local_files: Vec::new(),
            local_commands: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    pub fn add_alias(&mut self, alias: AliasRecord) {
        self.aliases.push(alias);
    }

    pub fn add_current(&mut self, title: impl Into<String>) {
        self.currents.push(CurrentObjective {
            title: title.into(),
            details: None,
            source_path: None,
        });
    }

    pub fn add_queue_item(&mut self, title: impl Into<String>, status: Option<String>) {
        self.queues.push(QueueItem {
            title: title.into(),
            details: None,
            status,
            priority: None,
            source_path: None,
        });
    }
}
