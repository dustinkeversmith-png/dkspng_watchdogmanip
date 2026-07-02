use std::collections::{BTreeMap, BTreeSet};

use crate::parse::model::{BoundaryKind, CommandKind};

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub kind: CommandKind,
    pub canonical: String,
    pub aliases: Vec<String>,
    pub expected_parameters: Vec<String>,
    pub expected_members: Vec<String>,
    pub boundary: BoundaryKind,
    pub allow_loose_body: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CommandRegistry {
    by_alias: BTreeMap<String, CommandSpec>,
    canonical_names: BTreeSet<String>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, spec: CommandSpec) {
        self.canonical_names.insert(spec.canonical.clone());
        let mut aliases = spec.aliases.clone();
        aliases.push(spec.canonical.clone());
        for alias in aliases {
            self.by_alias.insert(normalize_alias(&alias), spec.clone());
        }
    }

    pub fn lookup_chain(&self, chain: &[String]) -> Option<CommandSpec> {
        if chain.is_empty() {
            return None;
        }
        let joined = normalize_alias(&chain.join(" "));
        if let Some(spec) = self.by_alias.get(&joined) {
            return Some(spec.clone());
        }
        for width in (1..=chain.len()).rev() {
            let probe = normalize_alias(&chain[..width].join(" "));
            if let Some(spec) = self.by_alias.get(&probe) {
                return Some(spec.clone());
            }
        }
        None
    }

    pub fn lookup_name(&self, name: &str) -> Option<CommandSpec> {
        self.by_alias.get(&normalize_alias(name)).cloned()
    }
}

fn normalize_alias(s: &str) -> String {
    s.trim()
        .trim_start_matches('@')
        .to_ascii_lowercase()
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn default_registry() -> CommandRegistry {
    use BoundaryKind::*;
    use CommandKind::*;

    let mut r = CommandRegistry::new();
    let specs = vec![
        spec(
            Task,
            "task",
            &["todo", "objective"],
            &["task_name", "task_details"],
            &["Title", "Description", "Content", "RelativeContext"],
            UntilNextCommand,
        ),
        spec(
            Idea,
            "idea",
            &["floating idea"],
            &[],
            &["Title", "Description", "Content", "RelativeContext"],
            UntilNextCommand,
        ),
        spec(
            Deferred,
            "deferred",
            &["defer", "later", "deffered"],
            &[],
            &["Title", "Description", "Content", "RelativeContext"],
            UntilNextCommand,
        ),
        spec(
            Progressive,
            "progressive",
            &["main feature", "promote"],
            &[],
            &["Title", "Description", "Content"],
            UntilNextCommand,
        ),
        spec(
            Thought,
            "thought",
            &["note"],
            &[],
            &["Title", "Description", "Content", "RelativeContext"],
            UntilNextCommand,
        ),
        spec(
            Project,
            "project",
            &["workspace project"],
            &[],
            &["Title", "Description", "Content", "Ideas"],
            UntilNextCommand,
        ),
        spec(
            Tutorial,
            "tutorial",
            &["guide"],
            &[],
            &["Title", "Description", "Content", "Shortcuts", "Commands"],
            UntilNextCommand,
        ),
        spec(
            Prompt,
            "prompt",
            &["composer prompt", "cursor prompt"],
            &[],
            &["Title", "Description", "Content"],
            UntilNextCommand,
        ),
        spec(
            Tags,
            "tags",
            &["tag"],
            &["tags"],
            &["Tags"],
            UntilNextCommand,
        ),
        spec(
            Alias,
            "alias",
            &["shortcut"],
            &[],
            &[
                "Name",
                "Target",
                "File",
                "Folder",
                "Path",
                "Environment",
                "Position",
            ],
            UntilNextCommand,
        ),
        spec(
            Categories,
            "categories",
            &["category"],
            &["categories"],
            &["Categories"],
            UntilNextCommand,
        ),
        spec(
            Goals,
            "goals",
            &["goal"],
            &[],
            &["Title", "Description", "Content"],
            UntilNextCommand,
        ),
        spec(
            Algodocutize,
            "algodocutize",
            &["algorithmize", "algodocumentize"],
            &[],
            &["Input", "Output", "Steps"],
            UntilNextCommand,
        ),
        spec(
            Deprecated,
            "deprecated",
            &["obsolete"],
            &[],
            &["Reason", "Replacement"],
            UntilNextCommand,
        ),
        spec(
            MacroClipboard,
            "macro clipboard",
            &["macro @clipboard", "clipboard macro"],
            &[],
            &["Title", "Content", "Tags"],
            UntilNextCommand,
        ),
        spec(
            Enqueue,
            "enqueue",
            &["queue", "enqueue[x]"],
            &["queue_name"],
            &["Item", "Queue", "Priority"],
            UntilNextCommand,
        ),
        spec(
            Groups,
            "groups",
            &["group"],
            &[],
            &["Title", "Environments", "Workspaces", "Modules"],
            UntilNextCommand,
        ),
        spec(
            ObjectiveQueue,
            "objectivequeue",
            &["objective queue", "queue objective"],
            &[],
            &["Objectives"],
            UntilNextCommand,
        ),
        spec(
            Reference,
            "reference",
            &["ref"],
            &["filepath"],
            &["Path", "Url", "Target"],
            SameLine,
        ),
        spec(
            Before,
            "before",
            &["backreference"],
            &[],
            &["Context"],
            UntilNextCommand,
        ),
        spec(
            QA,
            "q/a",
            &["qa", "question answer"],
            &[],
            &["Questions", "Answers"],
            UntilNextCommand,
        ),
        spec(
            Current,
            "current",
            &["current objective"],
            &[],
            &["Objective"],
            UntilNextCommand,
        ),
        spec(
            In,
            "in",
            &["inside", "context"],
            &["scope"],
            &["Environment", "Workspace", "Module"],
            UntilNextCommand,
        ),
        spec(Complete, "complete", &["done"], &[], &["Target"], SameLine),
        spec(Building, "building", &["build"], &[], &["Target"], SameLine),
        spec(
            Adapting,
            "adapting",
            &["adapterd", "adapting"],
            &[],
            &["Target"],
            SameLine,
        ),
    ];
    for s in specs {
        r.register(s);
    }
    r
}

fn spec(
    kind: CommandKind,
    canonical: &str,
    aliases: &[&str],
    params: &[&str],
    members: &[&str],
    boundary: BoundaryKind,
) -> CommandSpec {
    CommandSpec {
        kind,
        canonical: canonical.to_string(),
        aliases: aliases.iter().map(|s| s.to_string()).collect(),
        expected_parameters: params.iter().map(|s| s.to_string()).collect(),
        expected_members: members.iter().map(|s| s.to_string()).collect(),
        boundary,
        allow_loose_body: true,
    }
}
