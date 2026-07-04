use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::parse::model::{BoundaryKind, CommandKind};
use crate::parse::shape::{CommandShapeKind, ParameterShapeKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandBodyPolicy {
    CaptureIfPresent,
    MarkerOnly,
    PreferInline,
    PreferBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub name: String,
    pub required: bool,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
    pub shape_hints: Vec<ParameterShapeKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberSpec {
    pub name: String,
    pub required: bool,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
    pub value_shape_hints: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub kind: CommandKind,
    pub canonical: String,
    pub aliases: Vec<String>,
    pub parameters: Vec<ParameterSpec>,
    pub optional_members: Vec<MemberSpec>,
    pub required_members: Vec<MemberSpec>,
    pub accepted_layouts: Vec<CommandLayoutKind>,
    pub accepted_shapes: Vec<CommandShapeKind>,
    pub body_policy: CommandBodyPolicy,
    pub boundary: BoundaryKind,
}

impl CommandSpec {
    pub fn parameter_names(&self) -> Vec<String> {
        self.parameters
            .iter()
            .flat_map(|p| std::iter::once(p.name.clone()).chain(p.aliases.clone()))
            .collect()
    }

    pub fn member_names(&self) -> Vec<String> {
        self.optional_members
            .iter()
            .chain(self.required_members.iter())
            .flat_map(|m| std::iter::once(m.name.clone()).chain(m.aliases.clone()))
            .collect()
    }

    pub fn optional_parameters(&self) -> Vec<String> {
        self.parameters.iter().map(|p| p.name.clone()).collect()
    }

    pub fn optional_members(&self) -> Vec<String> {
        self.optional_members
            .iter()
            .map(|m| m.name.clone())
            .collect()
    }

    pub fn expected_parameters(&self) -> Vec<String> {
        self.optional_parameters()
    }

    pub fn expected_members(&self) -> Vec<String> {
        self.member_names()
    }

    pub fn members_with_tag(&self, tag: &str) -> Vec<&MemberSpec> {
        self.optional_members
            .iter()
            .chain(self.required_members.iter())
            .filter(|m| m.tags.iter().any(|t| t == tag))
            .collect()
    }

    pub fn parameters_with_tag(&self, tag: &str) -> Vec<&ParameterSpec> {
        self.parameters
            .iter()
            .filter(|p| p.tags.iter().any(|t| t == tag))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandLayoutKind {
    Inline,
    Block,
    HeadingSection,
    ListItem,
    KeyValue,
    Prose,
}

#[derive(Debug, Clone)]
pub struct CommandRegistry {
    by_alias: BTreeMap<String, CommandSpec>,
    canonical_names: BTreeSet<String>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            by_alias: BTreeMap::new(),
            canonical_names: BTreeSet::new(),
        }
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

impl Default for CommandRegistry {
    fn default() -> Self {
        use BoundaryKind::*;
        use CommandKind::*;

        let mut registry = CommandRegistry::new();
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
        for entry in specs {
            registry.register(entry);
        }
        registry
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

pub fn parameter(name: &str, required: bool, aliases: &[&str], tags: &[&str]) -> ParameterSpec {
    ParameterSpec {
        name: name.to_string(),
        required,
        aliases: aliases.iter().map(|s| s.to_string()).collect(),
        tags: tags.iter().map(|s| s.to_string()).collect(),
        shape_hints: vec![ParameterShapeKind::SingleLooseParameter],
    }
}

pub fn member(name: &str, required: bool, aliases: &[&str], tags: &[&str]) -> MemberSpec {
    MemberSpec {
        name: name.to_string(),
        required,
        aliases: aliases.iter().map(|s| s.to_string()).collect(),
        tags: tags.iter().map(|s| s.to_string()).collect(),
        value_shape_hints: default_value_shape_hints(name),
    }
}

fn default_value_shape_hints(name: &str) -> Vec<String> {
    match name.to_ascii_lowercase().as_str() {
        "path" | "file" | "folder" | "url" | "target" | "filepath" => {
            vec!["path".to_string()]
        }
        "tags" | "categories" => vec!["list".to_string()],
        "title" | "name" | "objective" => vec!["title".to_string()],
        "description" | "content" | "body" => vec!["prose".to_string()],
        _ => vec!["text".to_string()],
    }
}

fn default_member_tags(name: &str) -> Vec<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "title" | "name" => vec!["title_candidate", "display"],
        "description" | "desc" => vec!["description"],
        "content" | "body" => vec!["body"],
        "path" | "file" | "folder" | "url" | "target" => vec!["navigation_target", "reference"],
        "tags" => vec!["tags"],
        "objective" | "objectives" => vec!["title_candidate", "objective"],
        _ => vec!["member"],
    }
}

fn default_parameter_tags(name: &str) -> Vec<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "filepath" | "path" | "scope" => vec!["navigation_target"],
        "title" | "name" | "task_name" => vec!["title_candidate"],
        "tags" | "categories" | "queue_name" => vec!["metadata"],
        _ => vec!["parameter"],
    }
}

fn spec(
    kind: CommandKind,
    canonical: &str,
    aliases: &[&str],
    params: &[&str],
    members: &[&str],
    boundary: BoundaryKind,
) -> CommandSpec {
    let parameters: Vec<ParameterSpec> = params
        .iter()
        .map(|name| parameter(name, false, &[], &default_parameter_tags(name)))
        .collect();
    let optional_members: Vec<MemberSpec> = members
        .iter()
        .map(|name| {
            let aliases = default_member_aliases(name);
            member(name, false, &aliases, &default_member_tags(name))
        })
        .collect();
    CommandSpec {
        kind,
        canonical: canonical.to_string(),
        aliases: aliases.iter().map(|s| s.to_string()).collect(),
        parameters,
        optional_members,
        required_members: Vec::new(),
        accepted_layouts: vec![CommandLayoutKind::Block, CommandLayoutKind::Inline],
        accepted_shapes: vec![
            CommandShapeKind::InlineTitle,
            CommandShapeKind::KeyValueMembers,
            CommandShapeKind::ProseOnly,
            CommandShapeKind::Mixed,
        ],
        body_policy: CommandBodyPolicy::CaptureIfPresent,
        boundary,
    }
}

fn default_member_aliases(name: &str) -> Vec<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "title" => vec!["name", "label"],
        "description" => vec!["desc"],
        "path" => vec!["file", "target"],
        "target" => vec!["path", "file"],
        _ => vec![],
    }
}
