//! Intent Analyzer — understand what the user is doing and why.
//!
//! Combines observations from all providers (browser, terminal, active window,
//! file activity, errors) to produce a structured "intent summary" that tells
//! the AI copilot:
//! - What the user is currently doing
//! - What their likely intent/goal is
//! - What confidence level we have in this analysis
//! - What related infrastructure/components are involved
//! - What the next logical steps might be
//!
//! This is the "brain" layer that connects raw observations into coherent
//! understanding, enabling proactive AI guidance.

use std::sync::{Arc, Mutex};

use crate::browser::{BrowserContext, BrowserErrorSeverity};
use crate::correlation::CorrelationEngine;
use crate::error_detector::{DetectedError, ErrorSeverity};
use crate::semantic_analyzer::{IntentCategory, SemanticAnalyzer};
use crate::session_tracker::{SessionState, TroubleshootingSession};

// ── Intent understanding structures ───────────────────────────────

/// What the user is doing based on observation.
#[derive(Debug, Clone)]
pub struct UserActivity {
    /// A one-line description of current activity.
    pub description: String,
    /// What category of activity this falls into.
    pub category: ActivityCategory,
    /// Confidence (0.0–1.0) that this is correct.
    pub confidence: f32,
}

/// High-level activity category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityCategory {
    /// Debugging / troubleshooting an issue.
    Troubleshooting,
    /// Deploying or configuring infrastructure.
    Deployment,
    /// Monitoring or checking system health.
    Monitoring,
    /// Managing or administering systems.
    Administration,
    /// Developing or coding.
    Development,
    /// General navigation / reading.
    Browsing,
    /// Communication (email, chat, etc.).
    Communication,
    /// AI Vision analysis detected something noteworthy on screen.
    VisualInsight,
    /// AI Vision detected an error or problem on screen.
    VisualError,
    /// Unknown or mixed intent.
    Unknown,
}

/// The overall user intent detected from all observations.
#[derive(Debug, Clone)]
pub struct UserIntent {
    /// High-level intent description.
    pub intent: String,
    /// Confidence in this intent (0.0–1.0).
    pub confidence: f32,
    /// Activity category.
    pub activity_category: ActivityCategory,
    /// Infrastructure components involved.
    pub infrastructure_targets: Vec<String>,
    /// Related actions the user is performing.
    pub related_actions: Vec<String>,
    /// What the user might be trying to achieve (goal).
    pub goal: Option<String>,
    /// Suggested next steps.
    pub suggested_next_steps: Vec<String>,
}

/// Represents a command correctness check result.
#[derive(Debug, Clone)]
pub struct CommandCorrectness {
    /// The command that was checked.
    pub command: String,
    /// Whether the command is generally correct/best-practice.
    pub is_correct: bool,
    /// Confidence in this assessment (0.0-1.0).
    pub confidence: f32,
    /// If incorrect, what's the suggested correct approach.
    pub suggested_alternative: Option<String>,
    /// Additional context explaining the assessment.
    pub explanation: String,
}

/// Structured intent summary for AI consumption.
#[derive(Debug, Clone)]
pub struct IntentSummary {
    /// What the user is doing right now.
    pub current_activity: Vec<UserActivity>,
    /// The overall detected intent.
    pub intent: Option<UserIntent>,
    /// Detected issues/problems.
    pub issues: Vec<IssueReport>,
    /// Infrastructure context summary.
    pub infrastructure_context: Vec<String>,
    /// Suggested guidance.
    pub suggested_guidance: Vec<String>,
    /// Command correctness check results for terminal commands.
    pub command_correctness: Vec<CommandCorrectness>,
}

