use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEventRecord {
    pub id: Option<i64>,
    pub timestamp_unix_ms: i64,
    pub event_type: String,
    pub source: String,
    pub target_kind: String,
    pub target_value: String,
    pub context_id: Option<String>,
    pub workspace_id: Option<String>,
    pub metadata: BTreeMap<String, serde_json::Value>,
}
