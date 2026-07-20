# Troubleshooting Workflow Framework

**Phase 10** — Wiki Labs AI Copilot

---

## Overview

The Troubleshooting Workflow Framework creates reusable templates for how engineers solve common problems. Each workflow defines a sequence of investigation steps, decision points, and recommended actions.

## Design Principles

1. **Reusable** — Workflows are defined once and applied to any incident.
2. **Guided** — Steps progress from evidence collection → diagnosis → resolution.
3. **Adaptive** — Steps can be skipped if already completed; branches exist for different outcomes.
4. **Engineer-driven** — The AI guides; the engineer executes.

## Architecture

```
Decision Engine (detects problem type)
    ↓
┌─────────────────────────────────┐
│  Troubleshooting Workflow       │
│  Framework                      │
│                                 │
│  • WorkflowTemplate             │
│  • WorkflowStep                 │
│  • WorkflowState                │
│  • WorkflowProgress             │
│  • Built-in templates           │
└─────────────────────────────────┘
    ↓
Timeline (tracks workflow progress)
```

## Key Types

### WorkflowTemplate

A reusable workflow definition:

```rust
pub struct WorkflowTemplate {
    id: String,                    // e.g. "openshift-app-failure"
    name: String,                  // e.g. "OpenShift Application Failure"
    problem_category: String,      // e.g. "Application unavailable"
    description: String,           // Human-readable description
    steps: Vec<WorkflowStep>,      // Ordered investigation steps
    warnings: Vec<String>,         // Safety warnings
    documentation_refs: Vec<String>, // Documentation URLs/paths
    recommended_commands: Vec<RecommendedCommand>,
}
```

### WorkflowStep

Individual investigation step:

```rust
pub struct WorkflowStep {
    id: String,           // e.g. "check-pod-status"
    title: String,        // e.g. "Check Pod Status"
    description: String,  // What to look for
    commands: Vec<String>,// e.g. ["oc get pods", "oc describe pod <pod>"]
    required_observations: Vec<String>, // What evidence to collect
    expected_outcomes: Vec<String>,    // What results are expected
    decision_points: Vec<DecisionPoint>,
    risk_level: String,   // e.g. "Low", "Medium"
    requires_approval: bool, // Whether engineer approval is needed
    documentation_refs: Vec<String>,
}
```

### WorkflowState

Where the workflow currently stands:

```rust
pub struct WorkflowState {
    template_id: String,
    current_step_index: usize,
    completed_steps: Vec<String>,
    current_step_observation: Option<String>,
    started_at: DateTime<Utc>,
}
```

### WorkflowProgress

```rust
pub struct WorkflowProgress {
    state: WorkflowState,
    template: WorkflowTemplate,
}
```

## Built-in Workflows

### OpenShift Application Failure

```
1. [check-pod-status] → Check pod status and restart counts
   ↓
2. [check-events] → Check pod events for warnings/errors
   ↓
3. [check-logs] → Check application logs
   ↓
4. [check-resources] → Check resource limits (CPU, memory)
   ↓
5. [check-network] → Check network connectivity
   ↓
6. [check-deployment] → Check deployment history
   ↓
7. [check-configuration] → Review configuration
```

### Linux Performance Investigation

```
1. [check-cpu] → Check CPU utilization
   ↓
2. [check-memory] → Check memory usage
   ↓
3. [check-disk] → Check disk usage and I/O
   ↓
4. [check-network] → Check network throughput
   ↓
5. [check-logs] → Check system logs for errors
```

### VMware VM Performance Issue

```
1. [check-vm-status] → Check VM status and power state
   ↓
2. [check-resources] → Check CPU, memory, disk allocation
   ↓
3. [check-host] → Check host health
   ↓
4. [check-datastore] → Check datastore status
   ↓
5. [check-logs] → Check VM and host logs
```

### Database Connection Failure

```
1. [check-database-status] → Check database service status
   ↓
2. [check-network] → Check network connectivity to database
   ↓
3. [check-credentials] → Check connection credentials
   ↓
4. [check-configuration] → Review database configuration
   ↓
5. [check-logs] → Check database and application logs
```

### Nagios Alert Investigation

```
1. [check-alert-status] → Check Nagios alert status
   ↓
2. [check-host-status] → Check affected host status
   ↓
3. [check-service-status] → Check affected service status
   ↓
4. [check-history] → Check alert history and trends
   ↓
5. [check-configuration] → Check Nagios configuration
```

## Usage

```rust
// Get a workflow template
let template = WorkflowRegistry::get("openshift-app-failure");

// Start a new workflow instance
let progress = WorkflowProgress::new(template.clone());

// Mark the first step as complete
progress.mark_step_complete("check-pod-status", "Pod in CrashLoopBackOff");

// Get the next step
let next_step = progress.next_step();
// → Some(WorkflowStep { title: "Check Pod Events", ... })

// Check progress
let pct = progress.completion_percentage(); // → 16.67%
```

## Validation Checklist

- ✅ Workflows are reusable templates (not incident-specific)
- ✅ Each step has commands, observations, and expected outcomes
- ✅ Decision points guide branching logic
- ✅ Risk levels and warnings included
- ✅ Documentation references on each step
- ✅ Progress tracking with completion percentage
- ✅ 20 unit tests covering workflow management