/// A reported issue found during observation analysis.
#[derive(Debug, Clone)]
pub struct IssueReport {
    pub severity: IssueSeverity,
    pub title: String,
    pub description: String,
    pub source: String, // "browser", "terminal", "system", etc.
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// The intent analyzer — synthesizes all observations into understanding.
#[derive(Debug, Clone)]
pub struct IntentAnalyzer {
    semantic_analyzer: Arc<SemanticAnalyzer>,
    state: Arc<Mutex<IntentAnalyzerState>>,
    /// Optional reference to VisionAnalyzerProvider for cross-context correlation.
    vision_provider: Arc<Mutex<Option<crate::vision_analyzer::VisionAnalysisResult>>>,
}

#[derive(Debug, Clone)]
struct IntentAnalyzerState {
    /// Last analyzed intent.
    last_intent: Option<UserIntent>,
    /// History of intents (last 20).
    intent_history: Vec<UserIntent>,
}

impl IntentAnalyzer {
    pub fn new() -> Self {
        Self {
            semantic_analyzer: Arc::new(SemanticAnalyzer::new()),
            state: Arc::new(Mutex::new(IntentAnalyzerState {
                last_intent: None,
                intent_history: Vec::new(),
            })),
            vision_provider: Arc::new(Mutex::new(None)),
        }
    }

    /// Analyze all available observations and produce a complete intent summary.
    pub fn analyze(
        &self,
        browser_ctx: Option<&BrowserContext>,
        terminal_command: Option<&str>,
        terminal_output: Option<&str>,
        active_app: Option<&str>,
        errors: &[DetectedError],
        session_state: Option<&TroubleshootingSession>,
        _correlation: &CorrelationEngine,
        vision: Option<&crate::vision_analyzer::VisionAnalysisResult>,
    ) -> IntentSummary {
        let mut summary = IntentSummary {
            current_activity: Vec::new(),
            intent: None,
            issues: Vec::new(),
            infrastructure_context: Vec::new(),
            suggested_guidance: Vec::new(),
            command_correctness: Vec::new(),
        };

        // 1. Analyze browser activity
        if let Some(ctx) = browser_ctx {
            self.analyze_browser(ctx, &mut summary);
        }

        // 2. Analyze terminal commands and output
        if let Some(cmd) = terminal_command {
            self.analyze_terminal_command(cmd, terminal_output, &mut summary);
        }

        // 3. Analyze active application
        if let Some(app) = active_app {
            self.analyze_active_app(app, &mut summary);
        }

        // 4. Analyze detected errors
        if !errors.is_empty() {
            self.analyze_errors(errors, &mut summary);
        }

        // 5. Analyze troubleshooting session state
        if let Some(session) = session_state {
            self.analyze_session(session, &mut summary);
        }

        // 5.5. Analyze Vision AI results (Phase 3)
        if let Some(vision_ref) = vision {
            self.analyze_vision(vision_ref, &mut summary);
        }

        // 6. Synthesize overall intent from all signals
        if !summary.current_activity.is_empty() || !summary.issues.is_empty() {
            let intent = self.synthesize_intent(&summary);
            if let Some(ref i) = intent {
                let mut state = self.state.lock().unwrap();
                state.last_intent = Some(i.clone());
                state.intent_history.push(i.clone());
                if state.intent_history.len() > 20 {
                    let cutoff = state.intent_history.len() - 20;
                    state.intent_history.drain(0..cutoff);
                }
            }
            summary.intent = intent;
        }

        // 7. Generate suggested guidance
        if let Some(ref intent) = summary.intent {
            let guidance = self.generate_guidance(intent);
            summary.suggested_guidance.extend(guidance);
        }

        summary
    }

