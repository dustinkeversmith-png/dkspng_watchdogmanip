pub const CONTEXT_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS contexts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    root_path TEXT NOT NULL,
    parent_id TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}'
);
CREATE TABLE IF NOT EXISTS context_edges (
    parent_id TEXT NOT NULL,
    child_id TEXT NOT NULL,
    PRIMARY KEY(parent_id, child_id)
);
CREATE TABLE IF NOT EXISTS context_files (
    context_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    PRIMARY KEY(context_id, file_path)
);
CREATE TABLE IF NOT EXISTS context_commands (
    context_id TEXT NOT NULL,
    command_id TEXT NOT NULL,
    PRIMARY KEY(context_id, command_id)
);
CREATE TABLE IF NOT EXISTS context_aliases (
    context_id TEXT NOT NULL,
    alias_name TEXT NOT NULL,
    target_json TEXT NOT NULL,
    PRIMARY KEY(context_id, alias_name)
);
CREATE TABLE IF NOT EXISTS context_currents (
    context_id TEXT NOT NULL,
    title TEXT NOT NULL,
    details TEXT,
    PRIMARY KEY(context_id, title)
);
"#;

pub const CONTEXT_REQUIRED_TABLES: &[&str] = &[
    "contexts",
    "context_edges",
    "context_files",
    "context_commands",
    "context_aliases",
    "context_currents",
];

use crate::database::migrations::{apply_migration_batch, Migration};

pub fn context_migrations() -> Vec<Migration> {
    vec![Migration {
        name: "context_schema_v1",
        sql: CONTEXT_SCHEMA_SQL,
    }]
}

pub fn create_context_schema(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    for migration in context_migrations() {
        apply_migration_batch(conn, migration.sql)?;
    }
    Ok(())
}
