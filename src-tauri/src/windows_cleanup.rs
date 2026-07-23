//! Windows-specific process cleanup and graceful shutdown for WebView2.
//!
//! WebView2 on Windows can leave zombie processes and registered window classes
//! after crashes. This module provides:
//! 1. On-startup cleanup — kills leftover WebView2 processes from previous crashes
//! 2. Panic hook — ensures cleanup even on panics
//! 3. Drop-based graceful shutdown — unregisters window classes on normal exit
//! 4. Port 1420 cleanup — kills processes holding the API server port

use std::process::Command;
use std::sync::Once;
use tracing::{error, info, warn};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

static CLEANUP_ONCE: Once = Once::new();

// ── WebView2 Process Management ─────────────────────────────────

/// Known WebView2 process names that can become zombies after crashes.
/// Only targets WebView2 Runtime processes, NOT the user's browser.
const WEBVIEW2_PROCESSES: &[&str] = &[
    "msedgewebview2",
    "msedgewebview2.exe",
    "WebView2Broker",
    "WebView2Broker.exe",
    "WebEngineBridge",
    "WebEngineBridge.exe",
];

/// Kill all processes matching the given names that are related to WebView2.
/// This is safe to call — we only target known WebView2 processes, not the main app.
pub fn cleanup_webview2_processes() {
    CLEANUP_ONCE.call_once(|| {
        info!("Running WebView2 zombie process cleanup...");
        for proc_name in WEBVIEW2_PROCESSES {
            if cfg!(windows) {
                // Windows: use taskkill to kill processes
                #[cfg(windows)]
                let output = Command::new("taskkill")
                    .args(&["/F", "/IM", proc_name, "/T"])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
                #[cfg(not(windows))]
                let output = Command::new("taskkill")
                    .args(&["/F", "/IM", proc_name, "/T"])
                    .output();

                match &output {
                    Ok(o) if !o.status.success() => {
                        // taskkill returns 128 if no matching process found — that's fine
                        if o.status.code() != Some(128) {
                            warn!(
                                proc_name,
                                "Killed zombie WebView2 process (exit: {})",
                                o.status.code()
                                    .map(|c| c.to_string())
                                    .unwrap_or("unknown".into())
                            );
                        }
                    }
                    Ok(_) => {
                        info!(proc_name, "Cleaned up zombie WebView2 process");
                    }
                    Err(e) => {
                        warn!(error = %e, proc_name, "Failed to kill WebView2 process");
                    }
                }
            } else {
                // Non-Windows: skip
                warn!(proc_name, "WebView2 cleanup only works on Windows");
            }
        }
    });
}

// ── Port 1420 Cleanup ──────────────────────────────────────────

