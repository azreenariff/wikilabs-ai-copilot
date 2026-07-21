//! Performance benchmarking module for Wiki Labs AI Copilot v1.0.0.
//!
//! ## Overview
//!
//! This crate provides structured performance measurement for critical code paths:
//!
//! - **Application Startup** — Time from process start to ready state
//! - **AI Provider Communication** — Request/response latency for AI calls
//! - **Knowledge Indexing** — Document pipeline processing time
//! - **Skill Loading** — Skill module initialization time
//! - **Screen Capture / OCR** — Observation pipeline latency
//! - **Large Conversation Handling** — Context assembly time for long sessions
//!
//! ## Usage
//!
//! ```rust
//! use wikilabs_benchmark::{Benchmark, BenchmarkMetrics, BenchmarkTimer};
//!
//! // Create a named benchmark timer
//! let mut timer = BenchmarkTimer::new("startup");
//!
//! // ... perform work ...
//!
//! // Record the result
//! let metric = timer.finish();
//! ```
//!
//! ## Metrics Integration
//!
//! `BenchmarkMetrics` aggregates results across all runs and is designed to be
//! embedded in the diagnostics report produced by `src-tauri/src/config.rs`.

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Individual benchmark measurement result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Unique measurement ID (UUID v4).
    pub id: String,
    /// Name of the benchmark (e.g., "startup", "ai_response", "indexing").
    pub name: String,
    /// Wall-clock start time as UNIX timestamp (seconds).
    pub started_at: u64,
    /// Duration of the measured operation in nanoseconds.
    pub duration_ns: u64,
    /// Human-readable duration string.
    pub duration_human: String,
    /// Optional metadata with additional context.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Duration formatting helpers.
impl BenchmarkResult {
    /// Format the duration in the most appropriate unit.
    pub fn duration_formatted(&self) -> String {
        let ns = self.duration_ns;
        if ns < 1_000 {
            format!("{ns} ns")
        } else if ns < 1_000_000 {
            format!("{} µs", ns / 1_000)
        } else if ns < 1_000_000_000 {
            format!("{} ms", ns / 1_000_000)
        } else {
            format!("{:.2} s", ns as f64 / 1_000_000_000.0)
        }
    }
}

/// A timer that records the duration of an operation.
#[derive(Debug)]
pub struct BenchmarkTimer {
    name: String,
    start: Instant,
    metadata: HashMap<String, String>,
}

impl BenchmarkTimer {
    /// Create a new benchmark timer with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
            metadata: HashMap::new(),
        }
    }

    /// Attach arbitrary metadata to this measurement.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Finish the timer and return a [`BenchmarkResult`].
    pub fn finish(self) -> BenchmarkResult {
        let duration = self.start.elapsed();
        let duration_ns = duration.as_nanos() as u64;

        BenchmarkResult {
            id: format!(
                "{}_{}",
                self.name,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            ),
            name: self.name,
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            duration_ns,
            duration_human: (((duration.as_micros() as f64) / 1_000.0).ceil() as u64).to_string(),
            metadata: self.metadata,
        }
    }
}

/// Aggregated metrics for a named benchmark across multiple runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    /// Benchmark name.
    pub name: String,
    /// Number of runs recorded.
    pub run_count: usize,
    /// Shortest duration in nanoseconds.
    pub min_ns: u64,
    /// Longest duration in nanoseconds.
    pub max_ns: u64,
    /// Mean duration in nanoseconds.
    pub mean_ns: u64,
    /// Most recent duration in nanoseconds.
    pub latest_ns: u64,
    /// All recorded results (kept for inspection).
    #[serde(default)]
    pub results: Vec<BenchmarkResult>,
}

impl BenchmarkMetrics {
    /// Create a new empty metrics container.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            run_count: 0,
            min_ns: u64::MAX,
            max_ns: 0,
            mean_ns: 0,
            latest_ns: 0,
            results: Vec::new(),
        }
    }

    /// Record a new benchmark result.
    pub fn record(&mut self, result: BenchmarkResult) {
        let ns = result.duration_ns;
        self.run_count += 1;
        self.min_ns = self.min_ns.min(ns);
        self.max_ns = self.max_ns.max(ns);
        self.latest_ns = ns;
        // Update running mean
        self.mean_ns = (self.mean_ns * ((self.run_count - 1) as u64) + ns) / (self.run_count as u64);
        self.results.push(result);
    }

    /// Format key stats for display.
    pub fn summary(&self) -> String {
        format!(
            "{}: n={} min={}µs max={}µs mean={}µs latest={}µs",
            self.name,
            self.run_count,
            self.min_ns / 1_000,
            self.max_ns / 1_000,
            self.mean_ns / 1_000,
            self.latest_ns / 1_000,
        )
    }

    /// Create a JSON-serializable summary for diagnostics.
    pub fn diagnostic_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "run_count": self.run_count,
            "min_ns": self.min_ns,
            "max_ns": self.max_ns,
            "mean_ns": self.mean_ns,
            "latest_ns": self.latest_ns,
        })
    }
}

/// Registry that tracks metrics for all named benchmarks.
#[derive(Debug, Default, Clone)]
pub struct BenchmarkRegistry {
    metrics: HashMap<String, BenchmarkMetrics>,
}

