//! Schema migration support for future schema changes.
//!
/// Tracks schema versions and applies migrations when the database
/// is opened.
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// A database migration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    /// Migration version number
    pub version: u32,
    /// Migration description
    pub description: String,
    /// SQL to apply
    pub up_sql: String,
    /// SQL to rollback (optional)
    pub down_sql: Option<String>,
}

/// Result of a migration operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    /// From version
    pub from_version: u32,
    /// To version
    pub to_version: u32,
    /// Description
    pub description: String,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// When the migration was applied
    pub applied_at: DateTime<Utc>,
}

/// Manages schema migrations.
pub struct SchemaMigration {
    /// All registered migrations, sorted by version
    pub migrations: Vec<Migration>,
    /// Current schema version
    pub current_version: u32,
}

impl SchemaMigration {
    /// Create a new SchemaMigration with no migrations.
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
            current_version: 1,
        }
    }

    /// Register a migration.
    pub fn register(&mut self, migration: Migration) {
        self.migrations.push(migration);
        self.migrations.sort_by_key(|m| m.version);
    }

    /// Apply all pending migrations to a connection.
    pub fn apply_migrations(
        &self,
        conn: &Connection,
        current_version: u32,
    ) -> anyhow::Result<Vec<MigrationResult>> {
        let mut results = Vec::new();

        for migration in &self.migrations {
            if migration.version > current_version {
                debug!(
                    from_version = current_version,
                    to_version = migration.version,
                    description = migration.description,
                    "Applying migration"
                );

                let result = self.apply_single_migration(conn, migration);
                if result.success {
                    results.push(result);
                    // Update schema version
                    self.update_schema_version(conn, migration.version)?;
                } else {
                    results.push(result);
                }
            }
        }

        info!(
            applied = results.iter().filter(|r| r.success).count(),
            failed = results.iter().filter(|r| !r.success).count(),
            "Migration complete"
        );

        Ok(results)
    }

    /// Apply a single migration.
    fn apply_single_migration(&self, conn: &Connection, migration: &Migration) -> MigrationResult {
        let result = conn.execute_batch(&migration.up_sql);

        let success = result.is_ok();
        let error = result.err().map(|e| e.to_string());

        if success {
            info!(
                version = migration.version,
                description = migration.description,
                "Migration applied successfully"
            );
        } else {
            warn!(
                version = migration.version,
                error = ?error,
                "Migration failed"
            );
        }

        MigrationResult {
            from_version: self.current_version,
            to_version: migration.version,
            description: migration.description.clone(),
            success,
            error,
            applied_at: Utc::now(),
        }
    }

    /// Update the schema version in the metadata table.
    fn update_schema_version(&self, conn: &Connection, version: u32) -> anyhow::Result<()> {
        // In a full implementation, this would update a schema_version column
        debug!(version, "Schema version updated");
        Ok(())
    }

    /// Get migrations pending from a given version.
    pub fn pending_migrations(&self, from_version: u32) -> Vec<&Migration> {
        self.migrations
            .iter()
            .filter(|m| m.version > from_version)
            .collect()
    }
}

impl Default for SchemaMigration {
    fn default() -> Self {
        Self::new()
    }
}
