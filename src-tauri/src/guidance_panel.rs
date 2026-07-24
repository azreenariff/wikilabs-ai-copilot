//! Guidance Panel UI — Recommendation cards, evidence status, timeline, workflow progress, and feedback.
//!
//! Provides IPC commands for the frontend to:
//! - List and display recommendation cards
//! - View evidence status (collected / missing)
//! - View workflow progress
//! - View guidance timeline
//! - Provide engineer feedback on recommendations
//! - Switch copilot modes (Teaching, Balanced, Expert, Silent)
//!
//! Does NOT execute commands — guidance only.

use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

// ── Recommendation Card Types ────────────────────────────────────

/// Status of a recommendation card in the UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationStatus {
    /// Currently displayed and awaiting engineer response.
    Active,
    /// Engineer marked as useful.
    Accepted,
    /// Engineer marked as not useful or incorrect.
    Rejected,
    /// Already completed by the engineer.
    Skipped,
    /// Dismissed by engineer.
    Dismissed,
}

/// Risk level displayed on a recommendation card.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CardRiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl CardRiskLevel {
    pub fn color(&self) -> &'static str {
        match self {
            Self::None => "#4ade80",  // green
            Self::Low => "#facc15",   // yellow
            Self::Medium => "#fb923c", // orange
            Self::High => "#f87171",   // red
            Self::Critical => "#dc2626", // dark red
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::None => "✓",
            Self::Low => "⚠",
            Self::Medium => "⚠⚠",
            Self::High => "🔴",
            Self::Critical => "💀",
        }
    }
}

/// A recommendation card displayed in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationCard {
    pub id: String,
    pub title: String,
    pub technology: String,
    pub category: String,
    pub description: String,
    pub reason: String,
    pub confidence: f64,
    pub evidence: Vec<EvidenceItem>,
    pub recommended_next_step: Option<String>,
    pub reference_docs: Vec<ReferenceDoc>,
    pub risk_level: Option<CardRiskLevel>,
    pub status: RecommendationStatus,
    pub created_at: String,
}

/// Evidence item displayed inside a recommendation card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub source: String,
    pub description: String,
    pub confidence: f64,
}

/// Reference document link in a recommendation card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceDoc {
    pub title: String,
    pub url: Option<String>,
    pub relevance: String,
}

// ── Evidence Status Types ────────────────────────────────────────

/// Evidence item in the session evidence tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvidence {
    pub id: String,
    pub source: String,
    pub finding: String,
    pub importance: String, // "Required", "Important", "Optional"
    pub confidence: f64,
    pub collected_at: String,
}

/// Missing evidence that should be collected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingEvidenceItem {
    pub needed: String,
    pub description: String,
    pub importance: String,
}

/// Evidence status summary displayed in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStatus {
    pub collected: Vec<SessionEvidence>,
    pub missing: Vec<MissingEvidenceItem>,
    pub confidence: f64,
    pub is_sufficient: bool,
    pub collected_count: usize,
    pub missing_count: usize,
}

// ── Workflow Progress Types ──────────────────────────────────────

/// Step in a workflow displayed in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStepCard {
    pub id: String,
    pub title: String,
    pub description: String,
    pub commands: Vec<String>,
    pub status: WorkflowStepStatus,
    pub observation: Option<String>,
    pub risk_level: String,
    pub requires_approval: bool,
}

/// Progress of a workflow step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowStepStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
}

/// Active workflow progress displayed in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgressCard {
    pub workflow_id: String,
    pub workflow_name: String,
    pub problem_category: String,
    pub steps: Vec<WorkflowStepCard>,
    pub current_step_index: usize,
    pub completed_steps: Vec<String>,
    pub completion_percentage: f64,
    pub started_at: String,
}

// ── Timeline Types ───────────────────────────────────────────────

/// A timeline event displayed in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub timestamp: String,
    pub event_type: String, // "guidance", "action", "dismissal", "detection", "evidence", "missing"
    pub title: Option<String>,
    pub technology: Option<String>,
    pub finding: Option<String>,
    pub description: Option<String>,
    pub confidence: Option<f64>,
    pub recommendation_id: Option<String>,
}

// ── Feedback Types ───────────────────────────────────────────────

