//! Workspace namespace — maps workspaces to namespace hierarchies.

use super::namespace::Namespace;
use super::namespace::NamespaceManager;
use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::SqlitePool;
use tracing::debug;

/// Maps a workspace ID to its namespace hierarchy within a knowledge pack.
#[derive(Debug, Clone)]
pub struct WorkspaceNamespace {
    pub workspace_id: String,
    pub knowledge_pack: String,
    pub root_namespace: String,
    pub children: Vec<Namespace>,
    pub created_at: String,
    pub updated_at: String,
}

impl WorkspaceNamespace {
    /// Create a new workspace namespace with hierarchical sub-namespaces.
    pub async fn create(
        namespace_mgr: &NamespaceManager,
        workspace_id: &str,
        knowledge_pack: &str,
        dimensions: usize,
    ) -> Result<Self> {
        let now = Utc::now().to_rfc3339();
        let root = format!("workspace:{workspace_id}");

        // Create root namespace
        let root_ns = namespace_mgr
            .create(&root, Some(workspace_id), knowledge_pack, dimensions)
            .await
            .context("Failed to create root namespace")?;

        let children = vec![root_ns];

        Ok(Self {
            workspace_id: workspace_id.to_string(),
            knowledge_pack: knowledge_pack.to_string(),
            root_namespace: root,
            children,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Get or create a sub-namespace for a specific pack within the workspace.
    pub async fn get_or_create_pack_namespace(
        &self,
        namespace_mgr: &NamespaceManager,
        pack_name: &str,
        dimensions: usize,
    ) -> Result<Namespace> {
        let sub_name = format!("{}:{}", self.root_namespace, pack_name);

        match namespace_mgr.get(&sub_name).await {
            Ok(ns) => Ok(ns),
            Err(_) => {
                let ns = namespace_mgr
                    .create(&sub_name, Some(&self.workspace_id), &self.knowledge_pack, dimensions)
                    .await?;
                self.children.push(ns.clone());
                Ok(ns)
            }
        }
    }

    /// List all child namespaces for this workspace.
    pub async fn list_children(&self, namespace_mgr: &NamespaceManager) -> Result<Vec<Namespace>> {
        namespace_mgr.list().await
    }
}