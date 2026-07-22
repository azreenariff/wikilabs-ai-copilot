//! Copilot Engine — orchestrates all subsystems.
//!
//! The Copilot Engine is the main entry point. It coordinates:
//! - Decision Engine (filter, prioritize, time)
//! - Recommendation Engine (generate with context/evidence)
//! - Policy Engine (Minimal/Balanced/Teaching/Expert/Silent)
//! - Lifecycle Manager (Candidate→Ready→Displayed→Accepted/Dismissed→Completed/Archived)
//! - Session Memory (track accepted/dismissed/corrections)
//! - Conversation Context (follow-up, no repetition)
//! - Explainability (traceable reasoning)
//! - Human Approval (engineer must approve before action)
//! - Proactive Assistance (when/how to interrupt)
//! - Contextual Follow-Up (continue previous topics)

use crate::approval::{ApprovalRequest, HumanApproval};
use crate::cards::RecommendationCard;
use crate::decision::{DecisionContext, DecisionEngine};
use crate::explainability::Explainability;
use crate::lifecycle::RecommendationLifecycle;
use crate::memory::SessionMemory;
use crate::modes::CopilotMode;
use crate::policy::PolicyEngine;
use crate::proactive::{ProactiveAssistance, ProactiveSignal};
use crate::recommendation::{EngineeringContext, RecommendationEngine};
use crate::Recommendation;
use serde::{Deserialize, Serialize};

/// A result of the Copilot Engine processing a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotResult {
    pub recommendation: Recommendation,
    pub card: RecommendationCard,
    pub explanation: Explainability,
    pub approval_request: Option<ApprovalRequest>,
    pub should_show: bool,
    pub reason: String,
}

/// The main Copilot Engine that orchestrates all subsystems.
pub struct CopilotEngine {
    decision_engine: DecisionEngine,
    recommendation_engine: RecommendationEngine,
    policy_engine: PolicyEngine,
    lifecycle: RecommendationLifecycle,
    session_memory: SessionMemory,
    conversation_context: crate::conversation::ConversationContext,
    human_approval: HumanApproval,
    proactive: ProactiveAssistance,
    mode: CopilotMode,
}

impl CopilotEngine {
    pub fn new(mode: CopilotMode) -> Self {
        CopilotEngine {
            decision_engine: DecisionEngine::new(),
            recommendation_engine: RecommendationEngine::new(),
            policy_engine: PolicyEngine::new(mode.policy_level()),
            lifecycle: RecommendationLifecycle::new(),
            session_memory: SessionMemory::new(),
            conversation_context: crate::conversation::ConversationContext::new(),
            human_approval: HumanApproval::new(),
            proactive: ProactiveAssistance::new(),
            mode,
        }
    }

    pub fn with_mode(mut self, mode: CopilotMode) -> Self {
        self.mode = mode;
        self.policy_engine.set_level(mode.policy_level());
        self
    }

