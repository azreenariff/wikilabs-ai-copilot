/// Feature 2 — Engineering Recommendation Framework
///
/// Creates structured recommendations with complete metadata.
/// Every recommendation contains:
/// - Title, Technology, Category, Description, Reason
/// - Confidence, Evidence, Recommended next step
/// - Reference documentation, Risk warning (if applicable)
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Category of the engineering recommendation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendationCategory {
    /// Troubleshooting or diagnostic guidance.
    Troubleshooting,
    /// Configuration review.
    ConfigurationReview,
    /// Performance optimization.
    PerformanceOptimization,
    /// Security assessment.
    Security,
    /// Deployment or infrastructure change.
    Infrastructure,
    /// Informational or educational.
    Informational,
    /// Command execution guidance.
    CommandGuidance,
}

/// Evidence supporting a recommendation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationEvidence {
    /// Source of evidence (e.g., "Pod Logs", "Node Status").
    pub source: String,
    /// Description of what was observed.
    pub description: String,
    /// Confidence in this specific evidence.
    pub confidence: f64,
}

/// Reference documentation for a recommendation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceDoc {
    /// Title of the documentation.
    pub title: String,
    /// URL or path to the documentation.
    pub url: Option<String>,
    /// Brief description of why this doc is relevant.
    pub relevance: String,
}

/// Risk warning for a recommendation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskWarning {
    /// Level of risk (Low, Medium, High, Critical).
    pub level: RiskLevel,
    /// Description of what could go wrong.
    pub description: String,
    /// Mitigation or safety steps.
    pub mitigation: String,
}

/// Risk level for recommendations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// No risk — information only.
    None,
    /// Low risk — read-only operation.
    Low,
    /// Medium risk — may affect running services.
    Medium,
    /// High risk — may cause downtime or data loss.
    High,
    /// Critical risk — irreversible action.
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// A structured engineering recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringRecommendation {
    /// Unique ID for this recommendation.
    pub id: Uuid,
    /// Concise title of the recommendation.
    pub title: String,
    /// Technology this recommendation applies to.
    pub technology: String,
    /// Category of the recommendation.
    pub category: RecommendationCategory,
    /// Detailed description.
    pub description: String,
    /// Why this recommendation was made.
    pub reason: String,
    /// Overall confidence (0.0 - 1.0).
    pub confidence: f64,
    /// Evidence supporting this recommendation.
    pub evidence: Vec<RecommendationEvidence>,
    /// Recommended next step (CLI command, API call, etc.).
    pub recommended_next_step: Option<String>,
    /// Reference documentation sources.
    pub reference_docs: Vec<ReferenceDoc>,
    /// Risk warning if applicable.
    pub risk_warning: Option<RiskWarning>,
    /// Timestamp when this recommendation was generated.
    pub generated_at: DateTime<Utc>,
}

/// Builder for constructing recommendations.
pub struct RecommendationBuilder {
    title: String,
    technology: String,
    category: RecommendationCategory,
    description: String,
    reason: String,
    confidence: f64,
    evidence: Vec<RecommendationEvidence>,
    recommended_next_step: Option<String>,
    reference_docs: Vec<ReferenceDoc>,
    risk_warning: Option<RiskWarning>,
}

impl RecommendationBuilder {
    /// Create a new recommendation builder.
    pub fn new(title: &str, technology: &str, category: RecommendationCategory) -> Self {
        Self {
            title: title.to_string(),
            technology: technology.to_string(),
            category,
            description: String::new(),
            reason: String::new(),
            confidence: 0.0,
            evidence: Vec::new(),
            recommended_next_step: None,
            reference_docs: Vec::new(),
            risk_warning: None,
        }
    }

    /// Set the description.
    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Set the reason.
    pub fn reason(mut self, reason: &str) -> Self {
        self.reason = reason.to_string();
        self
    }

    /// Set the confidence score.
    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Add evidence.
    pub fn evidence(mut self, source: &str, description: &str, confidence: f64) -> Self {
        self.evidence.push(RecommendationEvidence {
            source: source.to_string(),
            description: description.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
        });
        self
    }

    /// Set the recommended next step.
    pub fn next_step(mut self, step: &str) -> Self {
        self.recommended_next_step = Some(step.to_string());
        self
    }

    /// Add reference documentation.
    pub fn reference(mut self, title: &str, url: Option<&str>, relevance: &str) -> Self {
        self.reference_docs.push(ReferenceDoc {
            title: title.to_string(),
            url: url.map(String::from),
            relevance: relevance.to_string(),
        });
        self
    }

    /// Set a risk warning.
    pub fn risk(mut self, level: RiskLevel, description: &str, mitigation: &str) -> Self {
        self.risk_warning = Some(RiskWarning {
            level,
            description: description.to_string(),
            mitigation: mitigation.to_string(),
        });
        self
    }

