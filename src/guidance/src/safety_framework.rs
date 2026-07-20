/// Feature 8 — Command Safety Framework
///
/// Classifies recommended commands by risk level and generates appropriate warnings.
///
/// Categories:
/// - Information gathering (read-only, no side effects)
/// - Diagnostic (reads state, may be slow)
/// - Configuration review (reads config without changing)
/// - Potentially disruptive (may affect running services)
/// - Dangerous (can modify or destroy data)
///
/// The AI must warn before showing risky commands.

use serde::{Deserialize, Serialize};

/// Risk level for a command or operation.
///
/// The AI classifies commands before recommending them.
/// Higher risk levels trigger mandatory warnings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CommandRiskLevel {
    /// Information gathering — read-only, no side effects.
    /// Examples: df -h, ps aux, cat /etc/hostname
    InformationGathering,
    /// Diagnostic — reads system state, may be slow or resource-intensive.
    /// Examples: strace, tcpdump, kubectl logs
    Diagnostic,
    /// Configuration review — reads config files without modifying.
    /// Examples: cat, grep, diff
    ConfigurationReview,
    /// Potentially disruptive — may restart services, drain nodes, or change behavior.
    /// Examples: oc rollout restart, kubectl cordon, systemctl restart
    PotentiallyDisruptive,
    /// Dangerous — can modify or destroy data, delete resources, or crash systems.
    /// Examples: rm -rf, kubectl delete, dd, wipefs
    Dangerous,
}

impl CommandRiskLevel {
    /// Check if this risk level requires a warning before showing to the engineer.
    pub fn requires_warning(&self) -> bool {
        matches!(self, Self::PotentiallyDisruptive | Self::Dangerous)
    }

    /// Check if this is a dangerous command that should be double-confirmed.
    /// Note: All risk levels use warnings. No level requires explicit confirmation.
    pub fn requires_confirmation(&self) -> bool {
        false
    }

    /// Get a human-readable label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::InformationGathering => "Low",
            Self::Diagnostic => "Low",
            Self::ConfigurationReview => "Low",
            Self::PotentiallyDisruptive => "Medium",
            Self::Dangerous => "High",
        }
    }

    /// Get the color indicator for UI display.
    pub fn color(&self) -> &'static str {
        match self {
            Self::InformationGathering => "🟢",
            Self::Diagnostic => "🟡",
            Self::ConfigurationReview => "🟡",
            Self::PotentiallyDisruptive => "🟠",
            Self::Dangerous => "🔴",
        }
    }
}

/// A safety assessment for a command recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyAssessment {
    /// The risk level of the command.
    pub risk_level: CommandRiskLevel,
    /// A brief warning message for the engineer.
    pub warning: Option<String>,
    /// A suggested mitigation or precaution.
    pub mitigation: Option<String>,
    /// Whether the command is safe for automated recommendation.
    pub is_safe_to_recommend: bool,
}

impl SafetyAssessment {
    /// Create a safe assessment for low-risk commands.
    pub fn safe(risk: CommandRiskLevel) -> Self {
        Self {
            risk_level: risk,
            warning: None,
            mitigation: None,
            is_safe_to_recommend: true,
        }
    }

    /// Create a warning assessment for risky commands.
    pub fn warning(risk: CommandRiskLevel, message: String) -> Self {
        let mitigation = Self::suggested_mitigation(&risk);
        // Potentially disruptive commands are safe to recommend (with warning).
        // Dangerous commands require explicit confirmation before recommending.
        let is_safe_to_recommend = matches!(risk, CommandRiskLevel::PotentiallyDisruptive);
        Self {
            risk_level: risk,
            warning: Some(message),
            mitigation,
            is_safe_to_recommend,
        }
    }

    /// Get a suggested mitigation for the risk level.
    fn suggested_mitigation(risk: &CommandRiskLevel) -> Option<String> {
        match risk {
            CommandRiskLevel::PotentiallyDisruptive => {
                Some("Ensure you have backup access. Consider running in a staging environment first.".to_string())
            }
            CommandRiskLevel::Dangerous => {
                Some("Verify the command target. Have a rollback plan ready. Consider dry-run if available.".to_string())
            }
            _ => None,
        }
    }
}

/// The safety framework evaluator.
///
/// Assesses commands and generates safety assessments.
pub struct SafetyFramework;

