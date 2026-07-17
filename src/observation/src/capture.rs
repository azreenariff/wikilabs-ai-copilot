//! Screen capture — platform-specific implementations.

pub struct ScreenCapture;

#[derive(Debug)]
pub struct CaptureResult {
    pub width: usize,
    pub height: usize,
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self
    }

    pub async fn capture(&self) -> anyhow::Result<CaptureResult> {
        // TODO: Capture screen (DXGI / CG / X11 / Wayland)
        anyhow::bail!("Not yet implemented")
    }
}