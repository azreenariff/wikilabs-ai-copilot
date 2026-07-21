//! MCP Transport layer — JSON-RPC over HTTP.

pub struct TransportLayer;

impl Default for TransportLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportLayer {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        // Stub: placeholder. Start JSON-RPC server on localhost.
        unimplemented!()
    }
}