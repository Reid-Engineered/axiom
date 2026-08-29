use std::path::Path;

use rusqlite::{params, Connection, Result};

mod schema;

#[cfg(test)]
mod tests;

pub use schema::LATEST_SCHEMA_VERSION;

/// Opens Axiom's single local database, configures SQLite, and applies pending migrations.
pub fn open(path: impl AsRef<Path>) -> Result<Connection> {
    let mut connection = Connection::open(path)?;
    configure(&connection)?;
    migrate(&mut connection)?;
    Ok(connection)
}

/// Opens a transient database with the production schema, useful to query code and tests.
pub fn open_in_memory() -> Result<Connection> {
    let mut connection = Connection::open_in_memory()?;
    configure(&connection)?;
    migrate(&mut connection)?;
    Ok(connection)
}

fn configure(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;",
    )
}

/// Applies every pending migration in order inside its own transaction.
pub fn migrate(connection: &mut Connection) -> Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );",
    )?;

    let current_version = connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get::<_, i64>(0),
    )?;

    for migration in schema::MIGRATIONS
        .iter()
        .filter(|migration| migration.version > current_version)
    {
        let transaction = connection.transaction()?;
        transaction.execute_batch(migration.sql)?;
        transaction.execute(
            "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
            params![migration.version, migration.name],
        )?;
        transaction.commit()?;
    }

    Ok(())
}
