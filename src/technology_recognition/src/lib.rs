//! Technology Recognition Engine
//!
//! Uses detection rules to identify technologies from observation events.
//! All technology detection comes from DetectionRules defined in skill packages —
//! no hardcoded technology logic exists in this engine.
//!
//! ## Architecture
//!
//! - **DetectionEngine** (`engine`): Core recognition engine matching events against rules
//! - **RecognitionPipeline** (`pipeline`): Multi-pass pipeline for combining evidence
//! - **EvidenceAggregator** (`aggregator`): Aggregates confidence from multiple detections

pub mod engine;
pub mod pipeline;
pub mod aggregator;

// Re-export key types for convenience
pub use engine::DetectionEngine;
pub use pipeline::run_pass;
pub use aggregator::aggregate_by_technology;