    /// Analyze browser activity and add to the summary.
    fn analyze_browser(&self, ctx: &BrowserContext, summary: &mut IntentSummary) {
        if let Some(ref url) = ctx.url {
            let mut activity = UserActivity {
                description: format!("Browsing {}", url),
                category: ActivityCategory::Browsing,
                confidence: 0.8,
            };

            // Determine specific activity from URL and content
            let url_lower = url.to_lowercase();

            if ctx.is_engineering_portal {
                activity.category = ActivityCategory::Monitoring;
                activity.description = format!(
                    "Viewing engineering portal: {} ({})",
                    ctx.title.as_deref().unwrap_or("unknown"),
                    url
                );
                activity.confidence = 0.9;

                // Infrastructure context
                summary.infrastructure_context.push(format!(
                    "Engineering portal: {} ({})",
                    url,
                    ctx.title.as_deref().unwrap_or("unknown")
                ));
            } else if url_lower.contains("mysql") || url_lower.contains("phpmyadmin") {
                activity.description = format!(
                    "Viewing MySQL/PHPMyAdmin: {}",
                    ctx.title.as_deref().unwrap_or("database manager")
                );
                summary.infrastructure_context.push("MySQL database interface active".to_string());
            } else if url_lower.contains("grafana") || url_lower.contains("prometheus") {
                activity.category = ActivityCategory::Monitoring;
                activity.description = format!(
                    "Viewing monitoring dashboard: {}",
                    ctx.title.as_deref().unwrap_or("dashboard")
                );
                summary.infrastructure_context.push("Monitoring dashboard (Grafana/Prometheus) active".to_string());
            } else if url_lower.contains("github") || url_lower.contains("gitlab") {
                activity.category = ActivityCategory::Development;
                activity.description = format!(
                    "Viewing code repository: {}",
                    ctx.title.as_deref().unwrap_or("repo")
                );
                summary.infrastructure_context.push("Code repository browser active".to_string());
            } else if url_lower.contains("kubernetes") || url_lower.contains("k8s") || url_lower.contains("openshift") {
                activity.category = ActivityCategory::Monitoring;
                activity.description = format!(
                    "Viewing Kubernetes/OpenShift console: {}",
                    ctx.title.as_deref().unwrap_or("console")
                );
                summary.infrastructure_context.push("Kubernetes/OpenShift console active".to_string());
            }

            // Detect errors from page content
            if let Some(ref text) = ctx.visible_text {
                let text_lower = text.to_lowercase();
                for (pattern, desc) in crate::browser::ERROR_PAGE_PATTERNS {
                    if text_lower.contains(pattern) {
                        summary.issues.push(IssueReport {
                            severity: self.browser_error_severity(pattern),
                            title: desc.to_string(),
                            description: format!("Detected '{}' in page content", desc),
                            source: "browser".to_string(),
                        });
                        // If error page found, upgrade to troubleshooting
                        if summary.issues.iter().any(|i| matches!(i.severity, IssueSeverity::High | IssueSeverity::Critical)) {
                            summary.current_activity.push(UserActivity {
                                description: format!("Investigating {} error page", desc),
                                category: ActivityCategory::Troubleshooting,
                                confidence: 0.9,
                            });
                        }
                        break;
                    }
                }
            }

            // Detect errors from page errors
            if !ctx.detected_errors.is_empty() {
                for err in &ctx.detected_errors {
                    summary.issues.push(IssueReport {
                        severity: match err.severity {
                            BrowserErrorSeverity::Low => IssueSeverity::Low,
                            BrowserErrorSeverity::Medium => IssueSeverity::Medium,
                            BrowserErrorSeverity::High => IssueSeverity::High,
                            BrowserErrorSeverity::Critical => IssueSeverity::Critical,
                        },
                        title: err.description.clone(),
                        description: format!("Browser error: {}", err.pattern),
                        source: "browser".to_string(),
                    });
                }
            }

            summary.current_activity.push(activity);
        }
    }

