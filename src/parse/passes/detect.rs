use once_cell::sync::Lazy;
use regex::Regex;

use crate::parse::{
    model::{CommandKind, CommandSeed, SourceDocument, TextSpan},
    registry::CommandRegistry,
};

static AT_COMMAND: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)^\s*(?:\d+[.)]\s*)?(?P<chain>(?:@[A-Za-z][A-Za-z0-9_/-]*(?:\s+|$))+)(?P<payload>.*)$",
    )
    .unwrap()
});
static HASH_HEADING: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*#{1,6}\s+(?P<title>.+)$").unwrap());

pub fn detect_command_seeds(doc: &SourceDocument, registry: &CommandRegistry) -> Vec<CommandSeed> {
    let mut seeds = Vec::new();
    for (line_idx, line) in doc.lines.iter().enumerate() {
        let text = line.text.trim_end();
        if text.trim().is_empty() {
            continue;
        }

        if let Some(caps) = AT_COMMAND.captures(text) {
            let chain_raw = caps.name("chain").unwrap().as_str().trim();
            let mut chain: Vec<String> = chain_raw
                .split_whitespace()
                .filter(|p| p.starts_with('@'))
                .map(|p| p.trim_start_matches('@').to_string())
                .collect();

            let payload = caps
                .name("payload")
                .map(|m| m.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            // Support commands written as @project idea instead of @project @idea.
            if chain.len() == 1 {
                if let Some(first_word) = payload.split_whitespace().next() {
                    let possible_two = vec![chain[0].clone(), first_word.to_string()];
                    if registry.lookup_chain(&possible_two).is_some() {
                        chain = possible_two;
                    }
                }
            }
            let spec = registry.lookup_chain(&chain);
            let kind = spec
                .as_ref()
                .map(|s| s.kind.clone())
                .unwrap_or_else(|| CommandKind::Unknown(chain.join(" ")));
            let confidence = if spec.is_some() { 0.96 } else { 0.55 };
            seeds.push(CommandSeed {
                raw_identity: chain_raw.to_string(),
                chain,
                canonical_kind: kind,
                inline_payload: payload,
                start_line_index: line_idx,
                span: TextSpan::new(line.start, line.end, line.number, line.number),
                confidence,
            });
            continue;
        }

        if let Some(caps) = HASH_HEADING.captures(text) {
            let title = caps.name("title").unwrap().as_str().trim().to_string();
            seeds.push(CommandSeed {
                raw_identity: "#".to_string(),
                chain: vec!["heading".to_string()],
                canonical_kind: CommandKind::Inferred("heading_section".to_string()),
                inline_payload: title,
                start_line_index: line_idx,
                span: TextSpan::new(line.start, line.end, line.number, line.number),
                confidence: 0.45,
            });
        }
    }
    seeds
}
