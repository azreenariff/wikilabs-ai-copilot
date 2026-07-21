//! Workspace history — chat session management via simple in-memory store.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct HistoryManager {
    histories: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl HistoryManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entry(&self, workspace_id: &str, content: &str) -> anyhow::Result<()> {
        let mut histories = self.histories.lock().unwrap();
        histories
            .entry(workspace_id.to_string())
            .or_default()
            .push(content.to_string());
        Ok(())
    }

    pub fn get_entries(&self, workspace_id: &str, limit: usize) -> anyhow::Result<Vec<String>> {
        let histories = self.histories.lock().unwrap();
        let entries = histories.get(workspace_id).cloned().unwrap_or_default();
        let start = if entries.len() > limit {
            entries.len() - limit
        } else {
            0
        };
        Ok(entries[start..].to_vec())
    }

    pub fn clear(&self, workspace_id: &str) {
        let mut histories = self.histories.lock().unwrap();
        histories.remove(workspace_id);
    }

    pub fn entry_count(&self, workspace_id: &str) -> usize {
        let histories = self.histories.lock().unwrap();
        histories.get(workspace_id).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self {
            histories: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}