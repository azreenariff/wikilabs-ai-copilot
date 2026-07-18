//! Engineering timeline entries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A timeline entry — a single record of engineering activity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineEntry {
    /// Unique entry ID.
    pub id: Uuid,
    /// Timestamp of the event.
    pub timestamp: DateTime<Utc>,
    /// Human-readable label (e.g., "Detected Kubernetes config").
    pub label: String,
    /// Source of this entry ("observation", "intent", "correction", "user").
    pub source: String,
    /// Free-form detail text.
    pub detail: String,
}

impl TimelineEntry {
    /// Create a new timeline entry with current timestamp.
    pub fn new(label: impl Into<String>, source: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            label: label.into(),
            source: source.into(),
            detail: detail.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_entry_creation() {
        let entry = TimelineEntry::new("Detected Rust project", "observation", "Cargo.toml found");
        assert_eq!(entry.source, "observation");
        assert_eq!(entry.label, "Detected Rust project");
        assert!(entry.id.is_nil() == false);
    }

    #[test]
    fn test_timeline_entry_serialization() {
        let entry = TimelineEntry::new("Test", "user", "test detail");
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: TimelineEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.label, "Test");
        assert_eq!(deserialized.detail, "test detail");
    }
}