use crate::history::stats::{FrequencyIndex, UsageStats};
use serde::{Deserialize, Serialize};

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
    let mut scored: Vec<ScoredTarget> = index.stats.values().filter_map(|stats| score_one(stats, query)).collect();
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(query.limit);
    scored
}

fn score_one(stats: &UsageStats, query: &SuggestionQuery) -> Option<ScoredTarget> {
    if let Some(text) = &query.text {
        let hay = format!("{} {}", stats.target_key.to_lowercase(), stats.label.to_lowercase());
        if !hay.contains(&text.to_lowercase()) { return None; }
    }
    let mut score = stats.total_count as f64 + (stats.recent_count as f64 * 2.0);
    let mut reason = vec![format!("{} total uses", stats.total_count)];
    if stats.recent_count > 0 { reason.push(format!("{} recent uses", stats.recent_count)); }
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
    Some(ScoredTarget { target_key: stats.target_key.clone(), label: stats.label.clone(), score, reason })
}
