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
        // Stub: placeholder. Implement screen capture (DXGI on Windows, CG on macOS, X11/Wayland on Linux).
        unimplemented!()
    }
}