/// Feedback type displayed as buttons in the UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FeedbackType {
    Useful,
    NotUseful,
    AlreadyCompleted,
    Incorrect,
    DifferentApproach,
}

impl FeedbackType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Useful => "Useful",
            Self::NotUseful => "Not useful",
            Self::AlreadyCompleted => "Already completed",
            Self::Incorrect => "Incorrect",
            Self::DifferentApproach => "Different approach",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Useful => "👍",
            Self::NotUseful => "👎",
            Self::AlreadyCompleted => "⏭",
            Self::Incorrect => "❌",
            Self::DifferentApproach => "🔄",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Useful => "#4ade80",
            Self::NotUseful => "#f87171",
            Self::AlreadyCompleted => "#facc15",
            Self::Incorrect => "#f87171",
            Self::DifferentApproach => "#60a5fa",
        }
    }
}

/// Recorded feedback from the engineer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedFeedback {
    pub id: String,
    pub recommendation_id: String,
    pub feedback_type: FeedbackType,
    pub notes: Option<String>,
    pub timestamp: String,
}

/// Feedback statistics displayed in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackStatsCard {
    pub total: usize,
    pub useful_count: usize,
    pub not_useful_count: usize,
    pub redundant_count: usize,
    pub incorrect_count: usize,
    pub different_approach_count: usize,
    pub average_helpfulness: f64,
    pub is_positive: bool,
    pub needs_adjustment: bool,
}

// ── Mode Selection ───────────────────────────────────────────────

/// Copilot mode selected in the UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CopilotMode {
    Teaching,
    Balanced,
    Expert,
    Silent,
}

impl CopilotMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Teaching => "Teaching",
            Self::Balanced => "Balanced",
            Self::Expert => "Expert",
            Self::Silent => "Silent",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Teaching => "Detailed explanations, assumes no prior knowledge",
            Self::Balanced => "Moderate explanations, default mode",
            Self::Expert => "Concise suggestions, assumes deep knowledge",
            Self::Silent => "No proactive suggestions, only responds to questions",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Teaching => "🎓",
            Self::Balanced => "⚖️",
            Self::Expert => "🧠",
            Self::Silent => "🤫",
        }
    }
}

// ── Guidance Panel State ─────────────────────────────────────────

/// The guidance panel backend.
///
/// Manages recommendation cards, evidence, workflows, timeline,
/// feedback, and copilot mode for the desktop UI.
pub struct GuidancePanel {
    /// Active recommendation cards.
    recommendations: Mutex<Vec<RecommendationCard>>,
    /// Session evidence.
    evidence: Mutex<EvidenceStatus>,
    /// Active workflow progress.
    workflow_progress: Mutex<Option<WorkflowProgressCard>>,
    /// Guidance timeline events.
    timeline: Mutex<Vec<TimelineEvent>>,
    /// Recorded feedback.
    feedback: Mutex<Vec<RecordedFeedback>>,
    /// Current copilot mode.
    current_mode: Mutex<CopilotMode>,
    /// Suppressed recommendation IDs.
    suppressed: Mutex<Vec<String>>,
}

impl GuidancePanel {
    /// Creates a new guidance panel.
    pub fn new() -> Self {
        Self {
            recommendations: Mutex::new(Vec::new()),
            evidence: Mutex::new(EvidenceStatus {
                collected: Vec::new(),
                missing: Vec::new(),
                confidence: 0.0,
                is_sufficient: false,
                collected_count: 0,
                missing_count: 0,
            }),
            workflow_progress: Mutex::new(None),
            timeline: Mutex::new(Vec::new()),
            feedback: Mutex::new(Vec::new()),
            current_mode: Mutex::new(CopilotMode::Balanced),
            suppressed: Mutex::new(Vec::new()),
        }
    }

