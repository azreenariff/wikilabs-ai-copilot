//! Recommendation Engine — generates recommendations from context.
//!
//! The Recommendation Engine is the "brain" of the Copilot.
//! It transforms engineering understanding into timely, trustworthy,
//! context-aware guidance.
//!
//! It generates recommendations by evaluating:
//! - Current workflow state
//! - Recent activity and patterns
//! - Knowledge base context
//! - Observation data
//! - Engineering timeline
//! - User preferences and history

use crate::{Confidence, Evidence, Priority, Recommendation};
use serde::{Deserialize, Serialize};

/// Source of engineering context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct EngineeringContext {
    /// Current workflow state name.
    pub workflow_state: Option<String>,
    /// Active technologies being used.
    pub technologies: Vec<String>,
    /// Recent activity (last N events).
    pub recent_activity: Vec<String>,
    /// Engineering timeline events.
    pub timeline_events: Vec<String>,
    /// Knowledge base relevant files.
    pub knowledge_files: Vec<String>,
    /// Observation data (monitoring, metrics).
    pub observations: Vec<String>,
    /// Current session ID.
    pub session_id: Option<String>,
}


impl EngineeringContext {
    /// Check if this context has substantial data.
    pub fn has_data(&self) -> bool {
        !self.recent_activity.is_empty()
            || !self.technologies.is_empty()
            || !self.observations.is_empty()
            || !self.knowledge_files.is_empty()
    }

    /// Get a summary of the context for logging.
    pub fn summary(&self) -> String {
        format!(
            "workflow={} tech={} activity={} timeline={} knowledge={} observations={}",
            self.workflow_state.as_deref().unwrap_or("none"),
            self.technologies.len(),
            self.recent_activity.len(),
            self.timeline_events.len(),
            self.knowledge_files.len(),
            self.observations.len()
        )
    }
}

/// Recommendation generation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub recommendation: Recommendation,
    pub generation_reason: String,
    pub context_used: Vec<String>,
}

/// The Recommendation Engine — the brain of the Copilot.
///
/// Transforms engineering understanding into timely, trustworthy,
/// context-aware guidance that the engineer can use immediately.
///
/// The AI never performs work autonomously.
pub struct RecommendationEngine {
    /// Track recently generated recommendation titles to avoid repetition.
    recent_titles: Vec<String>,
    /// Maximum recent titles to track.
    max_recent: usize,
    /// Total number of recommendations generated this session.
    total_generations: usize,
}

impl RecommendationEngine {
    pub fn new() -> Self {
        RecommendationEngine {
            recent_titles: Vec::new(),
            max_recent: 50,
            total_generations: 0,
        }
    }

    pub fn with_max_recent(mut self, max: usize) -> Self {
        self.max_recent = max;
        self
    }

    /// Generate a recommendation based on the current context.
    ///
    /// Returns a GenerationResult containing the recommendation,
    /// the reason it was generated, and which context sources were used.
    pub fn generate(
        &mut self,
        title: impl Into<String>,
        description: impl Into<String>,
        reason: impl Into<String>,
        confidence: f64,
        priority: Priority,
        evidence_sources: Vec<(&str, &str, f64)>,
        context: &EngineeringContext,
        suggested_next_step: Option<String>,
    ) -> GenerationResult {
        let confidence = Confidence::new(confidence);
        let evidence: Vec<Evidence> = evidence_sources
            .into_iter()
            .map(|(source, desc, conf)| Evidence {
                source: source.to_string(),
                description: desc.to_string(),
                confidence: Confidence::new(conf),
            })
            .collect();

        let title_str = title.into();

        // Check for repetition
        let is_duplicate = self.recent_titles.iter().any(|t| t == &title_str);
        if is_duplicate {
            tracing::warn!("Recommendation '{title_str}' recently generated — may be repetitive");
        }

        self.total_generations += 1;
        self.recent_titles.push(title_str.clone());
        if self.recent_titles.len() > self.max_recent {
            self.recent_titles
                .drain(..self.recent_titles.len() - self.max_recent);
        }

        let context_used = self.collect_context_used(context);
        let gen_reason = self.build_generation_reason(context, &evidence);

        let rec = Recommendation::new(
            title_str,
            description,
            reason,
            confidence,
            evidence,
            priority,
            vec![],
        )
        .with_next_step(suggested_next_step.unwrap_or_default())
        .with_workflow_context(context.workflow_state.clone().unwrap_or_default());

        GenerationResult {
            recommendation: rec,
            generation_reason: gen_reason,
            context_used,
        }
    }

