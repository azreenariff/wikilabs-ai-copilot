//! Workspace manager — create, switch, delete, configure.
//!
//! Uses the real `wikilabs-persistence` layer for all workspace CRUD operations.

use wikilabs_persistence::RepositoryFactory;

pub struct WorkspaceManager {
    repos: RepositoryFactory,
    active_id: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}

pub struct WorkspaceConfig {
    pub name: String,
    pub customer_name: String,
    pub technology_stack: Vec<String>,
}

impl WorkspaceManager {
    pub fn new(repos: RepositoryFactory) -> Self {
        Self {
            repos,
            active_id: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn create(&self, config: WorkspaceConfig) -> anyhow::Result<uuid::Uuid> {
        let id = uuid::Uuid::new_v4();
        let tech_stack = serde_json::to_string(&config.technology_stack)
            .map_err(|e| anyhow::anyhow!("Failed to serialize tech stack: {e}"))?;
        self.repos
            .workspace
            .insert(&id.to_string(), &config.name, &config.customer_name, &tech_stack)?;
        // Set as active workspace
        {
            let mut active = self.active_id.lock().unwrap();
            *active = Some(id.to_string());
        }
        tracing::info!(id = %id, name = %config.name, "Workspace created");
        Ok(id)
    }

    pub fn list(&self) -> anyhow::Result<Vec<(uuid::Uuid, String)>> {
        let workspaces = self.repos.workspace.list_all()?;
        let result: Vec<(uuid::Uuid, String)> = workspaces
            .iter()
            .map(|w| (uuid::Uuid::parse_str(&w.id).unwrap_or_default(), w.name.clone()))
            .collect();
        Ok(result)
    }

    pub fn switch(&self, id: uuid::Uuid) -> anyhow::Result<()> {
        // Verify workspace exists
        let exists = self.repos.workspace.get_by_id(&id.to_string())?.is_some();
        if !exists {
            anyhow::bail!("Workspace not found: {}", id);
        }
        {
            let mut active = self.active_id.lock().unwrap();
            *active = Some(id.to_string());
        }
        tracing::info!(id = %id, "Workspace switched");
        Ok(())
    }

    pub fn get_active(&self) -> anyhow::Result<Option<uuid::Uuid>> {
        let active = self.active_id.lock().unwrap();
        Ok(active.as_ref().map(|s| uuid::Uuid::parse_str(s).unwrap_or_default()))
    }

    pub fn delete(&self, id: uuid::Uuid) -> anyhow::Result<()> {
        self.repos.workspace.delete(&id.to_string())?;
        let mut active = self.active_id.lock().unwrap();
        if active.as_ref() == Some(&id.to_string()) {
            *active = None;
        }
        tracing::info!(id = %id, "Workspace deleted");
        Ok(())
    }

    pub fn get_by_id(&self, id: uuid::Uuid) -> anyhow::Result<Option<WorkspaceConfig>> {
        let ws = self.repos.workspace.get_by_id(&id.to_string())?;
        match ws {
            None => Ok(None),
            Some(ws_row) => {
                let tech_stack: Vec<String> = serde_json::from_str(&ws_row.technology_stack)
                    .unwrap_or_default();
                Ok(Some(WorkspaceConfig {
                    name: ws_row.name,
                    customer_name: ws_row.customer_name,
                    technology_stack: tech_stack,
                }))
            }
        }
    }
}