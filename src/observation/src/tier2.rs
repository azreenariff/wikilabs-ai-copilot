//! Tier 2 — Fast observation (1-2s latency).
//! App monitor, window detection.

pub struct Tier2Engine;

impl Tier2Engine {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        // TODO: Start window polling
        anyhow::bail!("Not yet implemented")
    }
}