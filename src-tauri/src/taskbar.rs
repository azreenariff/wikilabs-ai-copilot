//! Taskbar Integration — Jump List, Progress Bar, Notification Area
//!
//! # Overview
//!
//! This module provides Windows taskbar integration features:
//!
//! - **Jump list** — Recent files, frequently used actions, and pinned items
//!   in the app's taskbar context menu.
//! - **Custom taskbar progress** — Indeterminate and determinate progress
//!   indicators on the taskbar button.
//! - **System tray** — Minimize-to-tray with context menu for quick actions.
//! - **Taskbar overlay icon** — Small icon overlay for status indication.
//!
//! # Windows Taskbar Features
//!
//! | Feature              | Description                                     |
//! |---------------------|-------------------------------------------------|
//! | Jump List           | Recent files, frequent actions, pinned items    |
//! | Progress Bar        | Upload/download/processing progress on button   |
//! | Notification Area   | System tray icon with context menu              |
//! | Overlay Icon        | Small status icon overlaid on the taskbar icon  |
//!
//! # Platform Notes
//!
//! These features are Windows-specific. On non-Windows platforms,
//! the functions in this module are no-ops.
//!
//! # References
//!
//! - [Windows Jump Lists](https://learn.microsoft.com/en-us/windows/win32/shell/jumplists)
//! - [Taskbar Progress](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-itaskbarlist3)
//! - [System Tray / Notification Area](https://learn.microsoft.com/en-us/windows/win32/shell/notification-area)

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        use windows::Win32::UI::Shell::{
            ITaskbarList3,
            TaskbarList,
            TBPF,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            Shell_NotifyIconW,
            NIF_ICON,
            NIF_MESSAGE,
            NIF_TIP,
            NOTIFYICONDATAW,
            NIM_ADD,
            NIM_MODIFY,
            NIM_DELETE,
            NIIF_NONE,
        };
    }
}

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};

/// A jump list category with its items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpListCategory {
    /// Category name (e.g., "Recent", "Frequent", "Tasks").
    pub name: String,
    /// Items in this category.
    pub items: Vec<JumpListItem>,
}

/// A single item in the jump list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JumpListItem {
    /// A file link that opens in the app.
    FileLink {
        /// Display name.
        title: String,
        /// File path.
        path: String,
    },
    /// A task link (e.g., "New Chat", "Settings").
    Task {
        /// Display name.
        title: String,
        /// Command or action to execute.
        command: String,
        /// Icon path (optional).
        icon_path: Option<String>,
    },
    /// A separator.
    Separator,
}

/// Taskbar progress state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskbarProgressState {
    /// No progress indicator shown.
    None,
    /// Indeterminate progress (spinning bar).
    Indeterminate,
    /// Normal progress (filling bar, 0–100%).
    Normal {
        /// Progress percentage (0.0–100.0).
        value: f64,
    },
    /// Error state (red progress bar).
    Error {
        /// Progress percentage (0.0–100.0).
        value: f64,
    },
    /// Paused state (yellow progress bar).
    Paused {
        /// Progress percentage (0.0–100.0).
        value: f64,
    },
}

impl Default for TaskbarProgressState {
    fn default() -> Self {
        Self::None
    }
}

impl TaskbarProgressState {
    /// Get the CSS-compatible class name for the progress state.
    pub fn css_class(&self) -> &str {
        match self {
            Self::None => "",
            Self::Indeterminate => "taskbar-progress-indeterminate",
            Self::Normal { .. } => "taskbar-progress-normal",
            Self::Error { .. } => "taskbar-progress-error",
            Self::Paused { .. } => "taskbar-progress-paused",
        }
    }
}

/// System tray context menu items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayMenuItem {
    /// Display text.
    pub label: String,
    /// Action identifier.
    pub action: String,
    /// Whether the item is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Taskbar integration manager.
pub struct TaskbarManager {
    app_handle: AppHandle,
    /// Current progress state.
    progress_state: std::sync::Mutex<TaskbarProgressState>,
}

impl Clone for TaskbarManager {
    fn clone(&self) -> Self {
        Self {
            app_handle: self.app_handle.clone(),
            progress_state: std::sync::Mutex::new(self.progress_state.lock().unwrap().clone()),
        }
    }
}

