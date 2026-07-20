# Copilot Engine Lifecycle Manager

## Overview

The Lifecycle Manager tracks recommendation states and ensures recommendations flow through a proper lifecycle. It prevents duplicate displays and maintains state history for debugging.

## Lifecycle States

### State Definitions

| State | Description | Next Possible States |
|-------|-------------|---------------------|
| **Candidate** | Recommendation generated but not yet reviewed | Ready, Dismissed |
| **Ready** | Recommendation ready for display | Displayed, Dismissed |
| **Displayed** | Recommendation shown to engineer | Accepted, Dismissed |
| **Accepted** | Engineer accepted recommendation | Completed, Dismissed |
| **Completed** | Recommendation fully processed | (terminal) |
| **Dismissed** | Recommendation dismissed by engineer | (terminal) |
| **Archived** | Recommendation archived for reference | (terminal) |

### Terminal States

- **Completed**: Recommendation fully processed and closed
- **Dismissed**: Recommendation rejected by engineer
- **Archived**: Recommendation stored for future reference

## State Transitions

### Valid Transitions

```
Candidate → Ready     (Mark as ready for display)
Candidate → Dismissed (Dismiss before showing)

Ready → Displayed    (Show to engineer)
Ready → Dismissed    (Dismiss while ready)

Displayed → Accepted (Engineer accepts)
Displayed → Dismissed (Engineer dismisses)

Accepted → Completed (Work completed)
Accepted → Dismissed (Dismiss after acceptance - unusual)

Completed → Archived (Archive completed item)
Dismissed → Archived (Archive dismissed item)
```

### Invalid Transitions

```
Completed → Any     (Terminal state)
Dismissed → Any     (Terminal state)
Archived → Any      (Terminal state)

Candidate → Displayed (Must go through Ready first)
Ready → Accepted (Must be Displayed first)
Displayed → Completed (Must be Accepted first)
```

## Transition Validation

### is_valid_transition Function

```rust
fn is_valid_transition(from: LifecycleState, to: LifecycleState) -> bool {
    match from {
        LifecycleState::Candidate => matches!(to, LifecycleState::Ready | LifecycleState::Dismissed),
        LifecycleState::Ready => matches!(to, LifecycleState::Displayed | LifecycleState::Dismissed),
        LifecycleState::Displayed => matches!(to, LifecycleState::Accepted | LifecycleState::Dismissed),
        LifecycleState::Accepted => matches!(to, LifecycleState::Completed | LifecycleState::Dismissed),
        LifecycleState::Completed => false, // Terminal
        LifecycleState::Dismissed => false, // Terminal
        LifecycleState::Archived => false,  // Terminal
    }
}
```

## Lifecycle Manager Structure

```rust
pub struct RecommendationLifecycle {
    states: HashMap<uuid::Uuid, LifecycleState>,
    history: Vec<LifecycleRecord>,
    shown_ids: Vec<uuid::Uuid>,
    completed_ids: Vec<uuid::Uuid>,
    dismissed_ids: Vec<uuid::Uuid>,
    archived_ids: Vec<uuid::Uuid>,
}
```

### Fields

- `states`: Current state for each recommendation
- `history`: Complete history of all state transitions
- `shown_ids`: Recommendations that have been displayed
- `completed_ids`: Recommendations that have been completed
- `dismissed_ids`: Recommendations that have been dismissed
- `archived_ids`: Recommendations that have been archived

## Lifecycle Record

```rust
pub struct LifecycleRecord {
    pub recommendation_id: uuid::Uuid,
    pub from_state: LifecycleState,
    pub to_state: LifecycleState,
    pub timestamp: DateTime<Utc>,
}
```

### Record Creation

```rust
impl LifecycleRecord {
    pub fn new(recommendation_id: uuid::Uuid, to_state: LifecycleState) -> Self {
        Self {
            recommendation_id,
            from_state: to_state.clone(),
            to_state,
            timestamp: Utc::now(),
        }
    }
    
    pub fn with_previous(mut self, from_state: LifecycleState) -> Self {
        self.from_state = from_state;
        self
    }
}
```

## Lifecycle Methods

### mark_ready

```rust
pub fn mark_ready(&mut self, recommendation_id: uuid::Uuid) -> Result<(), String> {
    let current = self.transition(recommendation_id, LifecycleState::Candidate, LifecycleState::Ready)?;
    Ok(())
}
```

### mark_displayed

