//! Shared SQLite helpers used by domain stores (`parse`, `context`, `history`).
//!
//! Parse-command storage lives in [`crate::parse::database`]; this module re-exports
//! those types for older call sites.

pub mod connection;
pub mod migrations;

pub use crate::parse::database::{
    health::{check_sqlite_health, DatabaseHealthReport},
    model::{
        CommandSearchHit, CommandSearchOptions, DatabaseStats, DatabaseTableDump,
        NewParsedCommandRecord, StoredParsedCommandRecord,
    },
    sqlite::{CommandSqliteDatabase, ParseCommandStore},
};

use crate::parse::model::ParsedCommand;

/// Prefer [`NewParsedCommandRecord::from_parsed`] or [`ParseCommandStore::insert_parsed_command`].
pub fn new_record_from_parsed_command(
    source_name: impl Into<String>,
    command: ParsedCommand,
) -> NewParsedCommandRecord {
    NewParsedCommandRecord::from_parsed(source_name, command)
}