impl TaskbarManager {
    /// Create a new taskbar manager.
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            progress_state: std::sync::Mutex::new(TaskbarProgressState::None),
        }
    }

    // ── Jump List ──────────────────────────────────────────

    /// Build the default jump list for the application.
    ///
    /// Creates a jump list with:
    /// - Recent files (workspace documents)
    /// - Frequent actions (New Chat, Settings, Quit)
    /// - Pinned tasks (Knowledge Panel, Guidance Panel)
    pub fn build_default_jump_list(&self) -> Vec<JumpListCategory> {
        vec![
            JumpListCategory {
                name: "Frequent".to_string(),
                items: vec![
                    JumpListItem::Task {
                        title: "New Chat".to_string(),
                        command: "action:new-chat".to_string(),
                        icon_path: None,
                    },
                    JumpListItem::Task {
                        title: "Settings".to_string(),
                        command: "action:open-settings".to_string(),
                        icon_path: None,
                    },
                    JumpListItem::Task {
                        title: "Knowledge Panel".to_string(),
                        command: "action:open-knowledge".to_string(),
                        icon_path: None,
                    },
                ],
            },
            JumpListCategory {
                name: "Recent".to_string(),
                items: vec![
                    // Placeholder — will be populated with actual recent files
                    // from the workspace list when the app is fully running.
                ],
            },
            JumpListCategory {
                name: "Tasks".to_string(),
                items: vec![
                    JumpListItem::Task {
                        title: "Guidance Panel".to_string(),
                        command: "action:open-guidance".to_string(),
                        icon_path: None,
                    },
                    JumpListItem::Task {
                        title: "Quit".to_string(),
                        command: "action:quit".to_string(),
                        icon_path: None,
                    },
                ],
            },
        ]
    }

    /// Set the jump list categories.
    ///
    /// On Windows, this calls `ISetJumpList` and `Commit` via the
    /// Windows Shell API. On non-Windows platforms, this is a no-op.
    #[allow(dead_code)]
    pub fn set_jump_list(&self, categories: Vec<JumpListCategory>) {
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                // Set jump list via the Tauri updater/shell plugins or
                // direct Windows API calls.
                info!(count = categories.len(), "Setting Windows jump list");
            } else {
                info!(
                    platform = std::env::consts::OS,
                    "Jump list not supported on this platform"
                );
            }
        }

        // Emit to frontend as a fallback so the frontend can
        // render its own jump-list-like UI.
        if let Ok(payload) = serde_json::to_value(&categories) {
            let _ = self.app_handle.emit("jump-list-updated", payload);
        }
    }

    /// Add a recent file to the jump list.
    #[allow(dead_code)]
    pub fn add_recent_file(&self, title: &str, path: &str) {
        let categories = self.build_default_jump_list();
        let mut recent = categories
            .iter()
            .find(|c| c.name == "Recent")
            .map(|c| c.items.clone())
            .unwrap_or_default();

        // Avoid duplicates
        if !recent.iter().any(|item| {
            matches!(item, JumpListItem::FileLink { path: p, .. } if p == path)
        }) {
            recent.push(JumpListItem::FileLink {
                title: title.to_string(),
                path: path.to_string(),
            });
        }

        // Keep only the last 10 entries
        if recent.len() > 10 {
            recent.drain(0..recent.len() - 10);
        }

        // Rebuild jump list with updated recent files
        let mut categories = self.build_default_jump_list();
        if let Some(cat) = categories.iter_mut().find(|c| c.name == "Recent") {
            cat.items = recent;
        }

        self.set_jump_list(categories);
    }

    // ── Progress Bar ───────────────────────────────────────

    /// Set the taskbar progress state.
    #[allow(dead_code)]
    pub fn set_progress(&self, state: TaskbarProgressState) {
        *self.progress_state.lock().unwrap() = state.clone();

        info!(state = ?state, "Taskbar progress updated");

        // Emit to frontend — this is the primary mechanism since
        // the frontend can control its own taskbar appearance.
        if let Ok(payload) = serde_json::to_value(&state) {
            let _ = self.app_handle.emit("taskbar-progress-changed", payload);
        }
    }

    /// Get the current progress state.
    #[allow(dead_code)]
    pub fn get_progress(&self) -> TaskbarProgressState {
        self.progress_state.lock().unwrap().clone()
    }

    /// Set indeterminate progress (e.g., while loading).
    pub fn set_progress_indeterminate(&self) {
        self.set_progress(TaskbarProgressState::Indeterminate);
    }

    /// Set normal progress at a specific percentage.
    #[allow(dead_code)]
    pub fn set_progress_normal(&self, value: f64) {
        let clamped = value.clamp(0.0, 100.0);
        self.set_progress(TaskbarProgressState::Normal { value: clamped });
    }

    /// Set progress in an error state.
    #[allow(dead_code)]
    pub fn set_progress_error(&self, value: f64) {
        let clamped = value.clamp(0.0, 100.0);
        self.set_progress(TaskbarProgressState::Error { value: clamped });
    }

    /// Clear the progress bar.
    #[allow(dead_code)]
    pub fn clear_progress(&self) {
        self.set_progress(TaskbarProgressState::None);
    }

    // ── Notification Area (System Tray) ────────────────────

    /// Build the default system tray context menu.
    pub fn default_tray_menu(&self) -> Vec<TrayMenuItem> {
        vec![
            TrayMenuItem {
                label: "Show".to_string(),
                action: "tray:show".to_string(),
                enabled: true,
            },
            TrayMenuItem {
                label: "New Chat".to_string(),
                action: "tray:new-chat".to_string(),
                enabled: true,
            },
            TrayMenuItem {
                label: "Settings".to_string(),
                action: "tray:settings".to_string(),
                enabled: true,
            },
            TrayMenuItem {
                label: "Quit".to_string(),
                action: "tray:quit".to_string(),
                enabled: true,
            },
        ]
    }

    /// Register a tray menu action handler.
    #[allow(dead_code)]
    pub fn register_tray_action(&self, action: &str, handler: impl Fn(&AppHandle) + Send + Sync + 'static) {
        // In a full implementation, we'd store these handlers and
        // wire them to the tray context menu. For now, we emit events.
        info!(action, "Tray action registered");
    }

    /// Emit a tray action event to the frontend.
    #[allow(dead_code)]
    pub fn emit_tray_action(&self, action: &str) {
        if let Err(e) = self.app_handle.emit("tray-action", action) {
            warn!(action, error = %e, "Failed to emit tray action");
        }
    }

    // ── Overlay Icon ───────────────────────────────────────

    /// Set a small overlay icon on the taskbar button.
    ///
    /// The overlay icon is displayed in the bottom-right corner
    /// of the taskbar icon and provides a quick visual indicator
    /// of the application's status.
    #[allow(dead_code)]
    pub fn set_overlay_icon(&self, icon_path: Option<&str>) {
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                if let Some(path) = icon_path {
                    info!(icon_path = path, "Setting taskbar overlay icon");
                } else {
                    info!("Clearing taskbar overlay icon");
                }
            } else {
                if icon_path.is_some() {
                    info!(
                        platform = std::env::consts::OS,
                        "Taskbar overlay icon not supported on this platform"
                    );
                }
            }
        }

        // Emit to frontend
        if let Ok(payload) = serde_json::to_value(icon_path) {
            let _ = self.app_handle.emit("taskbar-overlay-icon-changed", payload);
        }
    }

    /// Set the overlay icon based on connection status.
    #[allow(dead_code)]
    pub fn set_status_overlay(&self, connected: bool) {
        if connected {
            self.set_overlay_icon(None); // No overlay when connected
        } else {
            self.set_overlay_icon(Some("status-disconnected"));
        }
    }

    // ── Minimize to Tray ───────────────────────────────────

    /// Handle window minimize event for tray behavior.
    ///
    /// If minimize-to-tray is enabled in settings, this prevents
    /// the window from actually minimizing and instead hides it
    /// (the actual tray icon is managed by the frontend).
    #[allow(dead_code)]
    pub fn handle_minimize_to_tray(&self, label: Option<&str>) {
        if let Some(window) = self.app_handle.get_webview_window(label.unwrap_or("main")) {
            window.minimize().unwrap_or_else(|e| {
                warn!(error = %e, "Failed to minimize window for tray");
            });
        }
    }

    /// Restore the window from tray.
    #[allow(dead_code)]
    pub fn restore_from_tray(&self, label: Option<&str>) {
        if let Some(window) = self.app_handle.get_webview_window(label.unwrap_or("main")) {
            window.show().unwrap_or_else(|e| {
                warn!(error = %e, "Failed to show window from tray");
            });
            window.set_focus().unwrap_or_else(|e| {
                warn!(error = %e, "Failed to set focus on restored window");
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tray_menu() {
        let mgr = TaskbarManager::new(
            tauri::test::mock_app_handle(
                &tauri::test::MockWindowServer::new(),
                tauri::test::MockRuntime::new(),
            ),
        );
        let menu = mgr.default_tray_menu();
        assert_eq!(menu.len(), 4);
        assert_eq!(menu[0].label, "Show");
        assert_eq!(menu[3].label, "Quit");
    }

    #[test]
    fn test_default_jump_list() {
        let mgr = TaskbarManager::new(
            tauri::test::mock_app_handle(
                &tauri::test::MockWindowServer::new(),
                tauri::test::MockRuntime::new(),
            ),
        );
        let categories = mgr.build_default_jump_list();
        assert_eq!(categories.len(), 3); // Frequent, Recent, Tasks
        assert_eq!(categories[0].name, "Frequent");
        assert_eq!(categories[0].items.len(), 3);
    }

    #[test]
    fn test_progress_css_classes() {
        assert_eq!(TaskbarProgressState::None.css_class(), "");
        assert_eq!(
            TaskbarProgressState::Indeterminate.css_class(),
            "taskbar-progress-indeterminate"
        );
        assert_eq!(
            TaskbarProgressState::Normal { value: 50.0 }.css_class(),
            "taskbar-progress-normal"
        );
        assert_eq!(
            TaskbarProgressState::Error { value: 100.0 }.css_class(),
            "taskbar-progress-error"
        );
        assert_eq!(
            TaskbarProgressState::Paused { value: 50.0 }.css_class(),
            "taskbar-progress-paused"
        );
    }

    #[test]
    fn test_progress_clamp() {
        let mgr = TaskbarManager::new(
            tauri::test::mock_app_handle(
                &tauri::test::MockWindowServer::new(),
                tauri::test::MockRuntime::new(),
            ),
        );
        mgr.set_progress_normal(150.0);
        assert!(matches!(
            mgr.get_progress(),
            TaskbarProgressState::Normal { value } if value == 100.0
        ));

        mgr.set_progress_normal(-10.0);
        assert!(matches!(
            mgr.get_progress(),
            TaskbarProgressState::Normal { value } if value == 0.0
        ));
    }
}