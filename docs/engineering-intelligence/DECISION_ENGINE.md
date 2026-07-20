# Copilot Engine Decision Engine

## Overview

The Decision Engine evaluates whether a recommendation should be shown to the engineer based on multiple filtering criteria. It ensures recommendations are timely, relevant, and not overwhelming.

## Decision Filters

### 1. Confidence Threshold

- **Critical priority**: Always shown regardless of confidence
- **Other priorities**: Minimum 0.5 confidence required
- **Low confidence (<0.3)**: Filtered unless Critical

```rust
if priority.is_urgent() {
    // Critical always passes
    return true;
}

if confidence.score < 0.5 {
    return false;
}
```

### 2. Evidence Requirement

- **Critical priority**: Shown even with no evidence
- **Other priorities**: At least one evidence source required

```rust
if !priority.is_urgent() && evidence_sources.is_empty() {
    return false;
}
```

### 3. User State

- **Paused**: All recommendations filtered (user not available)
- **Typing**: Suggestions blocked to avoid disruption

```rust
if is_user_paused {
    return false;
}

if is_user_typing {
    return false;
}
```

### 4. Session Limits

- **Maximum**: 5 recommendations per session
- **Tracking**: `DecisionEngine` tracks recommendations shown per session

```rust
if session_shown >= config.max_session_recommendations {
    return false;
}
```

### 5. Frequency Limits

- **Maximum**: 2 recommendations per minute
- **Tracking**: Timestamp-based sliding window

```rust
let now = Utc::now();
let recent_shown = recommendations_shown
    .iter()
    .filter(|ts| now.signed_duration_since(*ts).num_seconds() < 60)
    .count();

if recent_shown >= config.max_recommendations_per_minute {
    return false;
}
```

### 6. Repetition Avoidance

- **Tracking**: Set of recommendation IDs already shown
- **Prevention**: Same recommendation not displayed twice

```rust
if shown_ids.contains(&recommendation_id) {
    return false;
}
```

### 7. Workflow Relevance

- **Context**: Current workflow state if applicable
- **Filtering**: Recommendations not relevant to current workflow filtered out

```rust
if let Some(workflow_state) = &current_workflow_state {
    if !workflow_state.is_empty() {
        // Check relevance to current workflow
        if !is_relevant_to_workflow(&recommendation, workflow_state) {
            return false;
        }
    }
}
```

## Decision Outcome

The Decision Engine returns a `DecisionOutcome` containing:

```rust
pub struct DecisionOutcome {
    pub should_show: bool,
    pub adjusted_priority: Priority,
    pub confidence: Confidence,
    pub reasoning: Vec<String>,
}
```

**Fields:**
- `should_show`: Whether recommendation should be displayed
- `adjusted_priority`: Priority after adjustments (e.g., confidence boost for low-confidence Critical)
- `confidence`: Original confidence score
- `reasoning`: List of reasons for the decision (for debugging/explainability)

## Decision Rules Priority

Rules are evaluated in order:

1. **Critical override**: Critical priority bypasses confidence and evidence checks
2. **Low confidence filter**: Recommendations with confidence < 0.3 filtered
3. **Evidence requirement**: Non-Critical recommendations need evidence
4. **Paused state**: All recommendations filtered when user paused
5. **Typing state**: Suggestions blocked when user typing
6. **Session limit**: Max 5 recommendations per session
7. **Frequency limit**: Max 2 recommendations per minute
8. **Repetition check**: Prevent duplicate display
9. **Workflow relevance**: Filter mismatched recommendations

## Configuration

```rust
pub struct DecisionConfig {
    pub max_session_recommendations: u32,
    pub max_recommendations_per_minute: u32,
    pub min_confidence: f64,
}

impl Default for DecisionConfig {
    fn default() -> Self {
        Self {
            max_session_recommendations: 5,
            max_recommendations_per_minute: 2,
            min_confidence: 0.5,
        }
    }
}
```

## Usage

```rust
let mut engine = DecisionEngine::new();

// Evaluate recommendation
let outcome = engine.evaluate(
    recommendation_id,
    confidence,
    priority,
    is_user_typing,
    &context,
);

if outcome.should_show {
    engine.record_shown(recommendation_id);
    // Display recommendation
}
```

## Testing

Tests cover:
- Confidence threshold filtering
- Evidence requirement validation
- Paused state handling
- Typing state blocking
- Session limit enforcement
- Frequency limit enforcement
- Repetition avoidance
- Workflow relevance filtering
- Critical priority overrides
- Priority adjustment logic