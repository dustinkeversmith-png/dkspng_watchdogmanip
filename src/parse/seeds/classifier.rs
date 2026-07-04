use serde::{Deserialize, Serialize};

use crate::parse::model::{CommandKind, CommandSeed, TextSpan};
use crate::parse::pipeline::ParseContext;
use crate::parse::seeds::command::CommandSeedStrategy;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedClassifierKind {
    ExplicitAtCommand,
    ChainedAtCommand,
    ClassifierKeyword,
    StatusKeyword,
    ParameterKeyword,
    GlossaryIdentifier,
    ReferenceLike,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedClassifierMatch {
    pub kind: SeedClassifierKind,
    pub keyword: String,
    pub normalized_identity: Option<String>,
    pub line: usize,
    pub column: usize,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifierKeywordSpec {
    pub keyword: String,
    pub maps_to_identity: Option<String>,
    pub kind: SeedClassifierKind,
    pub aliases: Vec<String>,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SeedClassifierRegistry {
    pub command_keywords: Vec<ClassifierKeywordSpec>,
    pub status_keywords: Vec<ClassifierKeywordSpec>,
    pub parameter_keywords: Vec<ClassifierKeywordSpec>,
    pub glossary_keywords: Vec<ClassifierKeywordSpec>,
}

impl SeedClassifierRegistry {
    pub fn with_defaults() -> Self {
        let command_keywords = vec![
            kw("task", Some("task"), SeedClassifierKind::ClassifierKeyword, &["todo"]),
            kw("idea", Some("idea"), SeedClassifierKind::ClassifierKeyword, &[]),
            kw("project", Some("project"), SeedClassifierKind::ClassifierKeyword, &[]),
            kw("prompt", Some("prompt"), SeedClassifierKind::ClassifierKeyword, &[]),
            kw("tutorial", Some("tutorial"), SeedClassifierKind::ClassifierKeyword, &["guide"]),
            kw("deferred", Some("deferred"), SeedClassifierKind::ClassifierKeyword, &["defer"]),
            kw("current", Some("current"), SeedClassifierKind::ClassifierKeyword, &[]),
            kw(
                "reference",
                Some("reference"),
                SeedClassifierKind::ReferenceLike,
                &["ref"],
            ),
            kw("alias", Some("alias"), SeedClassifierKind::ClassifierKeyword, &[]),
            kw("goal", Some("goals"), SeedClassifierKind::ClassifierKeyword, &["goals"]),
        ];
        let status_keywords = vec![
            kw("done", None, SeedClassifierKind::StatusKeyword, &[]),
            kw("complete", None, SeedClassifierKind::StatusKeyword, &[]),
            kw("building", None, SeedClassifierKind::StatusKeyword, &[]),
            kw("adapting", None, SeedClassifierKind::StatusKeyword, &[]),
        ];
        Self {
            command_keywords,
            status_keywords,
            parameter_keywords: Vec::new(),
            glossary_keywords: Vec::new(),
        }
    }

    pub fn classify_line(&self, line: &str, line_no: usize) -> Vec<SeedClassifierMatch> {
        let trimmed = line.trim();
        if trimmed.starts_with('@') {
            return Vec::new();
        }
        let lower = trimmed.to_ascii_lowercase();
        let mut matches = Vec::new();

        if let Some(chain) = detect_chained_keywords(trimmed, &self.command_keywords) {
            matches.push(SeedClassifierMatch {
                kind: SeedClassifierKind::ChainedAtCommand,
                keyword: chain.join(" "),
                normalized_identity: Some(chain.join(" ")),
                line: line_no,
                column: 0,
                confidence: 0.72,
                reason: "chained classifier keywords at line start".to_string(),
            });
        }

        for spec in &self.command_keywords {
            if let Some(m) = match_keyword(trimmed, &lower, line_no, spec) {
                matches.push(m);
            }
        }
        for spec in &self.status_keywords {
            if let Some(m) = match_keyword(trimmed, &lower, line_no, spec) {
                matches.push(m);
            }
        }

        matches
    }
}

fn kw(
    keyword: &str,
    maps_to: Option<&str>,
    kind: SeedClassifierKind,
    aliases: &[&str],
) -> ClassifierKeywordSpec {
    ClassifierKeywordSpec {
        keyword: keyword.to_string(),
        maps_to_identity: maps_to.map(str::to_string),
        kind,
        aliases: aliases.iter().map(|s| s.to_string()).collect(),
        case_sensitive: false,
    }
}

fn match_keyword(
    trimmed: &str,
    lower: &str,
    line_no: usize,
    spec: &ClassifierKeywordSpec,
) -> Option<SeedClassifierMatch> {
    let patterns: Vec<String> = std::iter::once(spec.keyword.clone())
        .chain(spec.aliases.clone())
        .collect();
    for pat in patterns {
        let pat_lower = pat.to_ascii_lowercase();
        let prefixes = [
            format!("{pat_lower}:"),
            format!("{pat_lower} -"),
            format!("{pat_lower} —"),
            format!("{pat_lower} "),
        ];
        if prefixes.iter().any(|p| lower.starts_with(p))
            || lower.starts_with(&format!("{pat_lower}["))
        {
            return Some(SeedClassifierMatch {
                kind: spec.kind.clone(),
                keyword: pat,
                normalized_identity: spec.maps_to_identity.clone(),
                line: line_no,
                column: 0,
                confidence: 0.68,
                reason: format!("classifier keyword near line start: {}", spec.keyword),
            });
        }
        if spec.kind == SeedClassifierKind::ReferenceLike
            && (lower.starts_with("reference ") || lower.starts_with("ref "))
        {
            return Some(SeedClassifierMatch {
                kind: SeedClassifierKind::ReferenceLike,
                keyword: pat,
                normalized_identity: spec.maps_to_identity.clone(),
                line: line_no,
                column: 0,
                confidence: 0.7,
                reason: "reference-like classifier prefix".to_string(),
            });
        }
        let _ = trimmed;
    }
    None
}

fn detect_chained_keywords(trimmed: &str, specs: &[ClassifierKeywordSpec]) -> Option<Vec<String>> {
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.len() < 2 {
        return None;
    }
    let w0 = words[0].trim_end_matches(':').to_ascii_lowercase();
    let w1 = words[1].trim_end_matches(':').to_ascii_lowercase();
    let known: Vec<String> = specs
        .iter()
        .flat_map(|s| {
            std::iter::once(s.keyword.clone()).chain(s.aliases.clone())
        })
        .map(|k| k.to_ascii_lowercase())
        .collect();
    if known.contains(&w0) && known.contains(&w1) {
        if words.get(2).is_some_and(|w| [":", "-", "—"].contains(w))
            || trimmed.contains(':')
            || trimmed.contains('[')
        {
            return Some(vec![w0, w1]);
        }
    }
    None
}

pub struct ClassifierCommandSeedStrategy {
    registry: SeedClassifierRegistry,
}

impl ClassifierCommandSeedStrategy {
    pub fn with_defaults() -> Self {
        Self {
            registry: SeedClassifierRegistry::with_defaults(),
        }
    }
}

impl CommandSeedStrategy for ClassifierCommandSeedStrategy {
    fn name(&self) -> &'static str {
        "classifier_keyword"
    }

    fn detect(&self, ctx: &ParseContext) -> Vec<CommandSeed> {
        let doc = ctx.document;
        let command_registry = ctx.command_registry;
        let mut seeds = Vec::new();
        for (line_idx, line) in doc.lines.iter().enumerate() {
            for m in self.registry.classify_line(&line.text, line.number) {
                let Some(identity) = m.normalized_identity else {
                    continue;
                };
                let chain: Vec<String> = identity.split_whitespace().map(str::to_string).collect();
                let spec = command_registry.lookup_chain(&chain);
                let kind = spec
                    .as_ref()
                    .map(|s| s.kind.clone())
                    .unwrap_or_else(|| CommandKind::Inferred(identity.clone()));
                let payload = line
                    .text
                    .split_once(':')
                    .map(|(_, rest)| rest.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_default();
                seeds.push(CommandSeed {
                    raw_identity: format!("~{}", identity),
                    chain,
                    canonical_kind: kind,
                    inline_payload: payload,
                    start_line_index: line_idx,
                    span: TextSpan::new(line.start, line.end, line.number, line.number),
                    confidence: m.confidence,
                });
            }
        }
        seeds
    }
}
