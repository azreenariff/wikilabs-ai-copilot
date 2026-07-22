/// Feature 7 — Command Recommendation Engine
///
/// Suggests CLI commands, SQL queries, API examples, and configuration checks.
/// Every command includes purpose, expected output, risk level, and explanation.
///
/// The AI recommends commands but never executes them. The engineer performs
/// all actual work manually.
///
/// Example:
///
/// Command: df -h
/// Purpose: Check filesystem usage.
/// Expected output: Filesystem utilization percentage.
/// Risk: Low.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of command that can be recommended.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandType {
    /// Linux/Unix shell command.
    ShellCommand,
    /// Windows PowerShell command.
    PowerShellCommand,
    /// SQL query.
    SqlQuery,
    /// API request example.
    ApiExample,
    /// Configuration file check.
    ConfigurationCheck,
}

/// Risk classification for a command recommendation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandRisk {
    /// Information gathering — no side effects.
    InformationGathering,
    /// Diagnostic — reads state, may be slow.
    Diagnostic,
    /// Configuration review — reads config without changing.
    ConfigurationReview,
    /// Potentially disruptive — may affect running services.
    PotentiallyDisruptive,
    /// Dangerous — can modify or destroy data.
    Dangerous,
}

impl CommandRisk {
    /// Check if this risk level requires a warning before showing to the engineer.
    pub fn requires_warning(&self) -> bool {
        matches!(
            self,
            Self::PotentiallyDisruptive | Self::Dangerous
        )
    }

    /// Get a short label for the risk level.
    pub fn label(&self) -> &'static str {
        match self {
            Self::InformationGathering => "Low",
            Self::Diagnostic => "Low",
            Self::ConfigurationReview => "Low",
            Self::PotentiallyDisruptive => "Medium",
            Self::Dangerous => "High",
        }
    }
}

/// A command recommendation to show to the engineer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecommendation {
    /// Unique identifier for this recommendation.
    pub id: Uuid,
    /// The command to execute.
    pub command: String,
    /// The purpose of this command.
    pub purpose: String,
    /// What output is expected (or what the engineer should look for).
    pub expected_output: String,
    /// Risk level of executing this command.
    pub risk: CommandRisk,
    /// Detailed explanation of what the command does and why it is recommended.
    pub explanation: String,
    /// The technology this command applies to.
    pub technology: String,
    /// When this command was created (used for timeline tracking).
    pub timestamp: DateTime<Utc>,
}

impl CommandRecommendation {
    /// Create a new command recommendation.
    pub fn new(
        command: String,
        purpose: String,
        expected_output: String,
        risk: CommandRisk,
        explanation: String,
        technology: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            command,
            purpose,
            expected_output,
            risk,
            explanation,
            technology,
            timestamp: Utc::now(),
        }
    }

    /// Check if this recommendation requires a safety warning.
    pub fn requires_warning(&self) -> bool {
        self.risk.requires_warning()
    }

    /// Get the warning text if this command is risky.
    pub fn warning_text(&self) -> Option<String> {
        if !self.requires_warning() {
            return None;
        }
        match &self.risk {
            CommandRisk::PotentiallyDisruptive => {
                Some("WARNING: This command may affect running services. Review carefully before execution.".to_string())
            }
            CommandRisk::Dangerous => {
                Some("DANGER: This command can modify or destroy data. Verify before execution.".to_string())
            }
            _ => None,
        }
    }

    /// Format as a display string for the desktop UI.
    pub fn format_display(&self) -> String {
        let mut output = format!("{}\n", self.command);
        output.push_str(&format!("Purpose: {}\n", self.purpose));
        output.push_str(&format!("Expected: {}\n", self.expected_output));
        output.push_str(&format!("Risk: {} ({})\n", self.risk.label(), self.risk_requires_label()));
        output.push_str(&format!("Explanation: {}\n", self.explanation));
        if let Some(warning) = self.warning_text() {
            output.push_str(&format!("\n{}\n", warning));
        }
        output
    }

    /// Helper to get risk label string.
    fn risk_requires_label(&self) -> &'static str {
        match &self.risk {
            CommandRisk::InformationGathering => "information gathering",
            CommandRisk::Diagnostic => "diagnostic",
            CommandRisk::ConfigurationReview => "configuration review",
            CommandRisk::PotentiallyDisruptive => "potentially disruptive",
            CommandRisk::Dangerous => "dangerous",
        }
    }
}

