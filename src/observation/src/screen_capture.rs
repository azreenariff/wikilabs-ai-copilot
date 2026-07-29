//! Observation Framework — Real Screen Capture Provider
//!
//! Periodic screenshot capture using Win32 BitBlt → GDI Bitmap → PNG encoding.
//! Stores screenshots in a rotating buffer (last N captures) as base64-encoded PNG.
//!
//! Platform support:
//! - Windows: BitBlt + GDI (fully implemented)
//! - Linux: X11/xcb (stub — requires libxcb-dev)
//! - macOS: CGWindowListCopyWindowInfo/CoreGraphics (stub)
//!
//! The captured screenshots are emitted as ObservationEvents with base64-encoded
//! PNG data, suitable for sending to Vision AI models.
//!
//! Screen resolution is scaled down to max_width x max_height to reduce payload size.
//! Default: 1920x1080 max (from whatever the actual resolution is).

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::event::{EventType, ObservationEvent, ObservationPayload, ProviderType};
use crate::provider::{ObservationProvider, ProviderConfig, ProviderLifecycle, ProviderState};

/// Configuration for the screen capture provider.
#[derive(Debug, Clone)]
pub struct ScreenCaptureConfig {
    /// Interval between captures in seconds.
    pub poll_interval_secs: u64,
    /// Maximum screenshot width (screenshots are scaled down if larger).
    pub max_width: u32,
    /// Maximum screenshot height.
    pub max_height: u32,
    /// Number of recent screenshots to keep in the rotating buffer.
    pub buffer_size: usize,
    /// Whether to capture all monitors or just the primary.
    pub capture_all_monitors: bool,
}

impl Default for ScreenCaptureConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 30,
            max_width: 1920,
            max_height: 1080,
            buffer_size: 5,
            capture_all_monitors: false,
        }
    }
}

/// A captured screenshot (metadata + base64-encoded PNG data).
#[derive(Debug, Clone)]
pub struct CapturedScreenshot {
    /// Timestamp of capture.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Width of the captured image (after scaling).
    pub width: u32,
    /// Height of the captured image (after scaling).
    pub height: u32,
    /// Base64-encoded PNG data.
    pub data_base64: String,
    /// Primary window name that was in focus during capture.
    pub focused_window: Option<String>,
    /// Whether this was a full screen or window-specific capture.
    pub capture_type: ScreenshotType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenshotType {
    /// Full screen capture (all monitors or primary only).
    FullScreen,
    /// Single monitor capture.
    SingleMonitor,
    /// Window-specific capture.
    Window,
}

/// Screen capture provider state.
pub struct ScreenCaptureState {
    pub config: ProviderConfig,
    pub state: ProviderState,
    pub lifecycle: ProviderLifecycle,
    pub screen_config: ScreenCaptureConfig,
    pub last_screenshot: Option<CapturedScreenshot>,
    pub screenshot_buffer: Vec<CapturedScreenshot>,
}

impl ScreenCaptureState {
    fn new(config: ProviderConfig, screen_config: ScreenCaptureConfig) -> Self {
        Self {
            config,
            state: ProviderState::Disabled,
            lifecycle: ProviderLifecycle::new(),
            screen_config,
            last_screenshot: None,
            screenshot_buffer: Vec::new(),
        }
    }
}

/// Screen observation provider.
pub struct ScreenCaptureProvider {
    state: Arc<Mutex<ScreenCaptureState>>,
}

impl ScreenCaptureProvider {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ScreenCaptureState::new(
                ProviderConfig::default(),
                ScreenCaptureConfig::default(),
            ))),
        }
    }

    pub fn with_config(config: ProviderConfig, screen_config: ScreenCaptureConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(ScreenCaptureState::new(
                config,
                screen_config,
            ))),
        }
    }

    /// Capture the screen and return a base64-encoded PNG.
    fn capture(&self) -> Option<CapturedScreenshot> {
        #[cfg(target_os = "windows")]
        {
            screen_capture_windows::screen_capture_windows(
                self.state.lock().unwrap().screen_config.clone(),
            )
        }

        #[cfg(not(target_os = "windows"))]
        {
            None
        }
    }
}

