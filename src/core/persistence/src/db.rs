//! SQLite connection management and query helpers.

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, Row, Transaction, TransactionBehavior};
use std::sync::{Arc, Mutex};
use tracing::debug;

/// SQLite database connection manager.
///
/// Wraps the connection in `Arc<Mutex<>>` so it can be shared across
/// repositories (each repository holds its own `Arc`).
#[derive(Clone)]
pub struct Database {
    pub path: String,
    connection: Arc<Mutex<Connection>>,
}

impl Database {
    /// Open or create a database at the given path.
    pub fn new(path: &str) -> Result<Self> {
        debug!(path, "Opening SQLite database");
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open SQLite database at {}", path))?;

        // Enable WAL mode for better concurrent performance
        conn.pragma_update(None, "journal_mode", "WAL")?;
        // Enable foreign keys
        conn.pragma_update(None, "foreign_keys", "on")?;

        let db = Self {
            path: path.to_string(),
            connection: Arc::new(Mutex::new(conn)),
        };

        tracing::info!(path, "Database opened successfully");
        Ok(db)
    }

    /// Execute a batch of SQL statements.
    pub fn execute_batch(&self, sql: &str) -> Result<()> {
        debug!(sql_len = sql.len(), "Executing SQL batch");
        self.connection
            .lock()
            .expect("Database mutex poisoned")
            .execute_batch(sql)
            .with_context(|| "Failed to execute SQL batch")
    }

    /// Run a closure that returns a single row.
    /// Closures should return `Result<T, rusqlite::Error>` (from `row.get()`).
    pub fn query<T, F>(&self, sql: &str, params: &[&dyn rusqlite::ToSql], f: F) -> Result<T>
    where
        F: FnOnce(&Row) -> std::result::Result<T, rusqlite::Error>,
    {
        let conn = self.connection.lock().expect("Database mutex poisoned");
        let mut stmt = conn
            .prepare(sql)
            .with_context(|| format!("Failed to prepare query: {}", sql))?;

        stmt.query_row(params, f)
            .optional()
            .with_context(|| format!("No result found for query: {}", sql))
            .and_then(|v| v.ok_or_else(|| anyhow::anyhow!("No result found for query: {}", sql)))
    }

    /// Run a closure that returns multiple rows.
    pub fn query_all<T, F>(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
        f: F,
    ) -> Result<Vec<T>>
    where
        F: FnMut(&Row) -> std::result::Result<T, rusqlite::Error>,
    {
        let conn = self.connection.lock().expect("Database mutex poisoned");
        let mut stmt = conn
            .prepare(sql)
            .with_context(|| format!("Failed to prepare query: {}", sql))?;

        let rows = stmt.query_map(params, f)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Run an INSERT/UPDATE/DELETE that doesn't return data.
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize> {
        let conn = self.connection.lock().expect("Database mutex poisoned");
        let rows = conn
            .prepare(sql)
            .with_context(|| format!("Failed to prepare statement: {}", sql))?
            .execute(params)
            .with_context(|| "Failed to execute statement")?;
        Ok(rows)
    }

    /// Get the last inserted row ID.
    pub fn last_insert_rowid(&self) -> i64 {
        self.connection
            .lock()
            .expect("Database mutex poisoned")
            .last_insert_rowid()
    }

    /// Begin a transaction using a closure — the closure receives a
    /// transaction reference and the guard is dropped at the end.
    /// On error the transaction is rolled back; on success it commits.
    pub fn transaction<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Transaction<'_>) -> Result<T>,
    {
        let mut conn = self.connection.lock().expect("Database mutex poisoned");
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .with_context(|| "Failed to begin transaction")?;
        match f(&tx) {
            Ok(v) => {
                tx.commit()
                    .with_context(|| "Failed to commit transaction")?;
                Ok(v)
            }
            Err(e) => {
                let _ = tx.rollback();
                Err(e)
            }
        }
    }

    /// Get a reference to the raw connection (for advanced usage).
    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.connection.lock().expect("Database mutex poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_memory_database() {
        let db = Database::new(":memory:").unwrap();
        db.execute_batch(crate::schema::INIT_SQL).unwrap();
    }
}
