//! Active application monitor — window title, URL, process.

pub struct AppMonitor;

#[derive(Debug)]
pub struct AppContext {
    pub window_title: String,
    pub process_name: String,
    pub url: Option<String>,
}

impl AppMonitor {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_active(&self) -> anyhow::Result<AppContext> {
        // TODO: Query active window
        anyhow::bail!("Not yet implemented")
    }
}