    /// Analyze terminal command and add to the summary.
    fn analyze_terminal_command(&self, command: &str, output: Option<&str>, summary: &mut IntentSummary) {
        // Use semantic analyzer to understand the command
        if let Some(intent) = self.semantic_analyzer.analyze_command(command) {
            let mut activity = UserActivity {
                description: format!("Running: {}", command),
                category: self.intent_to_category(&intent.category),
                confidence: intent.confidence,
            };

            // Check terminal output for additional context
            if let Some(out) = output {
                let results = self.semantic_analyzer.analyze_output(out);
                for result in &results {
                    match result {
                        crate::semantic_analyzer::AnalysisResult::Error(msg) => {
                            summary.issues.push(IssueReport {
                                severity: IssueSeverity::High,
                                title: msg.clone(),
                                description: format!("Terminal output: {}", out.chars().take(200).collect::<String>()),
                                source: "terminal".to_string(),
                            });
                            activity.category = ActivityCategory::Troubleshooting;
                            activity.description = format!("Debugging error while running: {}", command);
                            activity.confidence = 0.85;
                        }
                        crate::semantic_analyzer::AnalysisResult::Warning(msg) => {
                            summary.issues.push(IssueReport {
                                severity: IssueSeverity::Medium,
                                title: msg.clone(),
                                description: format!("Terminal warning: {}", out.chars().take(200).collect::<String>()),
                                source: "terminal".to_string(),
                            });
                        }
                        crate::semantic_analyzer::AnalysisResult::Success(msg) => {
                            // Service is healthy, just note it
                            summary.infrastructure_context.push(msg.clone());
                        }
                    }
                }
            }

            summary.current_activity.push(activity);

            // Infrastructure targets
            if let Some(ref target) = intent.target {
                summary.infrastructure_context.push(format!(
                    "Terminal: {} ({})",
                    intent.action, target
                ));
            } else {
                summary.infrastructure_context.push(format!(
                    "Terminal: {}",
                    intent.action
                ));
            }

            // Phase 4: Command correctness checking
            let correctness = self.check_command_correctness(command, output);
            if !correctness.is_correct {
                // Add as a suggested guidance item
                if let Some(ref alt) = correctness.suggested_alternative {
                    summary.suggested_guidance.push(format!(
                        "⚠️ Command check: '{}' — {}. Suggestion: {}",
                        command.chars().take(80).collect::<String>(),
                        correctness.explanation,
                        alt.chars().take(300).collect::<String>()
                    ));
                }
            }
            summary.command_correctness.push(correctness);
        } else {
            // Unknown command — just note it
            summary.current_activity.push(UserActivity {
                description: format!("Running command: {}", command.chars().take(150).collect::<String>()),
                category: ActivityCategory::Unknown,
                confidence: 0.5,
            });

            // Still run correctness check on unknown commands
            let correctness = self.check_command_correctness(command, output);
            summary.command_correctness.push(correctness);
        }
    }

    /// Analyze active application and add to the summary.
    fn analyze_active_app(&self, app: &str, summary: &mut IntentSummary) {
        let app_lower = app.to_lowercase();
        let mut category = ActivityCategory::Unknown;
        let mut description = format!("Active app: {}", app);
        let mut confidence = 0.6;

        if app_lower.contains("terminal") || app_lower.contains("bash") || app_lower.contains("powershell")
            || app_lower.contains("ssh") || app_lower.contains("putty") || app_lower.contains("mintty") {
            category = ActivityCategory::Administration;
            description = format!("Working in terminal ({})", app);
            confidence = 0.8;
        } else if app_lower.contains("firefox") || app_lower.contains("chrome") || app_lower.contains("edge")
            || app_lower.contains("librewolf") {
            category = ActivityCategory::Browsing;
            description = format!("Using browser: {}", app);
            confidence = 0.8;
        } else if app_lower.contains("code") || app_lower.contains("vscode") || app_lower.contains("vim")
            || app_lower.contains("emacs") || app_lower.contains("intellij") {
            category = ActivityCategory::Development;
            description = format!("Using IDE: {}", app);
            confidence = 0.8;
        } else if app_lower.contains("mysql") || app_lower.contains("dBeaver") || app_lower.contains("pgadmin") {
            category = ActivityCategory::Development;
            description = format!("Using database tool: {}", app);
            summary.infrastructure_context.push(format!("Database client active: {}", app));
            confidence = 0.8;
        } else if app_lower.contains("docker") || app_lower.contains("kubernetes") || app_lower.contains("kube") {
            category = ActivityCategory::Monitoring;
            description = format!("Using container/orchestration tool: {}", app);
            confidence = 0.8;
        } else if app_lower.contains("gmail") || app_lower.contains("outlook") || app_lower.contains("thunderbird") {
            category = ActivityCategory::Communication;
            description = format!("Using email client: {}", app);
            confidence = 0.8;
        }

        summary.current_activity.push(UserActivity {
            description,
            category,
            confidence,
        });
    }

    /// Analyze detected errors and add to the summary.
    fn analyze_errors(&self, errors: &[DetectedError], summary: &mut IntentSummary) {
        for err in errors {
            summary.issues.push(IssueReport {
                severity: match err.severity {
                    ErrorSeverity::Low => IssueSeverity::Low,
                    ErrorSeverity::Medium => IssueSeverity::Medium,
                    ErrorSeverity::High => IssueSeverity::High,
                    ErrorSeverity::Critical => IssueSeverity::Critical,
                },
                title: err.title.clone(),
                description: err.description.clone(),
                source: format!("{:?}", err.source),
            });
        }
    }