    /// Returns a static instance.
    pub fn instance() -> &'static Self {
        static INSTANCE: Lazy<GuidancePanel> = Lazy::new(GuidancePanel::new);
        &INSTANCE
    }

    // ── Recommendation Card Commands ───────────────────────────

    /// Updates an active recommendation's status.
    pub async fn update_recommendation_status(&self, rec_id: &str, new_status: RecommendationStatus) -> Result<()> {
        let mut recs = self.recommendations.lock().await;
        if let Some(rec) = recs.iter_mut().find(|r| r.id == rec_id) {
            rec.status = new_status;
            tracing::info!(rec_id = %rec_id, "Recommendation status updated");
            return Ok(());
        }
        Err(anyhow::anyhow!("Recommendation '{}' not found", rec_id))
    }

    /// Dismisses a recommendation.
    pub async fn dismiss_recommendation(&self, rec_id: &str) -> Result<()> {
        self.update_recommendation_status(rec_id, RecommendationStatus::Dismissed).await
    }

    /// Returns all active (non-dismissed) recommendation cards.
    pub async fn active_recommendations(&self) -> Vec<RecommendationCard> {
        let recs = self.recommendations.lock().await;
        recs.iter()
            .filter(|r| r.status == RecommendationStatus::Active)
            .cloned()
            .collect()
    }

    /// Returns all recommendation cards regardless of status.
    pub async fn all_recommendations(&self) -> Vec<RecommendationCard> {
        let recs = self.recommendations.lock().await;
        recs.clone()
    }

    /// Adds a recommendation card generated from observations.
    pub async fn add_recommendation(
        &self,
        title: &str,
        description: &str,
        reason: &str,
        technology: &str,
        category: &str,
        confidence: f64,
        risk_level: CardRiskLevel,
        reference_docs: Vec<ReferenceDoc>,
        recommended_next_step: Option<String>,
    ) -> Result<()> {
        let mut recs = self.recommendations.lock().await;
        // Avoid duplicates: skip if same title already exists as Active
        if recs.iter().any(|r| r.title == title && r.status == RecommendationStatus::Active) {
            return Ok(());
        }
        recs.push(RecommendationCard {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            technology: technology.to_string(),
            category: category.to_string(),
            description: description.to_string(),
            reason: reason.to_string(),
            confidence,
            evidence: Vec::new(),
            recommended_next_step,
            reference_docs,
            risk_level: Some(risk_level),
            status: RecommendationStatus::Active,
            created_at: chrono::Utc::now().to_rfc3339(),
        });
        tracing::info!(title, "Generated recommendation");
        Ok(())
    }

    // ── Evidence Commands ──────────────────────────────────────

    /// Adds evidence to the session.
    pub async fn add_evidence(
        &self,
        source: &str,
        finding: &str,
        importance: &str,
        confidence: f64,
    ) -> Result<()> {
        let mut ev = self.evidence.lock().await;
        let evidence = SessionEvidence {
            id: Uuid::new_v4().to_string(),
            source: source.to_string(),
            finding: finding.to_string(),
            importance: importance.to_string(),
            confidence,
            collected_at: chrono::Utc::now().to_rfc3339(),
        };
        ev.collected.push(evidence);
        ev.collected_count = ev.collected.len();

        // Recalculate overall confidence
        if ev.collected_count > 0 {
            let sum: f64 = ev.collected.iter().map(|e| e.confidence).sum();
            ev.confidence = sum / ev.collected_count as f64;
        }

        // Update sufficiency
        let required_missing = ev.missing.iter().any(|m| m.importance == "Required");
        ev.is_sufficient = !required_missing;

        Ok(())
    }

    /// Marks evidence as missing.
    pub async fn mark_missing(&self, needed: &str, description: &str, importance: &str) -> Result<()> {
        let mut ev = self.evidence.lock().await;
        ev.missing.push(MissingEvidenceItem {
            needed: needed.to_string(),
            description: description.to_string(),
            importance: importance.to_string(),
        });
        ev.missing_count = ev.missing.len();
        Ok(())
    }

    /// Returns current evidence status.
    pub async fn get_evidence_status(&self) -> EvidenceStatus {
        self.evidence.lock().await.clone()
    }

    // ── Workflow Progress Commands ─────────────────────────────

    /// Starts a new workflow.
    pub async fn start_workflow(
        &self,
        workflow_id: &str,
        workflow_name: &str,
        problem_category: &str,
        steps: Vec<WorkflowStepCard>,
    ) -> Result<()> {
        let progress = WorkflowProgressCard {
            workflow_id: workflow_id.to_string(),
            workflow_name: workflow_name.to_string(),
            problem_category: problem_category.to_string(),
            steps,
            current_step_index: 0,
            completed_steps: Vec::new(),
            completion_percentage: 0.0,
            started_at: chrono::Utc::now().to_rfc3339(),
        };
        *self.workflow_progress.lock().await = Some(progress);
        Ok(())
    }

    /// Marks a step as completed.
    pub async fn complete_step(&self, step_id: &str, observation: Option<String>) -> Result<()> {
        let mut wp = self.workflow_progress.lock().await;
        let progress = wp.as_mut().ok_or_else(|| anyhow::anyhow!("No active workflow"))?;

        if let Some(step) = progress.steps.iter_mut().find(|s| s.id == step_id) {
            step.status = WorkflowStepStatus::Completed;
            step.observation = observation;
            progress.completed_steps.push(step_id.to_string());

            // Update completion percentage
            let total = progress.steps.len();
            progress.completion_percentage = if total > 0 {
                (progress.completed_steps.len() as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            // Move to next step
            if progress.completed_steps.len() < progress.steps.len() {
                let next_idx = progress.completed_steps.len();
                progress.current_step_index = next_idx;
                if next_idx < progress.steps.len() {
                    progress.steps[next_idx].status = WorkflowStepStatus::InProgress;
                }
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Step '{}' not found", step_id))
        }
    }

    /// Returns current workflow progress.
    pub async fn get_workflow_progress(&self) -> Option<WorkflowProgressCard> {
        self.workflow_progress.lock().await.clone()
    }

    // ── Timeline Commands ──────────────────────────────────────

    /// Adds an event to the guidance timeline.
    pub async fn add_timeline_event(
        &self,
        event_type: &str,
        title: Option<&str>,
        technology: Option<&str>,
        finding: Option<&str>,
        description: Option<&str>,
        confidence: Option<f64>,
        recommendation_id: Option<String>,
    ) -> Result<()> {
        let mut timeline = self.timeline.lock().await;
        timeline.push(TimelineEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type: event_type.to_string(),
            title: title.map(String::from),
            technology: technology.map(String::from),
            finding: finding.map(String::from),
            description: description.map(String::from),
            confidence,
            recommendation_id,
        });
        Ok(())
    }

    /// Returns all timeline events, ordered chronologically.
    pub async fn get_timeline(&self) -> Vec<TimelineEvent> {
        let timeline = self.timeline.lock().await;
        timeline.clone()
    }

    /// Returns recent events (last N minutes).
    pub async fn get_recent_events(&self, minutes: u64) -> Vec<TimelineEvent> {
        let timeline = self.timeline.lock().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::minutes(minutes as i64);
        timeline
            .iter()
            .filter(|e| {
                chrono::DateTime::parse_from_rfc3339(&e.timestamp)
                    .map(|dt| dt >= cutoff)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    // ── Feedback Commands ──────────────────────────────────────

    /// Records feedback for a recommendation.
    pub async fn record_feedback(
        &self,
        recommendation_id: &str,
        feedback_type: FeedbackType,
        notes: Option<&str>,
    ) -> Result<()> {
        let mut fb = self.feedback.lock().await;
        let mut suppressed = self.suppressed.lock().await;

        let record = RecordedFeedback {
            id: Uuid::new_v4().to_string(),
            recommendation_id: recommendation_id.to_string(),
            feedback_type: feedback_type.clone(),
            notes: notes.map(String::from),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        fb.push(record);

        // Suppress recommendation on negative feedback
        if matches!(
            feedback_type,
            FeedbackType::NotUseful | FeedbackType::Incorrect | FeedbackType::AlreadyCompleted
        )
            && !suppressed.contains(&recommendation_id.to_string()) {
                suppressed.push(recommendation_id.to_string());
                // Also update the card status if it exists
                let mut recs = self.recommendations.lock().await;
                if let Some(rec) = recs.iter_mut().find(|r| r.id == recommendation_id) {
                    rec.status = match feedback_type {
                        FeedbackType::AlreadyCompleted => RecommendationStatus::Skipped,
                        FeedbackType::Incorrect => RecommendationStatus::Rejected,
                        _ => RecommendationStatus::Rejected,
                    };
                }
            }

        Ok(())
    }

    /// Returns current feedback statistics.
    pub async fn get_feedback_stats(&self) -> FeedbackStatsCard {
        let fb = self.feedback.lock().await;
        let total = fb.len();
        let mut useful = 0;
        let mut not_useful = 0;
        let mut redundant = 0;
        let mut incorrect = 0;
        let mut different_approach = 0;
        let mut helpfulness_sum = 0.0;

        for record in fb.iter() {
            match &record.feedback_type {
                FeedbackType::Useful => {
                    useful += 1;
                    helpfulness_sum += 1.0;
                }
                FeedbackType::NotUseful => {
                    not_useful += 1;
                }
                FeedbackType::AlreadyCompleted => {
                    redundant += 1;
                    helpfulness_sum += 0.5;
                }
                FeedbackType::Incorrect => {
                    incorrect += 1;
                }
                FeedbackType::DifferentApproach => {
                    different_approach += 1;
                    helpfulness_sum += 0.5;
                }
            }
        }

        let average_helpfulness = if total > 0 {
            helpfulness_sum / total as f64
        } else {
            0.5
        };

        FeedbackStatsCard {
            total,
            useful_count: useful,
            not_useful_count: not_useful,
            redundant_count: redundant,
            incorrect_count: incorrect,
            different_approach_count: different_approach,
            average_helpfulness,
            is_positive: total > 0 && average_helpfulness > 0.6,
            needs_adjustment: total > 0 && (incorrect as f64 / total as f64) > 0.3,
        }
    }

    // ── Mode Commands ──────────────────────────────────────────

    /// Sets the current copilot mode.
    pub async fn set_mode(&self, mode: CopilotMode) -> Result<()> {
        *self.current_mode.lock().await = mode.clone();
        tracing::info!(mode = %mode.label(), "Copilot mode changed");
        Ok(())
    }

    /// Returns the current copilot mode.
    pub async fn get_mode(&self) -> CopilotMode {
        self.current_mode.lock().await.clone()
    }

    /// Returns all available modes for the UI.
    pub async fn available_modes(&self) -> Vec<CopilotMode> {
        vec![
            CopilotMode::Teaching,
            CopilotMode::Balanced,
            CopilotMode::Expert,
            CopilotMode::Silent,
        ]
    }

    // ── Utility Commands ───────────────────────────────────────

    /// Clears all guidance state (reset session).
    pub async fn clear_all(&self) -> Result<()> {
        self.recommendations.lock().await.clear();
        *self.evidence.lock().await = EvidenceStatus {
            collected: Vec::new(),
            missing: Vec::new(),
            confidence: 0.0,
            is_sufficient: false,
            collected_count: 0,
            missing_count: 0,
        };
        *self.workflow_progress.lock().await = None;
        self.timeline.lock().await.clear();
        self.feedback.lock().await.clear();
        *self.suppressed.lock().await = Vec::new();
        tracing::info!("Guidance panel state cleared");
        Ok(())
    }
}

// ── Tauri IPC Commands ───────────────────────────────────────────

/// Tauri IPC command to get all active recommendation cards.
#[tauri::command]
pub fn guidance_get_active_recommendations() -> Vec<RecommendationCard> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current().block_on(panel.active_recommendations())
}

/// Tauri IPC command to get all recommendation cards (any status).
#[tauri::command]
pub fn guidance_get_all_recommendations() -> Vec<RecommendationCard> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current().block_on(panel.all_recommendations())
}

/// Tauri IPC command to dismiss a recommendation card.
#[tauri::command]
pub fn guidance_dismiss_recommendation(rec_id: String) -> Result<(), String> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current()
        .block_on(panel.dismiss_recommendation(&rec_id))
        .map_err(|e| e.to_string())
}

/// Tauri IPC command to update recommendation status.
#[tauri::command]
pub fn guidance_update_recommendation_status(
    rec_id: String,
    status: RecommendationStatus,
) -> Result<(), String> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current()
        .block_on(panel.update_recommendation_status(&rec_id, status))
        .map_err(|e| e.to_string())
}

/// Tauri IPC command to get evidence status.
#[tauri::command]
pub fn guidance_get_evidence_status() -> EvidenceStatus {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current().block_on(panel.get_evidence_status())
}

/// Tauri IPC command to add evidence.
#[tauri::command]
pub fn guidance_add_evidence(
    source: String,
    finding: String,
    importance: String,
    confidence: f64,
) -> Result<(), String> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current()
        .block_on(panel.add_evidence(&source, &finding, &importance, confidence))
        .map_err(|e| e.to_string())
}

