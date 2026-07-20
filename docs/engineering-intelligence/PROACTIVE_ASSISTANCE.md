# Copilot Engine Proactive Assistance

## Overview

The Proactive Assistance module determines when and how to interrupt the engineer with recommendations. It balances providing timely assistance with avoiding notification fatigue.

## Signal Types

### 1. Error Detected

```rust
ProactiveSignal::ErrorDetected {
    error_type: String,
}
```

**Policy**: Always warrants interruption - errors require immediate attention.

**Confidence threshold**: N/A (always interrupts)

### 2. Idle Detection

```rust
ProactiveSignal::Idle {
    seconds_idle: u64,
}
```

**Policy**: Only interrupts if idle time exceeds configured threshold (default 60 seconds).

**Confidence threshold**: ≥0.5 to interrupt

**Rationale**: Engineer is available and not actively working.

### 3. Resource Threshold

```rust
ProactiveSignal::ResourceThreshold {
    resource_type: String,
    usage_percent: f64,
    threshold_percent: f64,
}
```

**Policy**: Interrupts when resource usage exceeds threshold.

**Confidence threshold**: ≥0.5 to interrupt

**Example**: Memory at 85% with 80% threshold.

### 4. Related Work

```rust
ProactiveSignal::RelatedWork {
    related_id: uuid::Uuid,
    confidence: f64,
}
```

**Policy**: Only interrupts if confidence exceeds configured threshold (default 0.7).

**Confidence threshold**: ≥configured threshold to interrupt

**Purpose**: Surface related recommendations that may be relevant.

### 5. High Confidence

```rust
ProactiveSignal::HighConfidence {
    confidence: f64,
    priority: Priority,
}
```

**Policy**: Always interrupts for high-confidence recommendations.

**Confidence threshold**: Always (confidence ≥0.9 assumed)

## Interrupt Policy

### should_interrupt Logic

```rust
pub fn should_interrupt(&self, signal: &ProactiveSignal, current_confidence: f64) -> bool {
    match signal {
        ProactiveSignal::ErrorDetected { .. } => true, // Always interrupt for errors
        
        ProactiveSignal::HighConfidence { .. } => true, // Always interrupt for high confidence
        
        ProactiveSignal::Idle { seconds_idle } => {
            *seconds_idle >= self.idle_threshold && current_confidence >= 0.5
        }
        
        ProactiveSignal::ResourceThreshold { usage_percent, threshold_percent, .. } => {
            *usage_percent >= *threshold_percent && current_confidence >= 0.5
        }
        
        ProactiveSignal::RelatedWork { confidence, .. } => {
            *confidence >= self.related_work_threshold
        }
    }
}
```

## Signal Classification

### ProactiveSignal Enum

```rust
#[derive(Debug, Clone)]
pub enum ProactiveSignal {
    ErrorDetected {
        error_type: String,
    },
    Idle {
        seconds_idle: u64,
    },
    ResourceThreshold {
        resource_type: String,
        usage_percent: f64,
        threshold_percent: f64,
    },
    RelatedWork {
        related_id: uuid::Uuid,
        confidence: f64,
    },
    HighConfidence {
        confidence: f64,
        priority: Priority,
    },
}
```

### Signal Classification Methods

```rust
impl ProactiveSignal {
    pub fn classification(&self) -> SignalClassification {
        match self {
            ProactiveSignal::ErrorDetected { .. } => SignalClassification::Error,
            ProactiveSignal::Idle { .. } => SignalClassification::Idle,
            ProactiveSignal::ResourceThreshold { .. } => SignalClassification::Resource,
            ProactiveSignal::RelatedWork { .. } => SignalClassification::Related,
            ProactiveSignal::HighConfidence { .. } => SignalClassification::HighConfidence,
        }
    }
    
    pub fn urgency(&self) -> ULevel {
        match self {
            ProactiveSignal::ErrorDetected { .. } => ULevel::High,
            ProactiveSignal::HighConfidence { confidence, .. } => {
                if *confidence >= 0.9 { ULevel::High } else { ULevel::Medium }
            }
            ProactiveSignal::ResourceThreshold { .. } => ULevel::Medium,
            ProactiveSignal::RelatedWork { .. } => ULevel::Low,
            ProactiveSignal::Idle { .. } => ULevel::Low,
        }
    }
}
```

