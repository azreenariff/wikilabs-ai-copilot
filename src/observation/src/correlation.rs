//! Cross-Context Correlation Engine
//!
//! Links observations across providers (browser URLs, terminal commands, active windows)
//! to build a coherent picture of what the user is doing.
//!
//! The engine matches browser URLs with terminal commands that reference the same
//! infrastructure (e.g., browsing the OpenShift console while running `kubectl` commands).

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use chrono::Utc;

/// A correlation between multiple observation sources.
#[derive(Debug, Clone)]
pub struct CorrelationRecord {
    pub id: String,
    pub first_seen: Instant,
    pub last_seen: Instant,
    pub count: u32,
    pub browser_url: Option<String>,
    pub browser_title: Option<String>,
    pub is_engineering_portal: bool,
    pub terminal_command: Option<String>,
    pub shell: Option<String>,
    pub active_app: Option<String>,
    pub confidence: f32,
    pub correlation_type: CorrelationType,
    pub explanation: String,
}

/// The type of correlation detected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorrelationType {
    InfrastructureContext,
    TerminalMatchesBrowser,
    ApplicationTerminalAlignment,
    MultiPortalSession,
    GeneralCorrelation,
}

/// A set of all active correlations.
#[derive(Debug, Clone)]
pub struct CorrelationSet {
    pub records: Vec<CorrelationRecord>,
    pub has_engineering_context: bool,
    pub summary: String,
}

/// Infrastructure keyword sets for cross-context matching.
/// Each entry maps an infrastructure name to a list of related command keywords.
const INFRA_KEYWORDS: &[(&str, &[&str])] = &[
    ("k8s", &["kubectl", "kubectx", "helm", "openshift", "oc"]),
    ("kubernetes", &["kubectl", "kubectx", "helm", "minikube"]),
    ("docker", &["docker", "podman", "containerd", "compose"]),
    ("aws", &["aws", "awscli", "s3", "ec2", "eks"]),
    ("gcp", &["gcloud", "gsutil", "kpt"]),
    ("azure", &["az", "azure", "azcli"]),
    ("nagios", &["nagios", "check_mk", "checkmk"]),
    ("grafana", &["grafana"]),
    ("prometheus", &["prometheus", "promql", "alertmanager"]),
    ("mysql", &["mysql", "mysqld", "mysqladmin", "pt-query-digest"]),
    ("postgresql", &["psql", "pg", "postgres", "pg_dump"]),
    ("redis", &["redis-cli", "redis-server"]),
    ("mongodb", &["mongo", "mongod", "mongosh"]),
    ("jenkins", &["jenkins", "pipeline", "jfrog"]),
    ("gitlab", &["gitlab", "gitlab-ci", "gitlab-runner"]),
    ("systemctl", &["systemctl", "journalctl", "service"]),
    ("ssh", &["ssh", "scp", "sftp", "rsync", "plink"]),
    ("network", &["ping", "nslookup", "dig", "tracert", "curl", "wget", "nc"]),
];

/// Shell correlation engine state.
#[derive(Debug, Clone)]
pub struct CorrelationState {
    pub last_browser_url: Option<String>,
    pub last_browser_title: Option<String>,
    pub last_browser_is_portal: bool,
    pub last_terminal_command: Option<String>,
    pub last_terminal_shell: Option<String>,
    pub last_active_app: Option<String>,
    pub correlations: Vec<CorrelationRecord>,
    pub last_scan: Option<Instant>,
}

impl Default for CorrelationState {
    fn default() -> Self {
        Self::new()
    }
}

impl CorrelationState {
    pub fn new() -> Self {
        Self {
            last_browser_url: None,
            last_browser_title: None,
            last_browser_is_portal: false,
            last_terminal_command: None,
            last_terminal_shell: None,
            last_active_app: None,
            correlations: Vec::new(),
            last_scan: None,
        }
    }
}

/// The correlation engine.
pub struct CorrelationEngine {
    state: Arc<Mutex<CorrelationState>>,
}