    /// Process an observation and return a recommendation result.
    pub fn process_observation(
        &mut self,
        observation: &str,
        context: &EngineeringContext,
    ) -> Option<CopilotResult> {
        // Generate recommendation from observation
        let gen_result = self
            .recommendation_engine
            .from_observations(&[observation.to_string()], context);
        if gen_result.is_empty() {
            return None;
        }

        let gen = gen_result.into_iter().next()?;
        let rec = gen.recommendation;
        let confidence = rec.confidence;
        let priority = rec.priority;

        // Check if topic was recently accepted or dismissed
        if self.session_memory.was_recently_accepted(&rec.title) {
            return None;
        }
        if !self.conversation_context.is_topic_acceptable(&rec.title) {
            return None;
        }

        // Decision Engine gate
        let decision_ctx = DecisionContext {
            is_idle: true,
            is_typing: false,
            recent_activity: context.recent_activity.clone(),
            ..Default::default()
        };

        let outcome = self.decision_engine.evaluate(
            rec.id,
            confidence,
            priority,
            !rec.evidence.is_empty(),
            &decision_ctx,
        );

        if !outcome.should_show {
            return None;
        }

        // Lifecycle: mark ready
        if self
            .lifecycle
            .state(rec.id)
            .unwrap_or(crate::lifecycle::LifecycleState::Candidate)
            != crate::lifecycle::LifecycleState::Ready
        {
            let _ = self.lifecycle.mark_ready(rec.id);
        }
        let _ = self.lifecycle.mark_displayed(rec.id);

        // Policy check
        let policy_ok = self.policy_engine.should_show(
            outcome.adjusted_confidence.score,
            outcome.adjusted_priority,
            true,
            None,
        );
        if !policy_ok {
            return None;
        }

        // Create explanation
        let explanation = Explainability::new(gen.generation_reason)
            .with_nodes(vec![
                crate::explainability::ExplanationNode::new(
                    "Evidence Source",
                    format!("Observation from monitoring: {}", observation),
                    vec![0],
                ),
                crate::explainability::ExplanationNode::new(
                    "Recommendation",
                    rec.description.clone(),
                    Vec::new(),
                )
                .mandatory(),
            ])
            .with_certainty(outcome.adjusted_confidence.score)
            .add_alternative("Do nothing")
            .add_limitation("Based on single observation");

        // Create card
        let card = RecommendationCard::from_recommendation(&rec);

        // Create approval request
        let approval = self.human_approval.create_request(
            rec.id,
            rec.suggested_next_step
                .clone()
                .unwrap_or_else(|| "Review and take action".to_string()),
        );

        self.session_memory
            .record_accepted(rec.id, rec.title.clone());
        self.conversation_context
            .record_shown(rec.title.clone(), rec.id);
        self.decision_engine.record_shown(rec.id);

        // Record signal for proactive tracking
        self.proactive
            .record_signal(ProactiveSignal::ErrorDetected {
                error_type: observation.to_string(),
            });

        Some(CopilotResult {
            recommendation: rec,
            card,
            explanation,
            approval_request: Some(approval),
            should_show: true,
            reason: outcome.reasoning.clone(),
        })
    }

    /// Process a user question.
    pub fn process_question(&mut self, question: &str, context: &EngineeringContext) -> String {
        self.conversation_context
            .record_question(question.to_string());
        let answer = self.generate_answer(question, context);
        self.conversation_context.record_answer(answer.clone());
        answer
    }

    /// Generate an answer based on a question.
    fn generate_answer(&self, question: &str, context: &EngineeringContext) -> String {
        let mut answer = String::new();

        // Check session memory for relevant history
        let acceptance_rate = self.session_memory.acceptance_rate();
        answer.push_str(&format!(
            "Based on our session, you've accepted {:.0}% of recommendations.\n\n",
            acceptance_rate * 100.0
        ));

        // Check conversation context for related topics
        let recent = self.conversation_context.recent_history(5);
        if !recent.is_empty() {
            answer.push_str("Recent topics discussed:\n");
            for turn in recent {
                answer.push_str(&format!("- {}\n", turn.content));
            }
        }

        // Context-aware answer
        if !context.technologies.is_empty() {
            answer.push_str(&format!(
                "\nTechnologies in context: {}\n",
                context.technologies.join(", ")
            ));
        }

        if question.to_lowercase().contains("memory")
            || question.to_lowercase().contains("resource")
        {
            answer.push_str(
                "For memory-related questions, review the monitoring data and follow SOP guidelines. "
                    .to_string()
                    .as_str(),
            );
        }

        if answer.is_empty()
            || answer
                == format!(
                    "Based on our session, you've accepted {:.0}% of recommendations.\n\n",
                    acceptance_rate * 100.0
                )
        {
            answer.push_str("I'm here to help. Ask me anything about your engineering context.");
        }

        answer
    }

    /// Record that the engineer accepted a recommendation.
    pub fn record_accepted(&mut self, recommendation_id: uuid::Uuid) {
        if let Some(id) = self
            .lifecycle
            .state(recommendation_id)
            .ok()
            .map(|_| recommendation_id)
        {
            let _ = self.lifecycle.mark_accepted(id);
            self.session_memory
                .record_accepted(id, format!("Rec {}", id));
        }
    }

    /// Record that the engineer dismissed a recommendation.
    pub fn record_dismissed(&mut self, recommendation_id: uuid::Uuid, reason: Option<String>) {
        if let Some(id) = self
            .lifecycle
            .state(recommendation_id)
            .ok()
            .map(|_| recommendation_id)
        {
            let _ = self.lifecycle.dismiss(id);
            self.session_memory.record_dismissed(id, reason);
            self.conversation_context
                .record_dismissed(format!("Rec {}", id), id);
        }
    }

