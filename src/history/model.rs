use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub type EventId = String;
pub type ContextId = String;
pub type WorkspaceId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventSource {
    Manual,
    MacroConsole,
    ShellHistory,
    NavigationEngine,
    Watchdog,
    EditorAdapter,
    WindowAdapter,
    Mock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Actor {
    User,
    System,
    Adapter(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum HistoryEventType {
    CommandExecuted,
    FileOpened,
    FileModified,
    FolderOpened,
    WindowFocused,
    WorkspaceActivated,
    AliasResolved,
    NavigationOpened,
    SearchPerformed,
    RoutineRan,
    WatchdogTriggered,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind")]
pub enum HistoryTarget {
    File { path: PathBuf, line: Option<u32>, column: Option<u32> },
    Folder { path: PathBuf },
    Command { shell: String, command: String, cwd: Option<PathBuf>, exit_code: Option<i32> },
    Window { title: String, app_name: Option<String>, process_name: Option<String> },
    Workspace { name: String, root_path: Option<PathBuf> },
    Alias { name: String, resolved_to: String },
    Search { query: String, scope: Option<String> },
    Unknown { label: String },
}

impl HistoryTarget {
    pub fn stable_key(&self) -> String {
        match self {
            HistoryTarget::File { path, .. } => format!("file:{}", path.display()),
            HistoryTarget::Folder { path } => format!("folder:{}", path.display()),
            HistoryTarget::Command { shell, command, cwd, .. } => {
                let cwd = cwd.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
                format!("command:{}:{}:{}", shell, cwd, command)
            }
            HistoryTarget::Window { title, app_name, process_name } => {
                format!("window:{}:{}:{}", app_name.clone().unwrap_or_default(), process_name.clone().unwrap_or_default(), title)
            }
            HistoryTarget::Workspace { name, root_path } => {
                format!("workspace:{}:{}", name, root_path.as_ref().map(|p| p.display().to_string()).unwrap_or_default())
            }
            HistoryTarget::Alias { name, resolved_to } => format!("alias:{}->{}", name, resolved_to),
            HistoryTarget::Search { query, scope } => format!("search:{}:{}", scope.clone().unwrap_or_default(), query),
            HistoryTarget::Unknown { label } => format!("unknown:{}", label),
        }
    }

    pub fn label(&self) -> String {
        match self {
            HistoryTarget::File { path, .. } => path.display().to_string(),
            HistoryTarget::Folder { path } => path.display().to_string(),
            HistoryTarget::Command { command, .. } => command.clone(),
            HistoryTarget::Window { title, .. } => title.clone(),
            HistoryTarget::Workspace { name, .. } => name.clone(),
            HistoryTarget::Alias { name, .. } => name.clone(),
            HistoryTarget::Search { query, .. } => query.clone(),
            HistoryTarget::Unknown { label } => label.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEvent {
    pub id: EventId,
    pub timestamp: DateTime<Utc>,
    pub source: EventSource,
    pub actor: Actor,
    pub event_type: HistoryEventType,
    pub target: HistoryTarget,
    pub context_id: Option<ContextId>,
    pub workspace_id: Option<WorkspaceId>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl HistoryEvent {
    pub fn new(id: impl Into<String>, event_type: HistoryEventType, target: HistoryTarget) -> Self {
        Self {
            id: id.into(),
            timestamp: Utc::now(),
            source: EventSource::Manual,
            actor: Actor::User,
            event_type,
            target,
            context_id: None,
            workspace_id: None,
            metadata: BTreeMap::new(),
        }
    }
}
