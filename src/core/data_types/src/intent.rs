//! Intent types that the AI can recognize from user input.

use serde::{Deserialize, Serialize};

/// Detected intent of a user query or system observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// User needs help diagnosing a problem.
    Troubleshooting,
    /// User wants to change or configure something.
    Configuration,
    /// User needs to deploy or roll back.
    Deployment,
    /// User wants documentation or reading material.
    Documentation,
    /// User is learning or exploring.
    Learning,
    /// Intent could not be determined.
    Unknown,
}

impl std::fmt::Display for Intent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Intent::Troubleshooting => write!(f, "troubleshooting"),
            Intent::Configuration => write!(f, "configuration"),
            Intent::Deployment => write!(f, "deployment"),
            Intent::Documentation => write!(f, "documentation"),
            Intent::Learning => write!(f, "learning"),
            Intent::Unknown => write!(f, "unknown"),
        }
    }
}
