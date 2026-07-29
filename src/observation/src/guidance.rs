//! Proactive AI Guidance Popup System
//!
//! Generates actionable, conversational suggestions based on observed context.
//! The guidance system is driven by rule-based heuristics — no AI model required.
//!
//! Guidance is generated when:
//! - Browser + terminal both reference the same infrastructure (correlated engineering work)
//! - Terminal commands indicate potential issues (errors, failures)
//! - New infrastructure tools are being used
//! - Multiple engineering portals are active simultaneously
//!
//! Guidance output is structured so the frontend can render it as a toast/popup.

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};

use crate::correlation::CorrelationEngine;
use crate::shell::ShellObserver;
use crate::terminal::TerminalProvider;

/// A single guidance suggestion.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuidanceSuggestion {
    /// Unique ID.
    pub id: String,
    /// When this was generated.
    pub generated_at: DateTime<Utc>,
    /// Severity level.
    pub severity: GuidanceSeverity,
    /// The suggestion text (conversational, like a teammate).
    pub message: String,
    /// Category of suggestion.
    pub category: GuidanceCategory,
    /// Whether this is action-oriented (has specific steps).
    pub is_actionable: bool,
    /// Confidence score (0.0 - 1.0) indicating how likely this suggestion is relevant.
    pub confidence: f32,
    /// Suggested actions (if applicable).
    pub suggested_actions: Vec<String>,
    /// Related correlations that triggered this.
    pub correlated_context: Vec<String>,
    /// Whether the user has dismissed it.
    pub dismissed: bool,
}

/// Severity of a guidance suggestion.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GuidanceSeverity {
    Info,
    Warning,
    Alert,
}

/// Category of guidance.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GuidanceCategory {
    /// Suggestion about infrastructure operations.
    Infrastructure,
    /// Warning about potential issues.
    PotentialIssue,
    /// Suggestion about tool usage patterns.
    ToolUsage,
    /// Reminder about best practices.
    BestPractice,
    /// Generic observation-based suggestion.
    Observation,
}

/// Guidance engine state.
#[derive(Debug, Clone)]
pub struct GuidanceState {
    /// Active (undismissed) suggestions.
    pub active_suggestions: Vec<GuidanceSuggestion>,
    /// Dismissed suggestions (kept for history).
    pub dismissed_suggestions: Vec<GuidanceSuggestion>,
    /// Number of suggestions generated.
    pub total_generated: u32,
    /// Time of last suggestion.
    pub last_suggestion_time: Option<DateTime<Utc>>,
}

/// The proactive guidance engine.
pub struct GuidanceEngine {
    state: Arc<Mutex<GuidanceState>>,
    correlation_engine: Arc<CorrelationEngine>,
    #[allow(dead_code)]
    terminal_provider: TerminalProvider,
    shell_observer: ShellObserver,
}

impl GuidanceEngine {
    pub fn new(correlation_engine: Arc<CorrelationEngine>) -> Self {
        Self {
            state: Arc::new(Mutex::new(GuidanceState {
                active_suggestions: Vec::new(),
                dismissed_suggestions: Vec::new(),
                total_generated: 0,
                last_suggestion_time: None,
            })),
            correlation_engine,
            terminal_provider: TerminalProvider::new(),
            shell_observer: ShellObserver::new(),
        }
    }

    /// Generate guidance suggestions based on current observation state.
    pub fn generate_suggestions(&self) -> Vec<GuidanceSuggestion> {
        let engineering = self.correlation_engine.scan();

        let mut suggestions = Vec::new();

        for record in &engineering.records {
            match record.correlation_type {
                crate::correlation::CorrelationType::InfrastructureContext => {
                    suggestions.push(self.build_infra_suggestion(record));
                }
                crate::correlation::CorrelationType::TerminalMatchesBrowser => {
                    suggestions.push(self.build_match_suggestion(record));
                }
                crate::correlation::CorrelationType::MultiPortalSession => {
                    suggestions.push(self.build_multi_portal_suggestion(record));
                }
                crate::correlation::CorrelationType::ApplicationTerminalAlignment => {
                    suggestions.push(self.build_app_alignment_suggestion(record));
                }
                crate::correlation::CorrelationType::GeneralCorrelation => {
                    suggestions.push(self.build_general_suggestion(record));
                }
            }
        }

        // Check for potential issues in terminal commands
        if let Some(cmd) = self.get_last_terminal_command() {
            suggestions.extend(self.check_for_issues(&cmd));
        }

        // Filter out dismissed suggestions
        suggestions.retain(|s| !s.dismissed);

        // Update state
        {
            let mut state = self.state.lock().unwrap();
            for _s in &suggestions {
                state.total_generated += 1;
                state.last_suggestion_time = Some(Utc::now());
            }
            // Keep only active (non-dismissed) suggestions, cap at 5
            let active: Vec<GuidanceSuggestion> = suggestions.iter()
                .filter(|s| !s.dismissed)
                .take(5)
                .cloned()
                .collect();
            state.active_suggestions = active;
        }

        suggestions
    }

