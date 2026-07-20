# Guidance Decision Engine

**Phase 10** — Wiki Labs AI Copilot

---

## Overview

The Guidance Decision Engine is responsible for deciding **when** to provide guidance, **what** guidance is relevant, **how detailed** it should be, and **whether** interruption is appropriate.

The AI observes, understands, explains, recommends, and guides — but never interrupts unnecessarily.

## Design Principles

1. **Context-aware** — Evaluates screen context, technology, workflow stage, and engineer activity before deciding.
2. **Adaptive detail** — Changes guidance detail level based on mode (Teaching → Comprehensive, Expert → Minimal).
3. **Respects engineer** — Never interrupts while the engineer is active (typing, interacting).
4. **Confidence-based** — Higher confidence justifies more assertive guidance.
5. **No repetition** — Avoids re-showing previous recommendations.

## Architecture

```
Observation Framework
    ↓
Engineering Intelligence Engine
    ↓
Knowledge Platform
    ↓
┌─────────────────────────┐
│  Guidance Decision      │
│  Engine                 │
│                         │
│  • evaluate(criteria)   │
│  • should_interrupt()   │
│  • mode-aware           │
└─────────────────────────┘
    ↓
Recommendation Framework
```

## Key Types

### GuidanceMode

Four modes control how the AI behaves:

| Mode | Detail Level | Interrupt | Description |
|------|-------------|-----------|-------------|
| `Balanced` | Standard | High confidence only | Default mode |
| `Teaching` | Comprehensive | High confidence only | Thorough explanations |
| `Expert` | Minimal | High confidence only | Brief, high-signal |
| `Silent` | Minimal | Never | Only when asked |

### DetailLevel

How detailed the guidance should be:

| Level | Description |
|-------|-------------|
| `Minimal` | One-liner, no explanation |
| `Standard` | Explanation included |
| `Comprehensive` | Full context, background, alternatives |

### DecisionCriteria

Inputs evaluated by the engine:

```rust
pub struct DecisionCriteria {
    screen_context: Option<ScreenContext>,  // Active app, URL, window title
    current_technology: Option<String>,     // e.g. "OpenShift"
    workflow_stage: Option<String>,         // e.g. "evidence collection"
    is_engineer_active: bool,               // Is the engineer typing/interacting?
    has_knowledge_sources: bool,            // Are knowledge sources available?
    confidence: f64,                        // 0.0 - 1.0 confidence of detection
    previous_recommendations: Vec<String>,  // Already shown in this session
    guidance_mode: GuidanceMode,            // Current mode
}
```

### GuidanceDecision

The output of the evaluation:

```rust
pub struct GuidanceDecision {
    should_guidance: bool,    // Whether to provide guidance at all
    detail_level: DetailLevel, // How detailed it should be
    should_interrupt: bool,    // Whether to interrupt the engineer
    reasoning: Vec<String>,   // Why this decision was made
}
```

## Decision Logic

The engine evaluates criteria in sequence:

1. **Confidence threshold** — If confidence < 0.5, set Minimal detail and note the limitation.
2. **Engineer activity** — If engineer is active, defer guidance (no interruption).
3. **Knowledge sources** — If no technology or knowledge detected, set Minimal detail.
4. **Guidance mode** — Adjust detail level and interrupt policy based on mode:
   - Teaching → Comprehensive, no interruption
   - Expert → Minimal
   - Silent → No guidance at all
   - Balanced → Standard detail
5. **Minimum level enforcement** — Ensure detail meets the configured minimum.

## Example: Decision Flow

```
Engineer opens OpenShift dashboard, idle, 0.92 confidence.

Criteria:
  screen_context: { app: "Firefox", url: "openshift.example.com" }
  current_technology: "OpenShift"
  workflow_stage: "application failure investigation"
  is_engineer_active: false
  has_knowledge_sources: true
  confidence: 0.92
  guidance_mode: Balanced

Evaluation:
  ✓ Confidence 0.92 >= 0.5 → Standard detail
  ✓ Engineer is idle → May interrupt (confidence >= 0.8)
  ✓ OpenShift detected + knowledge available → Standard detail
  ✓ Balanced mode → Standard detail, may interrupt
  ✓ No mode override → Standard detail

Decision:
  should_guidance: true
  detail_level: Standard
  should_interrupt: true
  reasoning: [
    "Confidence 0.92 above threshold 0.50",
    "Engineer is idle — guidance may be shown",
    "OpenShift technology detected with knowledge sources",
    "Balanced mode — standard detail level",
    "Guidance provided at standard detail level"
  ]
```

## Validation Checklist

- ✅ Guidance only, no execution
- ✅ No interruption while engineer is active
- ✅ Mode-based detail adjustment works correctly
- ✅ Confidence thresholds enforced
- ✅ Silent mode suppresses all guidance
- ✅ All decisions include reasoning
- ✅ 10 unit tests covering all modes and edge cases