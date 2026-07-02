use anyhow::{anyhow, Result};
use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct DatabaseHealthReport {
    pub online: bool,
    pub required_tables_present: bool,
    pub missing_tables: Vec<String>,
}

pub const REQUIRED_TABLES: &[&str] = &[
    "sources",
    "parsed_commands",
    "command_parameters",
    "command_tags",
    "command_references",
    "command_statuses",
    "command_fts",
];

pub fn check_sqlite_health(conn: &Connection) -> Result<DatabaseHealthReport> {
    let one: i64 = conn.query_row("SELECT 1", [], |row| row.get(0))?;

    if one != 1 {
        return Err(anyhow!("database health check failed: SELECT 1 did not return 1"));
    }

    let mut missing_tables = Vec::new();

    for table in REQUIRED_TABLES {
        let exists: i64 = conn.query_row(
            r#"
            SELECT COUNT(*)
            FROM sqlite_master
            WHERE name = ?1
              AND type IN ('table', 'virtual table')
            "#,
            [table],
            |row| row.get(0),
        )?;

        if exists == 0 {
            missing_tables.push((*table).to_string());
        }
    }

    Ok(DatabaseHealthReport {
        online: true,
        required_tables_present: missing_tables.is_empty(),
        missing_tables,
    })
}