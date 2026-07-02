use crate::parse::model::{CommandKind, ParseOutput, ParsedCommand};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParseDatabase {
    /// Full parsed documents keyed by source name/path.
    pub documents: BTreeMap<String, ParseOutput>,

    /// Flattened command index keyed by stable database command key.
    pub commands: BTreeMap<String, StoredParsedCommand>,

    /// Stringified command kind -> command keys.
    pub by_kind: BTreeMap<String, BTreeSet<String>>,

    /// tag -> command keys.
    pub by_tag: BTreeMap<String, BTreeSet<String>>,

    /// reference/path/symbol -> command keys.
    pub by_reference: BTreeMap<String, BTreeSet<String>>,

    /// lowercase token -> command keys.
    pub token_index: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredParsedCommand {
    pub db_key: String,
    pub source_name: String,
    pub command: ParsedCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseSearchHit {
    pub source_name: String,
    pub command_id: String,
    pub db_key: String,
    pub kind: CommandKind,
    pub title: Option<String>,
    pub content_preview: String,
    pub score: usize,
    pub matched_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseCommandView {
    pub db_key: String,
    pub source_name: String,
    pub command_id: String,
    pub kind: CommandKind,
    pub raw_identity: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<String>,
    pub tags: Vec<String>,
    pub references: Vec<String>,
    pub statuses: Vec<String>,
    pub content_preview: String,
    pub source_trace: String,
}

#[derive(Debug, Clone, Default)]
pub struct ParseSearchOptions {
    pub query: String,
    pub kind: Option<CommandKind>,
    pub tag: Option<String>,
    pub reference: Option<String>,
    pub source_name: Option<String>,
    pub limit: Option<usize>,
}

impl ParseDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.documents.clear();
        self.commands.clear();
        self.by_kind.clear();
        self.by_tag.clear();
        self.by_reference.clear();
        self.token_index.clear();
    }

    pub fn insert_output(&mut self, output: ParseOutput) {
        let source_name = output.source_name.clone();

        // If the source already exists, rebuild for correctness.
        if self.documents.contains_key(&source_name) {
            self.documents.insert(source_name.clone(), output);
            self.rebuild_indexes();
            return;
        }

        for command in &output.commands {
            self.insert_command_indexes(&source_name, command.clone());
        }

        self.documents.insert(source_name, output);
    }

    pub fn insert_many<I>(&mut self, outputs: I)
    where
        I: IntoIterator<Item = ParseOutput>,
    {
        for output in outputs {
            self.insert_output(output);
        }
    }

    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn all_command_views(&self) -> Vec<ParseCommandView> {
        self.commands
            .values()
            .map(|stored| command_view(stored))
            .collect()
    }

    pub fn command_view(&self, db_key: &str) -> Option<ParseCommandView> {
        self.commands.get(db_key).map(command_view)
    }

    pub fn commands_for_source(&self, source_name: &str) -> Vec<ParseCommandView> {
        self.commands
            .values()
            .filter(|stored| stored.source_name == source_name)
            .map(command_view)
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<ParseSearchHit> {
        self.search_with_options(ParseSearchOptions {
            query: query.to_string(),
            ..Default::default()
        })
    }

    pub fn search_by_kind(&self, kind: &CommandKind) -> Vec<ParseSearchHit> {
        self.search_with_options(ParseSearchOptions {
            query: String::new(),
            kind: Some(kind.clone()),
            ..Default::default()
        })
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<ParseSearchHit> {
        self.search_with_options(ParseSearchOptions {
            query: String::new(),
            tag: Some(tag.to_string()),
            ..Default::default()
        })
    }

    pub fn search_by_reference(&self, reference: &str) -> Vec<ParseSearchHit> {
        self.search_with_options(ParseSearchOptions {
            query: String::new(),
            reference: Some(reference.to_string()),
            ..Default::default()
        })
    }

    pub fn search_with_options(&self, options: ParseSearchOptions) -> Vec<ParseSearchHit> {
        let mut candidate_keys: Option<BTreeSet<String>> = None;

        if let Some(kind) = &options.kind {
            let keys = self
                .by_kind
                .get(&kind_key(kind))
                .cloned()
                .unwrap_or_default();
            candidate_keys = Some(intersect_candidate_keys(candidate_keys, keys));
        }

        if let Some(tag) = &options.tag {
            let keys = self
                .by_tag
                .get(&tag.to_ascii_lowercase())
                .cloned()
                .unwrap_or_default();
            candidate_keys = Some(intersect_candidate_keys(candidate_keys, keys));
        }

        if let Some(reference) = &options.reference {
            let keys = self
                .by_reference
                .get(&reference.to_ascii_lowercase())
                .cloned()
                .unwrap_or_default();
            candidate_keys = Some(intersect_candidate_keys(candidate_keys, keys));
        }

        let query_tokens = tokenize(&options.query);
        if !query_tokens.is_empty() {
            let mut token_keys = BTreeSet::new();
            for token in &query_tokens {
                if let Some(keys) = self.token_index.get(token) {
                    token_keys.extend(keys.iter().cloned());
                }
            }
            candidate_keys = Some(intersect_candidate_keys(candidate_keys, token_keys));
        }

        let keys: Vec<String> = match candidate_keys {
            Some(keys) => keys.into_iter().collect(),
            None => self.commands.keys().cloned().collect(),
        };

        let mut hits = Vec::new();

        for key in keys {
            let Some(stored) = self.commands.get(&key) else {
                continue;
            };

            if let Some(source_filter) = &options.source_name {
                if &stored.source_name != source_filter {
                    continue;
                }
            }

            if let Some(kind) = &options.kind {
                if &stored.command.kind != kind {
                    continue;
                }
            }

            if let Some(tag) = &options.tag {
                let tag = tag.to_ascii_lowercase();
                if !stored
                    .command
                    .tags
                    .iter()
                    .any(|t| t.to_ascii_lowercase() == tag)
                {
                    continue;
                }
            }

            if let Some(reference) = &options.reference {
                let reference = reference.to_ascii_lowercase();
                if !stored
                    .command
                    .references
                    .iter()
                    .any(|r| r.to_ascii_lowercase() == reference)
                {
                    continue;
                }
            }

            let (score, matched_fields) = score_command(&stored.command, &query_tokens);

            // Empty query means filter-only search; keep result with baseline score.
            if query_tokens.is_empty() || score > 0 {
                hits.push(ParseSearchHit {
                    source_name: stored.source_name.clone(),
                    command_id: stored.command.id.clone(),
                    db_key: stored.db_key.clone(),
                    kind: stored.command.kind.clone(),
                    title: stored.command.title.clone(),
                    content_preview: preview(&stored.command.content),
                    score: score.max(1),
                    matched_fields,
                });
            }
        }

        hits.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.source_name.cmp(&b.source_name))
                .then_with(|| a.command_id.cmp(&b.command_id))
        });

        if let Some(limit) = options.limit {
            hits.truncate(limit);
        }

        hits
    }

    pub fn rebuild_indexes(&mut self) {
        let documents: Vec<ParseOutput> = self.documents.values().cloned().collect();

        self.commands.clear();
        self.by_kind.clear();
        self.by_tag.clear();
        self.by_reference.clear();
        self.token_index.clear();

        for output in documents {
            for command in output.commands {
                self.insert_command_indexes(&output.source_name, command);
            }
        }
    }

    fn insert_command_indexes(&mut self, source_name: &str, command: ParsedCommand) {
        let db_key = make_db_key(source_name, &command.id);
        let stored = StoredParsedCommand {
            db_key: db_key.clone(),
            source_name: source_name.to_string(),
            command: command.clone(),
        };

        self.by_kind
            .entry(kind_key(&command.kind))
            .or_default()
            .insert(db_key.clone());

        for tag in &command.tags {
            self.by_tag
                .entry(tag.to_ascii_lowercase())
                .or_default()
                .insert(db_key.clone());
        }

        for reference in &command.references {
            self.by_reference
                .entry(reference.to_ascii_lowercase())
                .or_default()
                .insert(db_key.clone());
        }

        for token in command_tokens(&command) {
            self.token_index
                .entry(token)
                .or_default()
                .insert(db_key.clone());
        }

        self.commands.insert(db_key, stored);
    }
}

