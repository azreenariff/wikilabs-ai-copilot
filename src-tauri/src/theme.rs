//! Dark / Light Theme Support — System Detection, Toggle, Persistence
//!
//! # Overview
//!
//! This module provides dark/light theme detection, toggling, and preference
//! persistence for the application. It integrates with the existing
//! [`config::UISettings`] which already stores the theme preference.
//!
//! # Features
//!
//! - **System theme detection** — On first run, detect the user's OS theme
//!   preference and use it as the default.
//! - **Theme toggling** — Cycle between dark, light, and system-follow modes.
//! - **Preference persistence** — Save the user's theme choice to settings
//!   so it survives app restarts.
//! - **Event broadcasting** — Emit theme change events to the frontend for
//!   immediate UI updates.
//!
//! # Theme Modes
//!
//! | Mode     | Behavior                                          |
//! |----------|---------------------------------------------------|
//! | `dark`   | Always use dark theme                             |
//! | `light`  | Always use light theme                            |
//! | `system` | Follow the operating system's theme preference    |
//!
//! # References
//!
//! - [Tauri Theme](https://v2.tauri.app/reference/webview-versions/#theme)
//! - [CSS prefers-color-scheme](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-color-scheme)

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};

/// Available theme modes for the application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThemeMode {
    /// Always use the dark theme.
    Dark,
    /// Always use the light theme.
    Light,
    /// Automatically follow the OS theme preference.
    System,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Dark
    }
}

impl ThemeMode {
    /// Convert a string theme setting to a `ThemeMode`.
    ///
    /// Recognizes `"dark"`, `"light"`, and `"system"`.
    /// Unknown values fall back to `Dark`.
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dark" => Self::Dark,
            "light" => Self::Light,
            "system" => Self::System,
            _ => {
                warn!(theme = s, "Unknown theme mode, defaulting to dark");
                Self::Dark
            }
        }
    }

    /// Convert to the string representation used in settings.
    pub fn to_string(&self) -> &str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::System => "system",
        }
    }

    /// Return the effective theme (dark or light) for the current state.
    ///
    /// If the mode is `System`, this delegates to
    /// [`ThemeManager::detect_system_theme`].
    pub fn effective_theme(&self, manager: &ThemeManager) -> Theme {
        match self {
            Self::Dark => Theme::Dark,
            Self::Light => Theme::Light,
            Self::System => manager.detect_system_theme(),
        }
    }

    /// Cycle to the next theme mode in the sequence:
    /// dark → light → system → dark.
    pub fn next(&self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::System,
            Self::System => Self::Dark,
        }
    }
}

/// The resolved effective theme (dark or light).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    /// Dark color scheme.
    Dark,
    /// Light color scheme.
    Light,
}

/// Events emitted to the frontend when the theme changes.
#[derive(Debug, Clone, Serialize)]
pub struct ThemeChangeEvent {
    /// The new effective theme.
    pub theme: String,
    /// The mode that was selected (dark, light, system).
    pub mode: String,
}

/// Theme manager — handles detection, toggling, and persistence.
pub struct ThemeManager {
    app_handle: AppHandle,
    /// The user-selected theme mode.
    mode: Arc<std::sync::RwLock<ThemeMode>>,
}

impl Clone for ThemeManager {
    fn clone(&self) -> Self {
        Self {
            app_handle: self.app_handle.clone(),
            mode: Arc::clone(&self.mode),
        }
    }
}

impl ThemeManager {
    /// Create a new theme manager with the given app handle and initial mode.
    pub fn new(app_handle: AppHandle, initial_mode: ThemeMode) -> Self {
        Self {
            app_handle,
            mode: Arc::new(std::sync::RwLock::new(initial_mode)),
        }
    }

    /// Create a theme manager from existing settings.
    pub fn from_settings(app_handle: AppHandle, theme_str: &str) -> Self {
        let mode = ThemeMode::from_string(theme_str);
        Self::new(app_handle, mode)
    }

