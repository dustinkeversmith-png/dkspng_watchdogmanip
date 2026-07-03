use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

#[derive(Debug)]
pub struct DatabaseConnection {
    conn: Connection,
}

impl DatabaseConnection {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path.as_ref())
            .with_context(|| format!("failed to open sqlite db at {}", path.as_ref().display()))?;
        Ok(Self { conn })
    }

    pub fn open_existing(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path.as_ref())
            .with_context(|| format!("failed to open sqlite db at {}", path.as_ref().display()))?;
        Ok(Self { conn })
    }

    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

impl From<Connection> for DatabaseConnection {
    fn from(conn: Connection) -> Self {
        Self { conn }
    }
}

impl AsRef<Connection> for DatabaseConnection {
    fn as_ref(&self) -> &Connection {
        &self.conn
    }
}

impl AsMut<Connection> for DatabaseConnection {
    fn as_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}
