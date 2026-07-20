# Evidence Collection Framework

**Phase 10** — Wiki Labs AI Copilot

---

## Overview

The Evidence Collection Framework tracks troubleshooting evidence in a session. It records what has been found, what is still missing, and provides a basis for recommending the next investigation step.

## Design Principles

1. **Structured evidence** — Every piece of evidence has a source, finding, and importance level.
2. **Gap tracking** — The framework knows what's missing and recommends filling gaps.
3. **Confidence-based** — Evidence contributes to overall investigation confidence.
4. **Chainable** — Evidence can be combined and chained to build a narrative.

## Architecture

```
Observation Framework (collects raw data)
    ↓
┌─────────────────────────────────┐
│  Evidence Collection            │
│  Framework                      │
│                                 │
│  • Evidence                     │
│  • EvidenceCollection           │
│  • EvidenceEvaluation           │
│  • EvidenceChain                │
└─────────────────────────────────┘
    ↓
Decision Engine (uses evidence for recommendations)
```

## Key Types

### Evidence

A single piece of evidence:

```rust
pub struct Evidence {
    pub id: Uuid,                  // Unique identifier
    pub source: String,            // e.g. "Pod Logs", "Node Status"
    pub finding: String,           // e.g. "OOMKilled detected"
    pub importance: EvidenceImportance,
    pub confidence: f64,           // 0.0 - 1.0
    pub timestamp: DateTime<Utc>,
}
```

### EvidenceImportance

How critical the evidence is to the investigation:

```rust
pub enum EvidenceImportance {
    Required,   // Must have before recommending
    Important,  // Should have, but not blocking
    Optional,   // Nice to have
}
```

### EvidenceCollection

The collection of all evidence in a session:

```rust
pub struct EvidenceCollection {
    evidence: Vec<Evidence>,
    missing: Vec<MissingEvidence>,
}

pub struct MissingEvidence {
    pub needed: String,      // e.g. "Network connectivity test"
    pub description: String, // e.g. "May indicate DNS issue"
    pub importance: EvidenceImportance,
}
```

### EvidenceEvaluation

Evaluates the state of evidence collection:

```rust
pub struct EvidenceEvaluation {
    pub collected_count: usize,    // How many pieces collected
    pub missing_count: usize,      // How many still needed
    pub confidence: f64,           // Overall evidence confidence
    pub is_sufficient: bool,       // Enough to make a recommendation?
}
```

## Usage

### Collecting Evidence

```rust
let mut collection = EvidenceCollection::new();

// Add evidence from various sources
collection.add(
    Evidence {
        source: "Pod Logs".to_string(),
        finding: "OOMKilled detected".to_string(),
        importance: EvidenceImportance::Required,
        confidence: 0.95,
        timestamp: Utc::now(),
    },
);

collection.add(
    Evidence {
        source: "Node Status".to_string(),
        finding: "Node Ready, 2 pods evicted".to_string(),
        importance: EvidenceImportance::Important,
        confidence: 0.88,
        timestamp: Utc::now(),
    },
);

// Add missing evidence (what's still needed)
collection.mark_missing(
    "Network connectivity test",
    "May indicate DNS or network issue",
    EvidenceImportance::Important,
);

// Evaluate
let eval = collection.evaluate();
// → { collected: 2, missing: 1, confidence: 0.915, sufficient: false }
```

### Checking Evidence State

```rust
if let Some(gap) = collection.missing()[0] {
    println!("Missing evidence: {}", gap.needed);
    println!("Importance: {}", gap.importance);
    println!("Reason: {}", gap.description);
}

// Get evidence by source
let pod_evidence = collection.get_by_source("Pod Logs");
```

## Integration with Recommendation Framework

Evidence feeds into recommendations:

```rust
let rec = RecommendationBuilder::new(
    "Investigate OOMKilled Events",
    "OpenShift",
    RecommendationCategory::Troubleshooting,
)
.evidence(
    collection
        .evidence()
        .iter()
        .map(|e| RecommendationEvidence {
            source: e.source.clone(),
            description: e.finding.clone(),
            confidence: e.confidence,
        })
        .collect(),
)
.recommended_next_step(Some(
    "Review resource limits: oc describe pod <pod-name>"
    .to_string()
))
.build();
```

## Validation Checklist

- ✅ Evidence has source, finding, importance, confidence
- ✅ Missing evidence is tracked with description
- ✅ Evidence evaluation includes sufficiency check
- ✅ Confidence is averaged across collected evidence
- ✅ Required evidence blocks sufficiency
- ✅ Evidence is chainable for building narratives
- ✅ Timeline integration for evidence events