    /// Dismiss a suggestion.
    pub fn dismiss(&self, id: &str) {
        let mut state = self.state.lock().unwrap();
        if let Some(pos) = state.active_suggestions.iter().position(|s| s.id == id) {
            let suggestion = state.active_suggestions.remove(pos);
            let mut dismissed = suggestion;
            dismissed.dismissed = true;
            state.dismissed_suggestions.push(dismissed);
        }
    }

    /// Dismiss all suggestions.
    pub fn dismiss_all(&self) {
        let mut state = self.state.lock().unwrap();
        for s in &mut state.active_suggestions {
            s.dismissed = true;
        }
        let dismissed: Vec<GuidanceSuggestion> = state.active_suggestions.drain(..).collect();
        state.dismissed_suggestions.extend(dismissed);
    }

    /// Get current active suggestions.
    pub fn get_active_suggestions(&self) -> Vec<GuidanceSuggestion> {
        self.state.lock().unwrap().active_suggestions.clone()
    }

    /// Get all suggestions (including dismissed).
    pub fn get_all_suggestions(&self) -> Vec<GuidanceSuggestion> {
        let state = self.state.lock().unwrap();
        let mut all = state.active_suggestions.clone();
        all.extend(state.dismissed_suggestions.clone());
        all
    }

    /// Build infrastructure correlation suggestion.
    fn build_infra_suggestion(&self, record: &crate::correlation::CorrelationRecord) -> GuidanceSuggestion {
        let id = format!("infra-{}", uuid::Uuid::new_v4().simple());

        // Generate conversational, teammate-like guidance
        let message = if let (Some(url), Some(cmd)) = (&record.browser_url, &record.terminal_command) {
            format!(
                "I see you're working on {} and running {} in your terminal. \
                You might want to check {} status and verify the deployment is healthy.",
                self.shorten_url(url),
                cmd.split_whitespace().next().unwrap_or(""),
                self.extract_infra_name(url)
            )
        } else {
            "I noticed you're working with some infrastructure tools. \
            You might want to check the service status to make sure everything is running.".to_string()
        };

        let mut actions = Vec::new();
        if let Some(ref cmd) = record.terminal_command {
            if cmd.contains("kubectl") || cmd.contains("oc") {
                actions.push("Check pod status: kubectl get pods --all-namespaces".to_string());
                actions.push("Check cluster health: kubectl cluster-info".to_string());
            }
            if cmd.contains("docker") {
                actions.push("Check container logs: docker logs --tail 50 $(docker ps -q)".to_string());
                actions.push("Check disk usage: docker system df".to_string());
            }
            if cmd.contains("ssh") {
                actions.push("Check remote service status: ssh -t target 'systemctl status'".to_string());
            }
            if cmd.contains("systemctl") {
                actions.push("Check service status: systemctl status".to_string());
                actions.push("Check journal for errors: journalctl -xe --since '10 min ago'".to_string());
            }
        }

        let correlated = record.browser_url.iter()
            .chain(record.terminal_command.iter())
            .map(|s| s.chars().take(60).collect())
            .collect();

        GuidanceSuggestion {
            id,
            generated_at: Utc::now(),
            severity: GuidanceSeverity::Info,
            message,
            category: GuidanceCategory::Infrastructure,
            is_actionable: !actions.is_empty(),
            confidence: 0.7,
            suggested_actions: actions,
            correlated_context: correlated,
            dismissed: false,
        }
    }

    fn build_match_suggestion(&self, record: &crate::correlation::CorrelationRecord) -> GuidanceSuggestion {
            let id = format!("match-{}", uuid::Uuid::new_v4().simple());

            let message = if let (Some(url), Some(cmd)) = (&record.browser_url, &record.terminal_command) {
                format!(
                    "Your terminal command '{}' aligns with the {} portal you're viewing. \
                    Good timing — let me check if everything looks healthy.",
                    cmd.split_whitespace().next().unwrap_or(""),
                    self.extract_infra_name(url)
                )
            } else {
                "Your terminal activity aligns with the monitoring portal you're viewing. \
                Consider checking the related dashboard for real-time status."
                    .to_string()
            };

            GuidanceSuggestion {
                id,
                generated_at: Utc::now(),
                severity: GuidanceSeverity::Info,
                message,
                category: GuidanceCategory::Observation,
                is_actionable: false,
                confidence: 0.75,
                suggested_actions: Vec::new(),
                correlated_context: record.browser_url.iter()
                    .chain(record.terminal_command.iter())
                    .map(|s| s.chars().take(60).collect())
                    .collect(),
                dismissed: false,
            }
        }

