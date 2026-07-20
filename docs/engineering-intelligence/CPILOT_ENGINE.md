# Copilot Engine Architecture

## Overview

The Copilot Engine is the central orchestration layer for the Wiki Labs AI Copilot. It coordinates all subsystems in the observation→recommendation→approval loop, ensuring the AI never performs work autonomously while providing timely, context-aware assistance.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Copilot Engine                            │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │Decision  │  │Recommend-│  │Policy    │  │Lifecycle │   │
│  │Engine    │  │ation     │  │Engine    │  │Manager   │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │Session   │  │Conversation│ │Explain-  │  │Human     │   │
│  │Memory    │  │Context    │ │ability    │  │Approval  │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │Proactive │  │Contextual│  │Priority  │  │Modes     │   │
│  │Assistance│  │Follow-Up │  │Filter    │  │Config    │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │Cards     │  │Approval  │  │Engine    │  │Recommend-│   │
│  │(Display) │  │Request   │  │Orchestrat│  │ation     │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Core Modules

### 1. Decision Engine (`decision.rs`)

Evaluates whether a recommendation should be shown based on:
- **Confidence threshold**: Minimum 0.5 confidence (0.3 for Critical)
- **Evidence quality**: At least one evidence source required (except Critical)
- **User state**: Pauses don't filter, typing blocks suggestions
- **Session limits**: Max 5 recommendations per session
- **Frequency limits**: Max 2 recommendations per minute
- **Repetition avoidance**: Prevents showing same recommendation twice
- **Workflow relevance**: Filters based on current workflow state

**Key methods:**
- `evaluate()`: Run all filters and return DecisionOutcome
- `record_shown()`: Track displayed recommendations
- `record_intent_change()`: Handle user intent override

**Decision rules (priority order):**
1. Critical priority always shown regardless of confidence
2. Low confidence (<0.3) filtered unless Critical
3. No evidence filtered unless Critical
4. Paused state filters everything
5. Typing state blocks suggestions
6. Session limit (5) enforced
7. Frequency limit (2/minute) enforced
8. Repetition check prevents duplicate display
9. Workflow relevance filters mismatched recommendations

### 2. Recommendation Engine (`recommendation.rs`)

Generates recommendations from observation events:
- **Observation classification**: Classifies observations as error, resource, performance, security, or information
- **Template matching**: Uses predefined templates for common scenarios
- **Context awareness**: Incorporates technology stack, workspace, timeline
- **Evidence generation**: Auto-generates evidence from observations
- **Priority assignment**: Assigns Critical/Warning/Suggestion/Information

**Key methods:**
- `from_observations()`: Generate recommendations from observation list
- `generate()`: Create single recommendation from parameters
- `classify_observation()`: Classify observation type and assign priority

**Observation classifications:**
- **Error**: "error", "crash", "fail", "exception" → Critical priority
- **Resource**: "memory", "cpu", "disk", "pod", "container" → Warning priority
- **Performance**: "slow", "latency", "timeout", "bottleneck" → Warning priority
- **Security**: "vuln", "breach", "unauthorized", "exploit" → Critical priority
- **Information**: Everything else → Suggestion/Information priority

### 3. Policy Engine (`policy.rs`)

Enforces policy levels that control recommendation visibility:
- **Minimal**: Only Critical with high confidence
- **Balanced**: Critical + Warning with high confidence
- **Teaching**: All above Suggestion level
- **Expert**: All recommendations shown
- **Silent**: No recommendations shown

**Key methods:**
- `filter_recommendations()`: Apply policy level to recommendation list
- `with_level()`: Set policy level
- `config()`: Get policy configuration

**Policy filtering logic:**
```rust
match policy_level {
    PolicyLevel::Minimal => [Critical],
    PolicyLevel::Balanced => [Critical, Warning],
    PolicyLevel::Teaching => [Critical, Warning, Suggestion],
    PolicyLevel::Expert => [Critical, Warning, Suggestion, Information],
    PolicyLevel::Silent => [],
}
```

### 4. Lifecycle Manager (`lifecycle.rs`)

Manages recommendation state transitions:
- **States**: Candidate → Ready → Displayed → Accepted/Dismissed → Completed/Archived
- **No repetition**: Track shown recommendations to prevent re-display
- **State history**: Record all transitions for debugging

