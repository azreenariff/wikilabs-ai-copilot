//! Workspace-Pack association manager — stores and retrieves workspace-enablement status
//! for knowledge packs using SQLite.

use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use tracing::{debug, error};

use anyhow::Context;

/// Manages workspace-pack associations in SQLite.
pub struct WorkspaceKnowledgeStore {
    db: Connection,
}

impl WorkspaceKnowledgeStore {
    /// Creates a new store with the given SQLite connection.
    pub fn new(db: Connection) -> Self {
        Self { db }
    }

    /// Initializes the associations table.
    pub fn init(&self) -> Result<()> {
        self.db
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS workspace_pack_associations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                workspace_id TEXT NOT NULL,
                pack_name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(workspace_id, pack_name)
            );",
            )
            .context("Failed to create workspace_pack_associations table")?;

        debug!("Workspace-pack associations table initialized");
        Ok(())
    }

    /// Enables a knowledge pack for the given workspace.
    pub fn enable_pack(&mut self, workspace_id: &str, pack_name: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        self.db
            .execute(
                "INSERT OR REPLACE INTO workspace_pack_associations
             (workspace_id, pack_name, enabled, created_at, updated_at)
             VALUES (?1, ?2, 1, ?3, ?3)",
                params![workspace_id, pack_name, now],
            )
            .context("Failed to enable pack")?;

        debug!(workspace = %workspace_id, pack = %pack_name, "Pack enabled");
        Ok(())
    }

    /// Disables a knowledge pack for the given workspace.
    pub fn disable_pack(&mut self, workspace_id: &str, pack_name: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        let changes = self
            .db
            .execute(
                "UPDATE workspace_pack_associations
             SET enabled = 0, updated_at = ?2
             WHERE workspace_id = ?1 AND pack_name = ?3",
                params![workspace_id, now, pack_name],
            )
            .context("Failed to disable pack")?;

        if changes == 0 {
            // If no existing row, insert one as disabled
            self.db
                .execute(
                    "INSERT INTO workspace_pack_associations
                 (workspace_id, pack_name, enabled, created_at, updated_at)
                 VALUES (?1, ?2, 0, ?3, ?3)",
                    params![workspace_id, pack_name, now],
                )
                .context("Failed to insert disabled association")?;
        }

        debug!(workspace = %workspace_id, pack = %pack_name, "Pack disabled");
        Ok(())
    }

    /// Gets all enabled packs for the given workspace.
    pub fn get_enabled_packs(&self, workspace_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.db.prepare(
            "SELECT pack_name FROM workspace_pack_associations
             WHERE workspace_id = ?1 AND enabled = 1
             ORDER BY pack_name",
        )?;

        let packs: Vec<String> = stmt
            .query_map(params![workspace_id], |row| row.get(0))?
            .collect::<Result<_, _>>()?;

        debug!(workspace = %workspace_id, pack_count = packs.len(), "Retrieved enabled packs");
        Ok(packs)
    }

    /// Gets all disabled packs for the given workspace.
    pub fn get_disabled_packs(&self, workspace_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.db.prepare(
            "SELECT pack_name FROM workspace_pack_associations
             WHERE workspace_id = ?1 AND enabled = 0
             ORDER BY pack_name",
        )?;

        let packs: Vec<String> = stmt
            .query_map(params![workspace_id], |row| row.get(0))?
            .collect::<Result<_, _>>()?;

        Ok(packs)
    }

    /// Gets the full association status for all packs of a workspace.
    pub fn get_all_associations(&self, workspace_id: &str) -> Result<Vec<AssociationStatus>> {
        let mut stmt = self.db.prepare(
            "SELECT pack_name, enabled, created_at, updated_at
             FROM workspace_pack_associations
             WHERE workspace_id = ?1
             ORDER BY pack_name",
        )?;

        let associations: Vec<AssociationStatus> = stmt
            .query_map(params![workspace_id], |row| {
                Ok(AssociationStatus {
                    pack_name: row.get(0)?,
                    enabled: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })?
            .collect::<Result<_, _>>()?;

        Ok(associations)
    }

    /// Checks whether a specific pack is enabled for a workspace.
    pub fn is_pack_enabled(&self, workspace_id: &str, pack_name: &str) -> Result<bool> {
        let enabled = self.db.query_row(
            "SELECT enabled FROM workspace_pack_associations
             WHERE workspace_id = ?1 AND pack_name = ?2",
            params![workspace_id, pack_name],
            |row| row.get::<_, i32>(0),
        );

        match enabled {
            Ok(e) => Ok(e != 0),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => {
                error!(error = %e, "Error checking pack status");
                Err(anyhow::anyhow!("Database error: {}", e))
            }
        }
    }

    /// Removes an association entirely.
    pub fn remove_association(&mut self, workspace_id: &str, pack_name: &str) -> Result<()> {
        let _changes = self.db.execute(
            "DELETE FROM workspace_pack_associations
             WHERE workspace_id = ?1 AND pack_name = ?2",
            params![workspace_id, pack_name],
        )?;

        debug!(workspace = %workspace_id, pack = %pack_name, "Association removed");
        Ok(())
    }

    /// Gets all workspaces that have the given pack enabled.
    pub fn get_workspaces_for_pack(&self, pack_name: &str) -> Result<Vec<String>> {
        let mut stmt = self.db.prepare(
            "SELECT DISTINCT workspace_id FROM workspace_pack_associations
             WHERE pack_name = ?1 AND enabled = 1
             ORDER BY workspace_id",
        )?;

        let workspaces: Vec<String> = stmt
            .query_map(params![pack_name], |row| row.get(0))?
            .collect::<Result<_, _>>()?;

        Ok(workspaces)
    }
}

/// Association status record.
#[derive(Debug, Clone)]
pub struct AssociationStatus {
    pub pack_name: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Trait for creating a connection from a workspace ID.
pub trait WorkspaceConnection {
    fn connection(&self, workspace_id: &str) -> Result<Connection>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> Result<WorkspaceKnowledgeStore> {
        let db = Connection::open_in_memory()?;
        let store = WorkspaceKnowledgeStore::new(db);
        store.init()?;
        Ok(store)
    }

    #[test]
    fn test_enable_and_get_packs() {
        let mut store = make_store().unwrap();

        store.enable_pack("ws1", "pack-a").unwrap();
        store.enable_pack("ws1", "pack-b").unwrap();

        let enabled = store.get_enabled_packs("ws1").unwrap();
        assert_eq!(enabled.len(), 2);
        assert!(enabled.contains(&"pack-a".to_string()));
        assert!(enabled.contains(&"pack-b".to_string()));
    }

    #[test]
    fn test_disable_pack() {
        let mut store = make_store().unwrap();

        store.enable_pack("ws1", "pack-a").unwrap();
        store.disable_pack("ws1", "pack-a").unwrap();

        let enabled = store.get_enabled_packs("ws1").unwrap();
        assert!(enabled.is_empty());

        let disabled = store.get_disabled_packs("ws1").unwrap();
        assert_eq!(disabled.len(), 1);
        assert_eq!(disabled[0], "pack-a");
    }

    #[test]
    fn test_is_pack_enabled() {
        let mut store = make_store().unwrap();

        assert!(!store.is_pack_enabled("ws1", "pack-a").unwrap());

        store.enable_pack("ws1", "pack-a").unwrap();
        assert!(store.is_pack_enabled("ws1", "pack-a").unwrap());

        store.disable_pack("ws1", "pack-a").unwrap();
        assert!(!store.is_pack_enabled("ws1", "pack-a").unwrap());
    }

    #[test]
    fn test_remove_association() {
        let mut store = make_store().unwrap();

        store.enable_pack("ws1", "pack-a").unwrap();
        store.remove_association("ws1", "pack-a").unwrap();

        let enabled = store.get_enabled_packs("ws1").unwrap();
        assert!(enabled.is_empty());
    }

    #[test]
    fn test_get_all_associations() {
        let mut store = make_store().unwrap();

        store.enable_pack("ws1", "pack-a").unwrap();
        store.disable_pack("ws1", "pack-b").unwrap();

        let all = store.get_all_associations("ws1").unwrap();
        assert_eq!(all.len(), 2);
        assert!(all.iter().any(|a| a.pack_name == "pack-a" && a.enabled));
        assert!(all.iter().any(|a| a.pack_name == "pack-b" && !a.enabled));
    }

    #[test]
    fn test_get_workspaces_for_pack() {
        let mut store = make_store().unwrap();

        store.enable_pack("ws1", "shared").unwrap();
        store.enable_pack("ws2", "shared").unwrap();
        store.enable_pack("ws3", "shared").unwrap();
        store.disable_pack("ws2", "shared").unwrap();

        let consumers = store.get_workspaces_for_pack("shared").unwrap();
        assert_eq!(consumers.len(), 2);
        assert!(consumers.contains(&"ws1".to_string()));
        assert!(consumers.contains(&"ws3".to_string()));
    }
}