        /// Build suggestion for multi-portal sessions.
    fn build_multi_portal_suggestion(&self, record: &crate::correlation::CorrelationRecord) -> GuidanceSuggestion {
        let id = format!("multi-{}", uuid::Uuid::new_v4().simple());

        let message = "I notice you're working across multiple infrastructure tools right now. \
            You might want to check MySQL status and verify all services are healthy \
            before making changes.".to_string();

        let mut actions = Vec::new();
        if let Some(ref cmd) = record.terminal_command {
            if cmd.contains("mysql") || cmd.contains("postgres") {
                actions.push("Check database replication status".to_string());
                actions.push("Verify connection pool settings".to_string());
            }
        }

        GuidanceSuggestion {
            id,
            generated_at: Utc::now(),
            severity: GuidanceSeverity::Warning,
            message,
            category: GuidanceCategory::BestPractice,
            is_actionable: !actions.is_empty(),
            confidence: 0.65,
            suggested_actions: actions,
            correlated_context: record.browser_url.iter()
                .chain(record.terminal_command.iter())
                .map(|s| s.chars().take(60).collect())
                .collect(),
            dismissed: false,
        }
    }

    /// Build suggestion for app-alignment correlations.
    fn build_app_alignment_suggestion(&self, record: &crate::correlation::CorrelationRecord) -> GuidanceSuggestion {
        let id = format!("app-align-{}", uuid::Uuid::new_v4().simple());

        let message = if let Some(ref app) = record.active_app {
            format!(
                "I see you have {} open and running infrastructure commands. \
                You should also check the service logs to verify everything is running correctly.",
                app
            )
        } else {
            "I noticed you're running infrastructure commands. \
            Consider checking the service logs to verify everything is running correctly.".to_string()
        };

        GuidanceSuggestion {
            id,
            generated_at: Utc::now(),
            severity: GuidanceSeverity::Info,
            message,
            category: GuidanceCategory::Observation,
            is_actionable: false,
            confidence: 0.6,
            suggested_actions: Vec::new(),
            correlated_context: record.browser_url.iter()
                .chain(record.terminal_command.iter())
                .chain(record.active_app.iter())
                .map(|s| s.chars().take(60).collect())
                .collect(),
            dismissed: false,
        }
    }

    /// Build general suggestion.
    fn build_general_suggestion(&self, record: &crate::correlation::CorrelationRecord) -> GuidanceSuggestion {
        let id = format!("gen-{}", uuid::Uuid::new_v4().simple());

        let message = "I'm observing your activity and noticed some engineering work happening. \
            You might want to double-check the configuration you're working with.".to_string();

        GuidanceSuggestion {
            id,
            generated_at: Utc::now(),
            severity: GuidanceSeverity::Info,
            message,
            category: GuidanceCategory::Observation,
            is_actionable: false,
            confidence: 0.5,
            suggested_actions: Vec::new(),
            correlated_context: record.browser_url.iter()
                .chain(record.terminal_command.iter())
                .chain(record.active_app.iter())
                .map(|s| s.chars().take(60).collect())
                .collect(),
            dismissed: false,
        }
    }

    /// Check terminal command for potential issues.
    fn check_for_issues(&self, cmd: &str) -> Vec<GuidanceSuggestion> {
        let mut suggestions = Vec::new();
        let cmd_lower = cmd.to_lowercase();

        // Check for error patterns
        let error_patterns = [
            ("error", "I see there's an error in your output. You might want to check the service logs for more details."),
            ("fail", "I noticed a failure in your command output. Consider checking the service status and restart if needed."),
            ("timeout", "I see a timeout in your output. You might want to check network connectivity and service health."),
            ("refused", "I see a connection refused error. The target service might not be running — you should check its status."),
            ("denied", "I see a permission/denied error. You might need to check your credentials or access rights."),
            ("cannot", "I see a 'cannot' error in your output. Consider checking the file/directory permissions."),
        ];

        for (pattern, message) in error_patterns {
            if cmd_lower.contains(pattern) {
                let id = format!("issue-{}-{}", uuid::Uuid::new_v4().simple(), pattern);
                suggestions.push(GuidanceSuggestion {
                    id,
                    generated_at: Utc::now(),
                    severity: GuidanceSeverity::Warning,
                    message: message.to_string(),
                    category: GuidanceCategory::PotentialIssue,
                    is_actionable: true,
                    confidence: 0.8,
                    suggested_actions: vec![
                        format!("Run: {} status", self.extract_service_name(&cmd_lower, pattern)),
                        format!("Check logs: journalctl -xe --since '5 min ago'"),
                    ],
                    correlated_context: vec![cmd.chars().take(80).collect()],
                    dismissed: false,
                });
                break; // Only one issue suggestion at a time
            }
        }

        suggestions
    }