fn command_view(stored: &StoredParsedCommand) -> ParseCommandView {
    ParseCommandView {
        db_key: stored.db_key.clone(),
        source_name: stored.source_name.clone(),
        command_id: stored.command.id.clone(),
        kind: stored.command.kind.clone(),
        raw_identity: stored.command.raw_identity.clone(),
        title: stored.command.title.clone(),
        description: stored.command.description.clone(),
        parameters: stored.command.parameters.clone(),
        tags: stored.command.tags.clone(),
        references: stored.command.references.clone(),
        statuses: stored.command.statuses.clone(),
        content_preview: preview(&stored.command.content),
        source_trace: stored.command.source_trace.clone(),
    }
}

fn make_db_key(source_name: &str, command_id: &str) -> String {
    format!("{source_name}::{command_id}")
}

fn kind_key(kind: &CommandKind) -> String {
    match kind {
        CommandKind::Unknown(value) => format!("Unknown({value})"),
        CommandKind::Inferred(value) => format!("Inferred({value})"),
        other => format!("{other:?}"),
    }
}

fn command_tokens(command: &ParsedCommand) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();

    for text in [
        format!("{:?}", command.kind),
        command.raw_identity.clone(),
        command.title.clone().unwrap_or_default(),
        command.description.clone().unwrap_or_default(),
        command.content.clone(),
        command.parameters.join(" "),
        command.tags.join(" "),
        command.references.join(" "),
        command.statuses.join(" "),
        command.source_trace.clone(),
    ] {
        tokens.extend(tokenize(&text));
    }

    for (key, value) in &command.members {
        tokens.extend(tokenize(key));
        tokens.extend(tokenize(&value.to_string()));
    }

    tokens
}

