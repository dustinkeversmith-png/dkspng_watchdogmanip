use crate::database::health::{check_sqlite_health, DatabaseHealthReport};
use crate::database::model::{
    CommandSearchHit, CommandSearchOptions, DatabaseStats, NewParsedCommandRecord,
    StoredParsedCommandRecord,
};
use crate::parse::model::CommandKind;
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

use crate::database::model::DatabaseTableDump;
use serde_json::Value;

pub struct CommandSqliteDatabase {
    conn: Connection,
}

impl CommandSqliteDatabase {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path.as_ref())
            .with_context(|| format!("failed to open sqlite db at {}", path.as_ref().display()))?;

        let db = Self { conn };
        db.recreate_schema()?;
        Ok(db)
    }

    pub fn health_check(&self) -> anyhow::Result<DatabaseHealthReport> {
        check_sqlite_health(&self.conn)
    }

    pub fn is_online(&self) -> bool {
        self.health_check()
            .map(|report| report.online && report.required_tables_present)
            .unwrap_or(false)
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn open_existing(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path.as_ref())
            .with_context(|| format!("failed to open sqlite db at {}", path.as_ref().display()))?;

        let db = Self { conn };
        db.create_schema()?;
        Ok(db)
    }

    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.create_schema()?;
        Ok(db)
    }

    pub fn recreate_schema(&self) -> Result<()> {
        self.drop_schema()?;
        self.create_schema()?;
        Ok(())
    }

    pub fn drop_schema(&self) -> Result<()> {
        crate::parse::database::migrations::drop_parse_command_schema(&self.conn)
    }

    pub fn create_schema(&self) -> Result<()> {
        crate::parse::database::migrations::create_parse_command_schema(&self.conn)
    }

    pub fn create_sources_table(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_name TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )?;
        Ok(())
    }

    pub fn create_commands_table(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
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

            CREATE INDEX IF NOT EXISTS idx_parsed_commands_kind
                ON parsed_commands(kind);

            CREATE INDEX IF NOT EXISTS idx_parsed_commands_source
                ON parsed_commands(source_id);
            "#,
        )?;
        Ok(())
    }

    pub fn create_parameters_table(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS command_parameters (
                command_db_id INTEGER NOT NULL,
                position INTEGER NOT NULL,
                value TEXT NOT NULL,

                FOREIGN KEY(command_db_id) REFERENCES parsed_commands(id) ON DELETE CASCADE,
                PRIMARY KEY(command_db_id, position)
            );
            "#,
        )?;
        Ok(())
    }

    pub fn create_tags_table(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS command_tags (
                command_db_id INTEGER NOT NULL,
                tag TEXT NOT NULL,

                FOREIGN KEY(command_db_id) REFERENCES parsed_commands(id) ON DELETE CASCADE,
                PRIMARY KEY(command_db_id, tag)
            );

            CREATE INDEX IF NOT EXISTS idx_command_tags_tag
                ON command_tags(tag);
            "#,
        )?;
        Ok(())
    }

    pub fn create_references_table(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS command_references (
                command_db_id INTEGER NOT NULL,
                reference TEXT NOT NULL,

                FOREIGN KEY(command_db_id) REFERENCES parsed_commands(id) ON DELETE CASCADE,
                PRIMARY KEY(command_db_id, reference)
            );

            CREATE INDEX IF NOT EXISTS idx_command_references_reference
                ON command_references(reference);
            "#,
        )?;
        Ok(())
    }

    pub fn create_statuses_table(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS command_statuses (
                command_db_id INTEGER NOT NULL,
                status TEXT NOT NULL,

                FOREIGN KEY(command_db_id) REFERENCES parsed_commands(id) ON DELETE CASCADE,
                PRIMARY KEY(command_db_id, status)
            );
            "#,
        )?;
        Ok(())
    }

    pub fn create_fts_table(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
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
            "#,
        )?;
        Ok(())
    }

    pub fn insert_command(&self, command: &NewParsedCommandRecord) -> Result<i64> {
        let source_id = self.ensure_source(&command.source_name)?;

        self.conn.execute(
            r#"
            INSERT INTO parsed_commands(
                source_id,
                command_id,
                kind,
                kind_json,
                raw_identity,
                title,
                description,
                content,
                members_json,
                source_trace
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(source_id, command_id)
            DO UPDATE SET
                kind = excluded.kind,
                kind_json = excluded.kind_json,
                raw_identity = excluded.raw_identity,
                title = excluded.title,
                description = excluded.description,
                content = excluded.content,
                members_json = excluded.members_json,
                source_trace = excluded.source_trace
            "#,
            params![
                source_id,
                command.command_id,
                kind_to_key(&command.kind),
                serde_json::to_string(&command.kind)?,
                command.raw_identity,
                command.title,
                command.description,
                command.content,
                serde_json::to_string(&command.members)?,
                command.source_trace,
            ],
        )?;

        let command_db_id: i64 = self.conn.query_row(
            "SELECT id FROM parsed_commands WHERE source_id = ?1 AND command_id = ?2",
            params![source_id, command.command_id],
            |row| row.get(0),
        )?;

        self.replace_command_children(command_db_id, command)?;
        self.replace_fts(command_db_id, command)?;

        Ok(command_db_id)
    }

    pub fn get_command(&self, id: i64) -> Result<Option<StoredParsedCommandRecord>> {
        let command = self
            .conn
            .query_row(
                r#"
                SELECT
                    pc.id,
                    s.source_name,
                    pc.command_id,
                    pc.kind_json,
                    pc.raw_identity,
                    pc.title,
                    pc.description,
                    pc.content,
                    pc.members_json,
                    pc.source_trace
                FROM parsed_commands pc
                JOIN sources s ON s.id = pc.source_id
                WHERE pc.id = ?1
                "#,
                params![id],
                |row| {
                    let kind_json: String = row.get(3)?;
                    let members_json: String = row.get(8)?;

                    let kind: CommandKind =
                        serde_json::from_str(&kind_json).map_err(to_sql_json_err)?;
                    let members = serde_json::from_str(&members_json).map_err(to_sql_json_err)?;

                    Ok(StoredParsedCommandRecord {
                        id: row.get(0)?,
                        source_name: row.get(1)?,
                        command_id: row.get(2)?,
                        kind,
                        raw_identity: row.get(4)?,
                        title: row.get(5)?,
                        description: row.get(6)?,
                        content: row.get(7)?,
                        members,
                        parameters: Vec::new(),
                        tags: Vec::new(),
                        references: Vec::new(),
                        statuses: Vec::new(),
                        source_trace: row.get(9)?,
                    })
                },
            )
            .optional()?;

        let Some(mut command) = command else {
            return Ok(None);
        };

        command.parameters = self.load_strings(
            "SELECT value FROM command_parameters WHERE command_db_id = ?1 ORDER BY position",
            id,
        )?;

        command.tags = self.load_strings(
            "SELECT tag FROM command_tags WHERE command_db_id = ?1 ORDER BY tag",
            id,
        )?;

        command.references = self.load_strings(
            "SELECT reference FROM command_references WHERE command_db_id = ?1 ORDER BY reference",
            id,
        )?;

        command.statuses = self.load_strings(
            "SELECT status FROM command_statuses WHERE command_db_id = ?1 ORDER BY status",
            id,
        )?;

        Ok(Some(command))
    }

    pub fn search(&self, options: CommandSearchOptions) -> Result<Vec<CommandSearchHit>> {
        let limit = options.limit.unwrap_or(50) as i64;

        let mut sql = String::from(
            r#"
            SELECT
                pc.id,
                s.source_name,
                pc.command_id,
                pc.kind_json,
                pc.raw_identity,
                pc.title,
                pc.content,
                1 AS score
            FROM parsed_commands pc
            JOIN sources s ON s.id = pc.source_id
            WHERE 1 = 1
            "#,
        );

        if let Some(source_name) = &options.source_name {
            sql.push_str(&format!(
                " AND s.source_name = '{}'",
                escape_sql_literal(source_name)
            ));
        }

        if let Some(kind) = &options.kind {
            sql.push_str(&format!(
                " AND pc.kind = '{}'",
                escape_sql_literal(&kind_to_key(kind))
            ));
        }

        if let Some(tag) = &options.tag {
            sql.push_str(&format!(
                " AND pc.id IN (SELECT command_db_id FROM command_tags WHERE tag = '{}')",
                escape_sql_literal(&tag.to_ascii_lowercase())
            ));
        }

        if let Some(reference) = &options.reference {
            sql.push_str(&format!(
                " AND pc.id IN (SELECT command_db_id FROM command_references WHERE reference = '{}')",
                escape_sql_literal(&reference.to_ascii_lowercase())
            ));
        }

        if let Some(query) = &options.query {
            if !query.trim().is_empty() {
                sql.push_str(&format!(
                    " AND pc.id IN (SELECT command_db_id FROM command_fts WHERE command_fts MATCH '{}')",
                    escape_fts_query(query)
                ));
            }
        }

        sql.push_str(" ORDER BY pc.id ASC LIMIT ");
        sql.push_str(&limit.to_string());

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            let kind_json: String = row.get(3)?;
            let kind: CommandKind = serde_json::from_str(&kind_json).map_err(to_sql_json_err)?;
            let content: String = row.get(6)?;

            Ok(CommandSearchHit {
                id: row.get(0)?,
                source_name: row.get(1)?,
                command_id: row.get(2)?,
                kind,
                raw_identity: row.get(4)?,
                title: row.get(5)?,
                content_preview: preview(&content),
                score: row.get(7)?,
            })
        })?;

        let mut hits = Vec::new();
        for row in rows {
            hits.push(row?);
        }

        Ok(hits)
    }

    pub fn stats(&self) -> Result<DatabaseStats> {
        Ok(DatabaseStats {
            source_count: self
                .conn
                .query_row("SELECT COUNT(*) FROM sources", [], |row| row.get(0))?,
            command_count: self.conn.query_row(
                "SELECT COUNT(*) FROM parsed_commands",
                [],
                |row| row.get(0),
            )?,
            tag_count: self.conn.query_row(
                "SELECT COUNT(DISTINCT tag) FROM command_tags",
                [],
                |row| row.get(0),
            )?,
            reference_count: self.conn.query_row(
                "SELECT COUNT(DISTINCT reference) FROM command_references",
                [],
                |row| row.get(0),
            )?,
        })
    }

    fn ensure_source(&self, source_name: &str) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO sources(source_name, updated_at)
            VALUES (?1, CURRENT_TIMESTAMP)
            ON CONFLICT(source_name)
            DO UPDATE SET updated_at = CURRENT_TIMESTAMP
            "#,
            params![source_name],
        )?;

        let source_id = self.conn.query_row(
            "SELECT id FROM sources WHERE source_name = ?1",
            params![source_name],
            |row| row.get(0),
        )?;

        Ok(source_id)
    }

    fn replace_command_children(
        &self,
        command_db_id: i64,
        command: &NewParsedCommandRecord,
    ) -> Result<()> {
        self.conn.execute(
            "DELETE FROM command_parameters WHERE command_db_id = ?1",
            params![command_db_id],
        )?;
        self.conn.execute(
            "DELETE FROM command_tags WHERE command_db_id = ?1",
            params![command_db_id],
        )?;
        self.conn.execute(
            "DELETE FROM command_references WHERE command_db_id = ?1",
            params![command_db_id],
        )?;
        self.conn.execute(
            "DELETE FROM command_statuses WHERE command_db_id = ?1",
            params![command_db_id],
        )?;

        for (index, value) in command.parameters.iter().enumerate() {
            self.conn.execute(
                "INSERT INTO command_parameters(command_db_id, position, value) VALUES (?1, ?2, ?3)",
                params![command_db_id, index as i64, value],
            )?;
        }

        for tag in &command.tags {
            self.conn.execute(
                "INSERT INTO command_tags(command_db_id, tag) VALUES (?1, ?2)",
                params![command_db_id, tag.to_ascii_lowercase()],
            )?;
        }

        for reference in &command.references {
            self.conn.execute(
                "INSERT INTO command_references(command_db_id, reference) VALUES (?1, ?2)",
                params![command_db_id, reference.to_ascii_lowercase()],
            )?;
        }

        for status in &command.statuses {
            self.conn.execute(
                "INSERT INTO command_statuses(command_db_id, status) VALUES (?1, ?2)",
                params![command_db_id, status.to_ascii_lowercase()],
            )?;
        }

        Ok(())
    }

    fn replace_fts(&self, command_db_id: i64, command: &NewParsedCommandRecord) -> Result<()> {
        self.conn.execute(
            "DELETE FROM command_fts WHERE command_db_id = ?1",
            params![command_db_id],
        )?;

        self.conn.execute(
            r#"
            INSERT INTO command_fts(
                command_db_id,
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
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
            params![
                command_db_id,
                command.source_name,
                command.command_id,
                kind_to_key(&command.kind),
                command.raw_identity,
                command.title.clone().unwrap_or_default(),
                command.description.clone().unwrap_or_default(),
                command.content,
                command.parameters.join(" "),
                command.tags.join(" "),
                command.references.join(" "),
                command.statuses.join(" "),
                command.source_trace,
            ],
        )?;

        Ok(())
    }

    fn load_strings(&self, sql: &str, command_id: i64) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![command_id], |row| row.get::<_, String>(0))?;

        let mut values = Vec::new();
        for row in rows {
            values.push(row?);
        }

        Ok(values)
    }

    pub fn dump_table(&self, table_name: &str, limit: usize) -> anyhow::Result<DatabaseTableDump> {
        let allowed_tables = [
            "sources",
            "parsed_commands",
            "command_parameters",
            "command_tags",
            "command_references",
            "command_statuses",
        ];

        if !allowed_tables.contains(&table_name) {
            anyhow::bail!("table dump refused for unknown or unsafe table: {table_name}");
        }

        let count_sql = format!("SELECT COUNT(*) FROM {table_name}");
        let row_count: usize = self.conn.query_row(&count_sql, [], |row| row.get(0))?;

        let sql = format!("SELECT * FROM {table_name} LIMIT {limit}");
        let mut stmt = self.conn.prepare(&sql)?;

        let column_names = stmt
            .column_names()
            .iter()
            .map(|name| name.to_string())
            .collect::<Vec<_>>();

        let rows = stmt.query_map([], |row| {
            let mut object = serde_json::Map::new();

            for (index, column_name) in column_names.iter().enumerate() {
                let value = sqlite_value_to_json(row, index)?;
                object.insert(column_name.clone(), value);
            }

            Ok(Value::Object(object))
        })?;

        let mut out = Vec::new();

        for row in rows {
            out.push(row?);
        }

        Ok(DatabaseTableDump {
            table_name: table_name.to_string(),
            row_count,
            rows: out,
        })
    }

    pub fn dump_core_tables(
        &self,
        limit_per_table: usize,
    ) -> anyhow::Result<Vec<DatabaseTableDump>> {
        let tables = [
            "sources",
            "parsed_commands",
            "command_parameters",
            "command_tags",
            "command_references",
            "command_statuses",
        ];

        tables
            .iter()
            .map(|table| self.dump_table(table, limit_per_table))
            .collect()
    }
}

