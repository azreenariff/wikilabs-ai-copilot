//! Semantic Command Analyzer
//!
//! Understands what terminal commands *mean* — not just the literal text,
//! but the user's intent. This is the "brain" that lets the copilot reason
//! about what the user is trying to do.
//!
//! Examples:
//! - "systemctl status nagios" → user is checking nagios service health
//! - "kubectl get pods" → user is inspecting kubernetes workloads
//! - "curl -I https://example.com" → user is checking HTTP response headers

use std::sync::{Arc, Mutex};



// Semantic analyzer for command intent understanding

/// Represents an understood command intent.
#[derive(Debug, Clone)]
pub struct CommandIntent {
    /// The original command text.
    pub command: String,
    /// What the command is doing (human-readable).
    pub action: String,
    /// Target system/resource (e.g., "nagios", "kubernetes", "mysql").
    pub target: Option<String>,
    /// Category of operation.
    pub category: IntentCategory,
    /// Confidence score (0.0-1.0).
    pub confidence: f32,
    /// Additional context about the intent.
    pub explanation: String,
}

/// Category of command intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentCategory {
    ServiceHealthCheck,
    ServiceStartStop,
    LogInspection,
    NetworkDiagnostic,
    ConfigurationChange,
    DataQuery,
    Deployment,
    Troubleshooting,
    General,
}

/// Semantic analyzer state.
#[derive(Debug, Clone)]
pub struct AnalyzerState {
    /// Last analyzed command.
    pub last_intent: Option<CommandIntent>,
    /// History of analyzed intents.
    pub intent_history: Vec<CommandIntent>,
}

