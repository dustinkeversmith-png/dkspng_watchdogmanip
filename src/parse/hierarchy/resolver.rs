use crate::parse::boundary::CommandBlock;
use crate::parse::hierarchy::detector::HierarchyDetectorRegistry;
use crate::parse::hierarchy::model::{HierarchySignal, HierarchySignalKind, ParseHierarchyNode};
use crate::parse::model::{CommandKind, ParsedCommand, SourceDocument};

#[derive(Debug, Clone, Default)]
struct HeadingFrame {
    level: usize,
    title: String,
    command_id: Option<String>,
}

#[derive(Debug, Clone)]
struct ListFrame {
    level: usize,
    label: String,
    kind: HierarchySignalKind,
}

pub fn resolve_hierarchy(
    doc: &SourceDocument,
    blocks: &[CommandBlock],
    commands: &mut [ParsedCommand],
) -> Vec<ParseHierarchyNode> {
    let registry = HierarchyDetectorRegistry::with_defaults();
    resolve_hierarchy_with_detectors(doc, blocks, commands, &registry)
}

pub fn resolve_hierarchy_with_detectors(
    doc: &SourceDocument,
    blocks: &[CommandBlock],
    commands: &mut [ParsedCommand],
    detector_registry: &HierarchyDetectorRegistry,
) -> Vec<ParseHierarchyNode> {
    let signals = detector_registry.detect_all(doc, blocks);
    let mut heading_stack: Vec<HeadingFrame> = Vec::new();
    let mut list_stack: Vec<ListFrame> = Vec::new();
    let mut signal_idx = 0;
    let mut prior: Vec<(String, usize, Vec<String>)> = Vec::new();
    let mut nodes = Vec::new();

    for command in commands.iter_mut() {
        while signal_idx < signals.len() && signals[signal_idx].line <= command.span.line_start {
            apply_signal(&signals[signal_idx], &mut heading_stack, &mut list_stack);
            signal_idx += 1;
        }

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
            list_stack.push(ListFrame {
                level: list_stack.len() + 1,
                label: command.title.clone().unwrap_or_default(),
                kind: HierarchySignalKind::BulletList,
            });
        }

        let heading_context: Vec<String> = heading_stack
            .iter()
            .map(|frame| frame.title.clone())
            .filter(|title| !title.is_empty())
            .collect();
        let hierarchy_path = build_hierarchy_path(&heading_stack, &list_stack);
        let list_context = list_stack.last().map(|frame| frame.label.clone());

        command.heading_context = heading_context;
        command.list_context = list_context;
        command.hierarchy_path = hierarchy_path.clone();

        let parent_id = find_parent_id(command, &hierarchy_path, &prior, &heading_stack);
        command.parent_id = parent_id.clone();

        let signal_kinds = active_signal_kinds(&heading_stack, &list_stack);
        prior.push((
            command.id.clone(),
            command.span.line_start,
            hierarchy_path.clone(),
        ));

        nodes.push(ParseHierarchyNode {
            command_id: command.id.clone(),
            parent_id: parent_id.clone(),
            child_ids: Vec::new(),
            hierarchy_path,
            signal_kinds,
        });
    }

    attach_child_ids(&mut nodes);
    nodes
}

fn apply_signal(
    signal: &HierarchySignal,
    heading_stack: &mut Vec<HeadingFrame>,
    list_stack: &mut Vec<ListFrame>,
) {
    match signal.kind {
        HierarchySignalKind::MarkdownHeading => {
            while heading_stack
                .last()
                .is_some_and(|frame| frame.level >= signal.level)
            {
                heading_stack.pop();
            }
            heading_stack.push(HeadingFrame {
                level: signal.level,
                title: signal.label.clone().unwrap_or_default(),
                command_id: None,
            });
        }
        HierarchySignalKind::NumberedList | HierarchySignalKind::BulletList => {
            while list_stack
                .last()
                .is_some_and(|frame| frame.level >= signal.level)
            {
                list_stack.pop();
            }
            list_stack.push(ListFrame {
                level: signal.level,
                label: signal.label.clone().unwrap_or_default(),
                kind: signal.kind.clone(),
            });
        }
        HierarchySignalKind::Indentation => {
            if let Some(frame) = list_stack.last_mut() {
                frame.level = frame.level.max(signal.level);
            } else {
                list_stack.push(ListFrame {
                    level: signal.level,
                    label: String::new(),
                    kind: HierarchySignalKind::Indentation,
                });
            }
        }
        _ => {}
    }
}

fn build_hierarchy_path(heading_stack: &[HeadingFrame], list_stack: &[ListFrame]) -> Vec<String> {
    heading_stack
        .iter()
        .map(|frame| frame.title.clone())
        .chain(
            list_stack
                .iter()
                .map(|frame| frame.label.clone())
                .filter(|label| !label.is_empty()),
        )
        .filter(|part| !part.is_empty())
        .collect()
}

fn active_signal_kinds(
    heading_stack: &[HeadingFrame],
    list_stack: &[ListFrame],
) -> Vec<HierarchySignalKind> {
    let mut kinds = Vec::new();
    if !heading_stack.is_empty() {
        kinds.push(HierarchySignalKind::MarkdownHeading);
    }
    for frame in list_stack {
        if !kinds.contains(&frame.kind) {
            kinds.push(frame.kind.clone());
        }
    }
    kinds
}

fn find_parent_id(
    command: &ParsedCommand,
    path: &[String],
    prior: &[(String, usize, Vec<String>)],
    heading_stack: &[HeadingFrame],
) -> Option<String> {
    if matches!(command.kind, CommandKind::Inferred(ref s) if s == "heading_section") {
        return heading_stack
            .iter()
            .rev()
            .nth(1)
            .and_then(|frame| frame.command_id.clone());
    }

    for (id, line, prior_path) in prior.iter().rev() {
        if *line >= command.span.line_start {
            continue;
        }
        if prior_path.len() < path.len() && path.starts_with(prior_path) {
            return Some(id.clone());
        }
    }

    let heading_prefix: Vec<String> = heading_stack
        .iter()
        .map(|frame| frame.title.clone())
        .filter(|title| !title.is_empty())
        .collect();

    for (id, line, prior_path) in prior.iter().rev() {
        if *line >= command.span.line_start {
            continue;
        }
        if !heading_prefix.is_empty()
            && prior_path.len() <= heading_prefix.len()
            && heading_prefix.starts_with(prior_path)
        {
            return Some(id.clone());
        }
        if prior_path == path {
            return Some(id.clone());
        }
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