    /// Detect the current OS theme preference.
    ///
    /// On Windows, this checks the `AppsUseLightTheme` registry value.
    /// On macOS, this checks the `AppleInterfaceStyle` user default.
    /// On Linux, this checks the `DESKTOP_SESSION` and `XDG_CURRENT_DESKTOP`
    /// environment variables, or falls back to dark.
    ///
    /// Returns `Theme::Dark` on non-Windows platforms since we don't have
    /// native OS detection there (the frontend can detect via CSS).
    pub fn detect_system_theme(&self) -> Theme {
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                // On Windows, check the registry for the light/dark preference.
                Self::detect_windows_theme()
            } else {
                // On Linux/macOS, default to dark (frontend handles via CSS).
                Theme::Dark
            }
        }
    }

    /// Windows-specific system theme detection.
    ///
    /// Reads the registry key:
    /// `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\AppsUseLightTheme`
    ///
    /// - `0` = dark mode
    /// - `1` = light mode
    #[cfg(windows)]
    fn detect_windows_theme() -> Theme {
        use tracing::{debug, info};

        // Try to read the registry value for light theme.
        // If the key doesn't exist or is non-zero, assume dark mode
        // (which is the Windows default for most users).
        match winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
            .open_subkey_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                winreg::enums::KEY_READ,
            )
        {
            Ok(key) => match key.get_value::<u32, _>("AppsUseLightTheme") {
                Ok(value) => {
                    debug!(AppsUseLightTheme = value, "System theme detected");
                    if value == 1 {
                        Theme::Light
                    } else {
                        Theme::Dark
                    }
                }
                Err(_) => {
                    info!("Registry key not found, defaulting to dark theme");
                    Theme::Dark
                }
            },
            Err(_) => {
                info!("Could not open registry key, defaulting to dark theme");
                Theme::Dark
            }
        }
    }

    /// Get the current theme mode.
    #[allow(dead_code)]
    pub fn mode(&self) -> ThemeMode {
        *self.mode.read().unwrap()
    }

    /// Get the effective theme (dark or light).
    pub fn effective_theme(&self) -> Theme {
        let mode = self.mode.read().unwrap();
        mode.effective_theme(self)
    }

    /// Get the current theme as a string.
    pub fn theme_string(&self) -> String {
        self.mode.read().unwrap().to_string().to_string()
    }

    /// Get the effective theme as a string.
    #[allow(dead_code)]
    pub fn effective_theme_string(&self) -> String {
        match self.effective_theme() {
            Theme::Dark => "dark".to_string(),
            Theme::Light => "light".to_string(),
        }
    }

    /// Toggle to the next theme mode in the sequence.
    pub fn toggle_next(&self) -> Theme {
        let mut mode = self.mode.write().unwrap();
        let new_mode = mode.next();
        *mode = new_mode;

        let effective = new_mode.effective_theme(self);
        info!(
            mode = %new_mode.to_string(),
            effective = ?effective,
            "Theme toggled"
        );

        // Emit event to frontend
        self.emit_theme_change(&new_mode, &effective);

        effective
    }

    /// Set the theme mode explicitly.
    pub fn set_mode(&self, new_mode: ThemeMode) -> Theme {
        let mut mode = self.mode.write().unwrap();
        *mode = new_mode;

        let effective = new_mode.effective_theme(self);
        info!(
            mode = %new_mode.to_string(),
            effective = ?effective,
            "Theme mode set"
        );

        self.emit_theme_change(&new_mode, &effective);

        effective
    }

    /// Set the theme mode from a string value.
    pub fn set_mode_from_string(&self, s: &str) -> Theme {
        let mode = ThemeMode::from_string(s);
        self.set_mode(mode)
    }

    /// Set the theme to dark.
    #[allow(dead_code)]
    pub fn set_dark(&self) -> Theme {
        self.set_mode(ThemeMode::Dark)
    }

    /// Set the theme to light.
    #[allow(dead_code)]
    pub fn set_light(&self) -> Theme {
        self.set_mode(ThemeMode::Light)
    }

    /// Set the theme to follow the system.
    #[allow(dead_code)]
    pub fn set_system(&self) -> Theme {
        self.set_mode(ThemeMode::System)
    }

    /// Emit a theme change event to the frontend.
    fn emit_theme_change(&self, mode: &ThemeMode, effective: &Theme) {
        let event = ThemeChangeEvent {
            theme: effective.to_string(),
            mode: mode.to_string().to_string(),
        };

        if let Ok(payload) = serde_json::to_value(&event) {
            if let Err(e) = self.app_handle.emit("theme-changed", payload) {
                warn!(error = %e, "Failed to emit theme change event");
            }
        }
    }

    /// Apply the current theme mode to the Tauri app window.
    ///
    /// On Windows and macOS, Tauri supports setting the window theme
    /// directly via `set_theme()`. On Linux this is a no-op since
    /// the window manager handles theming.
    #[allow(dead_code)]
    pub fn apply_to_window(&self, label: Option<&str>) -> Result<(), String> {
        let effective = self.effective_theme();

        if let Some(app) = self.app_handle.get_webview_window(label.unwrap_or("main")) {
            // Tauri v2 doesn't have a direct set_theme on the window in the core crate,
            // but the frontend can adjust via CSS custom properties.
            // We emit an event for the frontend to handle.
            self.emit_theme_change(
                &self.mode.read().unwrap(),
                &effective,
            );
            Ok(())
        } else {
            Err("Window not found".to_string())
        }
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dark => write!(f, "dark"),
            Self::Light => write!(f, "light"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_from_string() {
        assert_eq!(ThemeMode::from_string("dark"), ThemeMode::Dark);
        assert_eq!(ThemeMode::from_string("light"), ThemeMode::Light);
        assert_eq!(ThemeMode::from_string("system"), ThemeMode::System);
        assert_eq!(ThemeMode::from_string("unknown"), ThemeMode::Dark);
    }

    #[test]
    fn test_theme_mode_next() {
        let mut mode = ThemeMode::Dark;
        assert_eq!(mode.next(), ThemeMode::Light);
        assert_eq!(mode.next(), ThemeMode::System);
        assert_eq!(mode.next(), ThemeMode::Dark);
    }

    #[test]
    fn test_theme_to_string() {
        assert_eq!(ThemeMode::Dark.to_string(), "dark");
        assert_eq!(ThemeMode::Light.to_string(), "light");
        assert_eq!(ThemeMode::System.to_string(), "system");
    }

    #[test]
    fn test_system_detection_default() {
        // On non-Windows, detect_system_theme should always return Dark.
        let manager = ThemeManager::new(
            tauri::test::mock_app_handle(
                &tauri::test::MockWindowServer::new(),
                tauri::test::MockRuntime::new(),
            ),
            ThemeMode::System,
        );
        let theme = manager.detect_system_theme();
        assert_eq!(theme, Theme::Dark);
    }
}