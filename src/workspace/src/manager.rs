//! Workspace manager — create, switch, delete, configure.
//!
//! Uses the real `wikilabs-persistence` layer for all workspace CRUD operations.

use wikilabs_persistence::RepositoryFactory;

#[cfg(test)]
mod tests {
    use super::*;
    use wikilabs_persistence::Database;

    fn setup_db() -> RepositoryFactory {
        let db = Database::new(":memory:").unwrap();
        db.execute_batch(wikilabs_persistence::schema::INIT_SQL).unwrap();
        RepositoryFactory::new(db)
    }

    #[test]
    fn test_workspace_manager_new() {
        let repos = setup_db();
        let wm = WorkspaceManager::new(repos);
        let active = wm.get_active().unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn test_workspace_manager_create() {
        let repos = setup_db();
        let wm = WorkspaceManager::new(repos);
        let id = wm.create(WorkspaceConfig {
            name: "test-ws".to_string(),
            customer_name: "test-customer".to_string(),
            technology_stack: vec!["rust".to_string()],
        }).unwrap();
        let list = wm.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].1, "test-ws");
    }

    #[test]
    fn test_workspace_manager_switch() {
        let repos = setup_db();
        let wm = WorkspaceManager::new(repos);
        let id = wm.create(WorkspaceConfig {
            name: "ws1".to_string(),
            customer_name: "c1".to_string(),
            technology_stack: vec![],
        }).unwrap();
        wm.switch(id).unwrap();
        let active = wm.get_active().unwrap();
        assert_eq!(active.unwrap(), id);
    }

    #[test]
    fn test_workspace_manager_switch_nonexistent() {
        let repos = setup_db();
        let wm = WorkspaceManager::new(repos);
        let id = uuid::Uuid::new_v4();
        let result = wm.switch(id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Workspace not found"));
    }

    #[test]
    fn test_workspace_manager_delete() {
        let repos = setup_db();
        let wm = WorkspaceManager::new(repos);
        let id = wm.create(WorkspaceConfig {
            name: "ws1".to_string(),
            customer_name: "c1".to_string(),
            technology_stack: vec![],
        }).unwrap();
        wm.switch(id).unwrap();
        wm.delete(id).unwrap();
        let active = wm.get_active().unwrap();
        assert!(active.is_none());
        let list = wm.list().unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_workspace_manager_get_by_id() {
        let repos = setup_db();
        let wm = WorkspaceManager::new(repos);
        let id = wm.create(WorkspaceConfig {
            name: "ws1".to_string(),
            customer_name: "c1".to_string(),
            technology_stack: vec!["kubernetes".to_string(), "docker".to_string()],
        }).unwrap();
        let config = wm.get_by_id(id).unwrap().unwrap();
        assert_eq!(config.name, "ws1");
        assert_eq!(config.customer_name, "c1");
        assert_eq!(config.technology_stack, vec!["kubernetes", "docker"]);
    }

    #[test]
    fn test_workspace_manager_get_by_id_nonexistent() {
        let repos = setup_db();
        let wm = WorkspaceManager::new(repos);
        let result = wm.get_by_id(uuid::Uuid::new_v4()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_workspace_manager_list_empty() {
        let repos = setup_db();
        let wm = WorkspaceManager::new(repos);
        let list = wm.list().unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_workspace_manager_multiple_workspaces() {
        let repos = setup_db();
        let wm = WorkspaceManager::new(repos);
        wm.create(WorkspaceConfig {
            name: "ws1".to_string(),
            customer_name: "c1".to_string(),
            technology_stack: vec![],
        }).unwrap();
        wm.create(WorkspaceConfig {
            name: "ws2".to_string(),
            customer_name: "c2".to_string(),
            technology_stack: vec!["go".to_string()],
        }).unwrap();
        let list = wm.list().unwrap();
        assert_eq!(list.len(), 2);
    }
}

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