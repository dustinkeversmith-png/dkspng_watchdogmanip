use crate::parse::hierarchy::model::ParseHierarchyNode;
use crate::parse::model::{CommandKind, ParsedCommand, SourceDocument};

#[derive(Debug, Clone, Default)]
struct HeadingFrame {
    level: usize,
    title: String,
    command_id: Option<String>,
}

pub fn resolve_hierarchy(
    _doc: &SourceDocument,
    commands: &mut [ParsedCommand],
) -> Vec<ParseHierarchyNode> {
    let heading_stack: Vec<HeadingFrame> = Vec::new();
    let mut heading_stack = heading_stack;
    let mut list_context: Option<String> = None;
    let mut nodes = Vec::new();

    for command in commands.iter_mut() {
        if matches!(command.kind, CommandKind::Inferred(ref s) if s == "heading_section") {
            let level = heading_stack.len() + 1;
            while heading_stack
                .last()
                .is_some_and(|frame| frame.level >= level)
            {
                heading_stack.pop();
            }
            heading_stack.push(HeadingFrame {
                level,
                title: command.title.clone().unwrap_or_default(),
                command_id: Some(command.id.clone()),
            });
        }

        if command.raw_identity.starts_with("- ")
            || command
                .content
                .lines()
                .any(|l| l.trim_start().starts_with("- "))
        {
            list_context = command.title.clone();
        }

        let hierarchy_path: Vec<String> = heading_stack
            .iter()
            .map(|frame| frame.title.clone())
            .filter(|title| !title.is_empty())
            .collect();

        command.heading_context = hierarchy_path.clone();
        command.list_context = list_context.clone();
        command.hierarchy_path = hierarchy_path.clone();

        let parent_id = find_parent_id(&heading_stack, command);
        command.parent_id = parent_id.clone();

        nodes.push(ParseHierarchyNode {
            command_id: command.id.clone(),
            parent_id: parent_id.clone(),
            child_ids: Vec::new(),
            hierarchy_path: hierarchy_path.clone(),
        });
    }

    attach_child_ids(&mut nodes);
    nodes
}

fn find_parent_id(heading_stack: &[HeadingFrame], command: &ParsedCommand) -> Option<String> {
    if matches!(command.kind, CommandKind::Inferred(ref s) if s == "heading_section") {
        return heading_stack
            .iter()
            .rev()
            .nth(1)
            .and_then(|frame| frame.command_id.clone());
    }

    heading_stack
        .iter()
        .rev()
        .find_map(|frame| frame.command_id.clone())
}

fn attach_child_ids(nodes: &mut [ParseHierarchyNode]) {
    let ids: Vec<_> = nodes
        .iter()
        .map(|node| (node.command_id.clone(), node.parent_id.clone()))
        .collect();

    for (command_id, parent_id) in &ids {
        if let Some(parent) = parent_id {
            if let Some(parent_node) = nodes.iter_mut().find(|n| &n.command_id == parent) {
                if !parent_node.child_ids.contains(command_id) {
                    parent_node.child_ids.push(command_id.clone());
                }
            }
        }
    }
}