**Key methods:**
- `mark_ready()`: Transition Candidate → Ready
- `mark_displayed()`: Transition Ready → Displayed
- `mark_accepted()`: Transition Displayed → Accepted
- `dismiss()`: Transition to Dismissed (terminal)
- `mark_completed()`: Transition Accepted → Completed
- `is_ready()`: Check if recommendation is ready to show
- `is_shown()`: Check if recommendation was already displayed

**State transitions:**
```
Candidate → Ready → Displayed → Accepted → Completed
              → Dismissed (terminal)
                    → Archived
```

### 5. Session Memory (`memory.rs`)

Tracks engineer interactions for personalization:
- **Acceptance rate**: Track recommendations accepted vs total
- **Dismissal reasons**: Record why recommendations were dismissed
- **Correction tracking**: Record engineer corrections to recommendations
- **Topic analysis**: Identify topics engineer frequently corrects
- **Confidence adjustment**: Reduce confidence on frequently corrected topics

**Key methods:**
- `record_accepted()`: Track recommendation acceptance
- `record_dismissed()`: Track dismissal with reason
- `record_correction()`: Track engineer correction
- `acceptance_rate()`: Calculate acceptance percentage
- `correction_rate()`: Calculate correction percentage
- `confidence_adjustment_for_topic()`: Calculate confidence penalty

### 6. Conversation Context (`conversation.rs`)

Maintains conversation history and follow-up context:
- **Turn tracking**: Record questions and answers
- **Topic tracking**: Identify ongoing topics for follow-up
- **No repetition**: Track discussed recommendations to avoid re-mentioning
- **Context awareness**: Maintain context for multi-turn conversations

**Key methods:**
- `record_shown()`: Track recommendation display
- `record_dismissed()`: Track recommendation dismissal
- `record_corrected()`: Track recommendation correction
- `get_context_summary()`: Get conversation summary
- `pending_topics()`: Identify topics needing follow-up

### 7. Explainability (`explainability.rs`)

Provides traceable reasoning for recommendations:
- **Reason trees**: Build explanation trees with mandatory and optional nodes
- **Evidence mapping**: Link explanation nodes to evidence sources
- **Certainty scoring**: Include confidence in explanations
- **Human-readable output**: Generate explanations for engineers

**Key methods:**
- `build_reason_tree()`: Create explanation tree
- `with_nodes()`: Add explanation nodes
- `add_node()`: Add individual explanation node
- `generate_explanation()`: Render explanation for display

### 8. Human Approval (`approval.rs`)

Enforces human-in-the-loop requirements:
- **Approval requests**: Create approval requests for actions
- **Approval states**: Pending → Approved/Denied/AutoApproved
- **Auto-approval**: Low-risk actions can be auto-approved
- **Audit trail**: Track all approval decisions

**Key methods:**
- `create_request()`: Create new approval request
- `approve()`: Approve request by ID
- `deny()`: Deny request with reason
- `is_resolved()`: Check if request is approved or denied
- `can_proceed()`: Check if recommendation can proceed

### 9. Proactive Assistance (`proactive.rs`)

Determines when to interrupt the engineer:
- **Signal types**: Error detected, idle detection, resource threshold, related work, high confidence
- **Signal classification**: Categorize signals by type and urgency
- **Flooding prevention**: Prevent notification spam
- **Interrupt policy**: Determine if interruption is warranted

**Key methods:**
- `should_interrupt()`: Check if engineer should be interrupted
- `record_signal()`: Track signal history
- `is_flooding()`: Check if signals exceed threshold
- `with_idle_threshold()`: Configure idle detection threshold

### 10. Contextual Follow-Up (`follow_up.rs`)

Generates contextual follow-up suggestions:
- **Keyword analysis**: Analyze recommendation title for follow-up triggers
- **Priority-specific follow-ups**: Generate follow-ups based on recommendation priority
- **Deduplication**: Prevent duplicate follow-up suggestions
- **Guidance**: Provide actionable next steps

**Key methods:**
- `generate()`: Generate follow-ups for single recommendation
- `generate_batch()`: Generate follow-ups for multiple recommendations
- `contains_keyword()`: Check if text contains trigger keywords

