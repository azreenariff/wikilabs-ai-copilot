# Engineering Recommendation Framework

**Phase 10** — Wiki Labs AI Copilot

---

## Overview

The Engineering Recommendation Framework creates structured, metadata-rich recommendations. Every recommendation contains the title, technology, category, description, reason, confidence, evidence, recommended next step, reference documentation, and risk warning (if applicable).

## Design Principles

1. **Structured** — Every recommendation follows a defined schema.
2. **Transparent** — Engineers see the reasoning, evidence, and confidence behind each suggestion.
3. **Traceable** — Each recommendation has a unique ID for timeline tracking.
4. **Risk-aware** — Recommendations include risk warnings when applicable.

## Architecture

```
Decision Engine (decides what to recommend)
    ↓
┌─────────────────────────────────┐
│  Engineering Recommendation     │
│  Framework                      │
│                                 │
│  • EngineeringRecommendation    │
│  • RecommendationBuilder        │
│  • RecommendationCategory       │
│  • RecommendationEvidence       │
│  • ReferenceDoc                 │
│  • RiskWarning                  │
│  • RiskLevel                    │
└─────────────────────────────────┘
    ↓
Desktop UI (displays recommendation cards)
```

## Key Types

### EngineeringRecommendation

The complete recommendation structure:

```rust
pub struct EngineeringRecommendation {
    id: Uuid,                          // Unique identifier
    title: String,                     // Concise title
    technology: String,                // e.g. "OpenShift"
    category: RecommendationCategory,  // Type of recommendation
    description: String,               // Detailed description
    reason: String,                    // Why this recommendation was made
    confidence: f64,                   // 0.0 - 1.0 overall confidence
    evidence: Vec<RecommendationEvidence>,
    recommended_next_step: Option<String>,
    reference_docs: Vec<ReferenceDoc>,
    risk_warning: Option<RiskWarning>,
    generated_at: DateTime<Utc>,
}
```

### RecommendationCategory

Six categories classify recommendations:

| Category | Description | Example |
|----------|-------------|---------|
| `Troubleshooting` | Diagnostic guidance | Check pod events |
| `ConfigurationReview` | Config review | Review resource limits |
| `PerformanceOptimization` | Performance tuning | Check CPU throttling |
| `Security` | Security assessment | Verify RBAC settings |
| `Infrastructure` | Deployment/infra change | Update deployment replicas |
| `Informational` | Educational content | Explain pod lifecycle |
| `CommandGuidance` | CLI command suggestions | Run `oc logs` |

### RecommendationEvidence

Evidence supporting a recommendation:

```rust
pub struct RecommendationEvidence {
    source: String,      // e.g. "Pod Logs"
    description: String, // e.g. "OOMKilled detected"
    confidence: f64,     // 0.0 - 1.0 confidence of this evidence
}
```

### ReferenceDoc

Documentation references:

```rust
pub struct ReferenceDoc {
    title: String,       // e.g. "OpenShift Troubleshooting Guide"
    url: Option<String>, // e.g. Some("https://docs.openshift.com/...")
    relevance: String,   // e.g. "Pod restart investigation steps"
}
```

### RiskWarning

Risk assessment for recommendations:

```rust
pub struct RiskWarning {
    level: RiskLevel,    // None, Low, Medium, High, Critical
    description: String, // e.g. "This command may restart services"
    mitigation: String,  // e.g. "Run during maintenance window"
}
```

## Usage

### Building a Recommendation

```rust
let rec = RecommendationBuilder::new(
    "Check OpenShift Pod Events",
    "OpenShift",
    RecommendationCategory::Troubleshooting,
)
.description("Pod restart count increasing")
.reason("Current troubleshooting workflow indicates evidence collection is incomplete")
.confidence(0.92)
.evidence(vec![
    RecommendationEvidence {
        source: "Pod Status".to_string(),
        description: "Pod restart count increasing".to_string(),
        confidence: 0.95,
    },
    RecommendationEvidence {
        source: "Deployment History".to_string(),
        description: "Deployment recently changed".to_string(),
        confidence: 0.85,
    },
])
.recommended_next_step(Some("oc get events --sort-by='.lastTimestamp'".to_string()))
.reference_docs(vec![
    ReferenceDoc {
        title: "OpenShift Troubleshooting Guide".to_string(),
        url: Some("https://docs.openshift.com/container-platform/latest/troubleshooting/...".to_string()),
        relevance: "Pod restart investigation".to_string(),
    },
])
.risk_warning(Some(RiskWarning {
    level: RiskLevel::Low,
    description: "Read-only command, no side effects".to_string(),
    mitigation: "N/A — information gathering only".to_string(),
}))
.build();
```

### Displaying a Recommendation

```rust
println!("=== {} ===", rec.title);
println!("Technology: {}", rec.technology);
println!("Confidence: {:.0}%", rec.confidence * 100.0);
println!("Reason: {}", rec.reason);
if let Some(next) = &rec.recommended_next_step {
    println!("Command: {}", next);
}
if let Some(risk) = &rec.risk_warning {
    println!("WARNING: {} — {}", risk.level, risk.description);
}
```

## Validation Checklist

- ✅ Recommendations contain all required fields
- ✅ Every recommendation has a unique ID
- ✅ Confidence scores are 0.0–1.0
- ✅ Evidence is structured and source-tracked
- ✅ Risk warnings present for risky recommendations
- ✅ Reference documentation with URLs included
- ✅ Timestamps recorded for each recommendation
- ✅ Builder pattern prevents incomplete recommendations