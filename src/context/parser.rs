use crate::context::error::{ContextError, Result};
use crate::context::index::ContextIndex;
use crate::context::model::{AliasRecord, AliasTargetRef, ContextNode, CurrentObjective, InheritancePolicy, QueueItem, SymbolRecord};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ParseConfig {
    pub default_context_id: String,
}

impl Default for ParseConfig {
    fn default() -> Self {
        Self { default_context_id: "project".to_string() }
    }
}

pub fn build_index_from_document(input: &str, config: ParseConfig) -> Result<ContextIndex> {
    let mut index = ContextIndex::new();
    index.upsert_context(ContextNode::new(&config.default_context_id, &config.default_context_id, "."));

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        parse_line_into_index(line, &config, &mut index)?;
    }
    Ok(index)
}

fn parse_line_into_index(line: &str, config: &ParseConfig, index: &mut ContextIndex) -> Result<()> {
    let mut parts = line.split_whitespace();
    let Some(head) = parts.next() else { return Ok(()); };
    let rest: Vec<&str> = parts.collect();

    match head.to_ascii_lowercase().as_str() {
        "@context" => {
            let ctx = parse_context(&rest)?;
            index.upsert_context(ctx);
        }
        "@alias" => {
            let (context_id, alias) = parse_alias(&rest, config)?;
            ensure_context(index, &context_id);
            index.add_alias(&context_id, alias)?;
        }
        "@current" => {
            let context_id = option_value(&rest, "context").unwrap_or_else(|| config.default_context_id.clone());
            let title = filtered_title(&rest, &["context"]);
            ensure_context(index, &context_id);
            index.context_mut(&context_id)?.currents.push(CurrentObjective { title, details: None, source_path: None });
        }
        "@queue" | "@enqueue" => {
            let context_id = option_value(&rest, "context").unwrap_or_else(|| config.default_context_id.clone());
            let status = option_value(&rest, "status");
            let priority = option_value(&rest, "priority");
            let title = filtered_title(&rest, &["context", "status", "priority"]);
            ensure_context(index, &context_id);
            index.add_queue_item(&context_id, QueueItem { title, details: None, status, priority, source_path: None })?;
        }
        "@symbol" => {
            if rest.len() < 2 {
                return Err(ContextError::ParseError("@Symbol requires: <kind> <name> [context=...] [path=...] [line=...]".into()));
            }
            let kind = rest[0].to_string();
            let name = rest[1].to_string();
            let context_id = option_value(&rest[2..], "context").unwrap_or_else(|| config.default_context_id.clone());
            let source_path = option_value(&rest[2..], "path").map(PathBuf::from);
            let line = option_value(&rest[2..], "line").and_then(|v| v.parse::<u32>().ok());
            ensure_context(index, &context_id);
            index.add_symbol(&context_id, SymbolRecord { name, kind, source_path, line, context_id: None })?;
        }
        _ => {}
    }
    Ok(())
}

fn ensure_context(index: &mut ContextIndex, context_id: &str) {
    if !index.contexts.contains_key(context_id) {
        index.upsert_context(ContextNode::new(context_id, context_id, "."));
    }
}

fn parse_context(rest: &[&str]) -> Result<ContextNode> {
    if rest.len() < 2 {
        return Err(ContextError::ParseError("@Context requires: <id> <root_path> [parent=...] [inheritance=...] [imports=a,b]".into()));
    }
    let mut ctx = ContextNode::new(rest[0], rest[0], rest[1]);
    if let Some(parent) = option_value(&rest[2..], "parent") {
        ctx.parent_id = Some(parent);
    }
    if let Some(inheritance) = option_value(&rest[2..], "inheritance") {
        ctx.inheritance = parse_inheritance(&inheritance)?;
    }
    if let Some(imports) = option_value(&rest[2..], "imports") {
        ctx.imports = imports.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    }
    Ok(ctx)
}

fn parse_alias(rest: &[&str], config: &ParseConfig) -> Result<(String, AliasRecord)> {
    if rest.len() < 2 {
        return Err(ContextError::ParseError("@Alias requires: <name> <target> [context=...] [type=file|folder|symbol|context|search]".into()));
    }
    let name = rest[0].to_string();
    let raw_target = rest[1];
    let context_id = option_value(&rest[2..], "context").unwrap_or_else(|| config.default_context_id.clone());
    let target_type = option_value(&rest[2..], "type").unwrap_or_else(|| infer_target_type(raw_target));
    let target = match target_type.as_str() {
        "folder" => AliasTargetRef::Folder { path: PathBuf::from(raw_target) },
        "symbol" => AliasTargetRef::Symbol { name: raw_target.to_string(), kind: option_value(&rest[2..], "kind") },
        "context" => AliasTargetRef::Context { context_id: raw_target.to_string() },
        "workspace" => AliasTargetRef::Workspace { workspace_id: raw_target.to_string() },
        "command" => AliasTargetRef::Command { command_name: raw_target.to_string(), args: option_value(&rest[2..], "args").map(|v| v.split(',').map(|s| s.to_string()).collect()).unwrap_or_default() },
        "search" => AliasTargetRef::Search { query: raw_target.to_string(), scope: option_value(&rest[2..], "scope") },
        _ => AliasTargetRef::File {
            path: PathBuf::from(raw_target),
            line: option_value(&rest[2..], "line").and_then(|v| v.parse::<u32>().ok()),
            column: option_value(&rest[2..], "col").or_else(|| option_value(&rest[2..], "column")).and_then(|v| v.parse::<u32>().ok()),
            marker: option_value(&rest[2..], "marker"),
        },
    };
    Ok((context_id, AliasRecord::new(name, target)))
}

fn option_value(tokens: &[&str], key: &str) -> Option<String> {
    let prefix = format!("{key}=");
    tokens.iter().find_map(|token| token.strip_prefix(&prefix).map(|value| value.to_string()))
}

fn filtered_title(tokens: &[&str], option_keys: &[&str]) -> String {
    tokens
        .iter()
        .filter(|token| !option_keys.iter().any(|key| token.starts_with(&format!("{key}="))))
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

fn infer_target_type(raw: &str) -> String {
    if raw.ends_with('/') || (!raw.contains('.') && raw.starts_with("./")) {
        "folder".to_string()
    } else {
        "file".to_string()
    }
}

fn parse_inheritance(value: &str) -> Result<InheritancePolicy> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Ok(InheritancePolicy::None),
        "parent" | "parent_only" => Ok(InheritancePolicy::ParentOnly),
        "ancestors" => Ok(InheritancePolicy::Ancestors),
        "project" | "project_root" => Ok(InheritancePolicy::ProjectRoot),
        "workspace" | "workspace_root" => Ok(InheritancePolicy::WorkspaceRoot),
        "explicit" | "explicit_imports_only" => Ok(InheritancePolicy::ExplicitImportsOnly),
        other => Err(ContextError::ParseError(format!("unknown inheritance policy: {other}"))),
    }
}
