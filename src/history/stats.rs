use crate::history::model::{ContextId, HistoryEvent, HistoryEventType, WorkspaceId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageStats {
    pub target_key: String,
    pub label: String,
    pub event_type_counts: BTreeMap<HistoryEventType, u64>,
    pub total_count: u64,
    pub recent_count: u64,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub context_counts: BTreeMap<ContextId, u64>,
    pub workspace_counts: BTreeMap<WorkspaceId, u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrequencyIndex {
    pub stats: BTreeMap<String, UsageStats>,
}

impl FrequencyIndex {
    pub fn build(events: &[HistoryEvent], recent_window_days: i64) -> Self {
        let mut stats = BTreeMap::<String, UsageStats>::new();
        let now = events
            .iter()
            .map(|e| e.timestamp)
            .max()
            .unwrap_or_else(Utc::now);
        let recent_cutoff = now - chrono::Duration::days(recent_window_days);

        for event in events {
            let key = event.target.stable_key();
            let entry = stats.entry(key.clone()).or_insert_with(|| UsageStats {
                target_key: key,
                label: event.target.label(),
                event_type_counts: BTreeMap::new(),
                total_count: 0,
                recent_count: 0,
                first_seen_at: event.timestamp,
                last_seen_at: event.timestamp,
                context_counts: BTreeMap::new(),
                workspace_counts: BTreeMap::new(),
            });
            entry.total_count += 1;
            if event.timestamp >= recent_cutoff {
                entry.recent_count += 1;
            }
            *entry
                .event_type_counts
                .entry(event.event_type.clone())
                .or_insert(0) += 1;
            if event.timestamp < entry.first_seen_at {
                entry.first_seen_at = event.timestamp;
            }
            if event.timestamp > entry.last_seen_at {
                entry.last_seen_at = event.timestamp;
            }
            if let Some(ctx) = &event.context_id {
                *entry.context_counts.entry(ctx.clone()).or_insert(0) += 1;
            }
            if let Some(ws) = &event.workspace_id {
                *entry.workspace_counts.entry(ws.clone()).or_insert(0) += 1;
            }
        }

        Self { stats }
    }

    pub fn top(&self, limit: usize) -> Vec<&UsageStats> {
        let mut values: Vec<_> = self.stats.values().collect();
        values.sort_by(|a, b| {
            b.total_count
                .cmp(&a.total_count)
                .then_with(|| b.last_seen_at.cmp(&a.last_seen_at))
        });
        values.into_iter().take(limit).collect()
    }
}
