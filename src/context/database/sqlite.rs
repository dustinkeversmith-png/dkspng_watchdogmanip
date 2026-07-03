use crate::context::database::model::{ContextCommandRecord, ContextFileRecord, ContextRecord};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

pub struct ContextStore {
    conn: Connection,
}

impl ContextStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path.as_ref())
            .with_context(|| format!("failed opening context db at {}", path.as_ref().display()))?;
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
        crate::context::database::migrations::create_context_schema(&self.conn)
    }

    pub fn health_check(&self) -> Result<bool> {
        crate::database::migrations::tables_present(
            &self.conn,
            crate::context::database::migrations::CONTEXT_REQUIRED_TABLES,
        )
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn insert_context(&self, record: &ContextRecord) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO contexts(id, name, root_path, parent_id, metadata_json)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                root_path = excluded.root_path,
                parent_id = excluded.parent_id,
                metadata_json = excluded.metadata_json
            "#,
            params![
                record.id,
                record.name,
                record.root_path.display().to_string(),
                record.parent_id,
                serde_json::to_string(&record.metadata)?,
            ],
        )?;
        if let Some(parent_id) = &record.parent_id {
            self.conn.execute(
                "INSERT OR IGNORE INTO context_edges(parent_id, child_id) VALUES (?1, ?2)",
                params![parent_id, record.id],
            )?;
        }
        Ok(())
    }

    pub fn insert_file(&self, record: &ContextFileRecord) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO context_files(context_id, file_path) VALUES (?1, ?2)",
            params![record.context_id, record.file_path.display().to_string()],
        )?;
        Ok(())
    }

    pub fn insert_command(&self, record: &ContextCommandRecord) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO context_commands(context_id, command_id) VALUES (?1, ?2)",
            params![record.context_id, record.command_id],
        )?;
        Ok(())
    }

    pub fn context_count(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM contexts", [], |row| row.get(0))?)
    }
}
