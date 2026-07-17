//! Cross-skill context bus — skills publish events, other skills subscribe.

pub struct ContextBus;

impl ContextBus {
    pub fn new() -> Self {
        Self
    }

    pub fn publish(&self, _event: serde_json::Value) -> anyhow::Result<()> {
        // TODO: Publish context event to bus
        anyhow::bail!("Not yet implemented")
    }

    pub fn subscribe(&self, _handler: impl FnMut(serde_json::Value) + Send + 'static) {
        // TODO: Register handler for context events
    }
}