    /// Analyze troubleshooting session state.
    fn analyze_session(&self, session: &TroubleshootingSession, summary: &mut IntentSummary) {
        if session.state != SessionState::Idle {
            summary.current_activity.push(UserActivity {
                description: format!(
                    "Troubleshooting session: {:?} ({} steps completed)",
                    session.state,
                    session.steps.len()
                ),
                category: ActivityCategory::Troubleshooting,
                confidence: 0.8,
            });

            summary.infrastructure_context.push(format!(
                "Troubleshooting target: {}",
                session.target_system.as_deref().unwrap_or("unknown")
            ));

            if let Some(ref hyp) = session.current_hypothesis {
                summary.infrastructure_context.push(format!(
                    "Active hypothesis: {}",
                    hyp.chars().take(200).collect::<String>()
                ));
            }

            if let Some(ref next) = session.suggested_next_step {
                summary.suggested_guidance.push(format!(
                    "Session tracker suggests next step: {}",
                    next.chars().take(200).collect::<String>()
                ));
            }
        }
    }

    /// Analyze Vision AI analysis result and add to the summary.
    /// This is the Phase 3 bridge: raw system events → Vision analysis → intent synthesis.
    fn analyze_vision(&self, vision: &crate::vision_analyzer::VisionAnalysisResult, summary: &mut IntentSummary) {
        // Record the focused app context that was analyzed
        if let Some(ref app) = vision.focused_app {
            summary.current_activity.push(UserActivity {
                description: format!("Screen focused: {}", app),
                category: ActivityCategory::Unknown,
                confidence: 0.9,
            });
        }

        // If AI detected user activity on screen, add it
        if let Some(ref activity) = vision.user_activity {
            summary.current_activity.push(UserActivity {
                description: format!("AI Vision detected activity: {}", activity),
                category: ActivityCategory::Unknown,
                confidence: 0.8,
            });
        }

        // If AI detected an inferred intent from the screen, add it
        if let Some(ref intent) = vision.inferred_intent {
            summary.infrastructure_context.push(format!("User's screen activity: {}", intent));
        }

        // If AI detected errors on screen, add them as high-severity issues
        for err in &vision.errors_detected {
            summary.issues.push(IssueReport {
                severity: IssueSeverity::High,
                title: format!("Screen error detected: {}", err.description),
                description: format!("Vision AI detected a {} severity error on screen", err.severity),
                source: "vision_analyzer".to_string(),
            });
        }

        // If AI found suggestions, add them as guidance
        for sug in &vision.suggestions {
            summary.suggested_guidance.push(format!("AI Vision suggests: {}", sug));
        }
    }

