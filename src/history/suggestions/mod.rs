use crate::history::database::HistoryEventRecord;
use crate::history::stats::{FrequencyIndex, UsageStats};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SuggestionRequest {
    pub query: Option<String>,
    pub context_id: Option<String>,
    pub workspace_id: Option<String>,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SuggestionResult {
    pub target_kind: String,
    pub target_value: String,
    pub score: f64,
    pub reasons: Vec<String>,
}

pub fn suggest_from_index(
    index: &FrequencyIndex,
    request: &SuggestionRequest,
) -> Vec<SuggestionResult> {
    let legacy_query = SuggestionQuery {
        text: request.query.clone(),
        context_id: request.context_id.clone(),
        workspace_id: request.workspace_id.clone(),
        limit: request.limit,
    };
    suggest(index, &legacy_query)
        .into_iter()
        .map(|hit| SuggestionResult {
            target_kind: "legacy".to_string(),
            target_value: hit.label,
            score: hit.score,
            reasons: hit.reason,
        })
        .collect()
}

pub fn suggest_from_events(
    events: &[HistoryEventRecord],
    request: &SuggestionRequest,
) -> Vec<SuggestionResult> {
    let mut scores: std::collections::BTreeMap<(String, String), (f64, Vec<String>)> =
        std::collections::BTreeMap::new();

    for event in events {
        if let Some(query) = &request.query {
            let hay = format!("{} {}", event.target_kind, event.target_value).to_ascii_lowercase();
            if !hay.contains(&query.to_ascii_lowercase()) {
                continue;
            }
        }
        if let Some(context_id) = &request.context_id {
            if event.context_id.as_deref() != Some(context_id.as_str()) {
                continue;
            }
        }

        let key = (event.target_kind.clone(), event.target_value.clone());
        let entry = scores.entry(key).or_insert((0.0, Vec::new()));
        entry.0 += 1.0;
        entry.1.push(format!("event_type={}", event.event_type));
    }

    let mut results: Vec<SuggestionResult> = scores
        .into_iter()
        .map(
            |((target_kind, target_value), (score, reasons))| SuggestionResult {
                target_kind,
                target_value,
                score,
                reasons,
            },
        )
        .collect();

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(request.limit);
    results
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SuggestionQuery {
    pub text: Option<String>,
    pub context_id: Option<String>,
    pub workspace_id: Option<String>,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoredTarget {
    pub target_key: String,
    pub label: String,
    pub score: f64,
    pub reason: Vec<String>,
}

pub fn suggest(index: &FrequencyIndex, query: &SuggestionQuery) -> Vec<ScoredTarget> {
    let mut scored: Vec<ScoredTarget> = index
        .stats
        .values()
        .filter_map(|stats| score_one(stats, query))
        .collect();
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(query.limit);
    scored
}

fn score_one(stats: &UsageStats, query: &SuggestionQuery) -> Option<ScoredTarget> {
    if let Some(text) = &query.text {
        let hay = format!(
            "{} {}",
            stats.target_key.to_lowercase(),
            stats.label.to_lowercase()
        );
        if !hay.contains(&text.to_lowercase()) {
            return None;
        }
    }
    let mut score = stats.total_count as f64 + (stats.recent_count as f64 * 2.0);
    let mut reason = vec![format!("{} total uses", stats.total_count)];
    if stats.recent_count > 0 {
        reason.push(format!("{} recent uses", stats.recent_count));
    }
    if let Some(ctx) = &query.context_id {
        if let Some(count) = stats.context_counts.get(ctx) {
            score += (*count as f64) * 3.0;
            reason.push(format!("{} uses in context {}", count, ctx));
        }
    }
    if let Some(ws) = &query.workspace_id {
        if let Some(count) = stats.workspace_counts.get(ws) {
            score += (*count as f64) * 2.5;
            reason.push(format!("{} uses in workspace {}", count, ws));
        }
    }
    Some(ScoredTarget {
        target_key: stats.target_key.clone(),
        label: stats.label.clone(),
        score,
        reason,
    })
}
