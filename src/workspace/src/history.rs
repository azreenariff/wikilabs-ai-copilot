//! Workspace history — chat session management via simple in-memory store.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct HistoryManager {
    histories: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl HistoryManager {
    pub fn new() -> Self {
        Self {
            histories: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_entry(&self, workspace_id: &str, content: &str) -> anyhow::Result<()> {
        let mut histories = self.histories.lock().unwrap();
        histories
            .entry(workspace_id.to_string())
            .or_insert_with(Vec::new)
            .push(content.to_string());
        Ok(())
    }

    pub fn get_entries(
        &self,
        workspace_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<String>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_history_manager() {
        let hm = HistoryManager::new();
        assert_eq!(hm.entry_count("ws1"), 0);
    }

    #[test]
    fn test_add_entry() {
        let hm = HistoryManager::new();
        hm.add_entry("ws1", "message 1").unwrap();
        hm.add_entry("ws1", "message 2").unwrap();
        assert_eq!(hm.entry_count("ws1"), 2);
    }

    #[test]
    fn test_get_entries() {
        let hm = HistoryManager::new();
        hm.add_entry("ws1", "a").unwrap();
        hm.add_entry("ws1", "b").unwrap();
        hm.add_entry("ws1", "c").unwrap();

        let entries = hm.get_entries("ws1", 10).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], "a");
        assert_eq!(entries[2], "c");
    }

    #[test]
    fn test_get_entries_limit() {
        let hm = HistoryManager::new();
        for i in 0..10 {
            hm.add_entry("ws1", &format!("msg{}", i)).unwrap();
        }
        let entries = hm.get_entries("ws1", 3).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], "msg7");
        assert_eq!(entries[2], "msg9");
    }

    #[test]
    fn test_empty_workspace() {
        let hm = HistoryManager::new();
        let entries = hm.get_entries("nonexistent", 10).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_isolation() {
        let hm = HistoryManager::new();
        hm.add_entry("ws1", "msg1").unwrap();
        hm.add_entry("ws2", "msg2").unwrap();

        assert_eq!(hm.entry_count("ws1"), 1);
        assert_eq!(hm.entry_count("ws2"), 1);
    }
}