    /// Synthesize an overall intent from the collected signals.
    fn synthesize_intent(&self, summary: &IntentSummary) -> Option<UserIntent> {
        let mut confidence = 0.0f32;
        let mut activity_count = 0u32;
        let mut issues_count = 0u32;
        let mut has_troubleshooting = false;
        let mut has_deployment = false;
        let mut has_monitoring = false;
        let mut has_admin = false;
        let mut infrastructure_targets: Vec<String> = Vec::new();
        let mut related_actions: Vec<String> = Vec::new();

        for activity in &summary.current_activity {
            activity_count += 1;
            confidence += activity.confidence;
            if activity.description.len() > 200 {
                related_actions.push(activity.description.chars().take(200).collect::<String>());
            } else {
                related_actions.push(activity.description.clone());
            }

            match activity.category {
                ActivityCategory::Troubleshooting => has_troubleshooting = true,
                ActivityCategory::Deployment => has_deployment = true,
                ActivityCategory::Monitoring => has_monitoring = true,
                ActivityCategory::Administration => has_admin = true,
                _ => {}
            }
        }

        for issue in &summary.issues {
            issues_count += 1;
            if matches!(issue.severity, IssueSeverity::High | IssueSeverity::Critical) {
                has_troubleshooting = true;
                confidence += 0.9;
            }
        }

        for ctx in &summary.infrastructure_context {
            if !ctx.is_empty() && ctx.len() < 100 {
                infrastructure_targets.push(ctx.clone());
            }
        }

        confidence = confidence / (activity_count + 1).max(1) as f32;
        confidence = confidence.min(0.99);

        let (intent, category, goal) = if has_troubleshooting {
            (
                "Troubleshooting an issue".to_string(),
                ActivityCategory::Troubleshooting,
                Some("User appears to be diagnosing and fixing a problem".to_string()),
            )
        } else if has_deployment {
            (
                "Deploying or configuring infrastructure".to_string(),
                ActivityCategory::Deployment,
                Some("User is setting up, deploying, or configuring systems".to_string()),
            )
        } else if has_monitoring {
            (
                "Monitoring system health and performance".to_string(),
                ActivityCategory::Monitoring,
                Some("User is checking the status and performance of infrastructure".to_string()),
            )
        } else if has_admin {
            (
                "Performing system administration tasks".to_string(),
                ActivityCategory::Administration,
                Some("User is managing systems, services, or infrastructure".to_string()),
            )
        } else if activity_count == 1 && activity_count > 0 {
            (
                "General browsing or navigation".to_string(),
                ActivityCategory::Browsing,
                Some("User is viewing information or navigating".to_string()),
            )
        } else if !related_actions.is_empty() {
            (
                "Mixed activity — multiple tasks in progress".to_string(),
                ActivityCategory::Unknown,
                Some("User is engaged in several activities".to_string()),
            )
        } else {
            return None;
        };

        // Generate suggested next steps
        let mut suggested_steps = Vec::new();
        if has_troubleshooting && issues_count > 0 {
            suggested_steps.push("Check the most recent error logs for root cause details".to_string());
            suggested_steps.push("Verify the affected service is running and accessible".to_string());
            if infrastructure_targets.len() > 1 {
                suggested_steps.push(format!(
                    "Cross-check correlations between {} to identify shared failure points",
                    infrastructure_targets.iter().take(3).map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                ));
            }
        }
        if has_deployment {
            suggested_steps.push("Verify prerequisites before proceeding".to_string());
            suggested_steps.push("Consider running a health check after deployment".to_string());
        }

        if activity_count > 0 && issues_count == 0 && !has_troubleshooting {
            suggested_steps.push("Everything looks stable — no immediate issues detected".to_string());
        }

        Some(UserIntent {
            intent,
            confidence,
            activity_category: category,
            infrastructure_targets,
            related_actions,
            goal,
            suggested_next_steps: suggested_steps,
        })
    }

    /// Generate guidance suggestions based on the detected intent.
    fn generate_guidance(&self, intent: &UserIntent) -> Vec<String> {
        let mut suggestions = Vec::new();
        match intent.activity_category {
            ActivityCategory::Troubleshooting => {
                let has_log_check = intent
                    .infrastructure_targets
                    .iter()
                    .any(|c| c.contains("log") || c.contains("journalctl") || c.contains("tail"));
                if !has_log_check && intent.confidence > 0.7 {
                    suggestions.push(
                        "You should check the recent logs for the affected service to understand what's failing".to_string()
                    );
                }

                let has_status_check = intent
                    .infrastructure_targets
                    .iter()
                    .any(|c| c.contains("status") || c.contains("systemctl"));
                if !has_status_check && intent.confidence > 0.7 {
                    suggestions.push(
                        "Before digging deeper, verify the service is actually down with `systemctl status <service>`".to_string()
                    );
                }
            }
            ActivityCategory::Deployment => {
                suggestions.push(
                    "Make sure to verify the deployment didn't introduce any regressions".to_string()
                );
            }
            ActivityCategory::Monitoring => {
                suggestions.push(
                    "Based on what you're monitoring, consider also checking related services for dependency issues".to_string()
                );
            }
            _ => {}
        }
        suggestions
    }

    /// Determine severity for browser error patterns.
    fn browser_error_severity(&self, pattern: &str) -> IssueSeverity {
        let p = pattern.to_lowercase();
        if p.contains("500") || p.contains("502") || p.contains("503") || p.contains("504") {
            IssueSeverity::High
        } else if p.contains("403") || p.contains("404") {
            IssueSeverity::Medium
        } else if p.contains("connection refused") || p.contains("unable to connect") {
            IssueSeverity::High
        } else if p.contains("dns") || p.contains("ssl") || p.contains("certificate") {
            IssueSeverity::High
        } else if p.contains("error") || p.contains("warning") {
            IssueSeverity::Medium
        } else {
            IssueSeverity::Low
        }
    }

