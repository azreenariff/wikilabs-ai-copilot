# Copilot Engine Session Memory

## Overview

Session Memory tracks engineer interactions with recommendations to enable personalization and confidence adjustment. It monitors acceptance rates, dismissal reasons, and corrections to improve future recommendations.

## Tracked Metrics

### Acceptance Rate

Percentage of recommendations the engineer has accepted:

```rust
pub fn acceptance_rate(&self) -> f64 {
    let total = self.acceptance_count + self.dismissal_count + self.ignored_count + self.correction_count;
    if total == 0 {
        return 1.0; // No history yet — assume all good
    }
    self.acceptance_count as f64 / total as f64
}
```

### Correction Rate

Percentage of recommendations the engineer corrected:

```rust
pub fn correction_rate(&self) -> f64 {
    let total = self.acceptance_count + self.dismissal_count + self.ignored_count + self.correction_count;
    if total == 0 {
        return 0.0;
    }
    self.correction_count as f64 / total as f64
}
```

### Interaction Counts

Returns a tuple of `(acceptance, dismissal, ignored, correction)`:

```rust
pub fn counts(&self) -> (u32, u32, u32, u32) {
    (self.acceptance_count, self.dismissal_count, self.ignored_count, self.correction_count)
}
```

## Handling Records

### Handling Enum

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Handling {
    Accepted,
    Dismissed { reason: Option<String> },
    Ignored,
    Corrected { instead: String },
}
```

### HandlingRecord Structure

```rust
pub struct HandlingRecord {
    pub recommendation_id: uuid::Uuid,
    pub handling: Handling,
    pub timestamp: DateTime<Utc>,
}
```

## Recent Corrections

### Tracking Corrections

```rust
pub fn recent_corrections(&self) -> Vec<(uuid::Uuid, String)> {
    self.handling_history
        .iter()
        .filter_map(|record| {
            if let Handling::Corrected { instead } = &record.handling {
                Some((record.recommendation_id, instead.clone()))
            } else {
                None
            }
        })
        .collect()
}
```

### Confidence Adjustment

When an engineer frequently corrects recommendations on a topic, confidence is reduced:

```rust
pub fn confidence_adjustment_for_topic(&self, topic: &str) -> f64 {
    let recent = self.recent_corrections();
    let matching = recent
        .iter()
        .filter(|(_, instead)| {
            instead.to_lowercase().contains(&topic.to_lowercase())
        })
        .count();
    
    if matching > 3 {
        0.7 // Significant penalty — engineer frequently corrects
    } else if matching > 1 {
        0.85 // Moderate penalty
    } else {
        1.0 // No penalty
    }
}
```

## Accepted Titles Tracking

### Prevention of Repetition

```rust
pub fn record_accepted(&mut self, recommendation_id: uuid::Uuid, title: String) {
    self.acceptance_count += 1;
    self.handling_history.push(HandlingRecord {
        recommendation_id,
        handling: Handling::Accepted,
        timestamp: Utc::now(),
    });
    
    // Track accepted titles to avoid repetition
    self.accepted_titles.push(title);
    self.trim_history();
}
```

### Title Limits

- **Maximum titles**: 100 titles tracked
- **Trim strategy**: Oldest titles removed when limit exceeded
- **Purpose**: Prevent suggesting similar recommendations repeatedly

## Dismissal Tracking

### Reason Collection

```rust
pub fn record_dismissed(&mut self, recommendation_id: uuid::Uuid, reason: Option<String>) {
    self.dismissal_count += 1;
    self.handling_history.push(HandlingRecord {
        recommendation_id,
        handling: Handling::Dismissed { reason },
        timestamp: Utc::now(),
    });
    self.trim_history();
}
```

### Dismissal Analysis

Dismissal reasons can be analyzed to:
- Identify patterns in engineer rejection
- Improve recommendation quality
- Adjust confidence scores

## Correction Tracking

### Correction Details

```rust
pub fn record_correction(&mut self, recommendation_id: uuid::Uuid, instead: String) {
    self.correction_count += 1;
    self.handling_history.push(HandlingRecord {
        recommendation_id,
        handling: Handling::Corrected { instead },
        timestamp: Utc::now(),
    });
    self.trim_history();
}
```

### Correction Impact

Corrections are weighted more heavily than dismissals:
- **Corrections**: Engineer actively redirecting the AI
- **Dismissals**: Engineer simply rejecting
- **Ignored**: Engineer not engaging with recommendation

## Session History

### Handling History

```rust
pub struct SessionMemory {
    acceptance_count: u32,
    dismissal_count: u32,
    ignored_count: u32,
    correction_count: u32,
    handling_history: Vec<HandlingRecord>,
    accepted_titles: Vec<String>,
}
```

### History Limits

- **Maximum records**: 200 handling records
- **Trim strategy**: Oldest records removed when limit exceeded
- **Purpose**: Keep memory bounded while preserving recent history

## API Reference

### Creating Session Memory

```rust
let mut memory = SessionMemory::new();
```

### Recording Actions

```rust
// Record acceptance
memory.record_accepted(rec_id, "Fix memory leak".to_string());

// Record dismissal
memory.record_dismissed(rec_id, Some("Not relevant".to_string()));

// Record correction
memory.record_correction(rec_id, "Should fix network config instead".to_string());

// Record ignored
memory.record_ignored(rec_id);
```

### Querying Stats

```rust
let rate = memory.acceptance_rate();
let counts = memory.counts(); // (acceptance, dismissal, ignored, correction)
let adjustment = memory.confidence_adjustment_for_topic("network");
let recent = memory.recent_corrections();
```

### Clearing Memory

```rust
memory.clear();
```

## Integration with Decision Engine

Session Memory integrates with the Decision Engine to:

1. **Adjust confidence**: Reduce confidence for frequently corrected topics
2. **Prevent repetition**: Track accepted titles to avoid similar suggestions
3. **Improve quality**: Analyze dismissal patterns to refine recommendations

```rust
// In Decision Engine:
let adjustment = session_memory.confidence_adjustment_for_topic(&topic);
let adjusted_confidence = confidence.score * adjustment;

if adjusted_confidence < MIN_CONFIDENCE {
    return DecisionOutcome {
        should_show: false,
        reasoning: vec!["Low confidence after correction history".to_string()],
        ..
    };
}
```

## Testing

Tests cover:
- Acceptance rate calculation
- Dismissal rate calculation
- Correction tracking
- Ignored tracking
- Confidence adjustment for topics
- Accepted titles tracking
- History trimming (200 record limit)
- Title trimming (100 title limit)
- Stats counts
- Clear memory
- Empty memory defaults (1.0 acceptance rate)
- Handling enum variants