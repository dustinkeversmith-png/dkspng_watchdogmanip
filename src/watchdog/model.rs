use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub type WatchId = String;
pub type RuleId = String;
pub type RoutineId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WatchSpec {
    pub id: WatchId,
    pub name: String,
    pub root: PathBuf,
    pub recursive: bool,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default = "default_debounce")]
    pub debounce_ms: u64,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub rules: Vec<WatchRule>,
    #[serde(default)]
    pub routines: Vec<Routine>,
}

fn default_debounce() -> u64 {
    500
}
fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WatchRule {
    pub id: RuleId,
    pub name: String,
    pub trigger: WatchTrigger,
    #[serde(default)]
    pub conditions: Vec<WatchCondition>,
    #[serde(default)]
    pub actions: Vec<WatchAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WatchTrigger {
    FileCreated,
    FileModified,
    FileDeleted,
    FileRenamed,
    DirectoryCreated,
    DirectoryDeleted,
    Timer,
    Startup,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind")]
pub enum WatchCondition {
    PathContains { text: String },
    ExtensionIs { extension: String },
    MetadataEquals { key: String, value: String },
    ContextIs { context_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind")]
pub enum WatchAction {
    EmitHistoryEvent {
        event_type: String,
    },
    ReindexContext {
        context_id: String,
    },
    RefreshAliases {
        context_id: Option<String>,
    },
    RunCommand {
        command: String,
        cwd: Option<PathBuf>,
    },
    RunRoutine {
        routine_id: RoutineId,
    },
    WriteLog {
        message: String,
    },
    Notify {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Routine {
    pub id: RoutineId,
    pub name: String,
    #[serde(default)]
    pub steps: Vec<RoutineStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind")]
pub enum RoutineStep {
    RunCommand {
        command: String,
        cwd: Option<PathBuf>,
    },
    ReindexContext {
        context_id: String,
    },
    ScanFiles {
        root: PathBuf,
    },
    ParseMacros {
        files: Vec<PathBuf>,
    },
    RefreshAliases {
        context_id: Option<String>,
    },
    EmitEvent {
        event_type: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub trigger: WatchTrigger,
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
    pub context_id: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannedAction {
    pub watch_id: WatchId,
    pub rule_id: RuleId,
    pub event_id: String,
    pub action: WatchAction,
    pub reason: Vec<String>,
}