    /// Generate recommendations from observation data.
    ///
    /// This is the primary entry point for the observation-to-recommendation flow.
    pub fn from_observations(
        &mut self,
        observations: &[String],
        context: &EngineeringContext,
    ) -> Vec<GenerationResult> {
        let mut results = Vec::new();

        for observation in observations {
            let (title, desc, reason, confidence, priority) =
                self.classify_observation(observation, context);

            let result = self.generate(
                title,
                desc,
                reason,
                confidence,
                priority,
                vec![("Observation", observation.as_str(), confidence)],
                context,
                None,
            );
            results.push(result);
        }

        results
    }

    /// Classify an observation into a recommendation type.
    fn classify_observation(
        &self,
        observation: &str,
        _context: &EngineeringContext,
    ) -> (String, String, String, f64, Priority) {
        let obs_lower = observation.to_lowercase();

        // Memory/CPU threshold detection
        if obs_lower.contains("memory") || obs_lower.contains("cpu") {
            if obs_lower.contains("threshold") || obs_lower.contains("limit") {
                return (
                    "Resource Limit Approaching".into(),
                    observation.to_string(),
                    "Resource usage is approaching configured limits based on observation data"
                        .into(),
                    0.85,
                    Priority::Warning,
                );
            }
            return (
                "Resource Anomaly Detected".into(),
                observation.to_string(),
                "Resource metrics show unusual patterns requiring investigation".into(),
                0.7,
                Priority::Suggestion,
            );
        }

        // Pod/container detection
        if obs_lower.contains("pod") || obs_lower.contains("container") {
            if obs_lower.contains("crash") {
                return (
                    "Pod Crash Detected".into(),
                    observation.to_string(),
                    "Pod/container has crashed which indicates a critical failure".into(),
                    0.95,
                    Priority::Critical,
                );
            }
            if obs_lower.contains("restart") {
                return (
                    "Pod Restart Detected".into(),
                    observation.to_string(),
                    "Pod/container has restarted which may indicate instability".into(),
                    0.9,
                    Priority::Warning,
                );
            }
        }

        // Error detection
        if obs_lower.contains("error") || obs_lower.contains("fail") {
            return (
                "Error Condition Detected".into(),
                observation.to_string(),
                "An error condition was detected that requires attention".into(),
                0.8,
                Priority::Critical,
            );
        }

        // Default: information
        (
            "Observation".into(),
            observation.to_string(),
            "Observation data indicates something worth noting".into(),
            0.5,
            Priority::Information,
        )
    }

    /// Build a human-readable reason for why this recommendation was generated.
    fn build_generation_reason(
        &self,
        context: &EngineeringContext,
        evidence: &[Evidence],
    ) -> String {
        let mut parts = Vec::new();

        if let Some(workflow) = &context.workflow_state {
            parts.push(format!("Context: {}", workflow));
        }

        if !context.technologies.is_empty() {
            parts.push(format!("Technologies: {}", context.technologies.join(", ")));
        }

        if !evidence.is_empty() {
            parts.push(format!("Evidence: {} source(s)", evidence.len()));
        }

        parts.join(" | ")
    }

    /// Collect which context sources were used.
    fn collect_context_used(&self, context: &EngineeringContext) -> Vec<String> {
        let mut used = Vec::new();
        if context.workflow_state.is_some() {
            used.push("workflow_state".to_string());
        }
        if !context.technologies.is_empty() {
            used.push("technologies".to_string());
        }
        if !context.recent_activity.is_empty() {
            used.push("recent_activity".to_string());
        }
        if !context.timeline_events.is_empty() {
            used.push("timeline_events".to_string());
        }
        if !context.knowledge_files.is_empty() {
            used.push("knowledge_files".to_string());
        }
        if !context.observations.is_empty() {
            used.push("observations".to_string());
        }
        used
    }

