//! Namespace management — isolated search spaces per pack/workspace.

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::Connection;
use tracing::debug;

/// Represents a namespace entry.
#[derive(Debug, Clone)]
pub struct Namespace {
    pub id: i64,
    pub name: String,
    pub workspace_id: Option<String>,
    pub knowledge_pack: String,
    pub embedding_dimensions: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Manages namespace lifecycle.
pub struct NamespaceManager {
    pub conn: Connection,
}

impl NamespaceManager {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    /// Create a new namespace.
    pub fn create(
        &self,
        name: &str,
        workspace_id: Option<&str>,
        knowledge_pack: &str,
        dimensions: usize,
    ) -> Result<Namespace> {
        let _now = Utc::now().to_rfc3339();
        let mut stmt = self.conn.prepare(
            r#"
            INSERT INTO knowledge_namespaces (name, workspace_id, knowledge_pack, embedding_dimensions)
            VALUES (?1, ?2, ?3, ?4)
            RETURNING id, name, workspace_id, knowledge_pack, embedding_dimensions, created_at, updated_at
            "#,
        )?;

        let row = stmt
            .query_row(
                rusqlite::params![
                    name,
                    workspace_id.map(|s| s.to_string()),
                    knowledge_pack,
                    dimensions as i32,
                ],
                |row| {
                    Ok(Namespace {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        workspace_id: row.get(2)?,
                        knowledge_pack: row.get(3)?,
                        embedding_dimensions: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                },
            )
            .context("Failed to create namespace")?;

        Ok(row)
    }

    /// Get or create a namespace.
    pub fn get_or_create(
        &self,
        name: &str,
        workspace_id: Option<&str>,
        knowledge_pack: &str,
        dimensions: usize,
    ) -> Result<Namespace> {
        match self.get(name) {
            Ok(ns) => Ok(ns),
            Err(_) => self.create(name, workspace_id, knowledge_pack, dimensions),
        }
    }

    /// Get a namespace by name.
    pub fn get(&self, name: &str) -> Result<Namespace> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, workspace_id, knowledge_pack, embedding_dimensions, created_at, updated_at
            FROM knowledge_namespaces
            WHERE name = ?1
            "#,
        )?;

        stmt.query_row(rusqlite::params![name], |row| {
            Ok(Namespace {
                id: row.get(0)?,
                name: row.get(1)?,
                workspace_id: row.get(2)?,
                knowledge_pack: row.get(3)?,
                embedding_dimensions: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .context(format!("Namespace not found: {}", name))
    }

    /// List all namespaces.
    pub fn list(&self) -> Result<Vec<Namespace>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, workspace_id, knowledge_pack, embedding_dimensions, created_at, updated_at
            FROM knowledge_namespaces
            ORDER BY name
            "#,
        )?;

        let rows = stmt.query_map(rusqlite::params![], |row| {
            Ok(Namespace {
                id: row.get(0)?,
                name: row.get(1)?,
                workspace_id: row.get(2)?,
                knowledge_pack: row.get(3)?,
                embedding_dimensions: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .context("Failed to list namespaces")
    }

    /// Delete a namespace.
    pub fn delete(&self, name: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM knowledge_namespaces WHERE name = ?1",
            rusqlite::params![name],
        )?;

        debug!(namespace = name, "Deleted namespace");
        Ok(())
    }

    /// Get namespace by ID.
    pub fn get_by_id(&self, id: i64) -> Result<Namespace> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, workspace_id, knowledge_pack, embedding_dimensions, created_at, updated_at
            FROM knowledge_namespaces
            WHERE id = ?1
            "#,
        )?;

        stmt.query_row(rusqlite::params![id], |row| {
            Ok(Namespace {
                id: row.get(0)?,
                name: row.get(1)?,
                workspace_id: row.get(2)?,
                knowledge_pack: row.get(3)?,
                embedding_dimensions: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .context(format!("Namespace ID not found: {}", id))
    }
}
