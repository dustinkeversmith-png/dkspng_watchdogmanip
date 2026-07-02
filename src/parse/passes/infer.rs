use once_cell::sync::Lazy;
use regex::Regex;

use crate::parse::model::{CommandKind, CommandSeed, SourceDocument, TextSpan};

static TASK_VERB: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(fix|build|create|implement|revisit|compose|research|clean|refactor|add|remove|update)\b").unwrap()
});
static IDEA_HINT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(it would be cool|maybe|idea|what if|possibly|could)\b").unwrap()
});
static PATH_HINT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?x)(?:[A-Za-z]:\\|\./|\.\./|/|[\w.-]+/[\w./-]+)").unwrap());

pub fn infer_loose_objects(doc: &SourceDocument, existing: &[CommandSeed]) -> Vec<CommandSeed> {
    let mut covered = vec![false; doc.lines.len()];
    for seed in existing {
        if seed.start_line_index < covered.len() {
            covered[seed.start_line_index] = true;
        }
    }

    let mut seeds = Vec::new();
    for (idx, line) in doc.lines.iter().enumerate() {
        if covered[idx] {
            continue;
        }
        let trimmed = line.text.trim();
        if trimmed.len() < 8 || trimmed.starts_with('#') || trimmed.starts_with("```") {
            continue;
        }
        let (kind, confidence, why) = if TASK_VERB.is_match(trimmed)
            && (trimmed.starts_with('-') || trimmed.ends_with('?') || trimmed.contains("need to"))
        {
            (
                CommandKind::Inferred("loose_task".to_string()),
                0.42,
                "loose_task",
            )
        } else if IDEA_HINT.is_match(trimmed) {
            (
                CommandKind::Inferred("loose_idea".to_string()),
                0.38,
                "loose_idea",
            )
        } else if PATH_HINT.is_match(trimmed) {
            (CommandKind::Reference, 0.50, "path_reference")
        } else {
            continue;
        };
        seeds.push(CommandSeed {
            raw_identity: why.to_string(),
            chain: vec![why.to_string()],
            canonical_kind: kind,
            inline_payload: trimmed.to_string(),
            start_line_index: idx,
            span: TextSpan::new(line.start, line.end, line.number, line.number),
            confidence,
        });
    }
    seeds
}
