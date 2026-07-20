# Copilot Engine Recommendation Engine

## Overview

The Recommendation Engine generates actionable recommendations from observation events. It classifies observations, generates evidence, assigns priority, and incorporates engineering context.

## Observation Classification

### Classification Types

| Type | Keywords | Priority | Confidence |
|------|----------|----------|------------|
| **Error** | error, crash, fail, exception, panic | Critical | 0.95 |
| **Resource** | memory, cpu, disk, pod, container, limit | Warning | 0.85 |
| **Performance** | slow, latency, timeout, bottleneck | Warning | 0.75 |
| **Security** | vuln, breach, unauthorized, exploit | Critical | 0.90 |
| **Deprecation** | deprecated, end-of-life, migrate | Warning | 0.70 |
| **Information** | (everything else) | Suggestion | 0.50 |

### Classification Logic

```rust
fn classify_observation(&self, observation: &str, context: &EngineeringContext) -> ObservationType {
    let obs_lower = observation.to_lowercase();
    
    // Error detection
    if obs_lower.contains("error") || obs_lower.contains("crash") || 
       obs_lower.contains("fail") || obs_lower.contains("exception") {
        return ObservationType::Error;
    }
    
    // Resource detection
    if obs_lower.contains("memory") || obs_lower.contains("cpu") || 
       obs_lower.contains("disk") || obs_lower.contains("pod") || 
       obs_lower.contains("container") {
        return ObservationType::Resource;
    }
    
    // Performance detection
    if obs_lower.contains("slow") || obs_lower.contains("latency") || 
       obs_lower.contains("timeout") || obs_lower.contains("bottleneck") {
        return ObservationType::Performance;
    }
    
    // Security detection
    if obs_lower.contains("vuln") || obs_lower.contains("breach") || 
       obs_lower.contains("unauthorized") || obs_lower.contains("exploit") {
        return ObservationType::Security;
    }
    
    // Default: Information
    ObservationType::Information
}
```

## Recommendation Generation

### Generation Parameters

Each recommendation includes:

- **Title**: Concise summary of the issue
- **Description**: Detailed explanation
- **Reason**: Why this recommendation was made
- **Confidence**: Numeric confidence score (0.0 - 1.0)
- **Priority**: Critical/Warning/Suggestion/Information
- **Evidence**: List of evidence sources supporting the recommendation
- **Supporting documents**: References to relevant documentation
- **Suggested next step**: Optional actionable next step
- **Workflow context**: Optional workflow state context

### Template Matching

The engine uses predefined templates for common scenarios:

```rust
// Error template
(
    "Error Detected: {error_type}".into(),
    observation.to_string(),
    format!("An error was detected: {}", observation),
    0.95,
    Priority::Critical,
)

// Resource template
(
    "Resource Limit Approaching".into(),
    observation.to_string(),
    "Resource usage is approaching configured limits based on observation data",
    0.85,
    Priority::Warning,
)

// Performance template
(
    "Performance Degradation".into(),
    observation.to_string(),
    "Performance issue detected based on observation data",
    0.75,
    Priority::Warning,
)
```

## Context Awareness

### Engineering Context Integration

The recommendation engine incorporates:

- **Technology stack**: Tailors recommendations to detected technologies
- **Workspace context**: Considers workspace-specific information
- **Timeline**: References recent engineering activity
- **Prior recommendations**: Avoids repeating similar recommendations
- **Engineer preferences**: Incorporates known preferences

```rust
pub struct EngineeringContext {
    pub technologies: Vec<String>,
    pub workspace: String,
    pub timeline: Vec<TimelineEntry>,
    pub prior_recommendations: Vec<String>,
    pub workflow_state: Option<String>,
    pub user_preferences: HashMap<String, String>,
}
```

### Context-Enhanced Recommendations

