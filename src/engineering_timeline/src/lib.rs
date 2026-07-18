//! Engineering Timeline — Phase 7
//!
//! Maintains a chronological record of engineering activity, linking back to
//! source observation events.
//!
//! ## Architecture
//!
//! ```text
//! Observation Framework
//!     → EngineeringTimeline (append entries with timestamps)
//!     → TimelineEntry (id, timestamp, label, source, detail, related_observation_id)
//!     → Queries: by range, by source, by label pattern
//! ```
//!
//! ## Core Principles
//!
//! - Entries reference source observation events
//! - Chronological ordering is guaranteed
//! - Timeline is append-only (entries are never modified)
//! - Supports filtering by source, label pattern, and time range

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Timeline entry
// ---------------------------------------------------------------------------

/// A single timeline entry — a record of engineering activity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineEntry {
    /// Unique entry ID.
    pub id: Uuid,
    /// When this activity occurred.
    pub timestamp: DateTime<Utc>,
    /// Human-readable label (e.g., "Opened OpenShift Console").
    pub label: String,
    /// Source of this entry (observation, intent, correction, user).
    pub source: TimelineSource,
    /// Free-form detail text.
    pub detail: String,
    /// Reference to the source observation event ID (if any).
    pub related_observation_id: Option<Uuid>,
}

/// Where this timeline entry came from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TimelineSource {
    /// From the observation framework (command, browser, app event).
    Observation,
    /// From intent recognition.
    Intent,
    /// From human feedback / correction.
    Correction,
    /// Directly from the user.
    User,
    /// From the technology recognition engine.
    TechnologyRecognition,
    /// From the workflow engine.
    Workflow,
}

impl std::fmt::Display for TimelineSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimelineSource::Observation => write!(f, "observation"),
            TimelineSource::Intent => write!(f, "intent"),
            TimelineSource::Correction => write!(f, "correction"),
            TimelineSource::User => write!(f, "user"),
            TimelineSource::TechnologyRecognition => write!(f, "technology_recognition"),
            TimelineSource::Workflow => write!(f, "workflow"),
        }
    }
}

// ---------------------------------------------------------------------------
// Timeline engine
// ---------------------------------------------------------------------------

/// Engine that maintains the engineering activity timeline.
///
/// All entries are append-only and chronologically ordered.
pub struct EngineeringTimeline {
    entries: Vec<TimelineEntry>,
    max_entries: usize,
}

