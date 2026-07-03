use crate::history::model::*;
use chrono::Utc;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub trait HistoryAdapter {
    fn name(&self) -> &'static str;
    fn collect(&mut self) -> anyhow::Result<Vec<HistoryEvent>>;
}

#[derive(Debug, Default)]
pub struct MockHistoryAdapter;

impl HistoryAdapter for MockHistoryAdapter {
    fn name(&self) -> &'static str {
        "mock-history"
    }

    fn collect(&mut self) -> anyhow::Result<Vec<HistoryEvent>> {
        let now = Utc::now();
        let mut events = Vec::new();
        let rows = vec![
            (
                "evt_001",
                HistoryEventType::FileOpened,
                HistoryTarget::File {
                    path: PathBuf::from("./src/parser/mod.rs"),
                    line: Some(12),
                    column: None,
                },
                Some("parser"),
            ),
            (
                "evt_002",
                HistoryEventType::FileOpened,
                HistoryTarget::File {
                    path: PathBuf::from("./src/parser/mod.rs"),
                    line: Some(40),
                    column: None,
                },
                Some("parser"),
            ),
            (
                "evt_003",
                HistoryEventType::CommandExecuted,
                HistoryTarget::Command {
                    shell: "bash".into(),
                    command: "cargo test parser".into(),
                    cwd: Some(PathBuf::from(".")),
                    exit_code: Some(0),
                },
                Some("parser"),
            ),
            (
                "evt_004",
                HistoryEventType::FolderOpened,
                HistoryTarget::Folder {
                    path: PathBuf::from("./docs"),
                },
                Some("docs"),
            ),
            (
                "evt_005",
                HistoryEventType::SearchPerformed,
                HistoryTarget::Search {
                    query: "BoundarySolver".into(),
                    scope: Some("project".into()),
                },
                Some("parser"),
            ),
        ];
        for (i, (id, event_type, target, ctx)) in rows.into_iter().enumerate() {
            events.push(HistoryEvent {
                id: id.into(),
                timestamp: now - chrono::Duration::minutes((10 - i) as i64),
                source: EventSource::Mock,
                actor: Actor::Adapter("mock".into()),
                event_type,
                target,
                context_id: ctx.map(str::to_string),
                workspace_id: Some("macro_processor".into()),
                metadata: BTreeMap::new(),
            });
        }
        Ok(events)
    }
}
