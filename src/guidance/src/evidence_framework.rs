/// Feature 6 — Evidence Collection Framework
///
/// Tracks troubleshooting evidence state during a session.
/// Example:
///   Issue: Application unavailable
///   Evidence collected: ✓ Pod status, ✓ Recent events, ✓ Application logs
///   Missing: ✗ Network connectivity test
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of an evidence item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EvidenceStatus {
    /// Evidence has been collected and verified.
    Collected,
    /// Evidence has been collected but needs verification.
    Pending,
    /// Evidence is missing or could not be collected.
    Missing,
}

/// An individual evidence item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    /// Unique ID.
    pub id: Uuid,
    /// Name of the evidence source.
    pub source: String,
    /// What was found (if collected).
    pub finding: Option<String>,
    /// Status of the evidence.
    pub status: EvidenceStatus,
    /// When it was collected or last checked.
    pub timestamp: DateTime<Utc>,
    /// Why this evidence matters for troubleshooting.
    pub relevance: String,
}

/// The evidence collection state for a troubleshooting session.
pub struct EvidenceCollection {
    /// Items of evidence tracked.
    pub items: Vec<EvidenceItem>,
    /// The overall issue being investigated.
    pub issue: String,
    /// Session ID this evidence belongs to.
    pub session_id: Uuid,
}

impl Default for EvidenceCollection {
    fn default() -> Self {
        Self::new("Unknown issue".to_string())
    }
}

impl EvidenceCollection {
    /// Create a new evidence collection for an issue.
    pub fn new(issue: String) -> Self {
        Self {
            items: Vec::new(),
            issue,
            session_id: Uuid::new_v4(),
        }
    }

    /// Add a new evidence item.
    pub fn add_item(&mut self, source: &str, relevance: &str, status: EvidenceStatus, finding: Option<&str>) {
        self.items.push(EvidenceItem {
            id: Uuid::new_v4(),
            source: source.to_string(),
            finding: finding.map(String::from),
            status: status.clone(),
            timestamp: Utc::now(),
            relevance: relevance.to_string(),
        });
    }