impl EngineeringTimeline {
    /// Create a new timeline with the given max entry limit.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries),
            max_entries,
        }
    }

    /// Create a timeline with a default max of 10000 entries.
    pub fn default_max() -> Self {
        Self::new(10_000)
    }

    // ------------------------------------------------------------------
    // Entry addition
    // ------------------------------------------------------------------

    /// Add a new timeline entry.
    ///
    /// Returns the UUID of the newly added entry.
    pub fn append(&mut self, label: String, source: TimelineSource, detail: String) -> Uuid {
        let entry = TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            label,
            source,
            detail,
            related_observation_id: None,
        };
        self.append_entry(entry)
    }

    /// Add a new timeline entry with a reference to an observation event.
    pub fn append_with_observation(
        &mut self,
        label: String,
        source: TimelineSource,
        detail: String,
        observation_id: Uuid,
    ) -> Uuid {
        let entry = TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            label,
            source,
            detail,
            related_observation_id: Some(observation_id),
        };
        self.append_entry(entry)
    }

    /// Add a pre-built entry.
    fn append_entry(&mut self, entry: TimelineEntry) -> Uuid {
        let id = entry.id;

        // Ensure chronological order (entries should already be ordered)
        if let Some(last) = self.entries.last() {
            if entry.timestamp < last.timestamp {
                // Insert in correct position
                let pos = self
                    .entries
                    .iter()
                    .position(|e| e.timestamp > entry.timestamp)
                    .unwrap_or(self.entries.len());
                self.entries.insert(pos, entry);
            } else {
                self.entries.push(entry);
            }
        } else {
            self.entries.push(entry);
        }

        // Enforce max entries (remove oldest if over limit)
        if self.entries.len() > self.max_entries {
            self.entries.drain(0..self.entries.len() - self.max_entries);
        }

        id
    }

    // ------------------------------------------------------------------
    // Query methods
    // ------------------------------------------------------------------

    /// Get all entries.
    pub fn get_all(&self) -> &[TimelineEntry] {
        &self.entries
    }

    /// Get entries within a time range.
    pub fn get_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<TimelineEntry> {
        self.entries.iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get entries from the last N minutes.
    pub fn get_recent(&self, minutes: u64) -> Vec<TimelineEntry> {
        let cutoff = Utc::now() - Duration::minutes(minutes as i64);
        self.get_range(cutoff, Utc::now())
    }

    /// Get entries filtered by source.
    pub fn get_by_source(&self, source: &TimelineSource) -> Vec<TimelineEntry> {
        self.entries.iter()
            .filter(|e| &e.source == source)
            .cloned()
            .collect()
    }

    /// Get entries matching a label pattern (case-insensitive substring).
    pub fn get_by_label_pattern(&self, pattern: &str) -> Vec<TimelineEntry> {
        if self.entries.is_empty() || pattern.is_empty() {
            return Vec::new();
        }
        let pattern_lower = pattern.to_lowercase();
        self.entries.iter()
            .filter(|e| e.label.to_lowercase().contains(&pattern_lower))
            .cloned()
            .collect()
    }

    /// Get the most recent N entries.
    pub fn get_last(&self, n: usize) -> &[TimelineEntry] {
        let start = if self.entries.len() > n {
            self.entries.len() - n
        } else {
            0
        };
        &self.entries[start..]
    }

    /// Get entries for a specific observation event.
    pub fn get_by_observation(&self, observation_id: Uuid) -> Vec<TimelineEntry> {
        self.entries.iter()
            .filter(|e| e.related_observation_id == Some(observation_id))
            .cloned()
            .collect()
    }

    /// Get total entry count.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the timeline is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the first entry (oldest).
    pub fn first(&self) -> Option<&TimelineEntry> {
        self.entries.first()
    }

    /// Get the last entry (newest).
    pub fn last(&self) -> Option<&TimelineEntry> {
        self.entries.last()
    }

    /// Get a summary of activity by source.
    pub fn summary_by_source(&self) -> std::collections::HashMap<TimelineSource, usize> {
        let mut counts = std::collections::HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.source.clone()).or_insert(0) += 1;
        }
        counts
    }

    // ------------------------------------------------------------------
    // Serialization
    // ------------------------------------------------------------------

    /// Serialize the timeline to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.entries)
    }

    /// Deserialize the timeline from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let entries: Vec<TimelineEntry> = serde_json::from_str(json)?;
        Ok(Self {
            entries,
            max_entries: 10_000,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_creation() {
        let timeline = EngineeringTimeline::default_max();
        assert!(timeline.is_empty());
        assert_eq!(timeline.len(), 0);
        assert!(timeline.first().is_none());
        assert!(timeline.last().is_none());
    }

    #[test]
    fn test_append_entry() {
        let mut timeline = EngineeringTimeline::new(1000);
        let id = timeline.append(
            "Test event".to_string(),
            TimelineSource::Observation,
            "Test detail".to_string(),
        );
        assert!(!id.as_hyphenated().to_string().is_empty());
        assert_eq!(timeline.len(), 1);
        assert_eq!(timeline.first().unwrap().label, "Test event");
    }

    #[test]
    fn test_append_with_observation() {
        let mut timeline = EngineeringTimeline::new(1000);
        let obs_id = Uuid::new_v4();
        let id = timeline.append_with_observation(
            "Test event".to_string(),
            TimelineSource::Observation,
            "Test detail".to_string(),
            obs_id,
        );
        let entry = timeline.get_all().first().unwrap();
        assert_eq!(entry.related_observation_id, Some(obs_id));
    }

    #[test]
    fn test_get_recent() {
        let mut timeline = EngineeringTimeline::new(1000);

        // Add a recent entry
        timeline.append("Recent".to_string(), TimelineSource::Observation, "detail".to_string());

        // Add an old entry (simulate 2 hours ago)
        let old_id = Uuid::new_v4();
        let mut entry = TimelineEntry {
            id: old_id,
            timestamp: Utc::now() - Duration::hours(2),
            label: "Old".to_string(),
            source: TimelineSource::Observation,
            detail: "old detail".to_string(),
            related_observation_id: None,
        };
        timeline.entries.push(entry);

        let recent = timeline.get_recent(10);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].label, "Recent");
    }

    #[test]
    fn test_get_by_source() {
        let mut timeline = EngineeringTimeline::new(1000);

        timeline.append(
            "Obs event".to_string(),
            TimelineSource::Observation,
            "detail".to_string(),
        );
        timeline.append(
            "Intent event".to_string(),
            TimelineSource::Intent,
            "detail".to_string(),
        );
        timeline.append(
            "Another obs".to_string(),
            TimelineSource::Observation,
            "detail".to_string(),
        );

        let obs_entries = timeline.get_by_source(&TimelineSource::Observation);
        assert_eq!(obs_entries.len(), 2);

        let intent_entries = timeline.get_by_source(&TimelineSource::Intent);
        assert_eq!(intent_entries.len(), 1);
    }

    #[test]
    fn test_get_by_label_pattern() {
        let mut timeline = EngineeringTimeline::new(1000);

        timeline.append(
            "Executed oc get pods".to_string(),
            TimelineSource::Observation,
            "detail".to_string(),
        );
        timeline.append(
            "Checked oc version".to_string(),
            TimelineSource::Observation,
            "detail".to_string(),
        );
        timeline.append(
            "Viewed pod logs".to_string(),
            TimelineSource::Observation,
            "detail".to_string(),
        );

        let oc_entries = timeline.get_by_label_pattern("oc");
        assert_eq!(oc_entries.len(), 2);

        let empty = timeline.get_by_label_pattern("nonexistent");
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_get_by_observation() {
        let mut timeline = EngineeringTimeline::new(1000);
        let obs_id1 = Uuid::new_v4();
        let obs_id2 = Uuid::new_v4();

        timeline.append_with_observation(
            "Event 1".to_string(),
            TimelineSource::Observation,
            "detail".to_string(),
            obs_id1,
        );
        timeline.append_with_observation(
            "Event 2".to_string(),
            TimelineSource::Observation,
            "detail".to_string(),
            obs_id2,
        );
        timeline.append_with_observation(
            "Event 3".to_string(),
            TimelineSource::Observation,
            "detail".to_string(),
            obs_id1,
        );

        let entries1 = timeline.get_by_observation(obs_id1);
        assert_eq!(entries1.len(), 2);

        let entries2 = timeline.get_by_observation(obs_id2);
        assert_eq!(entries2.len(), 1);
    }

    #[test]
    fn test_get_range() {
        let mut timeline = EngineeringTimeline::new(1000);

        let now = Utc::now();
        let old = now - Duration::hours(3);
        let recent = now - Duration::minutes(30);

        // Old entry
        timeline.entries.push(TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: old,
            label: "Old".to_string(),
            source: TimelineSource::Observation,
            detail: "detail".to_string(),
            related_observation_id: None,
        });

        // Recent entry
        timeline.entries.push(TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: recent,
            label: "Recent".to_string(),
            source: TimelineSource::Observation,
            detail: "detail".to_string(),
            related_observation_id: None,
        });

        let range = timeline.get_range(old, now);
        assert_eq!(range.len(), 2);

        let recent_range = timeline.get_range(recent, now);
        assert_eq!(recent_range.len(), 1);
        assert_eq!(recent_range[0].label, "Recent");
    }

    #[test]
    fn test_get_last() {
        let mut timeline = EngineeringTimeline::new(1000);
        timeline.append("1".to_string(), TimelineSource::Observation, "detail".to_string());
        timeline.append("2".to_string(), TimelineSource::Observation, "detail".to_string());
        timeline.append("3".to_string(), TimelineSource::Observation, "detail".to_string());

        let last_2 = timeline.get_last(2);
        assert_eq!(last_2.len(), 2);
        assert_eq!(last_2[0].label, "2");
        assert_eq!(last_2[1].label, "3");
    }

    #[test]
    fn test_max_entries_enforcement() {
        let mut timeline = EngineeringTimeline::new(5);

        for i in 0..10 {
            timeline.append(
                format!("Entry {}", i),
                TimelineSource::Observation,
                "detail".to_string(),
            );
        }

        assert_eq!(timeline.len(), 5);
        // Should have entries 5-9
        assert_eq!(timeline.first().unwrap().label, "Entry 5");
        assert_eq!(timeline.last().unwrap().label, "Entry 9");
    }

    #[test]
    fn test_summary_by_source() {
        let mut timeline = EngineeringTimeline::new(1000);

        timeline.append("A".to_string(), TimelineSource::Observation, "detail".to_string());
        timeline.append("B".to_string(), TimelineSource::Intent, "detail".to_string());
        timeline.append("C".to_string(), TimelineSource::Observation, "detail".to_string());
        timeline.append("D".to_string(), TimelineSource::User, "detail".to_string());

        let summary = timeline.summary_by_source();
        assert_eq!(*summary.get(&TimelineSource::Observation).unwrap(), 2);
        assert_eq!(*summary.get(&TimelineSource::Intent).unwrap(), 1);
        assert_eq!(*summary.get(&TimelineSource::User).unwrap(), 1);
    }

    #[test]
    fn test_serialization() {
        let mut timeline = EngineeringTimeline::new(1000);
        timeline.append("Test".to_string(), TimelineSource::Observation, "detail".to_string());

        let json = timeline.to_json().unwrap();
        let restored = EngineeringTimeline::from_json(&json).unwrap();

        assert_eq!(restored.len(), 1);
        assert_eq!(restored.first().unwrap().label, "Test");
    }
}