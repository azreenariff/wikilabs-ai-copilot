//! Intent recognition engine.
//!
//! - Rule-based pattern matching (Phase 1)
//! - ML-based classification (Phase 2+)
//! - Confidence scoring
//! - "Unknown" intent state (first-class concept)
//! - Human correction mechanisms

pub mod engine;
pub mod model;
pub mod confidence;
pub mod correction;