impl Default for ScreenCaptureProvider {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl ObservationProvider for ScreenCaptureProvider {
    fn provider_type(&self) -> ProviderType { ProviderType::ScreenCapture }
    fn name(&self) -> &str { "ScreenCapture" }
    fn description(&self) -> &str {
        "Periodic screenshot capture with PNG encoding, rotating buffer, Vision AI integration"
    }
    fn config(&self) -> ProviderConfig { self.state.lock().unwrap().config.clone() }
    fn set_config(&mut self, config: ProviderConfig) { self.state.lock().unwrap().config = config; }
    fn state(&self) -> ProviderState { self.state.lock().unwrap().state.clone() }

    async fn start(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.lifecycle.start();
        state.state = ProviderState::Active;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.lifecycle.stop();
        state.state = ProviderState::Disabled;
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if matches!(state.state, ProviderState::Active) {
            state.state = ProviderState::Paused;
            Ok(())
        } else {
            Err("Provider is not currently active".to_string())
        }
    }

    async fn resume(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if matches!(state.state, ProviderState::Paused) {
            state.state = ProviderState::Active;
            Ok(())
        } else {
            Err("Provider is not currently paused".to_string())
        }
    }

    async fn observe(&self) -> Result<Vec<ObservationEvent>, String> {
        match self.capture() {
            Some(screenshot) => {
                // Update buffer
                {
                    let mut state = self.state.lock().unwrap();
                    state.last_screenshot = Some(screenshot.clone());
                    state.screenshot_buffer.push(screenshot.clone());
                    if state.screenshot_buffer.len() > state.screen_config.buffer_size {
                        state.screenshot_buffer.remove(0);
                    }
                }

                Ok(vec![ObservationEvent::new(
                    EventType::ScreenshotCaptured,
                    ProviderType::ScreenCapture,
                    format!("capture_{}x{}", screenshot.width, screenshot.height),
                    None,
                    ObservationPayload::new(serde_json::json!({
                        "width": screenshot.width,
                        "height": screenshot.height,
                        "focused_window": screenshot.focused_window,
                        "capture_type": format!("{:?}", screenshot.capture_type),
                        "data_base64_preview": format!("{}... ({} chars)", &screenshot.data_base64[..min(100, screenshot.data_base64.len())], screenshot.data_base64.len()),
                        "buffer_size": self.state.lock().unwrap().screenshot_buffer.len(),
                    })),
                )])
            }
            None => {
                Ok(vec![ObservationEvent::new(
                    EventType::ScreenshotCaptured,
                    ProviderType::ScreenCapture,
                    "no_capture".to_string(),
                    None,
                    ObservationPayload::new(serde_json::json!({
                        "status": "no_screenshot_available",
                        "platform": std::env::consts::OS,
                        "buffer_size": self.state.lock().unwrap().screenshot_buffer.len(),
                    })),
                )])
            }
        }
    }

    fn lifecycle(&self) -> crate::provider::ProviderLifecycle {
        self.state.lock().unwrap().lifecycle.clone()
    }

