# Copilot Engine Policy Engine

## Overview

The Policy Engine enforces policy levels that control which recommendations are shown to the engineer. It provides a spectrum from minimal (Critical only) to silent (no recommendations).

## Policy Levels

### Minimal
- Shows only Critical recommendations
- High confidence required (≥0.7)
- All evidence sources required

```rust
PolicyLevel::Minimal => {
    PriorityFilter::new(policy_level, vec![Priority::Critical])
}
```

### Balanced
- Shows Critical and Warning recommendations
- High confidence required (≥0.7)
- At least one evidence source

```rust
PolicyLevel::Balanced => {
    PriorityFilter::new(policy_level, vec![Priority::Critical, Priority::Warning])
}
```

### Teaching
- Shows Critical, Warning, and Suggestion recommendations
- Standard confidence threshold (≥0.5)
- At least one evidence source

```rust
PolicyLevel::Teaching => {
    PriorityFilter::new(policy_level, vec![Priority::Critical, Priority::Warning, Priority::Suggestion])
}
```

### Expert
- Shows all recommendations
- Standard confidence threshold (≥0.5)
- At least one evidence source

```rust
PolicyLevel::Expert => {
    PriorityFilter::new(policy_level, vec![Priority::Critical, Priority::Warning, Priority::Suggestion, Priority::Information])
}
```

### Silent
- No recommendations shown
- Useful for debugging or focused work

```rust
PolicyLevel::Silent => {
    PriorityFilter::new(policy_level, vec![])
}
```

## Policy Filtering

### Priority Mapping

Policy levels map to numeric priority scores:

| Policy Level   | Included Priorities              | Min Score |
|----------------|----------------------------------|-----------|
| Minimal        | Critical                         | 4         |
| Balanced       | Critical, Warning                | 3         |
| Teaching       | Critical, Warning, Suggestion    | 2         |
| Expert         | Critical, Warning, Suggestion, Information | 1 |
| Silent         | (none)                           | 0         |

### Confidence Thresholds

- **Minimal/Balanced**: High confidence required (≥0.7)
- **Teaching/Expert**: Standard confidence (≥0.5)
- **Silent**: No recommendations (no threshold check)

### Evidence Requirements

- **Critical priority**: Evidence not strictly required (bypass check)
- **Other priorities**: At least one evidence source required

## Policy Engine Configuration

```rust
pub struct PolicyConfig {
    pub level: PolicyLevel,
    pub min_confidence: f64,
    pub max_recommendations_per_minute: u32,
    pub interruption_cooldown: Duration,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            level: PolicyLevel::Balanced,
            min_confidence: 0.7,
            max_recommendations_per_minute: 2,
            interruption_cooldown: Duration::from_secs(60),
        }
    }
}
```

## Policy Engine API

### Creating Policy Engine

```rust
let engine = PolicyEngine::new(PolicyLevel::Balanced);
```

### Changing Policy Level

```rust
let mut engine = PolicyEngine::new(PolicyLevel::Minimal);
engine.with_level(PolicyLevel::Teaching);
```

### Filtering Recommendations

```rust
let filtered = engine.filter_recommendations(recommendations);
```

### Config

```rust
let config = engine.config();
```

## Policy Enforcement Flow

```
Recommendation
    ↓
Policy Level Check (Minimal/Balanced/Teaching/Expert/Silent)
    ↓
Priority Filter (Critical? Warning? etc.)
    ↓
Confidence Threshold (≥0.5 or ≥0.7 based on level)
    ↓
Evidence Requirement (at least one source)
    ↓
Frequency Limit (max N per minute)
    ↓
Interruption Cooldown (prevent rapid notifications)
    ↓
Pass/Fail Decision
```

## Policy Context

The policy context tracks:
- Current policy level
- Recommendations shown (timestamped)
- Last recommendation time (for frequency/cooldown tracking)
- Interrupted recommendations (for proactive assistance)

```rust
pub struct PolicyContext {
    policy_level: PolicyLevel,
    recommendations_shown: Vec<DateTime<Utc>>,
    last_recommendation_time: Option<DateTime<Utc>>,
    interrupted_recommendations: HashSet<uuid::Uuid>,
}
```

## Testing

Tests cover:
- Policy level filtering (all 5 levels)
- Confidence threshold enforcement
- Frequency limit enforcement
- Interruption cooldown
- Policy level changes
- Configuration defaults
- Priority score mapping
- Most urgent identification