/// Factory for creating common command recommendations.
pub struct CommandFactory;

impl CommandFactory {
    /// Create a command recommendation for checking OpenShift pod events.
    pub fn openshift_pod_events() -> CommandRecommendation {
        CommandRecommendation::new(
            "oc get events --sort-by=.lastTimestamp -A".to_string(),
            "Check recent cluster events sorted by timestamp.".to_string(),
            "Recent events including pod restarts, scheduling failures, or resource limits.".to_string(),
            CommandRisk::InformationGathering,
            "Events show the timeline of what the cluster has been doing. Sorting by lastTimestamp helps identify the most recent issues.".to_string(),
            "OpenShift".to_string(),
        )
    }

    /// Create a command recommendation for checking Linux disk usage.
    pub fn linux_disk_usage() -> CommandRecommendation {
        CommandRecommendation::new(
            "df -h".to_string(),
            "Check filesystem usage with human-readable sizes.".to_string(),
            "Filesystem utilization percentage for each mounted partition.".to_string(),
            CommandRisk::InformationGathering,
            "df -h shows disk usage across all mounted filesystems. Look for partitions above 80% utilization.".to_string(),
            "Linux".to_string(),
        )
    }

    /// Create a command recommendation for checking Linux process resource usage.
    pub fn linux_process_resources() -> CommandRecommendation {
        CommandRecommendation::new(
            "ps aux --sort=-%mem | head -20".to_string(),
            "Find the top 20 processes by memory usage.".to_string(),
            "Process names, PIDs, and memory/CPU usage for resource-intensive processes.".to_string(),
            CommandRisk::InformationGathering,
            "ps aux shows all running processes. Sorting by memory usage (--sort=-%mem) and limiting to 20 results shows which processes consume the most memory.".to_string(),
            "Linux".to_string(),
        )
    }

    /// Create a command recommendation for checking network connectivity.
    pub fn network_connectivity() -> CommandRecommendation {
        CommandRecommendation::new(
            "ping -c 3 8.8.8.8 && curl -s -o /dev/null -w '%{http_code}' https://example.com".to_string(),
            "Test basic network connectivity and HTTP access.".to_string(),
            "Ping response times and HTTP status codes from external hosts.".to_string(),
            CommandRisk::InformationGathering,
            "This two-step check verifies both ICMP connectivity and HTTP-level access. Useful for isolating whether issues are network-wide or application-specific.".to_string(),
            "Linux".to_string(),
        )
    }

    /// Create a command recommendation for checking system logs.
    pub fn check_system_logs() -> CommandRecommendation {
        CommandRecommendation::new(
            "journalctl -p err --since '1 hour ago' --no-pager".to_string(),
            "Check recent error-level system journal entries.".to_string(),
            "Error messages from system services in the past hour.".to_string(),
            CommandRisk::InformationGathering,
            "journalctl filters for priority 5 (error) and above. The --since flag limits to the past hour. --no-pager prevents output from being piped through less.".to_string(),
            "Linux".to_string(),
        )
    }

    /// Create a command recommendation for checking Kubernetes pod status.
    pub fn kubernetes_pod_status() -> CommandRecommendation {
        CommandRecommendation::new(
            "kubectl get pods --all-namespaces --field-selector=status.phase!=Running".to_string(),
            "Find all pods that are not in Running state.".to_string(),
            "Namespaces, pod names, and current state for pods in Error, Pending, or CrashLoopBackOff.".to_string(),
            CommandRisk::InformationGathering,
            "This query filters out healthy pods to quickly surface problematic ones. Useful for a high-level health check.".to_string(),
            "Kubernetes".to_string(),
        )
    }
}

/// The command recommendation manager.
///
/// Collects and tracks command recommendations across a session.
pub struct CommandRecommendationManager {
    recommendations: Vec<CommandRecommendation>,
}

impl Default for CommandRecommendationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRecommendationManager {
    /// Create a new empty command recommendation manager.
    pub fn new() -> Self {
        Self {
            recommendations: Vec::new(),
        }
    }

    /// Add a command recommendation.
    pub fn add(&mut self, rec: CommandRecommendation) {
        self.recommendations.push(rec);
    }

    /// Remove a recommendation by ID.
    pub fn remove(&mut self, id: &Uuid) -> Option<CommandRecommendation> {
        if let Some(pos) = self.recommendations.iter().position(|r| r.id == *id) {
            Some(self.recommendations.remove(pos))
        } else {
            None
        }
    }