    fn status_details(&self) -> HashMap<String, serde_json::Value> {
        let state = self.state.lock().unwrap();
        let mut details = HashMap::new();
        if let Some(ref s) = state.last_screenshot {
            details.insert("last_width".to_string(), serde_json::json!(s.width));
            details.insert("last_height".to_string(), serde_json::json!(s.height));
            details.insert("last_capture_time".to_string(), serde_json::json!(s.timestamp.to_rfc3339()));
            details.insert("last_focused_window".to_string(), serde_json::json!(s.focused_window));
        }
        details.insert("buffer_size".to_string(), serde_json::json!(state.screenshot_buffer.len()));
        details.insert("poll_interval_secs".to_string(), serde_json::json!(state.screen_config.poll_interval_secs));
        details.insert("platform".to_string(), serde_json::json!(std::env::consts::OS));
        details
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Helper: return min of two values
fn min(a: usize, b: usize) -> usize { if a < b { a } else { b } }

// ── Windows Screen Capture Implementation ───────────────────────────

#[cfg(target_os = "windows")]
mod screen_capture_windows {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC,
        DeleteObject, GetDC, GetDeviceCaps, ReleaseDC,
        SelectObject, SRCCOPY, BITMAPINFO, BITMAPINFOHEADER,
        DIB_RGB_COLORS, GetDIBits, HORZRES, VERTRES,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };
    use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;

    use crate::screen_capture::{CapturedScreenshot, ScreenshotType, ScreenCaptureConfig};
    use base64::Engine;
    use image::ImageEncoder;

    pub(super) fn screen_capture_windows(config: ScreenCaptureConfig) -> Option<CapturedScreenshot> {
        unsafe {
            // Get foreground window name
                        let foreground_hwnd = GetForegroundWindow();
                        let mut focused_window = String::new();
                        if !foreground_hwnd.0.is_null() {
                            let mut buf = vec![0u16; 512];
                            let len = GetWindowTextW(foreground_hwnd, &mut buf);
                            if len > 0 {
                                focused_window = String::from_utf16_lossy(&buf[..len as usize]);
                            } else {
                                focused_window.clear();
                            }
                        }

            // Get process name for more detail
            let mut process_name = String::new();
            if !foreground_hwnd.0.is_null() {
                let mut pid: u32 = 0;
                let _ = GetWindowThreadProcessId(foreground_hwnd, Some(&mut pid));
                if pid > 0 {
                    let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid);
                    if let Ok(proc_handle) = handle {
                        let mut exe_buf = [0u16; 260];
                        let exe_len = GetModuleFileNameExW(proc_handle, None, &mut exe_buf);
                        if exe_len > 0 {
                            let exe_path = String::from_utf16_lossy(&exe_buf[..exe_len as usize]);
                            let path = std::path::Path::new(&exe_path);
                            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                if !focused_window.is_empty() {
                                    focused_window = format!("{} — {}", name, focused_window);
                                } else {
                                    focused_window = name.to_string();
                                }
                            }
                        }
                        let _ = windows::Win32::Foundation::CloseHandle(proc_handle);
                    }
                }
            }

            // Capture the full primary monitor
            let dc = GetDC(HWND::default());
            if dc.is_invalid() {
                return None;
            }

            let screen_width = GetDeviceCaps(dc, HORZRES);
            let screen_height = GetDeviceCaps(dc, VERTRES);

            if screen_width <= 0 || screen_height <= 0 {
                let _ = ReleaseDC(HWND::default(), dc);
                return None;
            }

            // Scale down if needed
            let (width, height) = if (screen_width as i32) > config.max_width as i32 || (screen_height as i32) > config.max_height as i32 {
                let ratio = (config.max_width as f64 / screen_width as f64)
                    .min(config.max_height as f64 / screen_height as f64)
                    .max(0.1);
                let w = (screen_width as f64 * ratio).round() as i32;
                let h = (screen_height as f64 * ratio).round() as i32;
                (w.max(100), h.max(100))
            } else {
                (screen_width, screen_height)
            };

            // Create compatible DC and bitmap
            let mem_dc = CreateCompatibleDC(dc);
            if mem_dc.is_invalid() {
                let _ = ReleaseDC(HWND::default(), dc);
                return None;
            }

            // Create DIB section for direct pixel access
            let mut bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // top-down DIB
                    biPlanes: 1,
                    biBitCount: 32, // 32-bit RGBA
                    biCompression: 0, // BI_RGB
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [Default::default()], // 1-element array for 32-bit DIB
            };

            // Create DIB section
            let bitmap_handle = CreateCompatibleBitmap(dc, width, height);
            if bitmap_handle.is_invalid() {
                let _ = DeleteDC(mem_dc);
                let _ = ReleaseDC(HWND::default(), dc);
                return None;
            }