    /// Build the recommendation.
    pub fn build(self) -> EngineeringRecommendation {
        EngineeringRecommendation {
            id: Uuid::new_v4(),
            title: self.title,
            technology: self.technology,
            category: self.category,
            description: self.description,
            reason: self.reason,
            confidence: self.confidence,
            evidence: self.evidence,
            recommended_next_step: self.recommended_next_step,
            reference_docs: self.reference_docs,
            risk_warning: self.risk_warning,
            generated_at: Utc::now(),
        }
    }
}

/// Framework for managing engineering recommendations.
pub struct RecommendationFramework {
    generated_recommendations: Vec<EngineeringRecommendation>,
}

impl Default for RecommendationFramework {
    fn default() -> Self {
        Self::new()
    }
}

impl RecommendationFramework {
    /// Create a new recommendation framework.
    pub fn new() -> Self {
        Self {
            generated_recommendations: Vec::new(),
        }
    }

    /// Generate a recommendation using the builder pattern.
    pub fn generate(&mut self, builder: RecommendationBuilder) -> &EngineeringRecommendation {
        let rec = builder.build();
        self.generated_recommendations.push(rec);
        self.generated_recommendations.last().unwrap()
    }

    /// Get all generated recommendations.
    pub fn all(&self) -> &[EngineeringRecommendation] {
        &self.generated_recommendations
    }

    /// Get recommendation by ID.
    pub fn by_id(&self, id: &Uuid) -> Option<&EngineeringRecommendation> {
        self.generated_recommendations.iter().find(|r| r.id == *id)
    }

    /// Filter recommendations by category.
    pub fn by_category(&self, category: &RecommendationCategory) -> Vec<&EngineeringRecommendation> {
        self.generated_recommendations
            .iter()
            .filter(|r| &r.category == category)
            .collect()
    }

    /// Filter recommendations by technology.
    pub fn by_technology(&self, technology: &str) -> Vec<&EngineeringRecommendation> {
        self.generated_recommendations
            .iter()
            .filter(|r| r.technology.to_lowercase() == technology.to_lowercase())
            .collect()
    }

    /// Filter recommendations by minimum confidence.
    pub fn by_min_confidence(&self, min: f64) -> Vec<&EngineeringRecommendation> {
        self.generated_recommendations
            .iter()
            .filter(|r| r.confidence >= min)
            .collect()
    }

    /// Get count of generated recommendations.
    pub fn count(&self) -> usize {
        self.generated_recommendations.len()
    }

    /// Format a recommendation for display (minimal view).
    pub fn format_minimal(&self, rec: &EngineeringRecommendation) -> String {
        let mut output = format!("Recommendation: {}\n", rec.title);
        output.push_str(&format!("Technology: {}\n", rec.technology));
        output.push_str(&format!("Confidence: {:.0}%\n", rec.confidence * 100.0));

        if let Some(ref risk) = rec.risk_warning {
            output.push_str(&format!("Risk: {} — {}\n", risk.level, risk.description));
        }

        if let Some(ref step) = rec.recommended_next_step {
            output.push_str(&format!("Command: {}\n", step));
        }

        output
    }