/// Tauri IPC command to mark evidence as missing.
#[tauri::command]
pub fn guidance_mark_missing(
    needed: String,
    description: String,
    importance: String,
) -> Result<(), String> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current()
        .block_on(panel.mark_missing(&needed, &description, &importance))
        .map_err(|e| e.to_string())
}

/// Tauri IPC command to get workflow progress.
#[tauri::command]
pub fn guidance_get_workflow_progress() -> Option<WorkflowProgressCard> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current().block_on(panel.get_workflow_progress())
}

/// Tauri IPC command to start a workflow.
#[tauri::command]
pub fn guidance_start_workflow(
    workflow_id: String,
    workflow_name: String,
    problem_category: String,
    steps: Vec<WorkflowStepCard>,
) -> Result<(), String> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current()
        .block_on(panel.start_workflow(&workflow_id, &workflow_name, &problem_category, steps))
        .map_err(|e| e.to_string())
}

/// Tauri IPC command to complete a workflow step.
#[tauri::command]
pub fn guidance_complete_step(
    step_id: String,
    observation: Option<String>,
) -> Result<(), String> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current()
        .block_on(panel.complete_step(&step_id, observation))
        .map_err(|e| e.to_string())
}

/// Tauri IPC command to get the full timeline.
#[tauri::command]
pub fn guidance_get_timeline() -> Vec<TimelineEvent> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current().block_on(panel.get_timeline())
}

