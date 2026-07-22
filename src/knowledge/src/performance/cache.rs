//! LRU cache for recent retrieval results.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::debug;

/// A single cache entry with a time-to-live.
struct CacheEntry<T> {
    value: T,
    #[allow(dead_code)]
    created_at: Instant,
    expires_at: Instant,
}

/// An LRU cache for knowledge retrieval results.
///
/// Uses a hybrid of a HashMap for O(1) lookups and a sorted access-order
/// list for LRU eviction. When the cache exceeds `max_capacity`, the least
/// recently used entry is evicted. Entries also expire after `ttl`.
#[allow(dead_code)]
pub struct RetrievalCache<T> {
    max_capacity: usize,
    ttl: Duration,
    entries: HashMap<String, CacheEntry<T>>,
    access_order: Vec<String>,
}

impl<T> RetrievalCache<T> {
    /// Creates a new cache with the given max capacity and TTL.
    pub fn new(max_capacity: usize, ttl: Duration) -> Self {
        Self {
            max_capacity,
            ttl,
            entries: HashMap::new(),
            access_order: Vec::new(),
        }
    }

    /// Gets a value from the cache. Returns None if not found or expired.
    pub fn get(&mut self, key: &str) -> Option<&T> {
        let key_owned = key.to_string();

        // Check if entry exists and is not expired
        let entry = self.entries.get(&key_owned)?;
        if Instant::now() > entry.expires_at {
            self.remove(&key_owned);
            return None;
        }

        // Update access order for LRU
        self.access_order.retain(|k| k != &key_owned);
        self.access_order.push(key_owned.clone());

        self.entries.get(&key_owned).map(|e| &e.value)
    }

    /// Inserts a value into the cache, evicting if necessary.
    pub fn put(&mut self, key: String, value: T) {
        let now = Instant::now();
        let expires_at = now + self.ttl;

        // If key already exists, update it and refresh access order
        if self.entries.contains_key(&key) {
            let entry = self.entries.get_mut(&key).unwrap();
            entry.value = value;
            entry.expires_at = expires_at;
            self.access_order.retain(|k| k != &key);
            self.access_order.push(key.clone());
            return;
        }

        // Evict expired entries
        self.evict_expired();

        // Evict LRU entries if over capacity
        while self.entries.len() >= self.max_capacity {
            if let Some(oldest_key) = self.access_order.first().cloned() {
                self.remove(&oldest_key);
            } else {
                break;
            }
        }

        // Insert new entry
        let access_key = key.clone();
        self.entries.insert(
            key,
            CacheEntry {
                value,
                created_at: now,
                expires_at,
            },
        );
        self.access_order.push(access_key.clone());

        debug!(key = %access_key, cache_size = self.entries.len(), "Cached retrieval result");
    }

    /// Removes a specific entry from the cache.
    pub fn remove(&mut self, key: &str) {
        self.entries.remove(key);
        self.access_order.retain(|k| k != key);
    }

    /// Clears all entries from the cache.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        debug!("Cache cleared");
    }

    /// Evicts all expired entries.
    fn evict_expired(&mut self) {
        let now = Instant::now();
        let expired_keys: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| now > entry.expires_at)
            .map(|(key, _)| key.clone())
            .collect();

        for key in &expired_keys {
            self.entries.remove(key);
        }

        if !expired_keys.is_empty() {
            debug!(
                expired = expired_keys.len(),
                "Evicted expired cache entries"
            );
        }
    }

    /// Returns the number of entries in the cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of hits/misses (approximate — not tracked precisely).
    pub fn hit_rate(&self) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        (self.entries.len() as f64) / (self.max_capacity as f64)
    }

    /// Returns the configured max capacity.
    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get() {
        let mut cache: RetrievalCache<String> = RetrievalCache::new(10, Duration::from_secs(60));

        cache.put("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let mut cache: RetrievalCache<String> = RetrievalCache::new(10, Duration::from_secs(60));

        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache: RetrievalCache<String> = RetrievalCache::new(3, Duration::from_secs(60));

        cache.put("a".to_string(), "1".to_string());
        cache.put("b".to_string(), "2".to_string());
        cache.put("c".to_string(), "3".to_string());
        // Cache is now full

        // Access "a" to make it recently used
        cache.get("a");

        // Insert "d" — should evict "b" (least recently used)
        cache.put("d".to_string(), "4".to_string());

        assert_eq!(cache.get("a"), Some(&"1".to_string()));
        assert!(cache.get("b").is_none()); // evicted
        assert_eq!(cache.get("c"), Some(&"3".to_string()));
        assert_eq!(cache.get("d"), Some(&"4".to_string()));
    }

    #[test]
    fn test_ttl_expiry() {
        let mut cache: RetrievalCache<String> = RetrievalCache::new(10, Duration::from_millis(50));

        cache.put("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get("key1"), Some(&"value1".to_string()));

        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(100));

        assert!(cache.get("key1").is_none());
    }

    #[test]
    fn test_clear() {
        let mut cache: RetrievalCache<String> = RetrievalCache::new(10, Duration::from_secs(60));

        cache.put("a".to_string(), "1".to_string());
        cache.put("b".to_string(), "2".to_string());
        cache.clear();

        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_update_existing_key() {
        let mut cache: RetrievalCache<String> = RetrievalCache::new(10, Duration::from_secs(60));

        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key1".to_string(), "value2".to_string());

        assert_eq!(cache.get("key1"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut cache: RetrievalCache<String> = RetrievalCache::new(10, Duration::from_secs(60));

        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        cache.put("a".to_string(), "1".to_string());
        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 1);
    }
}