    /// Get all recommendations for a technology.
    pub fn by_technology(&self, tech: &str) -> Vec<&CommandRecommendation> {
        self.recommendations
            .iter()
            .filter(|r| r.technology == tech)
            .collect()
    }

    /// Get all recommendations that require warnings.
    pub fn with_warnings(&self) -> Vec<&CommandRecommendation> {
        self.recommendations
            .iter()
            .filter(|r| r.requires_warning())
            .collect()
    }

    /// Get count of total recommendations.
    pub fn count(&self) -> usize {
        self.recommendations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_risk_warning_levels() {
        assert!(!CommandRisk::InformationGathering.requires_warning());
        assert!(!CommandRisk::Diagnostic.requires_warning());
        assert!(!CommandRisk::ConfigurationReview.requires_warning());
        assert!(CommandRisk::PotentiallyDisruptive.requires_warning());
        assert!(CommandRisk::Dangerous.requires_warning());
    }

    #[test]
    fn test_command_risk_labels() {
        assert_eq!(CommandRisk::InformationGathering.label(), "Low");
        assert_eq!(CommandRisk::Dangerous.label(), "High");
    }

    #[test]
    fn test_command_recommendation_warning() {
        let rec = CommandRecommendation::new(
            "rm -rf /tmp/*".to_string(),
            "Clear temp directory".to_string(),
            "Files removed".to_string(),
            CommandRisk::Dangerous,
            "Destructive operation".to_string(),
            "Linux".to_string(),
        );

        assert!(rec.requires_warning());
        assert!(rec.warning_text().is_some());
        assert!(rec.warning_text().unwrap().contains("DANGER"));
    }

    #[test]
    fn test_command_recommendation_no_warning() {
        let rec = CommandRecommendation::new(
            "ls -la".to_string(),
            "List files".to_string(),
            "Directory contents".to_string(),
            CommandRisk::InformationGathering,
            "Read-only operation".to_string(),
            "Linux".to_string(),
        );

        assert!(!rec.requires_warning());
        assert!(rec.warning_text().is_none());
    }

    #[test]
    fn test_command_factory_openshift() {
        let rec = CommandFactory::openshift_pod_events();
        assert_eq!(rec.command, "oc get events --sort-by=.lastTimestamp -A");
        assert_eq!(rec.technology, "OpenShift");
        assert_eq!(rec.risk, CommandRisk::InformationGathering);
    }

    #[test]
    fn test_command_factory_linux_disk() {
        let rec = CommandFactory::linux_disk_usage();
        assert_eq!(rec.command, "df -h");
        assert_eq!(rec.risk, CommandRisk::InformationGathering);
    }

    #[test]
    fn test_command_manager_add_and_count() {
        let mut manager = CommandRecommendationManager::new();
        manager.add(CommandFactory::openshift_pod_events());
        manager.add(CommandFactory::linux_disk_usage());
        assert_eq!(manager.count(), 2);
    }

    #[test]
    fn test_command_manager_by_technology() {
        let mut manager = CommandRecommendationManager::new();
        manager.add(CommandFactory::openshift_pod_events());
        manager.add(CommandFactory::linux_disk_usage());
        manager.add(CommandFactory::kubernetes_pod_status());

        let openshift = manager.by_technology("OpenShift");
        assert_eq!(openshift.len(), 1);

        let linux = manager.by_technology("Linux");
        assert_eq!(linux.len(), 1);

        let k8s = manager.by_technology("Kubernetes");
        assert_eq!(k8s.len(), 1);
    }

    #[test]
    fn test_command_manager_with_warnings() {
        let mut manager = CommandRecommendationManager::new();
        manager.add(CommandFactory::linux_disk_usage()); // No warning

        let dangerous = CommandRecommendation::new(
            "shutdown now".to_string(),
            "Shut down system".to_string(),
            "System powered off".to_string(),
            CommandRisk::Dangerous,
            "Destructive".to_string(),
            "Linux".to_string(),
        );
        manager.add(dangerous);

        assert_eq!(manager.with_warnings().len(), 1);
    }

    #[test]
    fn test_command_format_display() {
        let rec = CommandFactory::linux_disk_usage();
        let display = rec.format_display();

        assert!(display.contains("df -h"));
        assert!(display.contains("Purpose:"));
        assert!(display.contains("Risk:"));
        assert!(display.contains("Explanation:"));
    }
}