impl BenchmarkRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    /// Get or create a named metrics bucket.
    pub fn get_or_create(&mut self, name: &str) -> &mut BenchmarkMetrics {
        self.metrics
            .entry(name.to_string())
            .or_insert_with(|| BenchmarkMetrics::new(name))
    }

    /// Record a benchmark result.
    pub fn record(&mut self, result: BenchmarkResult) {
        self.get_or_create(&result.name).record(result);
    }

    /// Get the metrics for a named benchmark.
    pub fn get(&self, name: &str) -> Option<&BenchmarkMetrics> {
        self.metrics.get(name)
    }

    /// Get all metrics as a JSON value for diagnostics.
    pub fn to_diagnostics(&self) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        for (name, m) in &self.metrics {
            obj.insert(name.clone(), m.diagnostic_summary());
        }
        serde_json::Value::Object(obj)
    }

    /// Get a human-readable summary of all registered metrics.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        for m in self.metrics.values() {
            lines.push(m.summary());
        }
        lines.join(" | ")
    }
}

// ── Predefined Benchmark Names ────────────────────────────────────────

/// Category names for organized benchmark tracking.
pub mod categories {
    /// Application startup from process launch to ready state.
    pub const STARTUP: &str = "startup";

    /// AI provider communication — request to response.
    pub const AI_RESPONSE: &str = "ai_response";

    /// Knowledge base document indexing pipeline.
    pub const KNOWLEDGE_INDEXING: &str = "knowledge_indexing";

    /// Skill module loading and initialization.
    pub const SKILL_LOADING: &str = "skill_loading";

    /// Screen capture and OCR pipeline latency.
    pub const SCREEN_CAPTURE: &str = "screen_capture";

    /// OCR text extraction time.
    pub const OCR_PROCESSING: &str = "ocr_processing";

    /// Large conversation context assembly time.
    pub const LARGE_CONVERSATION: &str = "large_conversation";

    /// Database query latency.
    pub const DB_QUERY: &str = "db_query";

    /// Settings/configuration load time.
    pub const CONFIG_LOAD: &str = "config_load";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_timer() {
        let timer = BenchmarkTimer::new("test");
        std::thread::sleep(Duration::from_millis(10));
        let result = timer.finish();
        assert_eq!(result.name, "test");
        assert!(result.duration_ns > 10_000_000); // > 10ms in ns
    }

    #[test]
    fn test_benchmark_metrics_aggregation() {
        let mut metrics = BenchmarkMetrics::new("agg_test");
        for i in 0..5 {
            let result = BenchmarkResult {
                id: format!("test_{}", i),
                name: "agg_test".to_string(),
                started_at: 0,
                duration_ns: 100_000_000 + i * 10_000_000, // 100-140ms in ns
                duration_human: "".to_string(),
                metadata: HashMap::new(),
            };
            metrics.record(result);
        }
        assert_eq!(metrics.run_count, 5);
        assert_eq!(metrics.min_ns, 100_000_000);
        assert_eq!(metrics.max_ns, 140_000_000);
    }

    #[test]
    fn test_benchmark_registry() {
        let mut registry = BenchmarkRegistry::new();
        let result = BenchmarkResult {
            id: "reg_test".to_string(),
            name: "startup".to_string(),
            started_at: 0,
            duration_ns: 50_000_000,
            duration_human: "".to_string(),
            metadata: HashMap::new(),
        };
        registry.record(result);

        let metrics = registry.get("startup").unwrap();
        assert_eq!(metrics.run_count, 1);
        assert_eq!(metrics.latest_ns, 50_000_000);

        // Second benchmark category
        let result2 = BenchmarkResult {
            id: "reg_test_2".to_string(),
            name: "ai_response".to_string(),
            started_at: 0,
            duration_ns: 200_000_000,
            duration_human: "".to_string(),
            metadata: HashMap::new(),
        };
        registry.record(result2);
        assert_eq!(registry.get("ai_response").unwrap().run_count, 1);
    }

    #[test]
    fn test_duration_formatted() {
        let r = BenchmarkResult {
            id: "fmt".to_string(),
            name: "test".to_string(),
            started_at: 0,
            duration_ns: 500,
            duration_human: "".to_string(),
            metadata: HashMap::new(),
        };
        assert_eq!(r.duration_formatted(), "500 ns");

        let r = BenchmarkResult {
            id: "fmt".to_string(),
            name: "test".to_string(),
            started_at: 0,
            duration_ns: 50_000,
            duration_human: "".to_string(),
            metadata: HashMap::new(),
        };
        assert_eq!(r.duration_formatted(), "50 µs");

        let r = BenchmarkResult {
            id: "fmt".to_string(),
            name: "test".to_string(),
            started_at: 0,
            duration_ns: 50_000_000,
            duration_human: "".to_string(),
            metadata: HashMap::new(),
        };
        assert_eq!(r.duration_formatted(), "50 ms");

        let r = BenchmarkResult {
            id: "fmt".to_string(),
            name: "test".to_string(),
            started_at: 0,
            duration_ns: 1_500_000_000,
            duration_human: "".to_string(),
            metadata: HashMap::new(),
        };
        assert_eq!(r.duration_formatted(), "1.50 s");
    }

    #[test]
    fn test_diagnostic_summary() {
        let mut metrics = BenchmarkMetrics::new("diag_test");
        metrics.record(BenchmarkResult {
            id: "d1".to_string(),
            name: "diag_test".to_string(),
            started_at: 1000,
            duration_ns: 100_000_000,
            duration_human: "".to_string(),
            metadata: HashMap::new(),
        });
        let summary = metrics.diagnostic_summary();
        assert_eq!(summary["run_count"], 1);
        assert_eq!(summary["min_ns"], 100_000_000);
    }
}