use crate::parse::extractors::model::{CommandExtractor, ExtractedCommandParts, ExtractionContext};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::json;

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

pub struct TitleExtractor;
impl CommandExtractor for TitleExtractor {
    fn name(&self) -> &'static str {
        "title"
    }
    fn extract(&self, context: &ExtractionContext) -> ExtractedCommandParts {
        let first = context
            .body
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(|line| line.trim_start_matches(['-', '*']).trim().to_string());
        ExtractedCommandParts {
            title: first,
            ..Default::default()
        }
    }
}

pub struct DescriptionExtractor;
impl CommandExtractor for DescriptionExtractor {
    fn name(&self) -> &'static str {
        "description"
    }
    fn extract(&self, context: &ExtractionContext) -> ExtractedCommandParts {
        let mut lines = context
            .body
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty());
        let _ = lines.next();
        let description = lines.take(3).collect::<Vec<_>>().join(" ");
        ExtractedCommandParts {
            description: if description.is_empty() {
                None
            } else {
                Some(description)
            },
            ..Default::default()
        }
    }
}

pub struct LooseParameterExtractor;
impl CommandExtractor for LooseParameterExtractor {
    fn name(&self) -> &'static str {
        "loose_parameters"
    }
    fn extract(&self, context: &ExtractionContext) -> ExtractedCommandParts {
        let parameters = context
            .seed
            .raw
            .split_whitespace()
            .filter(|s| !s.starts_with('@') && !s.contains(':'))
            .take(8)
            .map(|s| s.trim_matches(|c: char| c == ',' || c == ';').to_string())
            .filter(|s| !s.is_empty())
            .collect();
        ExtractedCommandParts {
            parameters,
            ..Default::default()
        }
    }
}

pub struct KeyValueMemberExtractor;
impl CommandExtractor for KeyValueMemberExtractor {
    fn name(&self) -> &'static str {
        "key_value_members"
    }
    fn extract(&self, context: &ExtractionContext) -> ExtractedCommandParts {
        let mut members = std::collections::BTreeMap::new();
        for line in context.body.lines() {
            if let Some(caps) = MEMBER_LINE.captures(line) {
                let key = caps.name("key").unwrap().as_str().trim().to_string();
                let value = caps
                    .name("value")
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                members.insert(key, json!(value));
            }
        }
        ExtractedCommandParts {
            members,
            ..Default::default()
        }
    }
}

pub struct TagExtractor;
impl CommandExtractor for TagExtractor {
    fn name(&self) -> &'static str {
        "tags"
    }
    fn extract(&self, context: &ExtractionContext) -> ExtractedCommandParts {
        let tags = TAG
            .captures_iter(&context.body)
            .map(|c| c["tag"].trim_start_matches('#').to_string())
            .collect();
        ExtractedCommandParts {
            tags,
            ..Default::default()
        }
    }
}

pub struct ReferenceExtractor;
impl CommandExtractor for ReferenceExtractor {
    fn name(&self) -> &'static str {
        "references"
    }
    fn extract(&self, context: &ExtractionContext) -> ExtractedCommandParts {
        let mut references: Vec<String> = URL
            .find_iter(&context.body)
            .map(|m| m.as_str().to_string())
            .collect();
        references.extend(
            PATHISH
                .find_iter(&context.body)
                .map(|m| m.as_str().to_string()),
        );
        references.sort();
        references.dedup();
        ExtractedCommandParts {
            references,
            ..Default::default()
        }
    }
}

pub struct StatusExtractor;
impl CommandExtractor for StatusExtractor {
    fn name(&self) -> &'static str {
        "statuses"
    }
    fn extract(&self, context: &ExtractionContext) -> ExtractedCommandParts {
        let statuses = INLINE_STATUS
            .captures_iter(&context.body)
            .map(|c| canonical_status(&c["status"]))
            .collect();
        ExtractedCommandParts {
            statuses,
            ..Default::default()
        }
    }
}

pub fn default_extractors() -> Vec<Box<dyn CommandExtractor>> {
    vec![
        Box::new(TitleExtractor),
        Box::new(DescriptionExtractor),
        Box::new(LooseParameterExtractor),
        Box::new(KeyValueMemberExtractor),
        Box::new(TagExtractor),
        Box::new(ReferenceExtractor),
        Box::new(StatusExtractor),
    ]
}

pub fn merge_parts(parts: &[ExtractedCommandParts]) -> ExtractedCommandParts {
    let mut merged = ExtractedCommandParts::default();
    for part in parts {
        if merged.title.is_none() {
            merged.title = part.title.clone();
        }
        if merged.description.is_none() {
            merged.description = part.description.clone();
        }
        merged.parameters.extend(part.parameters.clone());
        merged.members.extend(part.members.clone());
        merged.tags.extend(part.tags.clone());
        merged.references.extend(part.references.clone());
        merged.statuses.extend(part.statuses.clone());
    }
    merged.tags.sort();
    merged.tags.dedup();
    merged.references.sort();
    merged.references.dedup();
    merged.statuses.sort();
    merged.statuses.dedup();
    merged
}

fn canonical_status(s: &str) -> String {
    match s.to_ascii_lowercase().as_str() {
        "deffered" => "deferred".to_string(),
        "complete" => "done".to_string(),
        other => other.to_string(),
    }
}
