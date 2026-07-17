//! Database migrations — schema versioning and upgrades.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use tracing::{debug, info};

use crate::db::Database;
use crate::schema::INIT_SQL;

/// Current schema version — increment when migrating.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Track schema version and migration history.
pub struct MigrationManager {
    db: Database,
}

impl MigrationManager {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Initialize the database: create schema_version table if missing,
    /// run INIT_SQL, and record the initial version.
    pub fn initialize(&self) -> Result<()> {
        info!("Initializing database schema");

        // Create the schema_version tracking table first
        self.db.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version INTEGER NOT NULL UNIQUE,
                applied_at TEXT NOT NULL DEFAULT (datetime('now')),
                description TEXT DEFAULT ''
            );",
        )?;

        // Check if already initialized
        let version: u32 = self
            .db
            .conn()
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_versions",
                [],
                |row| row.get::<_, u32>(0),
            )
            .map_err(|e| anyhow::Error::from(e))?;

        if version == 0 {
            // Run the initial schema
            self.db.execute_batch(INIT_SQL)?;

            // Record version
            self.db.execute(
                "INSERT INTO schema_versions (version, description) VALUES (?, ?)",
                &[&CURRENT_SCHEMA_VERSION, &"Initial schema"],
            )?;

            info!(version = CURRENT_SCHEMA_VERSION, "Database initialized");
        } else {
            debug!(version, "Database already initialized");
        }

        Ok(())
    }

    /// Check if migrations are needed.
    pub fn needs_migration(&self) -> Result<bool> {
        let current = self.get_applied_version()?;
        Ok(current < CURRENT_SCHEMA_VERSION)
    }

    /// Get the highest applied schema version.
    pub fn get_applied_version(&self) -> Result<u32> {
        let version: u32 = self
            .db
            .conn()
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_versions",
                [],
                |row| row.get(0),
            )
            .context("Failed to read schema version")?;
        Ok(version)
    }

    /// Apply pending migrations.
    pub fn apply_pending(&self) -> Result<()> {
        let current = self.get_applied_version()?;
        if current >= CURRENT_SCHEMA_VERSION {
            debug!("No pending migrations");
            return Ok(());
        }

        info!(
            from = current,
            to = CURRENT_SCHEMA_VERSION,
            "Applying migrations"
        );

        for v in (current + 1)..=CURRENT_SCHEMA_VERSION {
            self.apply_migration(v)?;
        }

        Ok(())
    }

    /// Apply a single migration.
    fn apply_migration(&self, version: u32) -> Result<()> {
        info!(version, "Applying migration");

        match version {
            1 => self.migration_v1()?,
            _ => anyhow::bail!("Unknown migration version: {}", version),
        }

        self.db.execute(
            "INSERT INTO schema_versions (version, description) VALUES (?, ?)",
            &[&version, &format!("Migration v{version}")],
        )?;

        info!(version, "Migration applied successfully");
        Ok(())
    }

    /// Initial schema migration.
    fn migration_v1(&self) -> Result<()> {
        self.db.execute_batch(INIT_SQL)?;
        Ok(())
    }

    /// Get migration history.
    pub fn get_history(&self) -> Result<Vec<SchemaMigration>> {
        let rows = self.db.query_all(
            "SELECT version, applied_at, description FROM schema_versions ORDER BY version DESC",
            &[],
            |row| {
                let applied_at_str: String = row.get(1)?;
                let applied_at = chrono::NaiveDateTime::parse_from_str(&applied_at_str, "%Y-%m-%d %H:%M:%S")
                    .map(|naive| naive.and_utc())
                    .unwrap_or_else(|_| Utc::now());
                Ok(SchemaMigration {
                    version: row.get(0)?,
                    applied_at,
                    description: row.get(2)?,
                })
            },
        )?;
        Ok(rows)
    }
}

/// A single schema migration record.
#[derive(Debug, Clone)]
pub struct SchemaMigration {
    pub version: u32,
    pub applied_at: DateTime<Utc>,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_initialize() {
        let db = Database::new(":memory:").unwrap();
        let mgr = MigrationManager::new(db);
        mgr.initialize().unwrap();
        assert_eq!(mgr.get_applied_version().unwrap(), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_migration_already_initialized() {
        let db = Database::new(":memory:").unwrap();
        let mgr = MigrationManager::new(db);
        mgr.initialize().unwrap();
        // Second call should be idempotent
        mgr.initialize().unwrap();
        assert_eq!(mgr.get_applied_version().unwrap(), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_migration_history() {
        let db = Database::new(":memory:").unwrap();
        let mgr = MigrationManager::new(db);
        mgr.initialize().unwrap();
        let history = mgr.get_history().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].version, CURRENT_SCHEMA_VERSION);
    }
}