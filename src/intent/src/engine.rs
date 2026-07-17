//! Intent recognition engine — rule-based + ML.

pub struct IntentEngine;

impl IntentEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn recognize(&self, _context: &str) -> anyhow::Result<Intent> {
        // TODO: Classify intent from observation data
        anyhow::bail!("Not yet implemented")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Intent {
    Troubleshooting,
    Configuration,
    Deployment,
    Documentation,
    Learning,
    Unknown,
}