/// Feature 3 — Troubleshooting Workflow Framework
///
/// Reusable troubleshooting workflows for common engineering scenarios.
/// Each workflow contains:
/// - Problem category
/// - Required observations
/// - Recommended investigation sequence
/// - Decision points
/// - Common causes
/// - Recommended commands
/// - Warnings
/// - Documentation references

use serde::{Deserialize, Serialize};

/// Common technology domains for workflows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TechnologyDomain {
    OpenShift,
    Linux,
    Windows,
    VMware,
    Nagios,
    Checkmk,
    PostgreSQL,
    MySQL,
    MSSQL,
    Kubernetes,
    Docker,
    General,
}

/// Severity of the problem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProblemSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// A single step in a troubleshooting workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step number.
    pub step_number: u32,
    /// What to investigate or do.
    pub action: String,
    /// What to look for.
    pub what_to_check: String,
    /// CLI command to run (if applicable).
    pub command: Option<String>,
    /// Expected results if healthy.
    pub expected_result: String,
    /// Whether this step requires engineer approval.
    pub requires_approval: bool,
    /// Risk level of this step.
    pub risk_level: String,
}

/// A decision point in a troubleshooting workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPoint {
    /// Question to evaluate.
    pub question: String,
    /// Condition that, if met, changes the investigation path.
    pub condition_met: Vec<String>,
    /// Alternative path if condition is NOT met.
    pub alternative_path: Vec<String>,
}

/// A common cause identified during troubleshooting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonCause {
    /// Name of the cause.
    pub name: String,
    /// Probability (0.0 - 1.0).
    pub probability: f64,
    /// What to check to confirm.
    pub how_to_verify: String,
    /// Reference command or check.
    pub verification_command: Option<String>,
}

/// A troubleshooting workflow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TroubleshootingWorkflow {
    /// Unique name/ID of the workflow.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Technology domain this applies to.
    pub technology: TechnologyDomain,
    /// How serious the problem is.
    pub severity: ProblemSeverity,
    /// Description of the problem being addressed.
    pub description: String,
    /// Initial observations that suggest this workflow.
    pub trigger_observations: Vec<String>,
    /// Ordered steps for investigation.
    pub investigation_sequence: Vec<WorkflowStep>,
    /// Points where the path may diverge.
    pub decision_points: Vec<DecisionPoint>,
    /// Common causes to check.
    pub common_causes: Vec<CommonCause>,
    /// Commands to run (summary).
    pub recommended_commands: Vec<String>,
    /// Warnings about risky actions.
    pub warnings: Vec<String>,
    /// Reference documentation.
    pub documentation_refs: Vec<String>,
}

/// The troubleshooting workflow framework.
pub struct TroubleshootingWorkflowFramework {
    workflows: Vec<TroubleshootingWorkflow>,
    current_step_index: u32,
    current_workflow_id: Option<String>,
}

impl Default for TroubleshootingWorkflowFramework {
    fn default() -> Self {
        Self::new()
    }
}

impl TroubleshootingWorkflowFramework {
    /// Create a new workflow framework.
    pub fn new() -> Self {
        Self {
            workflows: Vec::new(),
            current_step_index: 0,
            current_workflow_id: None,
        }
    }

    /// Register a workflow.
    pub fn register(&mut self, workflow: TroubleshootingWorkflow) {
        self.workflows.push(workflow);
    }

    /// Find workflows that match the given observations.
    pub fn find_matching(&self, observations: &[String]) -> Vec<&TroubleshootingWorkflow> {
        self.workflows
            .iter()
            .filter(|wf| {
                observations.iter().any(|obs| {
                    wf.trigger_observations
                        .iter()
                        .any(|trigger| obs.to_lowercase().contains(&trigger.to_lowercase()))
                })
            })
            .collect()
    }

    /// Start a workflow by ID.
    pub fn start_workflow(&mut self, workflow_id: &str) -> bool {
        if self.workflows.iter().any(|wf| wf.id == workflow_id) {
            self.current_workflow_id = Some(workflow_id.to_string());
            self.current_step_index = 0;
            true
        } else {
            false
        }
    }

    /// Get the next step in the current workflow.
    pub fn next_step(&self) -> Option<&WorkflowStep> {
        let workflow = self.current_workflow_id.as_ref().and_then(|id| {
            self.workflows.iter().find(|wf| wf.id == *id)
        })?;

        workflow.investigation_sequence.get(self.current_step_index as usize)
    }

    /// Complete the current step and advance.
    pub fn complete_step(&mut self) {
        self.current_step_index += 1;
    }

    /// Get the current workflow's next step index.
    pub fn current_step_index(&self) -> u32 {
        self.current_step_index
    }

    /// Get the total number of steps in the current workflow.
    pub fn total_steps(&self) -> u32 {
        self.current_workflow_id
            .as_ref()
            .and_then(|id| self.workflows.iter().find(|wf| wf.id == *id))
            .map(|wf| wf.investigation_sequence.len() as u32)
            .unwrap_or(0)
    }

