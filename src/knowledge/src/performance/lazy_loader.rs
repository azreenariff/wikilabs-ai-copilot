//! Lazy loading — load knowledge packs only when needed.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

use super::cancellation::CancellationToken;

/// A lazily-loaded knowledge pack.
///
/// The actual pack data is loaded on first access and cached thereafter.
/// This avoids loading all packs at startup, reducing memory usage and
/// startup time.
pub struct LazyPackLoader {
    /// Registered pack paths (pack_name -> pack_dir path).
    packs: Arc<Mutex<HashMap<String, PathBuf>>>,
    /// Loaded pack data.
    loaded: Arc<Mutex<HashMap<String, PackData>>>,
}

/// Loaded pack data, cached after first load.
#[derive(Debug, Clone)]
pub struct PackData {
    pub name: String,
    pub version: String,
    pub document_count: usize,
    pub path: PathBuf,
}

impl LazyPackLoader {
    /// Creates a new lazy pack loader.
    pub fn new() -> Self {
        Self {
            packs: Arc::new(Mutex::new(HashMap::new())),
            loaded: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Registers a pack directory for lazy loading.
    pub async fn register_pack(&self, name: &str, path: PathBuf) {
        let mut packs = self.packs.lock().await;
        packs.insert(name.to_string(), path);
        debug!(pack = %name, path = %path.display(), "Registered pack for lazy loading");
    }

    /// Unregisters a pack.
    pub async fn unregister_pack(&self, name: &str) {
        let mut packs = self.packs.lock().await;
        let was_loaded = self.loaded.lock().await.contains_key(name);
        packs.remove(name);

        if was_loaded {
            self.loaded.lock().await.remove(name);
            debug!(pack = %name, "Unregistered and unloaded pack");
        } else {
            debug!(pack = %name, "Unregistered unloaded pack");
        }
    }

    /// Loads a pack on demand.
    ///
    /// If the pack has already been loaded, returns the cached data.
    /// Otherwise, loads it from the registered path and caches the result.
    pub async fn load_pack(
        &self,
        pack_name: &str,
        cancellation: CancellationToken,
    ) -> Option<PackData> {
        // If already loaded, return cached data
        {
            let loaded = self.loaded.lock().await;
            if let Some(data) = loaded.get(pack_name) {
                debug!(pack = %pack_name, "Returning cached pack data");
                return Some(data.clone());
            }
        }

        // Check if registered
        let pack_path = {
            let packs = self.packs.lock().await;
            packs.get(pack_name).cloned()?
        };

        // Check for cancellation
        if cancellation.is_cancelled() {
            debug!(pack = %pack_name, "Load cancelled");
            return None;
        }

        // Simulate loading — in real implementation this would read manifest.yaml,
        // count documents, validate, etc.
        info!(pack = %pack_name, path = %pack_path.display(), "Loading pack");

        // Parse manifest to get basic info
        let manifest_path = pack_path.join("manifest.yaml");
        let mut doc_count = 0;

        if manifest_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = serde_yaml::from_str::<crate::sdk::schema::Manifest>(&content)
                {
                    doc_count = manifest.documents.len();

                    // Check cancellation between operations
                    if cancellation.is_cancelled() {
                        debug!(pack = %pack_name, "Load cancelled after manifest read");
                        return None;
                    }
                }
            }
        }

        // Check cancellation before inserting into cache
        if cancellation.is_cancelled() {
            debug!(pack = %pack_name, "Load cancelled before caching");
            return None;
        }

        let data = PackData {
            name: pack_name.to_string(),
            version: "0.0.0".to_string(),
            document_count: doc_count,
            path: pack_path,
        };

        self.loaded.lock().await.insert(pack_name.to_string(), data.clone());

        info!(
            pack = %pack_name,
            doc_count = doc_count,
            "Pack loaded and cached"
        );

        Some(data)
    }

    /// Gets cached data for a pack without loading it first.
    pub async fn get_cached(&self, pack_name: &str) -> Option<PackData> {
        let loaded = self.loaded.lock().await;
        loaded.get(pack_name).cloned()
    }

    /// Returns all registered pack names.
    pub async fn registered_packs(&self) -> Vec<String> {
        let packs = self.packs.lock().await;
        packs.keys().cloned().collect()
    }

    /// Clears all cached data.
    pub async fn clear_cache(&self) {
        self.loaded.lock().await.clear();
        debug!("Cache cleared");
    }

    /// Returns the number of loaded (cached) packs.
    pub async fn loaded_count(&self) -> usize {
        let loaded = self.loaded.lock().await;
        loaded.len()
    }

    /// Returns the number of registered but not yet loaded packs.
    pub async fn unloaded_count(&self) -> usize {
        let packs = self.packs.lock().await;
        let loaded = self.loaded.lock().await;
        packs
            .keys()
            .filter(|k| !loaded.contains_key(*k))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_loader() -> LazyPackLoader {
        LazyPackLoader::new()
    }

    #[tokio::test]
    async fn test_register_and_register_again() {
        let loader = make_loader();
        let tmp = TempDir::new().unwrap();

        loader.register_pack("test-pack", tmp.path().to_path_buf()).await;

        let packs = loader.registered_packs().await;
        assert_eq!(packs.len(), 1);
        assert!(packs.contains(&"test-pack".to_string()));
    }

    #[tokio::test]
    async fn test_unregister() {
        let loader = make_loader();
        let tmp = TempDir::new().unwrap();

        loader.register_pack("test-pack", tmp.path().to_path_buf()).await;
        loader.unregister_pack("test-pack").await;

        let packs = loader.registered_packs().await;
        assert!(packs.is_empty());
    }

    #[tokio::test]
    async fn test_load_with_manifest() {
        let loader = make_loader();
        let tmp = TempDir::new().unwrap();

        fs::write(
            tmp.path().join("manifest.yaml"),
            "schema_version: '1.0'\nname: test-pack\nversion: '2.0.0'\ndescription: test\nauthor: test\nlicense: MIT\nformat_version: '1.0'\ndocuments:\n  - id: doc1\n    path: doc1.md\n    format: markdown\n    embed: true\n  - id: doc2\n    path: doc2.md\n    format: markdown\n    embed: true\ndependencies: []\n",
        )
        .unwrap();

        loader.register_pack("test-pack", tmp.path().to_path_buf()).await;

        let data = loader
            .load_pack("test-pack", CancellationToken::new())
            .await;

        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.name, "test-pack");
        assert_eq!(data.document_count, 2);
    }

    #[tokio::test]
    async fn test_load_cached() {
        let loader = make_loader();
        let tmp = TempDir::new().unwrap();

        loader.register_pack("test-pack", tmp.path().to_path_buf()).await;

        // First load
        let data1 = loader
            .load_pack("test-pack", CancellationToken::new())
            .await;
        assert!(data1.is_some());

        // Second load — should be cached
        let data2 = loader.get_cached("test-pack").await;
        assert!(data2.is_some());
        assert_eq!(data2.as_ref().unwrap().name, "test-pack");
    }

    #[tokio::test]
    async fn test_cancelled_load() {
        let loader = make_loader();
        let tmp = TempDir::new().unwrap();

        loader.register_pack("test-pack", tmp.path().to_path_buf()).await;

        let token = CancellationToken::new();
        token.cancel();

        let data = loader.load_pack("test-pack", token).await;
        assert!(data.is_none());
    }

    #[tokio::test]
    async fn test_load_nonexistent() {
        let loader = make_loader();
        let token = CancellationToken::new();

        let data = loader.load_pack("nonexistent", token).await;
        assert!(data.is_none());
    }
}