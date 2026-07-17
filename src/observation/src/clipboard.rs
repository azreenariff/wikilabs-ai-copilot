//! Clipboard observer — monitor clipboard changes.

pub struct ClipboardObserver;

impl ClipboardObserver {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_content(&self) -> anyhow::Result<String> {
        // TODO: Read clipboard
        anyhow::bail!("Not yet implemented")
    }
}