//! MCP Transport layer — JSON-RPC over HTTP.

pub struct TransportLayer;

impl TransportLayer {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        // TODO: Start JSON-RPC server on localhost
        anyhow::bail!("Not yet implemented")
    }
}