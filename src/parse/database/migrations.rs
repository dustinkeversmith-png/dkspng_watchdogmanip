pub const PARSE_COMMAND_DROP_SQL: &str = r#"
DROP TABLE IF EXISTS command_fts;
DROP TABLE IF EXISTS command_statuses;
DROP TABLE IF EXISTS command_references;
DROP TABLE IF EXISTS command_tags;
DROP TABLE IF EXISTS command_parameters;
DROP TABLE IF EXISTS parsed_commands;
DROP TABLE IF EXISTS sources;
"#;

pub const PARSE_COMMAND_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS parsed_commands (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL,
    command_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    kind_json TEXT NOT NULL,
    raw_identity TEXT NOT NULL,
    title TEXT,
    description TEXT,
    content TEXT NOT NULL,
    members_json TEXT NOT NULL,
    source_trace TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE,
    UNIQUE(source_id, command_id)
);

CREATE INDEX IF NOT EXISTS idx_parsed_commands_kind ON parsed_commands(kind);
CREATE INDEX IF NOT EXISTS idx_parsed_commands_source ON parsed_commands(source_id);

CREATE TABLE IF NOT EXISTS command_parameters (
    command_db_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    value TEXT NOT NULL,
    FOREIGN KEY(command_db_id) REFERENCES parsed_commands(id) ON DELETE CASCADE,
    PRIMARY KEY(command_db_id, position)
);

CREATE TABLE IF NOT EXISTS command_tags (
    command_db_id INTEGER NOT NULL,
    tag TEXT NOT NULL,
    FOREIGN KEY(command_db_id) REFERENCES parsed_commands(id) ON DELETE CASCADE,
    PRIMARY KEY(command_db_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_command_tags_tag ON command_tags(tag);

CREATE TABLE IF NOT EXISTS command_references (
    command_db_id INTEGER NOT NULL,
    reference TEXT NOT NULL,
    FOREIGN KEY(command_db_id) REFERENCES parsed_commands(id) ON DELETE CASCADE,
    PRIMARY KEY(command_db_id, reference)
);

CREATE INDEX IF NOT EXISTS idx_command_references_reference ON command_references(reference);

CREATE TABLE IF NOT EXISTS command_statuses (
    command_db_id INTEGER NOT NULL,
    status TEXT NOT NULL,
    FOREIGN KEY(command_db_id) REFERENCES parsed_commands(id) ON DELETE CASCADE,
    PRIMARY KEY(command_db_id, status)
);

CREATE VIRTUAL TABLE IF NOT EXISTS command_fts USING fts5(
    command_db_id UNINDEXED,
    source_name,
    command_id,
    kind,
    raw_identity,
    title,
    description,
    content,
    parameters,
    tags,
    refs_text,
    statuses,
    source_trace
);
"#;

pub const PARSE_COMMAND_REQUIRED_TABLES: &[&str] = &[
    "sources",
    "parsed_commands",
    "command_parameters",
    "command_tags",
    "command_references",
    "command_statuses",
    "command_fts",
];

use crate::database::migrations::{apply_migration_batch, Migration};

pub fn parse_command_migrations() -> Vec<Migration> {
    vec![Migration {
        name: "parse_command_schema_v1",
        sql: PARSE_COMMAND_SCHEMA_SQL,
    }]
}

pub fn create_parse_command_schema(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    for migration in parse_command_migrations() {
        apply_migration_batch(conn, migration.sql)?;
    }
    Ok(())
}

pub fn drop_parse_command_schema(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    apply_migration_batch(conn, PARSE_COMMAND_DROP_SQL)
}