```rust
pub fn mark_displayed(&mut self, recommendation_id: uuid::Uuid) -> Result<(), String> {
    self.transition(recommendation_id, LifecycleState::Ready, LifecycleState::Displayed)?;
    if !self.shown_ids.contains(&recommendation_id) {
        self.shown_ids.push(recommendation_id);
    }
    Ok(())
}
```

### mark_accepted

```rust
pub fn mark_accepted(&mut self, recommendation_id: uuid::Uuid) -> Result<(), String> {
    self.transition(recommendation_id, LifecycleState::Displayed, LifecycleState::Accepted)?;
    Ok(())
}
```

### dismiss

```rust
pub fn dismiss(&mut self, recommendation_id: uuid::Uuid) -> Result<(), String> {
    let current = self.state(recommendation_id)?;
    if current == LifecycleState::Candidate {
        self.transition(recommendation_id, LifecycleState::Candidate, LifecycleState::Dismissed)?;
    } else {
        self.transition(recommendation_id, current, LifecycleState::Dismissed)?;
    }
    Ok(())
}
```

### mark_completed

```rust
pub fn mark_completed(&mut self, recommendation_id: uuid::Uuid) -> Result<(), String> {
    self.transition(recommendation_id, LifecycleState::Accepted, LifecycleState::Completed)?;
    Ok(())
}
```

### state

```rust
pub fn state(&self, recommendation_id: uuid::Uuid) -> Result<LifecycleState, String> {
    match self.states.get(&recommendation_id) {
        Some(state) => Ok(state.clone()),
        None => Err(format!("Recommendation {} not found", recommendation_id)),
    }
}
```

### is_ready

```rust
pub fn is_ready(&self, recommendation_id: uuid::Uuid) -> bool {
    matches!(
        self.states.get(&recommendation_id),
        Some(&LifecycleState::Candidate | LifecycleState::Ready)
    )
}
```

### is_shown

```rust
pub fn is_shown(&self, recommendation_id: uuid::Uuid) -> bool {
    self.shown_ids.contains(&recommendation_id)
}
```

## Error Handling

### Transition Errors

The `transition` method returns `Result<(), String>` with errors for:
- **Invalid transition**: State transition not allowed
- **Not found**: Recommendation ID doesn't exist
- **Terminal state**: Attempting transition from terminal state

```rust
fn transition(&mut self, id: uuid::Uuid, from: LifecycleState, to: LifecycleState) -> Result<LifecycleState, String> {
    // Check if transition is valid
    if !is_valid_transition(from, to) {
        return Err(format!("Invalid transition from {:?} to {:?}", from, to));
    }
    
    // Get current state
    let current = self.states.get(&id)
        .ok_or_else(|| format!("Recommendation {} not found", id))?
        .clone();
    
    // Verify current state matches expected from state
    if current != from {
        return Err(format!("Expected state {:?} but found {:?}", from, current));
    }
    
    // Update state
    self.states.insert(id, to.clone());
    
    // Record transition
    self.record_transition(id, to.clone(), Some(from));
    
    // Update collections
    match to {
        LifecycleState::Dismissed => {
            if !self.dismissed_ids.contains(&id) {
                self.dismissed_ids.push(id);
            }
        }
        LifecycleState::Completed => {
            if !self.completed_ids.contains(&id) {
                self.completed_ids.push(id);
            }
        }
        LifecycleState::Archived => {
            if !self.archived_ids.contains(&id) {
                self.archived_ids.push(id);
            }
        }
        _ => {}
    }
    
    Ok(current)
}
```

## Usage Example

```rust
let mut lifecycle = RecommendationLifecycle::new();

// Add recommendation
let rec = Recommendation::new("Fix memory leak", ...);
lifecycle.add(&rec);

// Mark as ready
lifecycle.mark_ready(rec.id).unwrap();

// Check if ready
assert!(lifecycle.is_ready(rec.id));

// Mark as displayed
lifecycle.mark_displayed(rec.id).unwrap();
assert!(lifecycle.is_shown(rec.id));

// Engineer accepts
lifecycle.mark_accepted(rec.id).unwrap();

// Mark as completed
lifecycle.mark_completed(rec.id).unwrap();

// Cannot transition from Completed
assert!(lifecycle.transition(rec.id, LifecycleState::Completed, LifecycleState::Ready).is_err());
```

## Testing

Tests cover:
- Complete lifecycle flow (Candidate → Ready → Displayed → Accepted → Completed)
- Dismissal from various states
- Terminal state enforcement (no transitions from Completed/Dismissed/Archived)
- State history recording
- Shown/completed/dismissed/archived collections
- Invalid transition rejection
- Not-found error handling
- is_ready and is_shown methods
- Default state initialization