    /// Format a recommendation for display (full view).
    pub fn format_full(&self, rec: &EngineeringRecommendation) -> String {
        let mut output = self.format_minimal(rec);
        output.push_str(&format!("Category: {:?}\n", rec.category));
        output.push_str(&format!("Reason: {}\n\n", rec.reason));
        output.push_str("Evidence:\n");
        for ev in &rec.evidence {
            output.push_str(&format!(
                "  ✓ {} — {} (confidence: {:.0}%)\n",
                ev.source, ev.description, ev.confidence * 100.0
            ));
        }

        if !rec.reference_docs.is_empty() {
            output.push_str("Reference Documentation:\n");
            for doc in &rec.reference_docs {
                output.push_str(&format!("  • {} — {}\n", doc.title, doc.relevance));
                if let Some(ref url) = doc.url {
                    output.push_str(&format!("    URL: {}\n", url));
                }
            }
        }

        if let Some(ref risk) = rec.risk_warning {
            output.push_str(&format!("\nWARNING: {}\nMitigation: {}\n", risk.description, risk.mitigation));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_builder_openshift_events() {
        let framework = &mut RecommendationFramework::new();
        let rec = framework.generate(
            RecommendationBuilder::new(
                "Check OpenShift Pod Events",
                "OpenShift",
                RecommendationCategory::Troubleshooting,
            )
            .description("Pod restart detected. Review recent events for root cause.")
            .reason("Current troubleshooting workflow indicates evidence collection is incomplete.")
            .confidence(0.92)
            .evidence("Pod Status", "Pod restart count increasing", 0.85)
            .evidence("Deployment History", "Deployment recently changed", 0.80)
            .evidence("System Events", "Recent OOMKilled events detected", 0.90)
            .next_step("oc get events --sort-by=.lastTimestamp")
            .reference("OpenShift Troubleshooting Guide", Some("/docs/openshift/troubleshooting.md"), "Pod lifecycle")
            .risk(
                RiskLevel::Low,
                "Reading events does not modify cluster state.",
                "This is a read-only command. Safe to execute.",
            )
        );

        assert_eq!(rec.title, "Check OpenShift Pod Events");
        assert_eq!(rec.technology, "OpenShift");
        assert_eq!(rec.category, RecommendationCategory::Troubleshooting);
        assert_eq!(rec.confidence, 0.92);
        assert_eq!(rec.evidence.len(), 3);
        assert!(rec.risk_warning.is_some());
        assert!(rec.recommended_next_step.is_some());
        assert_eq!(framework.count(), 1);
    }

    #[test]
    fn test_recommendation_framework_by_category() {
        let framework = &mut RecommendationFramework::new();

        framework.generate(
            RecommendationBuilder::new("Check Pods", "OpenShift", RecommendationCategory::Troubleshooting)
                .confidence(0.8)
                .reason("Pod restart detected.")
        );

        framework.generate(
            RecommendationBuilder::new("Check Config", "Linux", RecommendationCategory::ConfigurationReview)
                .confidence(0.7)
                .reason("Configuration may be outdated.")
        );

        let troubleshooting = framework.by_category(&RecommendationCategory::Troubleshooting);
        assert_eq!(troubleshooting.len(), 1);

        let config = framework.by_category(&RecommendationCategory::ConfigurationReview);
        assert_eq!(config.len(), 1);
    }

    #[test]
    fn test_recommendation_framework_by_technology() {
        let framework = &mut RecommendationFramework::new();

        framework.generate(
            RecommendationBuilder::new("Check Pod", "OpenShift", RecommendationCategory::Troubleshooting)
                .confidence(0.8)
                .reason("Pod restart detected.")
        );

        framework.generate(
            RecommendationBuilder::new("Check Disk", "Linux", RecommendationCategory::PerformanceOptimization)
                .confidence(0.7)
                .reason("Disk space low.")
        );

        let openshift = framework.by_technology("openshift");
        assert_eq!(openshift.len(), 1);

        let linux = framework.by_technology("linux");
        assert_eq!(linux.len(), 1);
    }

    #[test]
    fn test_recommendation_framework_by_min_confidence() {
        let framework = &mut RecommendationFramework::new();

        framework.generate(
            RecommendationBuilder::new("High Conf", "Linux", RecommendationCategory::Troubleshooting)
                .confidence(0.9)
                .reason("High confidence issue.")
        );

        framework.generate(
            RecommendationBuilder::new("Low Conf", "Linux", RecommendationCategory::Informational)
                .confidence(0.3)
                .reason("Low confidence issue.")
        );

        let high = framework.by_min_confidence(0.5);
        assert_eq!(high.len(), 1);
        assert_eq!(high[0].title, "High Conf");
    }

    #[test]
    fn test_format_minimal() {
        let mut framework = RecommendationFramework::new();
        let rec = {
            framework.generate(
                RecommendationBuilder::new("Test Rec", "Linux", RecommendationCategory::Troubleshooting)
                    .confidence(0.85)
                    .reason("Test reason.")
                    .next_step("df -h")
            ).clone()
        };

        let minimal = framework.format_minimal(&rec);
        assert!(minimal.contains("Test Rec"));
        assert!(minimal.contains("Linux"));
        assert!(minimal.contains("df -h"));
    }

    #[test]
    fn test_format_full() {
        let mut framework = RecommendationFramework::new();
        let rec = {
            framework.generate(
                RecommendationBuilder::new("Full Test", "OpenShift", RecommendationCategory::Troubleshooting)
                    .confidence(0.9)
                    .reason("Full test reason.")
                    .evidence("Pod Status", "Pod restarting", 0.85)
                    .reference("OpenShift Docs", Some("https://example.com"), "Pod troubleshooting")
            ).clone()
        };

        let full = framework.format_full(&rec);
        assert!(full.contains("Evidence:"));
        assert!(full.contains("Pod Status"));
        assert!(full.contains("OpenShift Docs"));
        assert!(full.contains("Reference Documentation:"));
    }

    #[test]
    fn test_recommendation_has_uuid() {
        let mut framework = RecommendationFramework::new();
        let rec = {
            framework.generate(
                RecommendationBuilder::new("UUID Test", "Linux", RecommendationCategory::Informational)
                    .confidence(0.5)
                    .reason("Test UUID.")
            ).clone()
        };

        assert_ne!(rec.id, Uuid::nil());

        let found = framework.by_id(&rec.id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "UUID Test");
    }

    #[test]
    fn test_recommendation_safety_classification() {
        let framework = &mut RecommendationFramework::new();

        // No risk — informational
        let _info = framework.generate(
            RecommendationBuilder::new("Info", "Linux", RecommendationCategory::Informational)
                .confidence(0.5)
                .reason("Informational.")
        );

        // Critical risk — deployment
        let _critical = framework.generate(
            RecommendationBuilder::new("Critical", "OpenShift", RecommendationCategory::Infrastructure)
                .confidence(0.9)
                .reason("Critical deployment change.")
                .risk(RiskLevel::Critical, "This will restart all pods.", "Ensure backup before execution.")
        );

        let criticals = framework.by_min_confidence(0.5);
        assert_eq!(criticals.len(), 2);
    }
}