    /// Record that the engineer corrected a recommendation.
    pub fn record_correction(&mut self, recommendation_id: uuid::Uuid, instead: String) {
        if let Some(id) = self
            .lifecycle
            .state(recommendation_id)
            .ok()
            .map(|_| recommendation_id)
        {
            self.session_memory.record_correction(id, instead);
            self.conversation_context.record_corrected(
                format!("Rec {}", id),
                "Engineer took different action".to_string(),
                id,
            );
        }
    }

    /// Get recommended actions based on current context.
    pub fn get_recommendations(&mut self, _context: &EngineeringContext) -> Vec<RecommendationCard> {
        let mut results = Vec::new();

        // Get active recommendations from lifecycle
        let active = self.lifecycle.active_recommendations();
        for (id, _state) in active {
            if self.lifecycle.is_ready(id) {
                let rec = self.get_recommendation_by_id(id);
                if let Some(rec) = rec {
                    results.push(RecommendationCard::from_recommendation(&rec));
                }
            }
        }

        results
    }

    /// Get a recommendation by ID.
    fn get_recommendation_by_id(&self, _id: uuid::Uuid) -> Option<Recommendation> {
        // This would need access to stored recommendations
        // For now return None
        None
    }

    /// Switch copilot mode.
    pub fn switch_mode(&mut self, mode: CopilotMode) {
        self.mode = mode;
        self.policy_engine.set_level(mode.policy_level());
    }

    /// Get current mode.
    pub fn mode(&self) -> CopilotMode {
        self.mode
    }

    /// Get session stats.
    pub fn stats(&self) -> (u32, u32, u32, u32) {
        self.session_memory.counts()
    }

    /// Clear all session state.
    pub fn clear_session(&mut self) {
        self.session_memory.clear();
        self.conversation_context.clear();
        self.decision_engine.clear_shown();
    }
}

impl Default for CopilotEngine {
    fn default() -> Self {
        CopilotEngine::new(CopilotMode::Balanced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx() -> EngineeringContext {
        EngineeringContext {
            workflow_state: Some("Development".to_string()),
            technologies: vec!["Rust".to_string()],
            recent_activity: vec!["Edited src/lib.rs".to_string()],
            ..Default::default()
        }
    }

    #[test]
    fn test_engine_default_mode() {
        let engine = CopilotEngine::default();
        assert_eq!(engine.mode(), CopilotMode::Balanced);
    }

    #[test]
    fn test_engine_process_observation() {
        let mut engine = CopilotEngine::new(CopilotMode::Balanced);
        let ctx = make_ctx();
        let result = engine.process_observation("Pod memory at 90%", &ctx);
        // Should produce a result since error detection has high confidence
        assert!(result.is_some());
    }

    #[test]
    fn test_engine_process_question() {
        let mut engine = CopilotEngine::new(CopilotMode::Balanced);
        let ctx = make_ctx();
        let answer = engine.process_question("How is my memory?", &ctx);
        assert!(!answer.is_empty());
    }

    #[test]
    fn test_engine_mode_switch() {
        let mut engine = CopilotEngine::new(CopilotMode::Balanced);
        assert_eq!(engine.mode(), CopilotMode::Balanced);
        engine.switch_mode(CopilotMode::Silent);
        assert_eq!(engine.mode(), CopilotMode::Silent);
        engine.switch_mode(CopilotMode::Teaching);
        assert_eq!(engine.mode(), CopilotMode::Teaching);
    }

    #[test]
    fn test_engine_clear_session() {
        let mut engine = CopilotEngine::new(CopilotMode::Balanced);
        let ctx = make_ctx();
        let _ = engine.process_observation("Pod memory high", &ctx);
        engine.clear_session();
        let (a, d, i, c) = engine.stats();
        assert_eq!((a, d, i, c), (0, 0, 0, 0));
    }

    #[test]
    fn test_engine_process_silent_mode() {
        let mut engine = CopilotEngine::new(CopilotMode::Silent);
        let ctx = make_ctx();
        // Silent mode should filter out most recommendations
        let result = engine.process_observation("Pod memory at 90%", &ctx);
        // Should be None in silent mode (confidence 0.85 < 1.0)
        assert!(result.is_none());
    }

    #[test]
    fn test_engine_stats() {
        let engine = CopilotEngine::default();
        assert_eq!(engine.stats(), (0, 0, 0, 0));
    }
}