    /// Convert semantic intent category to activity category.
    fn intent_to_category(&self, category: &IntentCategory) -> ActivityCategory {
        match category {
            IntentCategory::ServiceHealthCheck => ActivityCategory::Monitoring,
            IntentCategory::ServiceStartStop => ActivityCategory::Deployment,
            IntentCategory::LogInspection => ActivityCategory::Troubleshooting,
            IntentCategory::NetworkDiagnostic => ActivityCategory::Troubleshooting,
            IntentCategory::ConfigurationChange => ActivityCategory::Deployment,
            IntentCategory::DataQuery => ActivityCategory::Development,
            IntentCategory::Deployment => ActivityCategory::Deployment,
            IntentCategory::Troubleshooting => ActivityCategory::Troubleshooting,
            IntentCategory::General => ActivityCategory::Administration,
        }
    }

    /// Set the latest Vision analysis result for cross-context correlation.
    pub fn set_vision_result(&self, result: crate::vision_analyzer::VisionAnalysisResult) {
        let mut guard = self.vision_provider.lock().unwrap();
        *guard = Some(result);
    }

    /// Get the latest Vision analysis result, if any.
    pub fn get_vision_result(&self) -> Option<crate::vision_analyzer::VisionAnalysisResult> {
        self.vision_provider.lock().unwrap().clone()
    }

    /// Get the last analyzed intent.
    pub fn get_last_intent(&self) -> Option<UserIntent> {
        self.state.lock().unwrap().last_intent.clone()
    }