**Follow-up categories:**
- **Security**: Critical follow-up required
- **Performance**: Warning-level validation needed
- **Refactoring**: Informational safety check
- **Test coverage**: Warning-level coverage check
- **Deprecation**: Critical migration warning
- **Next steps**: Guidance based on suggested_next_step

### 11. Priority Filtering (`priority.rs`)

Filters recommendations by priority level:
- **Priority levels**: Critical (4), Warning (3), Suggestion (2), Information (1)
- **Policy mapping**: Map policy levels to priority filters
- **Score-based filtering**: Filter by numeric priority score
- **Most urgent identification**: Find highest priority recommendation

**Key methods:**
- `default_for_policy()`: Create filter for policy level
- `passes()`: Check if priority passes filter
- `most_urgent()`: Find highest priority
- `to_priority_score()`: Convert Priority to numeric score

### 12. Modes (`modes.rs`)

Configures copilot operating modes:
- **Minimal**: Critical only
- **Balanced**: Critical + Warning
- **Teaching**: All above Suggestion
- **Expert**: All recommendations
- **Silent**: No recommendations

**Key methods:**
- `with_mode()`: Set operating mode
- `config()`: Get mode configuration
- `description()`: Get human-readable mode description

### 13. Recommendation Cards (`cards.rs`)

Generates display cards for recommendations:
- **Card structure**: Title, technology, confidence, priority, reason, actions
- **Action types**: Explain, Open Documentation, Dismiss, Mark Complete
- **Minimal mode**: Strip non-essential actions in minimal mode
- **Color coding**: Visual priority indicators

**Key methods:**
- `from_recommendation()`: Create card from recommendation
- `priority_color()`: Get color for priority level
- `with_actions()`: Set available actions
- `to_minimal_view()`: Generate minimal view

### 14. Copilot Engine (`engine.rs`)

Orchestrates all subsystems:
- **Observation processing**: Observe → Classify → Generate → Filter → Display
- **Question processing**: Answer engineer questions with context
- **Feedback recording**: Track accepted/dismissed/corrected
- **Stats tracking**: Monitor copilot performance

**Key methods:**
- `observe()`: Process observation and generate recommendation
- `process_question()`: Answer engineer questions
- `record_accepted()`: Record recommendation acceptance
- `record_dismissed()`: Record recommendation dismissal
- `record_correction()`: Record recommendation correction
- `stats()`: Get copilot statistics

## Data Flow

```
Observation Event
    ↓
Classification (error/resource/performance/security/info)
    ↓
Recommendation Generation (with evidence)
    ↓
Lifecycle: Candidate → Ready
    ↓
Decision Engine Evaluation (all filters)
    ↓
Policy Engine Filtering (policy level)
    ↓
Recommendation Display (with card)
    ↓
Human Approval (if required)
    ↓
Engineer Action (Accept/Dismiss/Correct)
    ↓
Lifecycle: Displayed → Accepted → Completed
    ↓
Session Memory Update
```

## Safety Constraints

1. **No autonomous execution**: AI never runs commands or makes changes
2. **Human approval required**: All recommendations require human review
3. **Evidence-based**: Every recommendation includes evidence
4. **Explainable**: All decisions are traceable and explainable
5. **Context-aware**: Recommendations tied to current context
6. **No repetition**: Same recommendation not shown twice
7. **Frequency limits**: Max 2 recommendations per minute
8. **Session limits**: Max 5 recommendations per session

## Integration

The Copilot Engine integrates with:
- **Observation Layer**: Receives observation events
- **AI Runtime**: Uses AI provider for enhanced recommendations
- **Skill System**: Leverages MCP skills for domain knowledge
- **Knowledge Base**: References local knowledge documents
- **Desktop UI**: Displays recommendations via Tauri commands

## Testing

132 unit tests covering:
- Decision engine filters and rules
- Recommendation generation and classification
- Policy level filtering
- Lifecycle state transitions
- Session memory tracking
- Conversation context management
- Proactive signal handling
- Follow-up suggestion generation
- Priority filtering
- Card generation and minimal view
- Approval request handling
- Explainability tree building
- Mode configuration
- Engine orchestration