use crate::history::database::model::HistoryEventRecord;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

pub struct HistoryStore {
    conn: Connection,
}

impl HistoryStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path.as_ref())
            .with_context(|| format!("failed opening history db at {}", path.as_ref().display()))?;
        let store = Self { conn };
        store.create_schema()?;
        Ok(store)
    }

    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.create_schema()?;
        Ok(store)
    }

    pub fn create_schema(&self) -> Result<()> {
        crate::history::database::migrations::create_history_schema(&self.conn)
    }

    pub fn health_check(&self) -> Result<bool> {
        crate::database::migrations::tables_present(
            &self.conn,
            crate::history::database::migrations::HISTORY_REQUIRED_TABLES,
        )
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn insert_event(&self, record: &HistoryEventRecord) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO history_events(
                timestamp_unix_ms, event_type, source, target_kind, target_value,
                context_id, workspace_id, metadata_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                record.timestamp_unix_ms,
                record.event_type,
                record.source,
                record.target_kind,
                record.target_value,
                record.context_id,
                record.workspace_id,
                serde_json::to_string(&record.metadata)?,
            ],
        )?;

        let event_id = self.conn.last_insert_rowid();

        self.conn.execute(
            r#"
            INSERT INTO history_targets(target_kind, target_value, use_count, last_seen_unix_ms)
            VALUES (?1, ?2, 1, ?3)
            ON CONFLICT(target_kind, target_value) DO UPDATE SET
                use_count = use_count + 1,
                last_seen_unix_ms = excluded.last_seen_unix_ms
            "#,
            params![
                record.target_kind,
                record.target_value,
                record.timestamp_unix_ms,
            ],
        )?;

        if let Some(context_id) = &record.context_id {
            self.conn.execute(
                "INSERT OR IGNORE INTO history_context_links(event_id, context_id) VALUES (?1, ?2)",
                params![event_id, context_id],
            )?;
        }

        Ok(event_id)
    }

    pub fn recent_events(&self, limit: usize) -> Result<Vec<HistoryEventRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp_unix_ms, event_type, source, target_kind, target_value, context_id, workspace_id, metadata_json
             FROM history_events ORDER BY timestamp_unix_ms DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let metadata_json: String = row.get(8)?;
            Ok(HistoryEventRecord {
                id: Some(row.get(0)?),
                timestamp_unix_ms: row.get(1)?,
                event_type: row.get(2)?,
                source: row.get(3)?,
                target_kind: row.get(4)?,
                target_value: row.get(5)?,
                context_id: row.get(6)?,
                workspace_id: row.get(7)?,
                metadata: serde_json::from_str(&metadata_json).unwrap_or_default(),
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn frequent_targets(&self, limit: usize) -> Result<Vec<(String, String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT target_kind, target_value, use_count FROM history_targets ORDER BY use_count DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn events_by_context(
        &self,
        context_id: &str,
        limit: usize,
    ) -> Result<Vec<HistoryEventRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT e.id, e.timestamp_unix_ms, e.event_type, e.source, e.target_kind, e.target_value,
                   e.context_id, e.workspace_id, e.metadata_json
            FROM history_events e
            JOIN history_context_links l ON l.event_id = e.id
            WHERE l.context_id = ?1
            ORDER BY e.timestamp_unix_ms DESC
            LIMIT ?2
            "#,
        )?;
        let rows = stmt.query_map(params![context_id, limit as i64], |row| {
            let metadata_json: String = row.get(8)?;
            Ok(HistoryEventRecord {
                id: Some(row.get(0)?),
                timestamp_unix_ms: row.get(1)?,
                event_type: row.get(2)?,
                source: row.get(3)?,
                target_kind: row.get(4)?,
                target_value: row.get(5)?,
                context_id: row.get(6)?,
                workspace_id: row.get(7)?,
                metadata: serde_json::from_str(&metadata_json).unwrap_or_default(),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}