    /// Check command correctness — compare terminal commands against common patterns
    /// and best practices. Returns structured results for each command.
    /// This is Phase 4: command correctness checking.
    pub fn check_command_correctness(&self, command: &str, output: Option<&str>) -> CommandCorrectness {
        let cmd_lower = command.to_lowercase().trim().to_string();

        // Check against common best-practice patterns
        // These are heuristics for common sysadmin/developer commands

        let mut is_correct = true;
        let mut suggestion: Option<String> = None;
        let mut confidence = 0.7;
        let mut explanation = String::from("Command follows standard patterns");

        // ── Dangerous commands without confirmation/safety ──
        if cmd_lower.contains("rm -rf /") || cmd_lower.contains("rm -rf /*") {
            is_correct = false;
            confidence = 0.95;
            suggestion = Some("⚠️ DANGEROUS: `rm -rf /` will delete EVERYTHING. Add a confirmation prompt or use `rm -i` to interactively confirm each file.".to_string());
            explanation = "Destructive command without safety measures".to_string();
        }

        // ── Systemctl without checking if service exists ──
        if cmd_lower.starts_with("systemctl restart ") || cmd_lower.starts_with("systemctl stop ") {
            let service_name = cmd_lower.split_whitespace().nth(2).unwrap_or("");
            if service_name.is_empty() {
                is_correct = false;
                confidence = 0.8;
                suggestion = Some("Add the service name: e.g., `systemctl restart nginx`".to_string());
                explanation = "Command missing service name argument".to_string();
            } else {
                // Best practice: check status first, then restart
                if !cmd_lower.contains("try-reload") {
                    suggestion = Some(format!(
                        "Best practice: check status first with `systemctl status {}`, then restart. If it's not running, `systemctl start {}` is more appropriate than restart.",
                        service_name, service_name
                    ));
                    is_correct = false;
                    confidence = 0.75;
                    explanation = "Missing pre-check — should verify status before restart".to_string();
                }
            }
        }

        // ── Docker without --force-recreate or restart ──
        if cmd_lower.contains("docker run ") && !cmd_lower.contains("--force-recreate") {
            // Not strictly wrong, but for idempotent deployments, --force-recreate is better
            if cmd_lower.contains("restart") || cmd_lower.contains("update") {
                suggestion = Some("For idempotent container updates, add `--force-recreate` to ensure the latest image is used.".to_string());
                confidence = 0.65;
                is_correct = false;
                explanation = "Consider using --force-recreate for idempotent container updates".to_string();
            }
        }

        // ── Nginx/Apache reload without testing config first ──
        if (cmd_lower.contains("nginx") && cmd_lower.contains("reload"))
            || (cmd_lower.contains("apache") || cmd_lower.contains("httpd"))
            && cmd_lower.contains("reload") {
            suggestion = Some("Test config first: run `nginx -t` or `apachectl configtest` before reloading to avoid downtime from bad configs.".to_string());
            confidence = 0.8;
            is_correct = false;
            explanation = "Should test config before reloading to prevent downtime".to_string();
        }

        // ── MySQL commands without specifying database ──
        if cmd_lower.starts_with("mysql") && !cmd_lower.contains("-e ") && !cmd_lower.contains("use ") {
            suggestion = Some("Consider specifying the database: `mysql -D dbname` to avoid querying the wrong database.".to_string());
            confidence = 0.6;
            is_correct = false;
            explanation = "No target database specified — risk of operating on wrong DB".to_string();
        }

        // ── Kubectl without namespace ──
        if cmd_lower.starts_with("kubectl") && !cmd_lower.contains("namespace") && !cmd_lower.contains("-n ") {
            // Not wrong, but missing namespace is common in multi-cluster environments
            if !cmd_lower.starts_with("kubectl get nodes") && !cmd_lower.starts_with("kubectl cluster-info") {
                suggestion = Some("Consider specifying a namespace: `kubectl -n <namespace> <command>` to avoid cross-namespace confusion.".to_string());
                confidence = 0.55;
                is_correct = false;
                explanation = "No namespace specified — may operate on wrong context in multi-cluster env".to_string();
            }
        }

        // ── General: commands that should use sudo ──
        if !cmd_lower.starts_with("sudo")
            && (cmd_lower.contains("systemctl")
                || cmd_lower.contains("journalctl")
                || cmd_lower.contains("iptables")
                || cmd_lower.contains("ufw ")
                || cmd_lower.contains("chmod 0")
                || cmd_lower.contains("chown root")) {
            suggestion = Some("This command typically requires elevated privileges. Consider using `sudo`.".to_string());
            confidence = 0.7;
            is_correct = false;
            explanation = "Likely needs elevated privileges — use `sudo`".to_string();
        }

        // ── General: piping cat to commands (useless use of cat) ──
        if cmd_lower.contains("cat ") && (cmd_lower.contains("| grep") || cmd_lower.contains("| awk") || cmd_lower.contains("| sed")) {
            suggestion = Some("Use the command directly instead of piping from `cat`: e.g., `grep pattern file.log` instead of `cat file.log | grep pattern`.".to_string());
            confidence = 0.85;
            is_correct = false;
            explanation = "Useless use of cat — use the command directly on the file".to_string();
        }

        // ── Check output for common warning patterns ──
        if let Some(out) = output {
            let out_lower = out.to_lowercase();
            if out_lower.contains("deprecated") || out_lower.contains("will be removed") {
                if !is_correct {
                    // Add to existing suggestion
                    if let Some(ref mut s) = suggestion {
                        s.push_str(" Additionally: output contains deprecation warnings.");
                    }
                } else {
                    is_correct = false;
                    confidence = 0.7;
                    suggestion = Some("⚠️ Output contains deprecation warnings — check if the command or flags are outdated.".to_string());
                    explanation = "Command produces deprecation warnings".to_string();
                }
            }
            if out_lower.contains("warning") || out_lower.contains("warn:") {
                if !is_correct {
                    if let Some(ref mut s) = suggestion {
                        s.push_str(" Output also contains warnings.");
                    }
                } else {
                    is_correct = false;
                    confidence = 0.65;
                    suggestion = Some("⚠️ Command output contains warnings — review for potential issues.".to_string());
                    explanation = "Command produced warnings in output".to_string();
                }
            }
        }

        CommandCorrectness {
            command: command.to_string(),
            is_correct,
            confidence,
            suggested_alternative: suggestion,
            explanation,
        }
    }
}

impl Default for IntentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}