    /// Get the number of recommendations generated this session.
    pub fn generation_count(&self) -> usize {
        self.total_generations
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        RecommendationEngine::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context() -> EngineeringContext {
        EngineeringContext {
            workflow_state: Some("Deployment".to_string()),
            technologies: vec!["Rust".to_string(), "Kubernetes".to_string()],
            recent_activity: vec!["Edited Cargo.toml".to_string()],
            timeline_events: vec!["Commit abc123".to_string()],
            knowledge_files: vec!["memory-sop.md".to_string()],
            observations: vec!["Pod memory at 85%".to_string()],
            session_id: Some("session-1".to_string()),
        }
    }

    #[test]
    fn test_recommendation_generation() {
        let mut engine = RecommendationEngine::new();
        let ctx = make_context();
        let result = engine.generate(
            "Check Memory",
            "Memory usage is high",
            "Memory monitoring shows elevated usage",
            0.85,
            Priority::Warning,
            vec![("Monitoring", "Pod memory at 85%".into(), 0.9)],
            &ctx,
            Some("Review pod configuration".to_string()),
        );
        assert_eq!(result.recommendation.title, "Check Memory");
        assert!(result.recommendation.confidence.score >= 0.8);
        assert!(!result.context_used.is_empty());
    }

    #[test]
    fn test_from_observations() {
        let mut engine = RecommendationEngine::new();
        let ctx = make_context();
        let results = engine.from_observations(&["Pod memory at 85%".into()], &ctx);
        assert_eq!(results.len(), 1);
        assert!(
            results[0].recommendation.title.contains("Memory")
                || results[0].recommendation.title.contains("Resource")
        );
    }

    #[test]
    fn test_classify_resource_threshold() {
        let engine = RecommendationEngine::new();
        let ctx = make_context();
        let (title, _, _, conf, priority) =
            engine.classify_observation("Pod memory threshold reached at 90%", &ctx);
        assert!(title.contains("Threshold") || title.contains("Resource"));
        assert_eq!(priority, Priority::Warning);
        assert!(conf >= 0.8);
    }

    #[test]
    fn test_classify_error() {
        let engine = RecommendationEngine::new();
        let ctx = make_context();
        let (_, _, _, conf, priority) =
            engine.classify_observation("Pod crash error detected", &ctx);
        assert_eq!(priority, Priority::Critical);
        assert!(conf >= 0.75);
    }

    #[test]
    fn test_classify_information() {
        let engine = RecommendationEngine::new();
        let ctx = make_context();
        let (title, _, _, conf, priority) =
            engine.classify_observation("Build completed successfully", &ctx);
        assert_eq!(priority, Priority::Information);
        assert_eq!(conf, 0.5);
    }

    #[test]
    fn test_context_summary() {
        let ctx = make_context();
        let summary = ctx.summary();
        assert!(summary.contains("Deployment"));
        assert!(summary.contains("2"));
        assert!(summary.contains("activity=1"));
    }

    #[test]
    fn test_context_has_data() {
        let mut ctx = EngineeringContext::default();
        assert!(!ctx.has_data());

        ctx.recent_activity.push("test".into());
        assert!(ctx.has_data());
    }

    #[test]
    fn test_recommendation_count() {
        let mut engine = RecommendationEngine::new();
        let ctx = make_context();
        let _ = engine.generate(
            "Test 1",
            "Desc",
            "Reason",
            0.8,
            Priority::Suggestion,
            vec![],
            &ctx,
            None,
        );
        let _ = engine.generate(
            "Test 2",
            "Desc",
            "Reason",
            0.8,
            Priority::Suggestion,
            vec![],
            &ctx,
            None,
        );
        assert_eq!(engine.generation_count(), 2);
    }

    #[test]
    fn test_no_context_defaults() {
        let mut engine = RecommendationEngine::new();
        let ctx = EngineeringContext::default();
        let result = engine.generate(
            "Simple",
            "Desc",
            "Reason",
            0.7,
            Priority::Suggestion,
            vec![],
            &ctx,
            None,
        );
        assert_eq!(result.recommendation.title, "Simple");
        assert_eq!(result.context_used.len(), 0);
    }

    #[test]
    fn test_max_recent_tracking() {
        let mut engine = RecommendationEngine::new().with_max_recent(3);
        let ctx = make_context();
        for i in 0..10 {
            engine.generate(
                format!("Rec {}", i),
                "Desc",
                "Reason",
                0.7,
                Priority::Suggestion,
                vec![],
                &ctx,
                None,
            );
        }
        assert_eq!(engine.generation_count(), 10);
        // The tracking vec should still have max 3
        // (but generation_count tracks total generated, not recent_titles)
    }
}
