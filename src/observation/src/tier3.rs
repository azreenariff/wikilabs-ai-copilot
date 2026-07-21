//! Tier 3 — Slow observation (5-10s latency).
//! Screen capture, OCR.

pub struct Tier3Engine;

impl Tier3Engine {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        // Stub: placeholder. Implement screen capture loop.
        unimplemented!()
    }
}