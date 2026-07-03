pub const HISTORY_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS history_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp_unix_ms INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    source TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    target_value TEXT NOT NULL,
    context_id TEXT,
    workspace_id TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}'
);
CREATE TABLE IF NOT EXISTS history_targets (
    target_kind TEXT NOT NULL,
    target_value TEXT NOT NULL,
    use_count INTEGER NOT NULL DEFAULT 0,
    last_seen_unix_ms INTEGER NOT NULL,
    PRIMARY KEY(target_kind, target_value)
);
CREATE TABLE IF NOT EXISTS history_context_links (
    event_id INTEGER NOT NULL,
    context_id TEXT NOT NULL,
    PRIMARY KEY(event_id, context_id)
);
CREATE TABLE IF NOT EXISTS history_metadata (
    event_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value_json TEXT NOT NULL,
    PRIMARY KEY(event_id, key)
);
"#;

pub const HISTORY_REQUIRED_TABLES: &[&str] = &[
    "history_events",
    "history_targets",
    "history_context_links",
    "history_metadata",
];

use crate::database::migrations::{apply_migration_batch, Migration};

pub fn history_migrations() -> Vec<Migration> {
    vec![Migration {
        name: "history_schema_v1",
        sql: HISTORY_SCHEMA_SQL,
    }]
}

pub fn create_history_schema(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    for migration in history_migrations() {
        apply_migration_batch(conn, migration.sql)?;
    }
    Ok(())
}