impl CorrelationEngine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CorrelationState::new())),
        }
    }

    /// Update with browser context.
    pub fn update_browser_context(
        &self,
        url: Option<String>,
        title: Option<String>,
        is_portal: bool,
    ) {
        let mut state = self.state.lock().unwrap();
        state.last_browser_url = url;
        state.last_browser_title = title;
        state.last_browser_is_portal = is_portal;
        state.last_scan = Some(Instant::now());
    }

    /// Update with terminal command.
    pub fn update_terminal_context(
        &self,
        command: Option<String>,
        shell: Option<String>,
    ) {
        let mut state = self.state.lock().unwrap();
        state.last_terminal_command = command;
        state.last_terminal_shell = shell;
        state.last_scan = Some(Instant::now());
    }

    /// Update with active application.
    pub fn update_active_app(&self, app: Option<String>) {
        let mut state = self.state.lock().unwrap();
        state.last_active_app = app;
        state.last_scan = Some(Instant::now());
    }

    /// Update with full browser context.
    pub fn update_browser_context_full(&self, context: crate::browser::BrowserContext) {
        let mut state = self.state.lock().unwrap();
        state.last_browser_url = context.url.clone();
        state.last_browser_title = context.title.clone();
        state.last_browser_is_portal = context.is_engineering_portal;
        state.last_scan = Some(Instant::now());
    }

    /// Get the last-known browser context as a full BrowserContext struct.
    pub fn get_browser_context(&self) -> Option<crate::browser::BrowserContext> {
        let state = self.state.lock().unwrap();
        let ctx = &state;
        Some(crate::browser::BrowserContext {
            browser_name: ctx
                .last_browser_url
                .as_ref()
                .map(|u| {
                    let lower = u.to_lowercase();
                    if lower.contains("firefox") || lower.contains("librewolf") {
                        "LibreWolf".to_string()
                    } else if lower.contains("chrome") || lower.contains("brave") {
                        "Chrome".to_string()
                    } else if lower.contains("edge") {
                        "Edge".to_string()
                    } else if lower.contains("safari") {
                        "Safari".to_string()
                    } else {
                        "Browser".to_string()
                    }
                })
                .unwrap_or_else(|| "Browser".to_string()),
            url: ctx.last_browser_url.clone(),
            title: ctx.last_browser_title.clone(),
            is_engineering_portal: ctx.last_browser_is_portal,
            visible_text: None,
            detected_errors: Vec::new(),
        })
    }

    /// Get the last-known terminal command.
    pub fn get_terminal_command(&self) -> Option<String> {
        let state = self.state.lock().unwrap();
        state.last_terminal_command.clone()
    }

    /// Run correlation analysis on current state.
    pub fn scan(&self) -> CorrelationSet {
        let mut state = self.state.lock().unwrap();
        let mut records = Vec::new();

        // Rule 1: Infrastructure context correlation
        if let (Some(ref url), Some(ref cmd)) = (&state.last_browser_url, &state.last_terminal_command) {
            let infra_matches = self.find_infra_matches(url, cmd);
            if !infra_matches.is_empty() {
                for match_info in &infra_matches {
                    let id = format!("infra-{}", uuid::Uuid::new_v4().simple());
                    records.push(CorrelationRecord {
                        id,
                        first_seen: Instant::now(),
                        last_seen: Instant::now(),
                        count: 1,
                        browser_url: Some(url.clone()),
                        browser_title: state.last_browser_title.clone(),
                        is_engineering_portal: state.last_browser_is_portal,
                        terminal_command: Some(cmd.clone()),
                        shell: state.last_terminal_shell.clone(),
                        active_app: state.last_active_app.clone(),
                        confidence: match_info.confidence,
                        correlation_type: CorrelationType::InfrastructureContext,
                        explanation: match_info.explanation.clone(),
                    });
                }
            }
        }

        // Rule 2: Terminal matches browser content
        if let (Some(ref url), Some(ref cmd)) = (&state.last_browser_url, &state.last_terminal_command) {
            if url.contains("grafana") && (cmd.contains("grafana") || cmd.contains("alert")) {
                records.push(CorrelationRecord {
                    id: format!("match-{}", uuid::Uuid::new_v4().simple()),
                    first_seen: Instant::now(),
                    last_seen: Instant::now(),
                    count: 1,
                    browser_url: Some(url.clone()),
                    browser_title: state.last_browser_title.clone(),
                    is_engineering_portal: true,
                    terminal_command: Some(cmd.clone()),
                    shell: state.last_terminal_shell.clone(),
                    active_app: state.last_active_app.clone(),
                    confidence: 0.85,
                    correlation_type: CorrelationType::TerminalMatchesBrowser,
                    explanation: format!(
                        "Terminal command '{}' matches browser portal '{}'",
                        cmd, url
                    ),
                });
            }
        }

        // Rule 3: Active app aligns with terminal
        if let (Some(ref app), Some(ref cmd)) = (&state.last_active_app, &state.last_terminal_command) {
            let app_keywords = self.extract_app_keywords(app);
            let cmd_keywords = self.extract_app_keywords(cmd);
            let shared = app_keywords.intersection(&cmd_keywords).count();
            if shared >= 1 {
                records.push(CorrelationRecord {
                    id: format!("app-align-{}", uuid::Uuid::new_v4().simple()),
                    first_seen: Instant::now(),
                    last_seen: Instant::now(),
                    count: 1,
                    browser_url: state.last_browser_url.clone(),
                    browser_title: state.last_browser_title.clone(),
                    is_engineering_portal: state.last_browser_is_portal,
                    terminal_command: Some(cmd.clone()),
                    shell: state.last_terminal_shell.clone(),
                    active_app: Some(app.clone()),
                    confidence: (shared as f32 * 0.3).min(0.7),
                    correlation_type: CorrelationType::ApplicationTerminalAlignment,
                    explanation: format!(
                        "Active app '{}' and terminal command '{}' share context keywords",
                        app, cmd
                    ),
                });
            }
        }

        // Rule 4: Multi-portal session detection
        if state.last_browser_is_portal {
            if let Some(ref cmd) = state.last_terminal_command {
                let infra_count = INFRA_KEYWORDS.iter().filter(|(_, cmds)| {
                    cmds.iter().any(|c| cmd.to_lowercase().contains(c))
                }).count();
                if infra_count >= 2 {
                    records.push(CorrelationRecord {
                        id: format!("multi-{}", uuid::Uuid::new_v4().simple()),
                        first_seen: Instant::now(),
                        last_seen: Instant::now(),
                        count: 1,
                        browser_url: state.last_browser_url.clone(),
                        browser_title: state.last_browser_title.clone(),
                        is_engineering_portal: true,
                        terminal_command: Some(cmd.clone()),
                        shell: state.last_terminal_shell.clone(),
                        active_app: state.last_active_app.clone(),
                        confidence: 0.75,
                        correlation_type: CorrelationType::MultiPortalSession,
                        explanation: format!(
                            "Multiple infrastructure tools detected: {} commands in terminal while viewing engineering portal",
                            infra_count
                        ),
                    });
                }
            }
        }

        // Store records (replace old ones after 5 minutes)
        let five_min_ago = Instant::now() - std::time::Duration::from_secs(300);
        state.correlations.retain(|r| r.last_seen > five_min_ago);
        state.correlations.append(&mut records);

        let has_engineering = state.correlations.iter().any(|r| {
            r.is_engineering_portal
                || r.terminal_command
                    .as_deref()
                    .map(|c| {
                        INFRA_KEYWORDS
                            .iter()
                            .any(|(_, cmds)| cmds.iter().any(|cmd| c.to_lowercase().contains(cmd)))
                    })
                    .unwrap_or(false)
        });

        let summary = if records.is_empty() {
            String::new()
        } else {
            format!("{} correlations detected: ", records.len())
        };

        CorrelationSet {
            records: state.correlations.clone(),
            has_engineering_context: has_engineering,
            summary,
        }
    }

    /// Find infrastructure keyword matches between URL and command.
    fn find_infra_matches(&self, url: &str, cmd: &str) -> Vec<InfraMatch> {
        let mut matches = Vec::new();
        let (url_lower, cmd_lower) = (url.to_lowercase(), cmd.to_lowercase());

        for (infra, related_cmds) in INFRA_KEYWORDS {
            let url_matches = url_lower.contains(infra)
                || related_cmds.iter().any(|c| url_lower.contains(c));
            let cmd_matches = related_cmds.iter().any(|c| cmd_lower.contains(c));

            if url_matches && cmd_matches {
                matches.push(InfraMatch {
                    infrastructure: infra.to_string(),
                    confidence: 0.9,
                    explanation: format!(
                        "Browser on '{}' and terminal '{}' both reference {}",
                        url, cmd, infra
                    ),
                });
            }
        }

        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        matches
    }

    /// Extract keywords from a string.
    fn extract_app_keywords(&self, text: &str) -> HashSet<String> {
        let lower = text.to_lowercase();
        INFRA_KEYWORDS.iter()
            .filter_map(|(infra, _)| {
                if lower.contains(infra) {
                    Some(infra.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get engineering-relevant correlations.
    pub fn get_engineering_correlations(&self) -> CorrelationSet {
        let state = self.state.lock().unwrap();
        let eng_records: Vec<CorrelationRecord> = state.correlations.iter()
            .filter(|r| r.is_engineering_portal || r.confidence > 0.7)
            .cloned()
            .collect();

        let has_engineering = !eng_records.is_empty();
        let summary = if eng_records.is_empty() {
            String::new()
        } else {
            format!("{} engineering correlations: ", eng_records.len())
        };

        CorrelationSet {
            records: eng_records,
            has_engineering_context: has_engineering,
            summary,
        }
    }

    /// Generate a human-readable summary of the current context.
    pub fn generate_context_summary(&self) -> String {
        let state = self.state.lock().unwrap();
        let mut parts: Vec<String> = Vec::new();

        if let Some(ref url) = state.last_browser_url {
            let portal = if state.last_browser_is_portal {
                " (engineering portal)"
            } else {
                ""
            };
            parts.push(format!("Browser: {}{}", url, portal));
        }

        if let Some(ref cmd) = state.last_terminal_command {
            let shell_name = state.last_terminal_shell.as_deref().unwrap_or("unknown");
            parts.push(format!("Terminal ({}): {}", shell_name, cmd));
        }

        if let Some(ref app) = state.last_active_app {
            parts.push(format!("Active app: {}", app));
        }

        if parts.is_empty() {
            "No observation context available".to_string()
        } else {
            format!("Observation context:\n  {}", parts.join("\n  "))
        }
    }

    /// Serialize current state for logging/debugging.
    pub fn serialize_state(&self) -> serde_json::Value {
        let state = self.state.lock().unwrap();
        serde_json::json!({
            "last_browser_url": state.last_browser_url,
            "last_browser_title": state.last_browser_title,
            "last_browser_is_portal": state.last_browser_is_portal,
            "last_terminal_command": state.last_terminal_command,
            "last_terminal_shell": state.last_terminal_shell,
            "last_active_app": state.last_active_app,
            "correlation_count": state.correlations.len(),
            "last_scan": state.last_scan.map(|_| Utc::now().to_rfc3339()),
        })
    }
}

/// An infrastructure keyword match.
#[derive(Debug, Clone)]
struct InfraMatch {
    #[allow(dead_code)]
    infrastructure: String,
    confidence: f32,
    explanation: String,
}

impl Default for CorrelationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_engine_creation() {
        let engine = CorrelationEngine::new();
        let set = engine.scan();
        assert!(!set.has_engineering_context);
    }

    #[test]
    fn test_infra_correlation() {
        let engine = CorrelationEngine::new();
        engine.update_browser_context(
            Some("https://openshift.example.com/console".to_string()),
            Some("OpenShift Console".to_string()),
            true,
        );
        engine.update_terminal_context(
            Some("kubectl get pods -n production".to_string()),
            Some("PowerShell".to_string()),
        );

        let set = engine.scan();
        assert!(set.has_engineering_context);
        let eng = engine.get_engineering_correlations();
        assert!(!eng.records.is_empty());
    }

    #[test]
    fn test_no_correlation() {
        let engine = CorrelationEngine::new();
        engine.update_browser_context(
            Some("https://github.com".to_string()),
            Some("GitHub".to_string()),
            false,
        );
        engine.update_terminal_context(
            Some("npm start".to_string()),
            Some("PowerShell".to_string()),
        );
        engine.scan();
        // Just ensure no panic — correlation may or may not exist
    }

    #[test]
    fn test_multi_infra_correlation() {
        let engine = CorrelationEngine::new();
        engine.update_browser_context(
            Some("https://grafana.example.com/dashboard".to_string()),
            Some("Grafana - Dashboards".to_string()),
            true,
        );
        engine.update_terminal_context(
            Some("kubectl get pods && curl localhost:9090".to_string()),
            Some("bash".to_string()),
        );
        let set = engine.scan();
        assert!(set.has_engineering_context);
    }

    #[test]
    fn test_context_summary() {
        let engine = CorrelationEngine::new();
        engine.update_browser_context(
            Some("https://grafana.example.com".to_string()),
            Some("Grafana".to_string()),
            true,
        );
        engine.update_terminal_context(
            Some("kubectl get pods".to_string()),
            Some("bash".to_string()),
        );

        let summary = engine.generate_context_summary();
        assert!(summary.contains("grafana"));
        assert!(summary.contains("kubectl"));
    }
}