pub fn new_record_from_parsed_command(
    source_name: impl Into<String>,
    command: crate::parse::model::ParsedCommand,
) -> NewParsedCommandRecord {
    NewParsedCommandRecord {
        source_name: source_name.into(),
        command_id: command.id,
        kind: command.kind,
        raw_identity: command.raw_identity,
        title: command.title,
        description: command.description,
        content: command.content,
        members: command.members,
        parameters: command.parameters,
        tags: command.tags,
        references: command.references,
        statuses: command.statuses,
        source_trace: command.source_trace,
    }
}

fn kind_to_key(kind: &CommandKind) -> String {
    match kind {
        CommandKind::Unknown(value) => format!("Unknown({value})"),
        CommandKind::Inferred(value) => format!("Inferred({value})"),
        other => format!("{other:?}"),
    }
}

fn preview(content: &str) -> String {
    content
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .chars()
        .take(140)
        .collect()
}

fn escape_sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

fn escape_fts_query(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| part.replace('"', ""))
        .collect::<Vec<_>>()
        .join(" ")
}

fn to_sql_json_err(err: serde_json::Error) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
}

fn sqlite_value_to_json(
    row: &rusqlite::Row<'_>,
    index: usize,
) -> rusqlite::Result<serde_json::Value> {
    use rusqlite::types::ValueRef;

    match row.get_ref(index)? {
        ValueRef::Null => Ok(serde_json::Value::Null),
        ValueRef::Integer(value) => Ok(serde_json::json!(value)),
        ValueRef::Real(value) => Ok(serde_json::json!(value)),
        ValueRef::Text(value) => {
            let text = String::from_utf8_lossy(value).to_string();

            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&text) {
                Ok(json_value)
            } else {
                Ok(serde_json::json!(text))
            }
        }
        ValueRef::Blob(value) => Ok(serde_json::json!({
            "blob_len": value.len()
        })),
    }
}
