use macro_os_engines::context::database::{ContextStore, CONTEXT_REQUIRED_TABLES};
use macro_os_engines::database::migrations::tables_present;
use macro_os_engines::history::database::{HistoryStore, HISTORY_REQUIRED_TABLES};
use macro_os_engines::parse::database::{
    create_parse_command_schema, drop_parse_command_schema, ParseCommandStore,
    PARSE_COMMAND_REQUIRED_TABLES,
};

#[test]
fn parse_command_domain_schema_is_independently_healthy() {
    let store = ParseCommandStore::open_memory().expect("parse store");
    assert!(
        store
            .health_check()
            .expect("health")
            .required_tables_present
    );
    assert!(tables_present(store.connection(), PARSE_COMMAND_REQUIRED_TABLES).expect("tables"));
}

#[test]
fn context_domain_schema_is_independently_healthy() {
    let store = ContextStore::open_memory().expect("context store");
    assert!(store.health_check().expect("health"));
    assert!(tables_present(store.connection(), CONTEXT_REQUIRED_TABLES).expect("tables"));
}

#[test]
fn history_domain_schema_is_independently_healthy() {
    let store = HistoryStore::open_memory().expect("history store");
    assert!(store.health_check().expect("health"));
    assert!(tables_present(store.connection(), HISTORY_REQUIRED_TABLES).expect("tables"));
}

#[test]
fn parse_migrations_can_drop_and_recreate_schema() {
    let conn = rusqlite::Connection::open_in_memory().expect("memory db");
    create_parse_command_schema(&conn).expect("create");
    drop_parse_command_schema(&conn).expect("drop");
    assert!(!tables_present(&conn, PARSE_COMMAND_REQUIRED_TABLES).expect("check"));
}
