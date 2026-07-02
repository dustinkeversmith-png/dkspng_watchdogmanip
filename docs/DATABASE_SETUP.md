# Database Setup Guide

This project currently uses SQLite through `rusqlite`.

SQLite is a local embedded database, so there is no separate database server to start. The database is considered "online" when the application can:

1. Open or create the `.sqlite` file.
2. Apply the schema.
3. Run a health check query.
4. Verify required tables exist.
5. Insert and read a parsed command record.

## Cargo Dependency

Make sure `Cargo.toml` includes:

```toml
[dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }