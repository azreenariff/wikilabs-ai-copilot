/// Feature 9 — Engineer Guidance Timeline
///
/// Tracks the timeline of AI guidance, showing:
/// - When guidance was provided
/// - What engineer did in response
/// - Which recommendations were relevant
/// - Overall troubleshooting progress

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of timeline event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimelineEventType {
    /// Guidance was provided.
    GuidanceProvided {
        /// Title of the guidance.
        title: String,
        /// Technology involved.
        technology: String,
        /// Confidence of the guidance.
        confidence: f64,
    },
    /// Engineer completed an action.
    EngineerActionCompleted {
        /// Description of what was completed.
        action: String,
        /// Whether it was relevant to the guidance.
        relevant: bool,
    },
    /// Engineer dismissed guidance.
    GuidanceDismissed {
        /// Title of the dismissed guidance.
        title: String,
        /// Reason for dismissal.
        reason: Option<String>,
    },
    /// Detection of technology or activity.
    Detection {
        /// Technology detected.
        technology: String,
        /// Activity description.
        activity: String,
    },
    /// Evidence was collected.
    EvidenceCollected {
        /// Source of evidence.
        source: String,
        /// What was found.
        finding: String,
    },
    /// Missing evidence identified.
    EvidenceMissing {
        /// What is still needed.
        needed: String,
        /// Why it matters.
        importance: String,
    },
}

/// A timeline event entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    /// Unique ID.
    pub id: Uuid,
    /// Timestamp of the event.
    pub timestamp: DateTime<Utc>,
    /// Type of event.
    pub event: TimelineEventType,
    /// Whether this is part of a specific recommendation thread.
    pub recommendation_id: Option<Uuid>,
}

/// The engineer guidance timeline.
pub struct EngineerGuidanceTimeline {
    entries: Vec<TimelineEntry>,
}

impl Default for EngineerGuidanceTimeline {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineerGuidanceTimeline {
    /// Create a new timeline.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Record that guidance was provided.
    pub fn record_guidance(
        &mut self,
        title: &str,
        technology: &str,
        confidence: f64,
        recommendation_id: Option<Uuid>,
    ) {
        self.entries.push(TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event: TimelineEventType::GuidanceProvided {
                title: title.to_string(),
                technology: technology.to_string(),
                confidence,
            },
            recommendation_id,
        });
    }

    /// Record that the engineer completed an action.
    pub fn record_engineer_action(
        &mut self,
        action: &str,
        relevant: bool,
        recommendation_id: Option<Uuid>,
    ) {
        self.entries.push(TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event: TimelineEventType::EngineerActionCompleted {
                action: action.to_string(),
                relevant,
            },
            recommendation_id,
        });
    }

    /// Record dismissed guidance.
    pub fn record_dismissal(
        &mut self,
        title: &str,
        reason: Option<&str>,
        recommendation_id: Option<Uuid>,
    ) {
        self.entries.push(TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event: TimelineEventType::GuidanceDismissed {
                title: title.to_string(),
                reason: reason.map(String::from),
            },
            recommendation_id,
        });
    }

    /// Record technology detection.
    pub fn record_detection(&mut self, technology: &str, activity: &str) {
        self.entries.push(TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event: TimelineEventType::Detection {
                technology: technology.to_string(),
                activity: activity.to_string(),
            },
            recommendation_id: None,
        });
    }

    /// Record evidence collection.
    pub fn record_evidence(&mut self, source: &str, finding: &str, recommendation_id: Option<Uuid>) {
        self.entries.push(TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event: TimelineEventType::EvidenceCollected {
                source: source.to_string(),
                finding: finding.to_string(),
            },
            recommendation_id,
        });
    }

    /// Record missing evidence.
    pub fn record_missing_evidence(&mut self, needed: &str, importance: &str, recommendation_id: Option<Uuid>) {
        self.entries.push(TimelineEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event: TimelineEventType::EvidenceMissing {
                needed: needed.to_string(),
                importance: importance.to_string(),
            },
            recommendation_id,
        });
    }

    /// Get all timeline entries, ordered chronologically.
    pub fn entries(&self) -> &[TimelineEntry] {
        &self.entries
    }

    /// Get entries within a time window (last N minutes).
    pub fn recent(&self, minutes: u64) -> Vec<&TimelineEntry> {
        let cutoff = Utc::now() - chrono::Duration::minutes(minutes as i64);
        self.entries.iter().filter(|e| e.timestamp >= cutoff).collect()
    }

    /// Get entries for a specific recommendation.
    pub fn by_recommendation(&self, id: &Uuid) -> Vec<&TimelineEntry> {
        self.entries
            .iter()
            .filter(|e| e.recommendation_id == Some(*id))
            .collect()
    }

    /// Get a human-readable summary of the timeline.
    pub fn summary(&self) -> String {
        if self.entries.is_empty() {
            return "Timeline is empty.".to_string();
        }

        let mut lines = Vec::new();
        for entry in &self.entries {
            let time = entry.timestamp.format("%H:%M");
            match &entry.event {
                TimelineEventType::GuidanceProvided { title, technology, confidence } => {
                    lines.push(format!(
                        "{} — Guidance: {} [{}] (confidence: {:.0}%)",
                        time, title, technology, confidence * 100.0
                    ));
                }
                TimelineEventType::EngineerActionCompleted { action, relevant } => {
                    lines.push(format!(
                        "{} — Engineer: {} {}",
                        time,
                        action,
                        if *relevant { "(relevant)" } else { "" }
                    ));
                }
                TimelineEventType::GuidanceDismissed { title, reason } => {
                    lines.push(format!(
                        "{} — Dismissed: {}{}",
                        time,
                        title,
                        reason.as_ref().map(|r| format!(" ({})", r)).unwrap_or_default()
                    ));
                }
                TimelineEventType::Detection { technology, activity } => {
                    lines.push(format!("{} — Detected: {} ({})", time, technology, activity));
                }
                TimelineEventType::EvidenceCollected { source, finding } => {
                    lines.push(format!("{} — Evidence: ✓ {} — {}", time, source, finding));
                }
                TimelineEventType::EvidenceMissing { needed, importance } => {
                    lines.push(format!("{} — Missing: ✗ {} — {}", time, needed, importance));
                }
            }
        }
        lines.join("\n")
    }

    /// Get count of entries.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Get the most recent event type.
    pub fn last_event_type(&self) -> Option<&TimelineEventType> {
        self.entries.last().map(|e| &e.event)
    }

    /// Get the time span of the timeline.
    pub fn time_span(&self) -> Option<chrono::Duration> {
        if self.entries.len() < 2 {
            return None;
        }
        let first = self.entries.first()?.timestamp;
        let last = self.entries.last()?.timestamp;
        Some(last - first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_records_guidance() {
        let mut timeline = EngineerGuidanceTimeline::new();
        timeline.record_guidance("Check Pods", "OpenShift", 0.9, None);
        assert_eq!(timeline.count(), 1);

        let entry = &timeline.entries()[0];
        assert!(matches!(
            entry.event,
            TimelineEventType::GuidanceProvided { ref title, .. } if title == "Check Pods"
        ));
    }

    #[test]
    fn test_timeline_records_engineer_action() {
        let mut timeline = EngineerGuidanceTimeline::new();
        let rec_id = Uuid::new_v4();
        timeline.record_guidance("Check Pods", "OpenShift", 0.9, Some(rec_id));
        timeline.record_engineer_action("Reviewed pod logs", true, Some(rec_id));

        assert_eq!(timeline.count(), 2);
        assert!(matches!(
            &timeline.entries()[1].event,
            TimelineEventType::EngineerActionCompleted { action, relevant } if action == "Reviewed pod logs" && *relevant
        ));
    }

    #[test]
    fn test_timeline_records_detection() {
        let mut timeline = EngineerGuidanceTimeline::new();
        timeline.record_detection("OpenShift", "Pod restart detected");

        assert_eq!(timeline.count(), 1);
        assert!(matches!(
            &timeline.entries()[0].event,
            TimelineEventType::Detection { technology, .. } if technology == "OpenShift"
        ));
    }

    #[test]
    fn test_timeline_records_evidence() {
        let mut timeline = EngineerGuidanceTimeline::new();
        let rec_id = Uuid::new_v4();
        timeline.record_evidence("Pod Logs", "OOMKilled detected", Some(rec_id));
        timeline.record_missing_evidence("Network connectivity", "May indicate DNS issue", Some(rec_id));

        assert_eq!(timeline.count(), 2);
        assert!(matches!(
            &timeline.entries()[1].event,
            TimelineEventType::EvidenceMissing { needed, .. } if needed == "Network connectivity"
        ));
    }

    #[test]
    fn test_timeline_by_recommendation() {
        let mut timeline = EngineerGuidanceTimeline::new();
        let rec_id = Uuid::new_v4();
        timeline.record_guidance("Check Pods", "OpenShift", 0.9, Some(rec_id));
        timeline.record_engineer_action("Checked pod status", true, Some(rec_id));
        timeline.record_guidance("Check Disk", "Linux", 0.8, None);

        let rec_entries = timeline.by_recommendation(&rec_id);
        assert_eq!(rec_entries.len(), 2);
    }

    #[test]
    fn test_timeline_summary() {
        let mut timeline = EngineerGuidanceTimeline::new();
        timeline.record_guidance("Check Pods", "OpenShift", 0.9, None);
        timeline.record_evidence("Pod Logs", "OOMKilled", None);

        let summary = timeline.summary();
        assert!(summary.contains("Check Pods"));
        assert!(summary.contains("OOMKilled"));
        assert!(summary.contains("✓"));
    }

    #[test]
    fn test_timeline_empty_summary() {
        let timeline = EngineerGuidanceTimeline::new();
        assert!(timeline.summary().contains("empty"));
    }

    #[test]
    fn test_timeline_last_event_type() {
        let mut timeline = EngineerGuidanceTimeline::new();
        timeline.record_guidance("Check Pods", "OpenShift", 0.9, None);
        timeline.record_engineer_action("Done", true, None);

        assert!(matches!(
            timeline.last_event_type(),
            Some(TimelineEventType::EngineerActionCompleted { .. })
        ));
    }

    #[test]
    fn test_timeline_time_span() {
        let mut timeline = EngineerGuidanceTimeline::new();
        timeline.record_guidance("Check", "Linux", 0.5, None);
        std::thread::sleep(std::time::Duration::from_millis(10));
        timeline.record_engineer_action("Done", true, None);

        let span = timeline.time_span();
        assert!(span.is_some());
        assert!(span.unwrap().num_seconds() >= 0);
    }

    #[test]
    fn test_timeline_recent() {
        let mut timeline = EngineerGuidanceTimeline::new();
        timeline.record_guidance("Old", "Linux", 0.5, None);

        let recent = timeline.recent(1);
        assert_eq!(recent.len(), 1);
    }

    #[test]
    fn test_timeline_dismissal() {
        let mut timeline = EngineerGuidanceTimeline::new();
        timeline.record_dismissal("Irrelevant suggestion", Some("Already checked"), None);

        assert!(matches!(
            &timeline.entries()[0].event,
            TimelineEventType::GuidanceDismissed { title, reason } if title == "Irrelevant suggestion" && reason.as_ref().map(|r| r.as_str()) == Some("Already checked")
        ));
    }
}