/// Tauri IPC command to add a timeline event.
#[tauri::command]
pub fn guidance_add_timeline_event(
    event_type: String,
    title: Option<String>,
    technology: Option<String>,
    finding: Option<String>,
    description: Option<String>,
    confidence: Option<f64>,
    recommendation_id: Option<String>,
) -> Result<(), String> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current().block_on(panel.add_timeline_event(
        &event_type,
        title.as_deref(),
        technology.as_deref(),
        finding.as_deref(),
        description.as_deref(),
        confidence,
        recommendation_id,
    ))
    .map_err(|e| e.to_string())
}

/// Tauri IPC command to get recent timeline events.
#[tauri::command]
pub fn guidance_get_recent_events(minutes: u64) -> Vec<TimelineEvent> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current().block_on(panel.get_recent_events(minutes))
}

/// Tauri IPC command to record feedback.
#[tauri::command]
pub fn guidance_record_feedback(
    recommendation_id: String,
    feedback_type: FeedbackType,
    notes: Option<String>,
) -> Result<(), String> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current().block_on(panel.record_feedback(&recommendation_id, feedback_type, notes.as_deref()))
        .map_err(|e| e.to_string())
}

/// Tauri IPC command to get feedback statistics.
#[tauri::command]
pub fn guidance_get_feedback_stats() -> FeedbackStatsCard {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current().block_on(panel.get_feedback_stats())
}

/// Tauri IPC command to set copilot mode.
#[tauri::command]
pub fn guidance_set_mode(mode: CopilotMode) -> Result<(), String> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current().block_on(panel.set_mode(mode)).map_err(|e| e.to_string())
}

/// Tauri IPC command to get current copilot mode.
#[tauri::command]
pub fn guidance_get_mode() -> CopilotMode {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current().block_on(panel.get_mode())
}

/// Tauri IPC command to get all available copilot modes.
#[tauri::command]
pub fn guidance_get_available_modes() -> Vec<CopilotMode> {
    let _panel = GuidancePanel::instance();
    vec![
        CopilotMode::Teaching,
        CopilotMode::Balanced,
        CopilotMode::Expert,
        CopilotMode::Silent,
    ]
}

/// Tauri IPC command to clear all guidance state.
#[tauri::command]
pub fn guidance_clear_all() -> Result<(), String> {
    let panel = GuidancePanel::instance();
    use tokio::runtime::Handle;
    Handle::current().block_on(panel.clear_all()).map_err(|e| e.to_string())
}