```rust
fn generate(
    &mut self,
    title: String,
    description: String,
    reason: String,
    confidence: f64,
    priority: Priority,
    evidence: Vec<(&str, &str, Confidence)>,
    context: &EngineeringContext,
    suggested_next_step: Option<String>,
) -> GenerationResult {
    // Incorporate technology context
    let mut tech_note = if !context.technologies.is_empty() {
        format!(" Technologies: {}", context.technologies.join(", "))
    } else {
        String::new()
    };
    
    // Check for repetition
    let is_duplicate = self.recent_titles.iter().any(|t| t == &title);
    if is_duplicate {
        tracing::warn!("Recommendation '{title}' recently generated — may be repetitive");
    }
    
    // Generate recommendation
    let rec = Recommendation::new(
        title,
        description,
        reason,
        Confidence::new(confidence),
        evidence.into_iter().map(|(source, desc, conf)| {
            Evidence {
                source: source.to_string(),
                description: desc.to_string(),
                confidence: conf,
            }
        }).collect(),
        priority,
        vec![],
    )
    .with_next_step(suggested_next_step.unwrap_or_default())
    .with_workflow_context(context.workflow_state.clone().unwrap_or_default());
    
    // Track generation
    self.total_generations += 1;
    self.recent_titles.push(title.clone());
    if self.recent_titles.len() > self.max_recent {
        self.recent_titles.drain(..self.recent_titles.len() - self.max_recent);
    }
    
    GenerationResult {
        recommendation: rec,
        context_used: self.collect_context_used(context),
    }
}
```

## Observation Processing

### from_observations API

```rust
pub fn from_observations(
    &mut self,
    observations: &[String],
    context: &EngineeringContext,
) -> Vec<GenerationResult> {
    let mut results = Vec::new();
    
    for observation in observations {
        // Classify observation
        let (title, description, reason, confidence, priority) = 
            self.classify_observation(observation, context);
        
        // Generate recommendation
        let result = self.generate(
            title,
            description,
            reason,
            confidence,
            priority,
            vec![("Observation", observation.as_str(), Confidence::new(confidence))],
            context,
            None,
        );
        
        results.push(result);
    }
    
    results
}
```

## Evidence Generation

### Auto-Evidence from Observations

The engine auto-generates evidence from observation events:

```rust
vec![
    (
        "Observation Source",
        observation.as_str(),
        Confidence::new(confidence),
    )
]
```

### Contextual Evidence

Additional evidence can be added from:
- **Knowledge base**: Referenced documentation
- **Previous interactions**: Engineer corrections or feedback
- **Technology detection**: Confirmed technology stack
- **Workflow state**: Current workflow context

## Deduplication

### Recent Titles Tracking

The engine tracks recent titles to avoid repetition:

```rust
pub struct RecommendationEngine {
    total_generations: u32,
    recent_titles: Vec<String>,
    max_recent: usize,
}
```

- **max_recent**: Configurable (default 100)
- **Drain strategy**: When limit exceeded, oldest titles are removed
- **Duplicate warning**: Log warning when duplicate detected

## Generation Results

### GenerationResult Structure

```rust
pub struct GenerationResult {
    pub recommendation: Recommendation,
    pub context_used: Vec<String>,
}
```

**Fields:**
- `recommendation`: The generated recommendation
- `context_used`: List of context sources that influenced the recommendation

## API Reference

### Creating Engine

```rust
let mut engine = RecommendationEngine::new();
```

### Generate from Observations

```rust
let results = engine.from_observations(
    &["Pod memory at 85%".to_string()],
    &engineering_context,
);
```

### Generate Single Recommendation

```rust
let result = engine.generate(
    "Title".to_string(),
    "Description".to_string(),
    "Reason".to_string(),
    0.85,
    Priority::Warning,
    vec![("Source", "Evidence", Confidence::new(0.85))],
    &context,
    Some("Next step".to_string()),
);
```

### Get Statistics

```rust
let count = engine.generation_count();
```

## Testing

Tests cover:
- Observation classification (all types)
- Recommendation generation
- Context incorporation
- Deduplication logic
- Evidence generation
- Statistics tracking
- Max recent title tracking
- Default context handling