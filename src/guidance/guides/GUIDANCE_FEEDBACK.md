# Guidance Feedback

**Phase 10** — Wiki Labs AI Copilot

---

## Overview

The Guidance Feedback System allows engineers to provide feedback on AI recommendations. Feedback influences current-session behavior only — it does not trigger autonomous learning.

## Design Principles

1. **Session-only** — Feedback affects only the current session.
2. **Adaptive** — The AI adjusts recommendations based on feedback patterns.
3. **No autonomous learning** — Feedback does not persist across sessions.
4. **Suppression** — Repeatedly suppressed recommendations are hidden.

## Architecture

```
Desktop UI (displays feedback buttons)
    ↓
┌─────────────────────────────────┐
│  Feedback System                │
│                                 │
│  • EngineerFeedback             │
│  • FeedbackType                 │
│  • FeedbackStats                │
│  • FeedbackSystem               │
└─────────────────────────────────┘
    ↓
Decision Engine (adjusts based on stats)
```

## Feedback Types

| Type | Description | Effect |
|------|-------------|--------|
| `Useful` | The recommendation helped | Positive signal |
| `NotUseful` | The recommendation did not help | Negative → suppress |
| `AlreadyCompleted` | The step was already done | Partially helpful → suppress |
| `Incorrect` | The recommendation was wrong | Negative → suppress |
| `DifferentApproach` | Engineer used different method | Partially helpful |

## Key Types

### EngineerFeedback

```rust
pub struct EngineerFeedback {
    pub id: Uuid,                    // Unique feedback ID
    pub recommendation_id: Uuid,     // Which recommendation
    pub feedback_type: FeedbackType, // Type of feedback
    pub notes: Option<String>,       // Optional notes
    pub timestamp: DateTime<Utc>,    // When given
}
```

### FeedbackStats

```rust
pub struct FeedbackStats {
    pub total: usize,
    pub useful_count: usize,
    pub not_useful_count: usize,
    pub redundant_count: usize,
    pub incorrect_count: usize,
    pub different_approach_count: usize,
    pub average_helpfulness: f64,    // 0.0 - 1.0
}
```

## Usage

### Recording Feedback

```rust
let mut feedback_system = FeedbackSystem::new();

// Record feedback on a recommendation
feedback_system.record_for(
    rec_id,
    FeedbackType::Useful,
    Some("Found the issue quickly"),
);

// Or create a feedback object directly
let fb = EngineerFeedback::new(
    rec_id,
    FeedbackType::Incorrect,
    Some("Wrong command for Kubernetes cluster"),
);
feedback_system.record(fb);
```

### Checking Feedback State

```rust
let stats = feedback_system.stats();
println!("Helpfulness: {:.0}%", stats.average_helpfulness * 100.0);

// Check if the system needs adjustment
if stats.needs_adjustment() {
    println!("Too many incorrect recommendations — reducing confidence");
}

// Check if a recommendation should be suppressed
if feedback_system.is_suppressed(&rec_id) {
    println!("Suppressing this recommendation type");
}
```

## Feedback → Behavior Mapping

```
Useful count / total > 60%  → Increase confidence threshold
Incorrect count / total > 30% → Reduce confidence, request more info
AlreadyCompleted > 50%      → Reduce proactive recommendations
Average helpfulness < 0.4   → Switch to Silent mode suggestion
```

## Validation Checklist

- ✅ Five feedback types implemented
- ✅ Feedback suppresses repeated recommendations
- ✅ Stats track session-wide patterns
- ✅ Average helpfulness calculated correctly
- ✅ Needs adjustment triggers on >30% incorrect rate
- ✅ Clear() resets session state
- ✅ No cross-session persistence
- ✅ 12 unit tests covering all feedback types and statistics