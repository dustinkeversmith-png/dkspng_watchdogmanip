use anyhow::{Context, Result};
use rusqlite::Connection;

pub fn apply_migration_batch(conn: &Connection, sql: &str) -> Result<()> {
    conn.execute_batch(sql)
        .with_context(|| "failed applying migration batch".to_string())
}

pub fn missing_tables(conn: &Connection, required: &[&str]) -> Result<Vec<String>> {
    let mut missing = Vec::new();
    for table in required {
        let exists: i64 = conn.query_row(
            r#"
            SELECT COUNT(*)
            FROM sqlite_master
            WHERE name = ?1
              AND type IN ('table', 'virtual table')
            "#,
            [*table],
            |row| row.get(0),
        )?;
        if exists == 0 {
            missing.push((*table).to_string());
        }
    }
    Ok(missing)
}

pub fn tables_present(conn: &Connection, required: &[&str]) -> Result<bool> {
    Ok(missing_tables(conn, required)?.is_empty())
}

pub struct Migration {
    pub name: &'static str,
    pub sql: &'static str,
}

pub fn apply_migrations(conn: &Connection, migrations: &[Migration]) -> Result<()> {
    for migration in migrations {
        apply_migration_batch(conn, migration.sql)
            .with_context(|| format!("migration {} failed", migration.name))?;
    }
    Ok(())
}
