//! Cross-skill context bus — skills publish events, other skills subscribe.

pub struct ContextBus;

impl Default for ContextBus {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextBus {
    pub fn new() -> Self {
        Self
    }

    pub fn publish(&self, _event: serde_json::Value) -> anyhow::Result<()> {
        // Stub: placeholder. Publish context event to bus.
        unimplemented!()
    }

    pub fn subscribe(&self, _handler: impl FnMut(serde_json::Value) + Send + 'static) {
        // Stub: placeholder. Register handler for context events.
    }
}
