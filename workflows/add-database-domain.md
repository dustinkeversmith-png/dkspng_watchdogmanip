# Workflow: Add database domain tables

Add or extend SQLite storage for parse, context, or history domains.

## Shared infrastructure

| File | Purpose |
|------|---------|
| `src/database/connection.rs` | `DatabaseConnection` open/open_memory |
| `src/database/migrations.rs` | `apply_migrations`, `tables_present`, `Migration` |

## Domain locations

| Domain | Migrations | Store | Required tables const |
|--------|------------|-------|------------------------|
| Parse commands | `src/parse/database/migrations.rs` | `CommandSqliteDatabase` | `PARSE_COMMAND_REQUIRED_TABLES` |
| Context | `src/context/database/migrations.rs` | `ContextStore` | `CONTEXT_REQUIRED_TABLES` |
| History | `src/history/database/migrations.rs` | `HistoryStore` | `HISTORY_REQUIRED_TABLES` |

## Steps

### 1. Add SQL to domain migrations

```rust
// src/context/database/migrations.rs
pub const CONTEXT_SCHEMA_SQL: &str = r#"
    -- existing tables...
    CREATE TABLE IF NOT EXISTS my_new_table (...);
"#;

pub const CONTEXT_REQUIRED_TABLES: &[&str] = &[
    // ...
    "my_new_table",
];
```

### 2. Add store methods

```rust
// src/context/database/sqlite.rs
pub fn insert_my_record(&self, record: &MyRecord) -> Result<()> {
    self.conn.execute("INSERT INTO my_new_table ...", params![...])?;
    Ok(())
}
```

### 3. Add model type

```rust
// src/context/database/model.rs
pub struct MyRecord { ... }
```

### 4. Health check

Stores expose `health_check()` using `tables_present()`. New tables must be in `*_REQUIRED_TABLES`.

### 5. Test

Add to `tests/database/database_health_test.rs` or domain-specific test:

```bash
cargo test --test database database_health
cargo test --test context   # if context store behavior
cargo test --test history   # if history store behavior
```

## Parse command inserts

Keep one-by-one insertion:

```rust
db.insert_command(&record)?;  // not insert_output()
```

## Checklist

- [ ] Migration SQL in domain `migrations.rs`
- [ ] Table in `*_REQUIRED_TABLES`
- [ ] Store insert/query methods
- [ ] Health test passes
- [ ] Glossary updated