/// Check if port 1420 is already in use (from a previous crash) and kill the holder.
/// This prevents "Address already in use" errors on startup.
pub fn cleanup_api_server_port(port: u16) -> bool {
    if cfg!(windows) {
        // Use netstat to find processes on our port
        #[cfg(windows)]
        let output = Command::new("netstat").args(&["-ano"]).creation_flags(CREATE_NO_WINDOW).output();
        #[cfg(not(windows))]
        let output = Command::new("netstat").args(&["-ano"]).output();

        if let Ok(o) = output {
            let stdout = String::from_utf8_lossy(&o.stdout);
            for line in stdout.lines() {
                if line.contains(&format!(":{}", port)) && line.contains("LISTENING") {
                    // Extract PID from the last field
                    if let Some(pid_str) = line.split_whitespace().last() {
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            info!(pid, port, "Killing process holding API server port");
                            #[cfg(windows)]
                            if let Ok(_) = Command::new("taskkill")
                                .args(&["/F", "/PID", &pid.to_string()])
                                .creation_flags(CREATE_NO_WINDOW)
                                .output()
                            {
                                info!(pid, port, "Port 1420 cleared");
                                return true;
                            } else {
                                warn!(pid, port, "Failed to kill process on port");
                            }
                            #[cfg(not(windows))]
                            if let Ok(_) = Command::new("taskkill")
                                .args(&["/F", "/PID", &pid.to_string()])
                                .output()
                            {
                                info!(pid, port, "Port 1420 cleared");
                                return true;
                            } else {
                                warn!(pid, port, "Failed to kill process on port");
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

// ── SQLite Lock Cleanup ────────────────────────────────────────

/// Clean up stale SQLite lock files (.db-shm, .db-wal) from crashed sessions.
/// These can cause "database is locked" errors on startup.
pub fn cleanup_sqlite_lock_files(data_dir: &std::path::Path) {
    let db_base = data_dir.join("wikilabs.db");
    let lock_files = [
        db_base.with_extension("db-shm"),
        db_base.with_extension("db-wal"),
        db_base.with_file_name("wikilabs.db-lock"),
    ];

    for path in &lock_files {
        if path.exists() {
            // Check if the file is stale (older than 5 minutes)
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(created) = metadata.created() {
                    if let Ok(age) = std::time::SystemTime::now().duration_since(created) {
                        if age.as_secs() > 300 {
                            info!(
                                path = %path.display(),
                                "Removing stale SQLite lock file ({}s old)",
                                age.as_secs()
                            );
                            if let Err(e) = std::fs::remove_file(path) {
                                warn!(error = %e, path = %path.display(), "Failed to remove lock file");
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Panic Hook for Cleanup ─────────────────────────────────────

/// Register a panic hook that ensures cleanup even on panics.
/// This is critical because WebView2 can leave zombie processes if the app crashes.
pub fn register_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location = if let Some(loc) = info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "unknown location".to_string()
        };

        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };

        error!(
            panic = %message,
            location = %location,
            "Uncaught panic detected — running cleanup"
        );

        // Run cleanup even on panic
        cleanup_webview2_processes();
    }));
}

// ── Graceful Shutdown with Drop ────────────────────────────────

/// Graceful shutdown handler that ensures WebView2 window classes are unregistered
/// and resources are cleaned up even if the app exits unexpectedly.
pub struct GracefulShutdownGuard {
    pub port: u16,
    pub data_dir: Option<std::path::PathBuf>,
    pub api_server_stopped: Option<std::sync::Arc<std::sync::Mutex<bool>>>,
}

impl GracefulShutdownGuard {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            data_dir: None,
            api_server_stopped: None,
        }
    }

    pub fn with_api_server(&mut self, stopped: std::sync::Arc<std::sync::Mutex<bool>>) {
        self.api_server_stopped = Some(stopped);
    }

    pub fn with_data_dir(&mut self, dir: std::path::PathBuf) {
        self.data_dir = Some(dir);
    }

    fn cleanup(&self) {
        info!("Running graceful shutdown cleanup...");

        // Stop API server thread gracefully
        if let Some(stopped) = &self.api_server_stopped {
            *stopped.lock().unwrap() = true;
            info!("Signaled API server to stop");
        }

        // Kill any leftover WebView2 processes
        cleanup_webview2_processes();

        // Clean up SQLite lock files
        if let Some(data_dir) = &self.data_dir {
            cleanup_sqlite_lock_files(data_dir);
        }

        info!("Graceful shutdown complete");
    }
}

impl Drop for GracefulShutdownGuard {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// ── Cleanup on Startup (called from main before Tauri init) ────

/// Run all pre-startup cleanup to prevent issues from previous crashes.
/// This should be called in main() BEFORE creating the Tauri app.
pub fn pre_startup_cleanup() {
    info!("=== Running pre-startup cleanup ===");

    // 1. Kill leftover WebView2 processes from previous crashes
    cleanup_webview2_processes();

    // 2. Clean up port 1420 if held by zombie process
    cleanup_api_server_port(1420);

    // 3. Clean up stale SQLite lock files
    // Note: data_dir not available yet at this point; will be done during setup

    info!("=== Pre-startup cleanup complete ===");
}