fn score_command(command: &ParsedCommand, query_tokens: &[String]) -> (usize, Vec<String>) {
    if query_tokens.is_empty() {
        return (1, vec!["filter".to_string()]);
    }

    let mut score = 0;
    let mut matched_fields = Vec::new();

    let kind = format!("{:?}", command.kind).to_ascii_lowercase();
    let raw_identity = command.raw_identity.to_ascii_lowercase();
    let title = command.title.as_deref().unwrap_or_default().to_ascii_lowercase();
    let description = command
        .description
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let content = command.content.to_ascii_lowercase();
    let tags = command.tags.join(" ").to_ascii_lowercase();
    let references = command.references.join(" ").to_ascii_lowercase();
    let parameters = command.parameters.join(" ").to_ascii_lowercase();
    let statuses = command.statuses.join(" ").to_ascii_lowercase();

    for token in query_tokens {
        if kind.contains(token) {
            score += 10;
            push_once(&mut matched_fields, "kind");
        }
        if raw_identity.contains(token) {
            score += 8;
            push_once(&mut matched_fields, "raw_identity");
        }
        if title.contains(token) {
            score += 7;
            push_once(&mut matched_fields, "title");
        }
        if tags.contains(token) {
            score += 6;
            push_once(&mut matched_fields, "tags");
        }
        if references.contains(token) {
            score += 6;
            push_once(&mut matched_fields, "references");
        }
        if parameters.contains(token) {
            score += 5;
            push_once(&mut matched_fields, "parameters");
        }
        if statuses.contains(token) {
            score += 4;
            push_once(&mut matched_fields, "statuses");
        }
        if description.contains(token) {
            score += 3;
            push_once(&mut matched_fields, "description");
        }
        if content.contains(token) {
            score += 2;
            push_once(&mut matched_fields, "content");
        }
    }

    (score, matched_fields)
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-' && c != '/')
        .map(|part| part.trim().to_ascii_lowercase())
        .filter(|part| part.len() >= 2)
        .collect()
}

fn preview(content: &str) -> String {
    let preview = content
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .chars()
        .take(140)
        .collect::<String>();

    preview
}

fn push_once(items: &mut Vec<String>, value: &str) {
    if !items.iter().any(|item| item == value) {
        items.push(value.to_string());
    }
}

fn intersect_candidate_keys(
    current: Option<BTreeSet<String>>,
    next: BTreeSet<String>,
) -> BTreeSet<String> {
    match current {
        Some(current) => current.intersection(&next).cloned().collect(),
        None => next,
    }
}