    /// Mark an evidence item as collected.
    pub fn mark_collected(&mut self, source: &str, finding: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.source == source) {
            item.status = EvidenceStatus::Collected;
            item.finding = Some(finding.to_string());
            item.timestamp = Utc::now();
        } else {
            self.add_item(source, "Collected during investigation", EvidenceStatus::Collected, Some(finding));
        }
    }

    /// Mark an evidence item as missing.
    pub fn mark_missing(&mut self, source: &str, reason: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.source == source) {
            item.status = EvidenceStatus::Missing;
            item.timestamp = Utc::now();
        } else {
            self.add_item(
                source,
                reason,
                EvidenceStatus::Missing,
                None,
            );
        }
    }

    /// Mark an evidence item as pending.
    pub fn mark_pending(&mut self, source: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.source == source) {
            item.status = EvidenceStatus::Pending;
            item.timestamp = Utc::now();
        } else {
            self.add_item(source, "Awaiting collection", EvidenceStatus::Pending, None);
        }
    }

    /// Get the count of collected evidence items.
    pub fn collected_count(&self) -> usize {
        self.items.iter().filter(|i| i.status == EvidenceStatus::Collected).count()
    }

    /// Get the count of missing evidence items.
    pub fn missing_count(&self) -> usize {
        self.items.iter().filter(|i| i.status == EvidenceStatus::Missing).count()
    }

    /// Get the count of pending evidence items.
    pub fn pending_count(&self) -> usize {
        self.items.iter().filter(|i| i.status == EvidenceStatus::Pending).count()
    }

    /// Get missing evidence items (what's still needed).
    pub fn missing(&self) -> Vec<&EvidenceItem> {
        self.items
            .iter()
            .filter(|i| i.status == EvidenceStatus::Missing)
            .collect()
    }

    /// Get pending evidence items.
    pub fn pending(&self) -> Vec<&EvidenceItem> {
        self.items
            .iter()
            .filter(|i| i.status == EvidenceStatus::Pending)
            .collect()
    }

    /// Get collected evidence items.
    pub fn collected(&self) -> Vec<&EvidenceItem> {
        self.items
            .iter()
            .filter(|i| i.status == EvidenceStatus::Collected)
            .collect()
    }

    /// Check if all evidence has been collected.
    pub fn is_complete(&self) -> bool {
        self.missing_count() == 0 && self.pending_count() == 0
    }

    /// Check if sufficient evidence exists to make a recommendation.
    pub fn is_sufficient(&self) -> bool {
        // At least 50% of evidence collected, and nothing critical is missing
        let total = self.items.len();
        if total == 0 {
            return false;
        }
        // If there are missing items, not sufficient regardless of percentage
        if self.missing_count() > 0 {
            return false;
        }
        self.collected_count() as f64 / total as f64 >= 0.5
    }

    /// Get a human-readable summary of evidence state.
    pub fn summary(&self) -> String {
        let mut output = format!("Evidence State for: {}\n", self.issue);
        output.push_str(&format!(
            "Collected: {}, Missing: {}, Pending: {}\n\n",
            self.collected_count(),
            self.missing_count(),
            self.pending_count()
        ));

        if !self.collected().is_empty() {
            output.push_str("✓ Collected:\n");
            for item in self.collected() {
                output.push_str(&format!("  • {} — {}\n", item.source, item.finding.as_deref().unwrap_or("N/A")));
            }
            output.push('\n');
        }

        if !self.pending().is_empty() {
            output.push_str("◐ Pending:\n");
            for item in self.pending() {
                output.push_str(&format!("  • {}\n", item.source));
            }
            output.push('\n');
        }

        if !self.missing().is_empty() {
            output.push_str("✗ Missing:\n");
            for item in self.missing() {
                output.push_str(&format!("  • {}\n", item.source));
            }
        }

        output
    }

    /// Get count of evidence items.
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Get the issue being investigated.
    pub fn issue(&self) -> &str {
        &self.issue
    }

    /// Get the session ID.
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    /// Clear all evidence.
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_collection_tracks_items() {
        let mut collection = EvidenceCollection::new("Application unavailable".to_string());

        collection.add_item("Pod Status", "Core system information", EvidenceStatus::Collected, Some("Pods Running"));
        collection.add_item("Recent Events", "Cluster events", EvidenceStatus::Collected, Some("OOMKilled events"));
        collection.add_item("Network Connectivity", "Pod network check", EvidenceStatus::Missing, None);

        assert_eq!(collection.collected_count(), 2);
        assert_eq!(collection.missing_count(), 1);
        assert_eq!(collection.pending_count(), 0);
        assert_eq!(collection.count(), 3);
    }

    #[test]
    fn test_evidence_mark_collected() {
        let mut collection = EvidenceCollection::new("Test issue".to_string());
        collection.add_item("Logs", "Application logs", EvidenceStatus::Pending, None);

        collection.mark_collected("Logs", "Found error: Connection refused");
        assert_eq!(collection.collected_count(), 1);
        assert_eq!(collection.pending_count(), 0);
    }

    #[test]
    fn test_evidence_mark_missing() {
        let mut collection = EvidenceCollection::new("Test".to_string());
        collection.add_item("Disk", "Disk usage", EvidenceStatus::Pending, None);

        collection.mark_missing("Disk", "Could not access disk info");
        assert_eq!(collection.missing_count(), 1);
    }

    #[test]
    fn test_evidence_is_complete() {
        let mut collection = EvidenceCollection::new("Test".to_string());
        collection.add_item("A", "Evidence A", EvidenceStatus::Collected, Some("OK"));
        collection.add_item("B", "Evidence B", EvidenceStatus::Collected, Some("OK"));

        assert!(collection.is_complete());
        assert!(collection.is_sufficient());
    }

    #[test]
    fn test_evidence_not_sufficient() {
        let mut collection = EvidenceCollection::new("Test".to_string());
        collection.add_item("A", "Evidence A", EvidenceStatus::Collected, Some("OK"));
        collection.add_item("B", "Evidence B", EvidenceStatus::Missing, None);

        assert!(!collection.is_complete());
        // 50% collected but 50% missing — not sufficient
        assert!(!collection.is_sufficient());
    }

    #[test]
    fn test_evidence_summary() {
        let mut collection = EvidenceCollection::new("App down".to_string());
        collection.add_item("Pod Status", "Core info", EvidenceStatus::Collected, Some("Running"));
        collection.add_item("Network", "Network check", EvidenceStatus::Missing, None);

        let summary = collection.summary();
        assert!(summary.contains("Pod Status"));
        assert!(summary.contains("Network"));
        assert!(summary.contains("✓"));
        assert!(summary.contains("✗"));
    }

    #[test]
    fn test_evidence_session_id_unique() {
        let c1 = EvidenceCollection::new("Issue 1".to_string());
        let c2 = EvidenceCollection::new("Issue 2".to_string());
        assert_ne!(c1.session_id(), c2.session_id());
    }

    #[test]
    fn test_evidence_clear() {
        let mut collection = EvidenceCollection::new("Test".to_string());
        collection.add_item("A", "Test", EvidenceStatus::Collected, Some("OK"));
        collection.add_item("B", "Test", EvidenceStatus::Missing, None);

        collection.clear();
        assert_eq!(collection.count(), 0);
        assert!(collection.is_complete());
    }

    #[test]
    fn test_evidence_pending() {
        let mut collection = EvidenceCollection::new("Test".to_string());
        collection.add_item("A", "Test", EvidenceStatus::Pending, None);
        collection.add_item("B", "Test", EvidenceStatus::Pending, None);

        assert_eq!(collection.pending_count(), 2);
    }

    #[test]
    fn test_evidence_mark_pending_new_source() {
        let mut collection = EvidenceCollection::new("Test".to_string());
        collection.mark_pending("New Source");
        assert_eq!(collection.pending_count(), 1);
    }
}