            let old_bitmap = SelectObject(mem_dc, bitmap_handle);

            // BitBlt the screen into our DIB
            let success = BitBlt(
                mem_dc,
                0, 0, width, height,
                dc,
                0, 0,
                SRCCOPY,
            );

            // Encode the bitmap to PNG using the image crate
            // Extract pixel data from the DIB
            let mut pixel_buffer = vec![0u8; (width * height * 4) as usize];
            let bits_result = GetDIBits(
                mem_dc,
                bitmap_handle,
                0,
                height as u32,
                Some(pixel_buffer.as_mut_ptr() as *mut _),
                &mut bitmap_info as *mut _,
                DIB_RGB_COLORS,
            );

            // Convert BGRA (from DIB) to RGB and create PNG
            let png_data = bgra_to_png(&pixel_buffer, width as u32, height as u32);

            // Restore old bitmap and cleanup
            SelectObject(mem_dc, old_bitmap);

            // Cleanup
            let _ = DeleteObject(bitmap_handle);
            let _ = SelectObject(mem_dc, old_bitmap);
            let _ = DeleteDC(mem_dc);
            let _ = ReleaseDC(HWND::default(), dc);

            if png_data.is_empty() {
                return None;
            }

            // Encode PNG to base64
            let data_base64 = base64::engine::general_purpose::STANDARD.encode(&png_data);

            Some(CapturedScreenshot {
                timestamp: chrono::Utc::now(),
                width: width as u32,
                height: height as u32,
                data_base64,
                focused_window: if focused_window.is_empty() { None } else { Some(focused_window) },
                capture_type: ScreenshotType::FullScreen,
            })
        }
    }

    /// Convert BGRA pixel buffer to PNG-encoded bytes using the image crate.
    fn bgra_to_png(buffer: &[u8], width: u32, height: u32) -> Vec<u8> {
        use image::{ExtendedColorType, Rgba, RgbaImage};

        // BGRA to RGBA — create RgbaImage
        let mut rgba_image = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                if idx + 3 < buffer.len() {
                    // BGRA → RGBA
                    let b = buffer[idx];
                    let g = buffer[idx + 1];
                    let r = buffer[idx + 2];
                    let a = buffer[idx + 3];
                    rgba_image.put_pixel(x, y, Rgba([r, g, b, a]));
                }
            }
        }

        // Encode to PNG in memory using image 0.25 API
        let mut data = Vec::new();
        let mut encoder = image::codecs::png::PngEncoder::new(&mut data);
        match encoder.write_image(
            &rgba_image,
            width,
            height,
            ExtendedColorType::Rgba8,
        ) {
            Ok(_) => data,
            Err(e) => {
                tracing::warn!("Failed to encode screenshot as PNG: {}", e);
                Vec::new()
            }
        }
    }
}

// ── Linux Screen Capture (Stub) ─────────────────────────────────────

#[cfg(not(target_os = "windows"))]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn screen_capture_windows(_config: ScreenCaptureConfig) -> Option<CapturedScreenshot> {
    // Linux: requires X11/xcb or Wayland xdg-desktop-portal
    // Implementation requires libxcb-dev, x11 crate
    None
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_capture_creation() {
        let provider = ScreenCaptureProvider::new();
        assert_eq!(provider.provider_type(), ProviderType::ScreenCapture);
        assert_eq!(provider.name(), "ScreenCapture");
    }

    #[test]
    fn test_provider_lifecycle() {
        let mut provider = ScreenCaptureProvider::new();
        assert_eq!(provider.state(), ProviderState::Disabled);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            assert!(provider.start().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Active);
            assert!(provider.pause().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Paused);
            assert!(provider.resume().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Active);
            assert!(provider.stop().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Disabled);
        });
    }

    #[test]
    fn test_observe_emits_event() {
        let mut provider = ScreenCaptureProvider::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let events = rt.block_on(async {
            provider.start().await.unwrap();
            provider.observe().await.unwrap()
        });
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, EventType::ScreenshotCaptured);
    }
}