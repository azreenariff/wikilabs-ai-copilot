//! Memory usage limits for knowledge operations.

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

/// Memory usage limit configuration.
pub struct MemoryLimitConfig {
    /// Maximum memory usage in bytes (soft limit).
    /// When reached, operations will be throttled.
    pub soft_limit_bytes: u64,
    /// Hard limit in bytes. When reached, operations will be cancelled.
    pub hard_limit_bytes: u64,
}

impl Default for MemoryLimitConfig {
    fn default() -> Self {
        // Default: 512MB soft, 1GB hard
        Self {
            soft_limit_bytes: 512 * 1024 * 1024,
            hard_limit_bytes: 1024 * 1024 * 1024,
        }
    }
}

/// Tracks current memory usage and enforces limits.
///
/// This is a heuristic tracker that estimates memory usage based on
/// the size of cached data, indexed vectors, and document content.
/// It does NOT use the OS RSS — that would require unsafe syscalls.
pub struct MemoryLimit {
    config: MemoryLimitConfig,
    /// Current estimated usage in bytes.
    current_usage: Arc<Mutex<u64>>,
}

impl MemoryLimit {
    /// Creates a new memory limit tracker.
    pub fn new(config: MemoryLimitConfig) -> Self {
        Self {
            config,
            current_usage: Arc::new(Mutex::new(0)),
        }
    }

    /// Creates a new memory limit tracker with default config.
    pub fn default_limit() -> Self {
        Self::new(MemoryLimitConfig::default())
    }

    /// Records estimated memory usage for a value.
    pub async fn record_usage(&self, estimated_bytes: u64) {
        let mut usage = self.current_usage.lock().await;
        *usage += estimated_bytes;

        let soft = self.config.soft_limit_bytes;
        let hard = self.config.hard_limit_bytes;
        let current = *usage;

        if current > hard {
            debug!(
                current = current,
                hard_limit = hard,
                "Hard memory limit exceeded!"
            );
        } else if current > soft {
            debug!(
                current = current,
                soft_limit = soft,
                "Soft memory limit exceeded — throttling may be needed"
            );
        }
    }

    /// Releases previously recorded usage.
    pub async fn release_usage(&self, estimated_bytes: u64) {
        let mut usage = self.current_usage.lock().await;
        *usage = (*usage).saturating_sub(estimated_bytes);
    }

    /// Returns the current estimated usage in bytes.
    pub async fn current_usage(&self) -> u64 {
        *self.current_usage.lock().await
    }

    /// Returns whether the soft limit has been exceeded.
    pub async fn exceeded_soft_limit(&self) -> bool {
        let usage = self.current_usage.lock().await;
        usage > self.config.soft_limit_bytes
    }

    /// Returns whether the hard limit has been exceeded.
    pub async fn exceeded_hard_limit(&self) -> bool {
        let usage = self.current_usage.lock().await;
        usage > self.config.hard_limit_bytes
    }

    /// Returns the configured soft limit.
    pub fn soft_limit(&self) -> u64 {
        self.config.soft_limit_bytes
    }

    /// Returns the configured hard limit.
    pub fn hard_limit(&self) -> u64 {
        self.config.hard_limit_bytes
    }

    /// Returns the memory usage as a human-readable string.
    pub async fn usage_string(&self) -> String {
        format_bytes(self.current_usage().await)
    }

    /// Returns the capacity remaining until the hard limit.
    pub async fn remaining_bytes(&self) -> u64 {
        let usage = self.current_usage.lock().await;
        self.config.hard_limit_bytes.saturating_sub(*usage)
    }

    /// Estimates the byte size of a string.
    pub fn estimate_string_bytes(s: &str) -> u64 {
        s.len() as u64
    }

    /// Estimates the byte size of a vector of f32.
    pub fn estimate_vector_bytes(dimensions: usize) -> u64 {
        (dimensions * std::mem::size_of::<f32>()) as u64
    }

    /// Checks if there is budget for an operation of the given size.
    pub async fn has_budget(&self, estimated_bytes: u64) -> bool {
        let usage = self.current_usage.lock().await;
        usage.saturating_add(estimated_bytes) <= self.config.hard_limit_bytes
    }
}

/// Formats bytes into a human-readable string (e.g., "1.2 MB").
pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[tokio::test]
    async fn test_record_and_release() {
        let limit = MemoryLimit::default_limit();

        assert_eq!(limit.current_usage().await, 0);

        limit.record_usage(1000).await;
        assert_eq!(limit.current_usage().await, 1000);

        limit.record_usage(2000).await;
        assert_eq!(limit.current_usage().await, 3000);

        limit.release_usage(1000).await;
        assert_eq!(limit.current_usage().await, 2000);
    }

    #[tokio::test]
    async fn test_limit_detection() {
        let config = MemoryLimitConfig {
            soft_limit_bytes: 100,
            hard_limit_bytes: 200,
        };
        let limit = MemoryLimit::new(config);

        assert!(!limit.exceeded_soft_limit().await);
        assert!(!limit.exceeded_hard_limit().await);

        limit.record_usage(150).await;
        assert!(limit.exceeded_soft_limit().await);
        assert!(!limit.exceeded_hard_limit().await);

        limit.record_usage(100).await;
        assert!(limit.exceeded_hard_limit().await);
    }

    #[tokio::test]
    async fn test_has_budget() {
        let config = MemoryLimitConfig {
            soft_limit_bytes: 500,
            hard_limit_bytes: 1000,
        };
        let limit = MemoryLimit::new(config);

        assert!(limit.has_budget(500).await);

        limit.record_usage(600).await;
        assert!(!limit.has_budget(500).await);
    }

    #[tokio::test]
    async fn test_estimate_string_bytes() {
        assert_eq!(MemoryLimit::estimate_string_bytes(""), 0);
        assert_eq!(MemoryLimit::estimate_string_bytes("hello"), 5);
    }

    #[tokio::test]
    async fn test_estimate_vector_bytes() {
        assert_eq!(MemoryLimit::estimate_vector_bytes(0), 0);
        assert_eq!(MemoryLimit::estimate_vector_bytes(384), 384 * 4); // f32 = 4 bytes
        assert_eq!(MemoryLimit::estimate_vector_bytes(768), 768 * 4);
    }

    #[tokio::test]
    async fn test_usage_string() {
        let limit = MemoryLimit::default_limit();
        assert_eq!(limit.usage_string().await, "0 B");

        limit.record_usage(1024).await;
        let usage_str = limit.usage_string().await;
        assert!(usage_str.contains("KB"));
    }
}