## Signal Classification

### SignalClassification Enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SignalClassification {
    Error,
    Idle,
    Resource,
    Related,
    HighConfidence,
}
```

### ULevel Enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ULevel {
    High,
    Medium,
    Low,
}
```

## Signal History

### Recent Signals Tracking

```rust
pub struct ProactiveAssistance {
    recent_signals: Vec<(ProactiveSignal, DateTime<Utc>)>,
    max_recent_signals: usize,
    idle_threshold: u64,
    related_work_threshold: f64,
}
```

### record_signal

```rust
pub fn record_signal(&mut self, signal: ProactiveSignal) {
    self.recent_signals.push((signal, Utc::now()));
    if self.recent_signals.len() > self.max_recent_signals {
        self.recent_signals
            .drain(..self.recent_signals.len() - self.max_recent_signals);
    }
}
```

## Flooding Prevention

### is_flooding

Prevents notification spam by checking signal frequency:

```rust
pub fn is_flooding(&self, window_seconds: u64) -> bool {
    let now = Utc::now();
    let recent = self
        .recent_signals
        .iter()
        .filter(|(_, ts)| {
            let delta = now.signed_duration_since(*ts);
            delta.num_seconds() < window_seconds as i64
        })
        .count();
    
    recent > 10
}
```

**Threshold**: More than 10 signals in the window = flooding

**Default window**: 60 seconds (configurable)

**Rationale**: More than 10 signals per minute indicates a cascading issue that should be batched.

### Flooding Response

When flooding is detected:
1. **Suppress individual signals**: Don't interrupt for each signal
2. **Batch summary**: Provide consolidated summary when engineer is idle
3. **Track flooding**: Record flooding event for analysis

## Configuration

### ProactiveAssistance Builder

```rust
impl ProactiveAssistance {
    pub fn new() -> Self {
        Self {
            recent_signals: Vec::new(),
            max_recent_signals: 50,
            idle_threshold: 60,
            related_work_threshold: 0.7,
        }
    }
    
    pub fn with_idle_threshold(mut self, threshold: u64) -> Self {
        self.idle_threshold = threshold;
        self
    }
    
    pub fn with_related_work_threshold(mut self, threshold: f64) -> Self {
        self.related_work_threshold = threshold;
        self
    }
    
    pub fn with_max_recent_signals(mut self, max: usize) -> Self {
        self.max_recent_signals = max;
        self
    }
}
```

### Default Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `idle_threshold` | 60 seconds | Minimum idle time to interrupt |
| `related_work_threshold` | 0.7 | Confidence threshold for related work |
| `max_recent_signals` | 50 | Maximum signals to track in history |
| `flooding_window` | 60 seconds | Time window for flooding detection |
| `flooding_threshold` | 10 | Maximum signals per window |

## Usage Example

```rust
let mut proactive = ProactiveAssistance::new();

// Record an error signal
proactive.record_signal(ProactiveSignal::ErrorDetected {
    error_type: "Pod crash detected".to_string(),
});

// Check if should interrupt for idle
let idle_signal = ProactiveSignal::Idle { seconds_idle: 120 };
let should_interrupt = proactive.should_interrupt(&idle_signal, 0.6);
assert!(should_interrupt); // Idle > 60s and confidence ≥ 0.5

// Check for flooding
let is_flooding = proactive.is_flooding(60);
assert!(!is_flooding); // Only 1 signal in window
```

## Testing

Tests cover:
- Error signal always warrants interruption
- Idle signal above threshold interrupts
- Idle signal below threshold doesn't interrupt
- Idle threshold configurable
- Resource threshold interrupts when exceeded
- High confidence always interrupts
- Related work respects threshold
- Flooding detection (>10 signals per window)
- Flooding resets (signals age out)
- Signal recording and history tracking
- Maximum recent signals limit