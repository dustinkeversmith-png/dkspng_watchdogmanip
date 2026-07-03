use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::json;
use std::collections::BTreeMap;

use crate::parse::model::{CommandKind, ParsedCommand};
use crate::parse::passes::boundary::CommandBlock;

static MEMBER_LINE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(?P<key>[A-Za-z][A-Za-z0-9 _/-]{1,40})\s*:\s*(?P<value>.*)$").unwrap()
});
static TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?P<tag>#[A-Za-z][A-Za-z0-9_-]+)").unwrap());
static INLINE_STATUS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\((?P<status>done|complete|deferred|deffered|building|adapting|blocked)\)")
        .unwrap()
});
static PATHISH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?x)(?:[A-Za-z]:\\[^\s]+|\./[^\s]+|\.\./[^\s]+|/[^\s]+|[\w.-]+/[\w./-]+)").unwrap()
});
static URL: Lazy<Regex> = Lazy::new(|| Regex::new(r"https?://[^\s)]+").unwrap());

pub fn extract_block(block: &CommandBlock, index: usize) -> ParsedCommand {
    let raw_body = block.body_lines.join("\n").trim().to_string();
    let mut members = BTreeMap::new();
    let mut content_lines = Vec::new();
    let mut title = None;
    let mut description = None;

    for line in &block.body_lines {
        if let Some(caps) = MEMBER_LINE.captures(line) {
            let key = caps.name("key").unwrap().as_str().trim().to_string();
            let value = caps
                .name("value")
                .map(|m| m.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            members.insert(key.clone(), json!(value));
            match key.to_ascii_lowercase().as_str() {
                "title" | "name" => title = Some(value),
                "description" | "desc" => description = Some(value),
                _ => {}
            }
        } else {
            content_lines.push(line.clone());
        }
    }

    if title.is_none() {
        title = infer_title(&block.seed.canonical_kind, &raw_body);
    }

    let tags = TAG
        .captures_iter(&raw_body)
        .map(|c| c["tag"].trim_start_matches('#').to_string())
        .collect();
    let mut references: Vec<String> = URL
        .find_iter(&raw_body)
        .map(|m| m.as_str().to_string())
        .collect();
    references.extend(PATHISH.find_iter(&raw_body).map(|m| m.as_str().to_string()));
    references.sort();
    references.dedup();
    let statuses = INLINE_STATUS
        .captures_iter(&raw_body)
        .map(|c| canonical_status(&c["status"]))
        .collect();

    let parameters = extract_parameters(&block.seed.inline_payload);
    let inferred = matches!(block.seed.canonical_kind, CommandKind::Inferred(_));
    let confidence_reason = if inferred {
        "Created from markdown/prose structure with low-confidence inferred identity.".to_string()
    } else if matches!(block.seed.canonical_kind, CommandKind::Unknown(_)) {
        "Unknown @command preserved for recovery instead of being discarded.".to_string()
    } else {
        "Explicit @command matched registry and body was captured by boundary solver.".to_string()
    };

    ParsedCommand {
        id: format!("cmd_{:04}", index + 1),
        kind: block.seed.canonical_kind.clone(),
        raw_identity: block.seed.raw_identity.clone(),
        aliases_seen: block.seed.chain.clone(),
        title,
        description,
        content: content_lines.join("\n").trim().to_string(),
        parameters,
        members,
        tags,
        references,
        statuses,
        inferred,
        confidence: block.seed.confidence,
        confidence_reason,
        boundary_kind: block.boundary_kind.clone(),
        span: block.span,
        source_trace: format!("lines {}-{}", block.span.line_start, block.span.line_end),
        parent_id: None,
        child_ids: Vec::new(),
        hierarchy_path: Vec::new(),
        heading_context: Vec::new(),
        list_context: None,
    }
}

fn infer_title(kind: &CommandKind, body: &str) -> Option<String> {
    let first = body.lines().map(str::trim).find(|l| !l.is_empty())?;
    let cleaned = first.trim_start_matches(['-', '*']).trim();
    if cleaned.len() > 80 {
        Some(format!("{}...", &cleaned[..77]))
    } else if cleaned.is_empty() {
        match kind {
            CommandKind::ObjectiveQueue => Some("Objective Queue".to_string()),
            _ => None,
        }
    } else {
        Some(cleaned.to_string())
    }
}

fn extract_parameters(payload: &str) -> Vec<String> {
    payload
        .split_whitespace()
        .filter(|s| !s.contains(':') && !s.starts_with('#'))
        .take(8)
        .map(|s| s.trim_matches(|c: char| c == ',' || c == ';').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn canonical_status(s: &str) -> String {
    match s.to_ascii_lowercase().as_str() {
        "deffered" => "deferred".to_string(),
        "complete" => "done".to_string(),
        other => other.to_string(),
    }
}