impl SafetyFramework {
    /// Assess a command by analyzing its keywords and patterns.
    pub fn assess(command: &str) -> SafetyAssessment {
        let cmd_lower = command.to_lowercase();

        // Dangerous patterns
        let dangerous_patterns = [
            "rm -rf", "rm -r /", "dd if=", "wipe", "mkfs", "fdisk",
            "shutdown", "reboot", "poweroff", "kill -9", ":(){ :|:& };:",
        ];
        for pattern in dangerous_patterns {
            if cmd_lower.contains(pattern) {
                return SafetyAssessment::warning(
                    CommandRiskLevel::Dangerous,
                    format!(
                        "DANGER: This command is destructive. '{}' is a high-risk operation that can destroy data or crash systems.",
                        command
                    ),
                );
            }
        }

        // Potentially disruptive patterns
        let disruptive_patterns = [
            "restart", "reload", "rollback", "cordon", "drain", "scale 0",
            "oc rollout", "kubectl rollout", "systemctl", "podman restart",
        ];
        for pattern in disruptive_patterns {
            if cmd_lower.contains(pattern) {
                return SafetyAssessment::warning(
                    CommandRiskLevel::PotentiallyDisruptive,
                    format!(
                        "WARNING: This command may affect running services. '{}' could restart or disrupt services.",
                        command
                    ),
                );
            }
        }

        // Configuration review patterns
        let review_patterns = ["cat", "grep", "diff", "head -", "tail -", "less", "more "];
        for pattern in review_patterns {
            if cmd_lower.contains(pattern) {
                return SafetyAssessment::safe(CommandRiskLevel::ConfigurationReview);
            }
        }

        // Diagnostic patterns
        let diagnostic_patterns = [
            "strace", "tcpdump", "kubectl logs", "oc logs", "journalctl",
            "top", "htop", "vmstat", "iostat", "sar", "perf",
            "kubectl describe", "oc describe",
        ];
        for pattern in diagnostic_patterns {
            if cmd_lower.contains(pattern) {
                return SafetyAssessment::safe(CommandRiskLevel::Diagnostic);
            }
        }

        // Default: information gathering
        SafetyAssessment::safe(CommandRiskLevel::InformationGathering)
    }

    /// Assess multiple commands and return assessments in order.
    pub fn assess_batch(commands: &[&str]) -> Vec<(String, SafetyAssessment)> {
        commands
            .iter()
            .map(|cmd| (cmd.to_string(), Self::assess(cmd)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_labels() {
        assert_eq!(CommandRiskLevel::InformationGathering.label(), "Low");
        assert_eq!(CommandRiskLevel::Dangerous.label(), "High");
        assert_eq!(CommandRiskLevel::PotentiallyDisruptive.label(), "Medium");
    }

    #[test]
    fn test_risk_level_colors() {
        assert_eq!(CommandRiskLevel::InformationGathering.color(), "🟢");
        assert_eq!(CommandRiskLevel::Dangerous.color(), "🔴");
    }

    #[test]
    fn test_requires_warning() {
        assert!(!CommandRiskLevel::InformationGathering.requires_warning());
        assert!(!CommandRiskLevel::Diagnostic.requires_warning());
        assert!(!CommandRiskLevel::ConfigurationReview.requires_warning());
        assert!(CommandRiskLevel::PotentiallyDisruptive.requires_warning());
        assert!(CommandRiskLevel::Dangerous.requires_warning());
    }

    #[test]
    fn test_requires_confirmation() {
        assert!(!CommandRiskLevel::Dangerous.requires_confirmation());
        assert!(!CommandRiskLevel::PotentiallyDisruptive.requires_confirmation());
    }

    #[test]
    fn test_assess_safe_command() {
        let assessment = SafetyFramework::assess("df -h");
        assert_eq!(assessment.risk_level, CommandRiskLevel::InformationGathering);
        assert!(assessment.is_safe_to_recommend);
        assert!(assessment.warning.is_none());
    }

    #[test]
    fn test_assess_dangerous_command() {
        let assessment = SafetyFramework::assess("rm -rf /tmp/*");
        assert_eq!(assessment.risk_level, CommandRiskLevel::Dangerous);
        assert!(!assessment.is_safe_to_recommend);
        assert!(assessment.warning.is_some());
        assert!(assessment.warning.as_ref().unwrap().contains("DANGER"));
    }

    #[test]
    fn test_assess_disruptive_command() {
        let assessment = SafetyFramework::assess("oc rollout restart deployment/myapp");
        assert_eq!(
            assessment.risk_level,
            CommandRiskLevel::PotentiallyDisruptive
        );
        assert!(assessment.is_safe_to_recommend);
        assert!(assessment.warning.is_some());
        assert!(assessment.warning.as_ref().unwrap().contains("WARNING"));
    }

    #[test]
    fn test_assess_configuration_review() {
        let assessment = SafetyFramework::assess("cat /etc/kubernetes/manifests");
        assert_eq!(
            assessment.risk_level,
            CommandRiskLevel::ConfigurationReview
        );
        assert!(assessment.is_safe_to_recommend);
    }

    #[test]
    fn test_assess_diagnostic_command() {
        let assessment = SafetyFramework::assess("kubectl logs mypod");
        assert_eq!(assessment.risk_level, CommandRiskLevel::Diagnostic);
        assert!(assessment.is_safe_to_recommend);
    }

    #[test]
    fn test_assess_batch() {
        let commands = vec!["df -h", "rm -rf /tmp", "kubectl logs app"];
        let assessments = SafetyFramework::assess_batch(&commands);
        assert_eq!(assessments.len(), 3);
        assert_eq!(
            assessments[0].1.risk_level,
            CommandRiskLevel::InformationGathering
        );
        assert_eq!(assessments[1].1.risk_level, CommandRiskLevel::Dangerous);
        assert_eq!(assessments[2].1.risk_level, CommandRiskLevel::Diagnostic);
    }

    #[test]
    fn test_safety_assessment_mitigation() {
        let dangerous = SafetyAssessment::warning(
            CommandRiskLevel::Dangerous,
            "This is dangerous".to_string(),
        );
        assert!(dangerous.mitigation.is_some());
        assert!(dangerous.mitigation.unwrap().contains("rollback"));

        let safe = SafetyAssessment::safe(CommandRiskLevel::InformationGathering);
        assert!(safe.mitigation.is_none());
    }
}