    /// Get the last terminal command from the shell observer.
    fn get_last_terminal_command(&self) -> Option<String> {
        let commands = self.shell_observer.get_engineering_commands();
        commands.last().map(|c| c.command.clone())
    }

    /// Helper: extract infrastructure name from URL.
    fn extract_infra_name(&self, url: &str) -> String {
        let url_lower = url.to_lowercase();
        if url_lower.contains("openshift") || url_lower.contains("kubernetes") || url_lower.contains("k8s") {
            "Kubernetes/OpenShift".to_string()
        } else if url_lower.contains("grafana") {
            "Grafana".to_string()
        } else if url_lower.contains("prometheus") {
            "Prometheus".to_string()
        } else if url_lower.contains("nagios") || url_lower.contains("checkmk") {
            "Monitoring".to_string()
        } else if url_lower.contains("jenkins") {
            "CI/CD".to_string()
        } else if url_lower.contains("gitlab") {
            "GitLab".to_string()
        } else if url_lower.contains("docker") || url_lower.contains("podman") {
            "Containers".to_string()
        } else if url_lower.contains("aws") {
            "AWS".to_string()
        } else if url_lower.contains("azure") {
            "Azure".to_string()
        } else if url_lower.contains("gcp") || url_lower.contains("google") {
            "Google Cloud".to_string()
        } else {
            "infrastructure".to_string()
        }
    }

    /// Helper: shorten URL for display.
    fn shorten_url(&self, url: &str) -> String {
        if url.len() > 60 {
            format!("{}...", &url[..57])
        } else {
            url.to_string()
        }
    }

    /// Helper: extract service name from command.
    fn extract_service_name(&self, cmd: &str, _error_pattern: &str) -> String {
        // Common service names to check
        let services = [
            "systemd", "docker", "nginx", "apache", "mysql", "postgresql",
            "redis", "mongo", "elasticsearch", "kibana", "prometheus",
            "grafana", "jenkins", "gitlab", "tomcat",
        ];

        for service in services {
            if cmd.contains(service) {
                return service.to_string();
            }
        }

        "service".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::correlation::CorrelationEngine;

    #[test]
    fn test_guidance_engine_creation() {
        let engine = CorrelationEngine::new();
        let ge = GuidanceEngine::new(Arc::new(engine));
        let suggestions = ge.generate_suggestions();
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_guidance_with_correlation() {
        let engine = Arc::new(CorrelationEngine::new());
        engine.update_browser_context(
            Some("https://openshift.example.com/console".to_string()),
            Some("OpenShift Console".to_string()),
            true,
        );
        engine.update_terminal_context(
            Some("kubectl get pods -n production".to_string()),
            Some("PowerShell".to_string()),
        );

        let ge = GuidanceEngine::new(engine);
        let suggestions = ge.generate_suggestions();
        // Should generate at least one infrastructure suggestion
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].category, GuidanceCategory::Infrastructure);
    }

    #[test]
    fn test_guidance_dismiss() {
        let engine = Arc::new(CorrelationEngine::new());
        engine.update_browser_context(
            Some("https://grafana.example.com/dashboard".to_string()),
            Some("Grafana".to_string()),
            true,
        );
        engine.update_terminal_context(
            Some("kubectl get pods".to_string()),
            Some("bash".to_string()),
        );

        let ge = GuidanceEngine::new(engine);
        let suggestions = ge.generate_suggestions();
        assert!(!suggestions.is_empty());

        // Dismiss the first suggestion
        ge.dismiss(&suggestions[0].id);
        assert!(ge.get_active_suggestions().is_empty());
    }

    #[test]
    fn test_guidance_dismiss_all() {
        let engine = Arc::new(CorrelationEngine::new());
        let ge = GuidanceEngine::new(engine);
        let _suggestions = ge.generate_suggestions();
        ge.dismiss_all();
        assert!(ge.get_active_suggestions().is_empty());
    }
}