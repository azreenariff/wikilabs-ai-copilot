//! Knowledge pack repository — manages a collection of knowledge packs,
//! including enable/disable, listing, and filtering.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::debug;

use super::KnowledgePack;

/// A read-write repository for knowledge packs.
///
/// The repository holds loaded packs in-memory and tracks their enabled/disabled
/// state. All mutations are synchronized via `RwLock`.
pub struct PackRepository {
    packs: Arc<RwLock<HashMap<String, KnowledgePack>>>,
}

impl Clone for PackRepository {
    fn clone(&self) -> Self {
        Self {
            packs: Arc::clone(&self.packs),
        }
    }
}

impl PackRepository {
    /// Create a new empty repository.
    pub fn new() -> Self {
        Self {
            packs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add or replace a pack in the repository.
    pub fn upsert(&self, pack: KnowledgePack) -> Result<()> {
        let mut map = self.packs.write().expect("PackRepository write lock poisoned");
        debug!(name = %pack.name(), "Registering pack in repository");
        map.insert(pack.name().to_string(), pack);
        Ok(())
    }

    /// Get a reference to a pack by name.
    pub fn get(&self, name: &str) -> Option<KnowledgePack> {
        let map = self.packs.read().expect("PackRepository read lock poisoned");
        map.get(name).cloned()
    }

    /// List all packs, optionally filtering by enabled state.
    pub fn list_all(&self) -> Vec<KnowledgePack> {
        let map = self.packs.read().expect("PackRepository read lock poisoned");
        map.values().cloned().collect()
    }

    /// List only enabled packs.
    pub fn list_enabled(&self) -> Vec<KnowledgePack> {
        let map = self.packs.read().expect("PackRepository read lock poisoned");
        map.values().filter(|p| p.enabled).cloned().collect()
    }

    /// List only disabled packs.
    pub fn list_disabled(&self) -> Vec<KnowledgePack> {
        let map = self.packs.read().expect("PackRepository read lock poisoned");
        map.values().filter(|p| !p.enabled).cloned().collect()
    }

    /// Enable a pack by name.
    pub fn enable(&self, name: &str) -> Result<()> {
        let mut map = self.packs.write().expect("PackRepository write lock poisoned");
        if let Some(pack) = map.get_mut(name) {
            pack.enabled = true;
            debug!(name, "Pack enabled");
        } else {
            anyhow::bail!("Pack not found: {}", name);
        }
        Ok(())
    }

    /// Disable a pack by name.
    pub fn disable(&self, name: &str) -> Result<()> {
        let mut map = self.packs.write().expect("PackRepository write lock poisoned");
        if let Some(pack) = map.get_mut(name) {
            pack.enabled = false;
            debug!(name, "Pack disabled");
        } else {
            anyhow::bail!("Pack not found: {}", name);
        }
        Ok(())
    }

    /// Toggle a pack's enabled state.
    pub fn toggle(&self, name: &str) -> Result<()> {
        let mut map = self.packs.write().expect("PackRepository write lock poisoned");
        if let Some(pack) = map.get_mut(name) {
            pack.enabled = !pack.enabled;
            debug!(name, enabled = pack.enabled, "Pack toggled");
        } else {
            anyhow::bail!("Pack not found: {}", name);
        }
        Ok(())
    }

    /// Remove a pack from the repository.
    pub fn remove(&self, name: &str) -> Result<KnowledgePack> {
        let mut map = self.packs.write().expect("PackRepository write lock poisoned");
        map.remove(name).ok_or_else(|| anyhow::anyhow!("Pack not found: {}", name))
    }

    /// Check if a pack exists in the repository.
    pub fn contains(&self, name: &str) -> bool {
        let map = self.packs.read().expect("PackRepository read lock poisoned");
        map.contains_key(name)
    }

    /// Return the total number of packs.
    pub fn count(&self) -> usize {
        let map = self.packs.read().expect("PackRepository read lock poisoned");
        map.len()
    }

    /// Return the number of enabled packs.
    pub fn enabled_count(&self) -> usize {
        let map = self.packs.read().expect("PackRepository read lock poisoned");
        map.values().filter(|p| p.enabled).count()
    }

    /// Check if all pack dependencies are satisfied.
    ///
    /// A dependency is satisfied if the required pack exists and is enabled.
    pub fn check_dependencies(&self, pack_name: &str) -> Result<Vec<String>> {
        let map = self.packs.read().expect("PackRepository read lock poisoned");
        let pack = map.get(pack_name).ok_or_else(|| anyhow::anyhow!("Pack not found: {}", pack_name))?;

        let mut unsatisfied = Vec::new();
        for dep in &pack.manifest.dependencies {
            if let Some(dep_pack) = map.get(&dep.name) {
                if !dep_pack.enabled {
                    unsatisfied.push(format!(
                        "Dependency '{}' exists but is disabled",
                        dep.name
                    ));
                }
            } else {
                unsatisfied.push(format!("Dependency '{}' is not installed", dep.name));
            }
        }

        if unsatisfied.is_empty() {
            debug!(pack_name, "All dependencies satisfied");
        } else {
            debug!(pack_name, ?unsatisfied, "Unsatisfied dependencies found");
        }

        Ok(unsatisfied)
    }
}