/// Semantic command analyzer.
#[derive(Debug, Clone)]
pub struct SemanticAnalyzer {
    state: Arc<Mutex<AnalyzerState>>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AnalyzerState {
                last_intent: None,
                intent_history: Vec::new(),
            })),
        }
    }

    /// Analyze a terminal command and return the understood intent.
    pub fn analyze_command(&self, command: &str) -> Option<CommandIntent> {
        let cmd_lower = command.to_lowercase().trim().to_string();
        let intent = self.match_intent(&cmd_lower, command);

        if let Some(ref i) = intent {
            let mut state = self.state.lock().unwrap();
            state.last_intent = Some(i.clone());
            // Keep last 50 intents in history
            state.intent_history.push(i.clone());
            if state.intent_history.len() > 50 {
                let cutoff = state.intent_history.len() - 50;
                state.intent_history.drain(0..cutoff);
            }
        }

        intent
    }

    /// Get the most recent intent.
    pub fn get_last_intent(&self) -> Option<CommandIntent> {
        self.state.lock().unwrap().last_intent.clone()
    }

    /// Get intent history (last 20).
    pub fn get_intent_history(&self) -> Vec<CommandIntent> {
        let state = self.state.lock().unwrap();
        let len = state.intent_history.len();
        if len <= 20 {
            state.intent_history.clone()
        } else {
            state.intent_history[len - 20..].to_vec()
        }
    }

    /// Analyze terminal output for errors/warnings.
    pub fn analyze_output(&self, output: &str) -> Vec<AnalysisResult> {
        let mut results = Vec::new();
        let output_lower = output.to_lowercase();

        // Check for critical error patterns
        let error_patterns = [
            ("error", AnalysisResult::Error("Error detected in output".to_string())),
            ("fail", AnalysisResult::Error("Failure detected".to_string())),
            ("error", AnalysisResult::Error("Error detected".to_string())),
            ("failed", AnalysisResult::Error("Operation failed".to_string())),
            ("unreachable", AnalysisResult::Warning("Target is unreachable".to_string())),
            ("timeout", AnalysisResult::Warning("Connection timeout".to_string())),
            ("refused", AnalysisResult::Warning("Connection refused".to_string())),
            ("denied", AnalysisResult::Warning("Access denied".to_string())),
            ("permission denied", AnalysisResult::Error("Permission denied".to_string())),
            ("no such file", AnalysisResult::Error("File not found".to_string())),
            ("not found", AnalysisResult::Warning("Resource not found".to_string())),
            ("does not exist", AnalysisResult::Warning("Resource does not exist".to_string())),
            ("segmentation fault", AnalysisResult::Error("Segmentation fault detected".to_string())),
            ("out of memory", AnalysisResult::Error("Out of memory".to_string())),
            ("oom-killer", AnalysisResult::Error("OOM killer activated".to_string())),
        ];

        for (pattern, result) in &error_patterns {
            if output_lower.contains(pattern.to_lowercase().as_str()) {
                results.push(result.clone());
            }
        }

        // Check for service status indicators
        if output_lower.contains("active (running)") {
            results.push(AnalysisResult::Success("Service is running normally".to_string()));
        } else if output_lower.contains("active (exited)") || output_lower.contains("inactive") {
            results.push(AnalysisResult::Warning("Service is not running".to_string()));
        } else if output_lower.contains("failed") && output_lower.contains("systemctl") {
            results.push(AnalysisResult::Error("Service has failed".to_string()));
        }

        // Check for container/pod status
        if output_lower.contains("running") && (output_lower.contains("pod") || output_lower.contains("container")) {
            results.push(AnalysisResult::Success("Container/pod is running".to_string()));
        } else if output_lower.contains("pending") || output_lower.contains("imagepullbackoff") || output_lower.contains("errimagepull") {
            results.push(AnalysisResult::Warning("Container/pod has issues".to_string()));
        } else if (output_lower.contains("error") || output_lower.contains("failed"))
            && (output_lower.contains("pod") || output_lower.contains("container")) {
            results.push(AnalysisResult::Error("Container/pod error detected".to_string()));
        }

        // Check for HTTP status codes
        if output_lower.contains("404") || output_lower.contains("not found") {
            results.push(AnalysisResult::Warning("HTTP 404 - resource not found".to_string()));
        }
        if output_lower.contains("500") || output_lower.contains("502") || output_lower.contains("503") {
            results.push(AnalysisResult::Error("HTTP server error detected".to_string()));
        }
        if output_lower.contains("connection refused") {
            results.push(AnalysisResult::Warning("Connection refused - service may be down".to_string()));
        }

        results
    }

    /// Match a command to a known intent pattern.
    fn match_intent(&self, cmd: &str, original: &str) -> Option<CommandIntent> {
        // ── Service health checks ──────────────────────────────
        if cmd.starts_with("systemctl status ") {
            let service = cmd.strip_prefix("systemctl status ").unwrap_or("").trim();
            return Some(CommandIntent {
                command: original.to_string(),
                action: format!("checking {} service health", service),
                target: if service.is_empty() { None } else { Some(service.to_string()) },
                category: IntentCategory::ServiceHealthCheck,
                confidence: 0.95,
                explanation: format!("User is checking the status of the '{}' service", service),
            });
        }

        // ── Service start/stop/restart ─────────────────────────
        if cmd.starts_with("systemctl start ") {
            let service = cmd.strip_prefix("systemctl start ").unwrap_or("").trim();
            return Some(CommandIntent {
                command: original.to_string(),
                action: format!("starting {} service", service),
                target: if service.is_empty() { None } else { Some(service.to_string()) },
                category: IntentCategory::ServiceStartStop,
                confidence: 0.95,
                explanation: format!("User is trying to start the '{}' service", service),
            });
        }

        if cmd.starts_with("systemctl restart ") {
            let service = cmd.strip_prefix("systemctl restart ").unwrap_or("").trim();
            return Some(CommandIntent {
                command: original.to_string(),
                action: format!("restarting {} service", service),
                target: if service.is_empty() { None } else { Some(service.to_string()) },
                category: IntentCategory::ServiceStartStop,
                confidence: 0.95,
                explanation: format!("User is restarting the '{}' service", service),
            });
        }

        // ── Service log inspection ─────────────────────────────
        if cmd.starts_with("journalctl -xe") || cmd.starts_with("journalctl -u ") || cmd.starts_with("journalctl --since") {
            let service = if cmd.contains("-u ") {
                let parts: Vec<&str> = cmd.split("-u ").collect();
                if parts.len() > 1 { Some(parts[1].split_whitespace().next()?.to_string()) } else { None }
            } else { None };

            return Some(CommandIntent {
                command: original.to_string(),
                action: "checking service logs for errors".to_string(),
                target: service,
                category: IntentCategory::LogInspection,
                confidence: 0.9,
                explanation: "User is investigating errors by checking systemd journal logs".to_string(),
            });
        }

        // ── Tail/follow logs ───────────────────────────────────
        if cmd.starts_with("tail -f ") || cmd.starts_with("tail -n ") || cmd.starts_with("grep ") {
            return Some(CommandIntent {
                command: original.to_string(),
                action: "inspecting log files".to_string(),
                target: None,
                category: IntentCategory::LogInspection,
                confidence: 0.85,
                explanation: "User is looking through log files for information".to_string(),
            });
        }

        // ── kubectl commands ───────────────────────────────────
        if cmd.starts_with("kubectl get pods") {
            let ns = if cmd.contains("-n ") || cmd.contains("--namespace ") {
                let parts: Vec<&str> = cmd.split("-n ").collect();
                if parts.len() > 1 { Some(parts[1].split_whitespace().next()?.to_string()) } else { None }
            } else { None };

            return Some(CommandIntent {
                command: original.to_string(),
                action: "checking pod status".to_string(),
                target: ns.or(Some("kubernetes".to_string())),
                category: IntentCategory::Troubleshooting,
                confidence: 0.95,
                explanation: "User is checking the status of Kubernetes pods".to_string(),
            });
        }

        if cmd.starts_with("kubectl describe ") {
            return Some(CommandIntent {
                command: original.to_string(),
                action: "inspecting detailed resource information".to_string(),
                target: Some("kubernetes".to_string()),
                category: IntentCategory::Troubleshooting,
                confidence: 0.9,
                explanation: "User is getting detailed information about a Kubernetes resource".to_string(),
            });
        }

        if cmd.starts_with("kubectl logs") {
            return Some(CommandIntent {
                command: original.to_string(),
                action: "checking pod logs".to_string(),
                target: Some("kubernetes".to_string()),
                category: IntentCategory::LogInspection,
                confidence: 0.95,
                explanation: "User is checking logs of a Kubernetes pod".to_string(),
            });
        }

        if cmd.starts_with("kubectl exec") || cmd.starts_with("kubectl debug") {
            return Some(CommandIntent {
                command: original.to_string(),
                action: "executing command in container".to_string(),
                target: Some("kubernetes".to_string()),
                category: IntentCategory::Troubleshooting,
                confidence: 0.9,
                explanation: "User is executing a command inside a Kubernetes container".to_string(),
            });
        }

        // ── docker commands ────────────────────────────────────
        if cmd.starts_with("docker logs") {
            return Some(CommandIntent {
                command: original.to_string(),
                action: "checking container logs".to_string(),
                target: Some("docker".to_string()),
                category: IntentCategory::LogInspection,
                confidence: 0.95,
                explanation: "User is checking logs of a Docker container".to_string(),
            });
        }

        if cmd.starts_with("docker ps") {
            return Some(CommandIntent {
                command: original.to_string(),
                action: "listing running containers".to_string(),
                target: Some("docker".to_string()),
                category: IntentCategory::Troubleshooting,
                confidence: 0.95,
                explanation: "User is checking which Docker containers are running".to_string(),
            });
        }

        if cmd.starts_with("docker restart") {
            return Some(CommandIntent {
                command: original.to_string(),
                action: "restarting a Docker container".to_string(),
                target: Some("docker".to_string()),
                category: IntentCategory::ServiceStartStop,
                confidence: 0.95,
                explanation: "User is restarting a Docker container".to_string(),
            });
        }

        // ── Network diagnostics ────────────────────────────────
        if cmd.starts_with("ping ") {
            let host = cmd.strip_prefix("ping ").unwrap_or("").split_whitespace().next();
            return Some(CommandIntent {
                command: original.to_string(),
                action: format!("pinging {}", host.unwrap_or("target")),
                target: host.map(String::from),
                category: IntentCategory::NetworkDiagnostic,
                confidence: 0.95,
                explanation: "User is testing network connectivity".to_string(),
            });
        }

        if cmd.starts_with("curl ") || cmd.starts_with("wget ") {
            let target_url = cmd.split_whitespace()
                .nth(1)
                .filter(|s| s.starts_with("http") || s.starts_with("//"));
            return Some(CommandIntent {
                command: original.to_string(),
                action: "checking HTTP endpoint".to_string(),
                target: target_url.map(String::from),
                category: IntentCategory::NetworkDiagnostic,
                confidence: 0.85,
                explanation: "User is testing HTTP connectivity to an endpoint".to_string(),
            });
        }

        if cmd.starts_with("nslookup ") || cmd.starts_with("dig ") || cmd.starts_with("host ") {
            return Some(CommandIntent {
                command: original.to_string(),
                action: "checking DNS resolution".to_string(),
                target: None,
                category: IntentCategory::NetworkDiagnostic,
                confidence: 0.9,
                explanation: "User is troubleshooting DNS resolution".to_string(),
            });
        }

        if cmd.starts_with("netstat ") || cmd.starts_with("ss ") || cmd.starts_with("lsof ") {
            return Some(CommandIntent {
                command: original.to_string(),
                action: "checking network connections".to_string(),
                target: None,
                category: IntentCategory::NetworkDiagnostic,
                confidence: 0.9,
                explanation: "User is inspecting network connections and ports".to_string(),
            });
        }

        // ── Database commands ──────────────────────────────────
        if cmd.starts_with("mysql ") || cmd.starts_with("psql ") || cmd.starts_with("mongo ") {
            return Some(CommandIntent {
                command: original.to_string(),
                action: "querying a database".to_string(),
                target: Some(cmd.split_whitespace().next().unwrap_or("").to_string()),
                category: IntentCategory::DataQuery,
                confidence: 0.9,
                explanation: "User is running commands against a database".to_string(),
            });
        }

        // ── SSH commands ───────────────────────────────────────
        if cmd.starts_with("ssh ") {
            let target = cmd.strip_prefix("ssh ").unwrap_or("")
                .split('@')
                .next_back()
                .map(String::from);
            return Some(CommandIntent {
                command: original.to_string(),
                action: "connecting to remote server via SSH".to_string(),
                target,
                category: IntentCategory::General,
                confidence: 0.9,
                explanation: "User is SSH-ing into a remote server".to_string(),
            });
        }

        // ── System info ────────────────────────────────────────
        if cmd.starts_with("df ") || cmd.starts_with("free ") || cmd.starts_with("top ") || cmd.starts_with("htop ") || cmd.starts_with("ps aux") || cmd.starts_with("uptime") {
            return Some(CommandIntent {
                command: original.to_string(),
                action: "checking system resources".to_string(),
                target: None,
                category: IntentCategory::Troubleshooting,
                confidence: 0.85,
                explanation: "User is checking system resource usage (disk, memory, processes)".to_string(),
            });
        }

        // ── grep for errors ────────────────────────────────────
        if cmd.starts_with("grep -i") && (cmd.contains("error") || cmd.contains("fail") || cmd.contains("warn")) {
            return Some(CommandIntent {
                command: original.to_string(),
                action: "searching for errors in logs".to_string(),
                target: None,
                category: IntentCategory::LogInspection,
                confidence: 0.9,
                explanation: "User is searching log files for error messages".to_string(),
            });
        }

        // ── General fallback ───────────────────────────────────
        if cmd.is_empty() || cmd.starts_with("cd ") || cmd.starts_with("ls ") || cmd.starts_with("pwd") || cmd.starts_with("cat ") {
            return Some(CommandIntent {
                command: original.to_string(),
                action: "navigating filesystem".to_string(),
                target: None,
                category: IntentCategory::General,
                confidence: 0.7,
                explanation: "User is performing basic filesystem operations".to_string(),
            });
        }

        None
    }
}

/// Result of analyzing output for issues.
#[derive(Debug, Clone)]
pub enum AnalysisResult {
    /// Error detected.
    Error(String),
    /// Warning detected.
    Warning(String),
    /// Service/status is healthy.
    Success(String),
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}