//! Observation engine — tiered pipeline.
//!
//! Tier 1: Instant (sub-ms) — shell integration, clipboard
//! Tier 2: Fast (1-2s) — app monitor, window detection
//! Tier 3: Slow (5-10s) — screen capture, OCR
//!
//! - Shell integration (bash, zsh, PowerShell)
//! - Active window detection
//! - Clipboard observer
//! - Screen capture (X11/Wayland/Wayland/CG/DXGI)
//! - OCR fallback (Tesseract)
//! - Adaptive interval: 1s active, 10s idle
//! - Credential pattern filtering

pub mod tier1;
pub mod tier2;
pub mod tier3;
pub mod shell;
pub mod app_monitor;
pub mod clipboard;
pub mod capture;
pub mod ocr;
pub mod credential_filter;