    /// Check if the current workflow is complete.
    pub fn is_complete(&self) -> bool {
        self.current_step_index >= self.total_steps()
    }

    /// Get workflow by ID.
    pub fn get(&self, id: &str) -> Option<&TroubleshootingWorkflow> {
        self.workflows.iter().find(|wf| wf.id == id)
    }

    /// Get all workflows.
    pub fn all(&self) -> &[TroubleshootingWorkflow] {
        &self.workflows
    }

    /// Get workflows for a specific technology domain.
    pub fn by_technology(&self, domain: &TechnologyDomain) -> Vec<&TroubleshootingWorkflow> {
        self.workflows
            .iter()
            .filter(|wf| &wf.technology == domain)
            .collect()
    }

    /// Format workflow for display.
    pub fn format_display(&self, workflow: &TroubleshootingWorkflow) -> String {
        let mut output = format!(
            "Workflow: {}\nTechnology: {:?}\nSeverity: {:?}\n\n",
            workflow.name, workflow.technology, workflow.severity
        );

        output.push_str(&format!("{}\n\n", workflow.description));

        output.push_str("Investigation Steps:\n");
        for step in &workflow.investigation_sequence {
            output.push_str(&format!(
                "  {}. {} [Risk: {}]\n",
                step.step_number, step.action, step.risk_level
            ));
            output.push_str(&format!("     Check: {}\n", step.what_to_check));
            if let Some(ref cmd) = step.command {
                output.push_str(&format!("     Command: {}\n", cmd));
            }
            output.push_str(&format!("     Expected: {}\n\n", step.expected_result));
        }

        if !workflow.recommended_commands.is_empty() {
            output.push_str("Recommended Commands:\n");
            for cmd in &workflow.recommended_commands {
                output.push_str(&format!("  • {}\n", cmd));
            }
        }

        if !workflow.warnings.is_empty() {
            output.push_str("\nWarnings:\n");
            for warn in &workflow.warnings {
                output.push_str(&format!("  ⚠ {}\n", warn));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_framework_registers_workflow() {
        let mut framework = TroubleshootingWorkflowFramework::new();
        let workflow = TroubleshootingWorkflow {
            id: "openshift-pod-failure".to_string(),
            name: "OpenShift Pod Failure".to_string(),
            technology: TechnologyDomain::OpenShift,
            severity: ProblemSeverity::High,
            description: "Troubleshoot OpenShift pod failures.".to_string(),
            trigger_observations: vec!["pod restart".to_string(), "pod crash".to_string()],
            investigation_sequence: vec![
                WorkflowStep {
                    step_number: 1,
                    action: "Check pod status".to_string(),
                    what_to_check: "Pod restart count, OOMKilled events".to_string(),
                    command: Some("oc get pods --all-namespaces".to_string()),
                    expected_result: "Pods in Running state".to_string(),
                    requires_approval: false,
                    risk_level: "Low".to_string(),
                },
                WorkflowStep {
                    step_number: 2,
                    action: "Check pod events".to_string(),
                    what_to_check: "Recent events and warnings".to_string(),
                    command: Some("oc get events --sort-by=.lastTimestamp".to_string()),
                    expected_result: "No critical events".to_string(),
                    requires_approval: false,
                    risk_level: "Low".to_string(),
                },
            ],
            decision_points: vec![],
            common_causes: vec![
                CommonCause {
                    name: "OOMKilled".to_string(),
                    probability: 0.7,
                    how_to_verify: "Check pod events for OOMKilled".to_string(),
                    verification_command: Some("oc get events --field-selector reason=OOMKilled".to_string()),
                },
            ],
            recommended_commands: vec![
                "oc get pods --all-namespaces".to_string(),
                "oc get events --sort-by=.lastTimestamp".to_string(),
            ],
            warnings: vec![
                "Do not restart pods without checking logs first.".to_string(),
            ],
            documentation_refs: vec!["OpenShift Troubleshooting Guide".to_string()],
        };

        framework.register(workflow);
        assert_eq!(framework.all().len(), 1);
    }

    #[test]
    fn test_workflow_framework_find_matching() {
        let mut framework = TroubleshootingWorkflowFramework::new();

        framework.register(TroubleshootingWorkflow {
            id: "pod-failure".to_string(),
            name: "Pod Failure".to_string(),
            technology: TechnologyDomain::OpenShift,
            severity: ProblemSeverity::High,
            description: "Pod failure".to_string(),
            trigger_observations: vec!["pod restart".to_string(), "crash".to_string()],
            investigation_sequence: vec![],
            decision_points: vec![],
            common_causes: vec![],
            recommended_commands: vec![],
            warnings: vec![],
            documentation_refs: vec![],
        });

        framework.register(TroubleshootingWorkflow {
            id: "disk-space".to_string(),
            name: "Disk Space Low".to_string(),
            technology: TechnologyDomain::Linux,
            severity: ProblemSeverity::Medium,
            description: "Disk space low".to_string(),
            trigger_observations: vec!["disk usage".to_string(), "filesystem full".to_string()],
            investigation_sequence: vec![],
            decision_points: vec![],
            common_causes: vec![],
            recommended_commands: vec![],
            warnings: vec![],
            documentation_refs: vec![],
        });

        let matches = framework.find_matching(&["Pod restart detected".to_string()]);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "pod-failure");

        let disk_matches = framework.find_matching(&["Disk usage is high".to_string()]);
        assert_eq!(disk_matches.len(), 1);
        assert_eq!(disk_matches[0].id, "disk-space");
    }

    #[test]
    fn test_workflow_framework_start_and_advance() {
        let mut framework = TroubleshootingWorkflowFramework::new();

        framework.register(TroubleshootingWorkflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            technology: TechnologyDomain::General,
            severity: ProblemSeverity::Low,
            description: "Test workflow".to_string(),
            trigger_observations: vec!["test".to_string()],
            investigation_sequence: vec![
                WorkflowStep {
                    step_number: 1, action: "Step 1".to_string(), what_to_check: "Check A".to_string(),
                    command: Some("cmd1".to_string()), expected_result: "OK".to_string(),
                    requires_approval: false, risk_level: "Low".to_string(),
                },
                WorkflowStep {
                    step_number: 2, action: "Step 2".to_string(), what_to_check: "Check B".to_string(),
                    command: Some("cmd2".to_string()), expected_result: "OK".to_string(),
                    requires_approval: false, risk_level: "Low".to_string(),
                },
            ],
            decision_points: vec![],
            common_causes: vec![],
            recommended_commands: vec![],
            warnings: vec![],
            documentation_refs: vec![],
        });

        assert!(framework.start_workflow("test-workflow"));
        assert_eq!(framework.current_step_index(), 0);
        assert!(!framework.is_complete());

        let step1 = framework.next_step();
        assert!(step1.is_some());
        assert_eq!(step1.unwrap().action, "Step 1");

        framework.complete_step();
        assert_eq!(framework.current_step_index(), 1);

        let step2 = framework.next_step();
        assert!(step2.is_some());
        assert_eq!(step2.unwrap().action, "Step 2");

        framework.complete_step();
        assert!(framework.is_complete());
        assert!(framework.next_step().is_none());
    }

    #[test]
    fn test_workflow_framework_by_technology() {
        let mut framework = TroubleshootingWorkflowFramework::new();

        framework.register(TroubleshootingWorkflow {
            id: "openshift-1".to_string(),
            name: "OpenShift 1".to_string(),
            technology: TechnologyDomain::OpenShift,
            severity: ProblemSeverity::Medium,
            description: "Test".to_string(),
            trigger_observations: vec![],
            investigation_sequence: vec![],
            decision_points: vec![],
            common_causes: vec![],
            recommended_commands: vec![],
            warnings: vec![],
            documentation_refs: vec![],
        });

        framework.register(TroubleshootingWorkflow {
            id: "linux-1".to_string(),
            name: "Linux 1".to_string(),
            technology: TechnologyDomain::Linux,
            severity: ProblemSeverity::Medium,
            description: "Test".to_string(),
            trigger_observations: vec![],
            investigation_sequence: vec![],
            decision_points: vec![],
            common_causes: vec![],
            recommended_commands: vec![],
            warnings: vec![],
            documentation_refs: vec![],
        });

        let openshift = framework.by_technology(&TechnologyDomain::OpenShift);
        assert_eq!(openshift.len(), 1);
        assert_eq!(openshift[0].id, "openshift-1");
    }

    #[test]
    fn test_workflow_format_display() {
        let mut framework = TroubleshootingWorkflowFramework::new();

        framework.register(TroubleshootingWorkflow {
            id: "display-test".to_string(),
            name: "Display Test".to_string(),
            technology: TechnologyDomain::OpenShift,
            severity: ProblemSeverity::High,
            description: "Display test workflow".to_string(),
            trigger_observations: vec!["test".to_string()],
            investigation_sequence: vec![
                WorkflowStep {
                    step_number: 1, action: "Check pods".to_string(), what_to_check: "Pod status".to_string(),
                    command: Some("oc get pods".to_string()), expected_result: "Running".to_string(),
                    requires_approval: false, risk_level: "Low".to_string(),
                },
            ],
            decision_points: vec![],
            common_causes: vec![],
            recommended_commands: vec!["oc get pods".to_string()],
            warnings: vec!["Do not restart pods without logs.".to_string()],
            documentation_refs: vec!["OpenShift Guide".to_string()],
        });

        let display = framework.format_display(framework.get("display-test").unwrap());
        assert!(display.contains("Display Test"));
        assert!(display.contains("oc get pods"));
        assert!(display.contains("Do not restart"));
    }

    #[test]
    fn test_workflow_framework_invalid_workflow() {
        let mut framework = TroubleshootingWorkflowFramework::new();
        assert!(!framework.start_workflow("nonexistent"));
    }
}