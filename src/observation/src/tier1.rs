//! Tier 1 — Instant observation (sub-ms latency).
//! Shell integration, clipboard observer.

pub struct Tier1Engine;

impl Tier1Engine {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        // Stub: placeholder. Implement shell integration hooks (bash, zsh, PowerShell).
        unimplemented!()
    }
}