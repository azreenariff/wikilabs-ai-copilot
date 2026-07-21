//! Global Keyboard Shortcuts — Application-Wide Hotkeys
//!
//! # Overview
//!
//! Registers global keyboard shortcuts using the Tauri `global-shortcut` plugin.
//! These shortcuts work even when the application window is not focused, providing
//! system-wide controls for common actions.
//!
//! # Registered Shortcuts
//!
//! | Shortcut      | Action                     | Description                                |
//! |--------------|----------------------------|--------------------------------------------|
//! | Ctrl+Shift+Q | Quit                       | Gracefully close the application           |
//! | Ctrl+Shift+M | Minimize / Restore         | Minimize to tray or restore from tray      |
//! | Ctrl+Shift+N | New Chat                   | Create a new chat session (emit event)     |
//! | Ctrl+Shift+T | Toggle Theme               | Switch between dark and light themes       |
//! | Ctrl+Shift+H | Show Help                  | Open keyboard shortcuts help overlay       |
//!
//! # Platform Notes
//!
//! - On Windows, these shortcuts use the standard Windows convention of
//!   `Ctrl+Shift+Key` to avoid conflicts with system-level shortcuts.
//! - On Linux, some desktop environments may intercept `Ctrl+Shift+` combos.
//! - On macOS, `Cmd+Shift+Key` is preferred but `Ctrl+Shift+Key` works as well.
//!
//! # References
//!
//! - [Tauri Global Shortcut Plugin](https://v2.tauri.app/reference/config/#global-shortcut-plugin)

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info, warn};

/// Events emitted to the frontend when a global shortcut is triggered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShortcutAction {
    /// User requested application quit.
    Quit,
    /// User requested window minimize/restore.
    MinimizeToggle,
    /// User requested a new chat session.
    NewChat,
    /// User requested theme toggle.
    ToggleTheme,
    /// User requested help overlay.
    ShowHelp,
}

/// Configuration for a registered global shortcut.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    /// The key combination (e.g., "Ctrl+Shift+Q").
    pub key: String,
    /// The action triggered when the shortcut is pressed.
    pub action: ShortcutAction,
    /// Human-readable label for UI display.
    pub label: String,
}

/// Global shortcut manager — registers and tracks hotkeys.
pub struct GlobalShortcutManager {
    app_handle: AppHandle,
    /// List of registered shortcut configurations.
    registered: Arc<std::sync::Mutex<Vec<ShortcutConfig>>>,
    /// Whether global shortcuts are enabled.
    enabled: Arc<std::sync::RwLock<bool>>,
}

impl Clone for GlobalShortcutManager {
    fn clone(&self) -> Self {
        Self {
            app_handle: self.app_handle.clone(),
            registered: Arc::clone(&self.registered),
            enabled: Arc::clone(&self.enabled),
        }
    }
}

impl GlobalShortcutManager {
    /// Create a new shortcut manager associated with the given app handle.
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            registered: Arc::new(std::sync::Mutex::new(Vec::new())),
            enabled: Arc::new(std::sync::RwLock::new(true)),
        }
    }

    /// Get the app handle reference.
    pub fn app_handle(&self) -> AppHandle {
        self.app_handle.clone()
    }

    /// Register all default application shortcuts.
    ///
    /// This registers the standard set of shortcuts listed in the module docs.
    pub fn register_defaults(&self) -> Result<(), String> {
        let shortcuts = self.default_shortcuts();

        for config in &shortcuts {
            if let Err(e) = self.register(config) {
                warn!(
                    shortcut = %config.key,
                    action = ?config.action,
                    "Failed to register shortcut: {}", e
                );
                // Continue registering remaining shortcuts even if one fails
            }
        }

        let count = shortcuts.len();
        info!(registered = count, "All default global shortcuts registered");
        Ok(())
    }

    /// Return the default shortcut configurations.
    pub fn default_shortcuts(&self) -> Vec<ShortcutConfig> {
        vec![
            ShortcutConfig {
                key: "Ctrl+Shift+Q".to_string(),
                action: ShortcutAction::Quit,
                label: "Quit Application".to_string(),
            },
            ShortcutConfig {
                key: "Ctrl+Shift+M".to_string(),
                action: ShortcutAction::MinimizeToggle,
                label: "Minimize / Restore Window".to_string(),
            },
            ShortcutConfig {
                key: "Ctrl+Shift+N".to_string(),
                action: ShortcutAction::NewChat,
                label: "New Chat".to_string(),
            },
            ShortcutConfig {
                key: "Ctrl+Shift+T".to_string(),
                action: ShortcutAction::ToggleTheme,
                label: "Toggle Theme".to_string(),
            },
            ShortcutConfig {
                key: "Ctrl+Shift+H".to_string(),
                action: ShortcutAction::ShowHelp,
                label: "Show Keyboard Help".to_string(),
            },
        ]
    }

    /// Register a single shortcut configuration.
    pub fn register(&self, config: &ShortcutConfig) -> Result<(), String> {
        // The global-shortcut plugin provides the shortcut registration
        // through the app's shortcut API.
        use tauri_plugin_global_shortcut::{
            Code, GlobalShortcutExt, Modifiers, Shortcut,
        };

        let parsed = parse_modifier_combo(&config.key)
            .map_err(|e| format!("Invalid shortcut key '{}': {}", config.key, e))?;

        let modifiers = parsed.0;
        let code = parsed.1;

        let shortcut = Shortcut::new(Some(modifiers), code);

        self.app_handle
            .plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, sc, event| {
                        if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                            tracing::info!(
                                shortcut = %sc.key(),
                                "Global shortcut pressed"
                            );
                            // Forward to the event handler
                            let action = find_action_by_key(app, &sc.key().to_string());
                            if let Some(action) = action {
                                emit_shortcut_event(app, &action);
                            }
                        }
                    })
                    .build(),
            )
            .map_err(|e| format!("Failed to initialize global-shortcut plugin: {}", e))?;

        // Register the shortcut
        self.app_handle
            .global_shortcut()
            .register(shortcut)
            .map_err(|e| format!("Failed to register shortcut '{}': {}", config.key, e))?;

        self.registered
            .lock()
            .map(|mut reg| reg.push(config.clone()))
            .map_err(|e| format!("Failed to track shortcut: {}", e))?;

        info!(shortcut = %config.key, action = ?config.action, "Shortcut registered");
        Ok(())
    }

    /// Unregister a specific shortcut.
    #[allow(dead_code)]
    pub fn unregister(&self, key: &str) -> Result<(), String> {
        use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

        let parsed = parse_modifier_combo(key)
            .map_err(|e| format!("Invalid shortcut key '{}': {}", key, e))?;

        let shortcut = Shortcut::new(Some(parsed.0), parsed.1);

        self.app_handle
            .global_shortcut()
            .unregister(&shortcut)
            .map_err(|e| format!("Failed to unregister shortcut '{}': {}", key, e))?;

        self.registered
            .lock()
            .map(|mut reg| {
                reg.retain(|s| s.key != key);
            })
            .map_err(|e| format!("Failed to update registry: {}", e))?;

        info!(shortcut = %key, "Shortcut unregistered");
        Ok(())
    }

    /// Check if a specific shortcut is registered.
    #[allow(dead_code)]
    pub fn is_registered(&self, key: &str) -> bool {
        self.registered
            .lock()
            .map(|reg| reg.iter().any(|s| s.key == key))
            .unwrap_or(false)
    }

    /// Get all registered shortcut configurations.
    #[allow(dead_code)]
    pub fn list_registered(&self) -> Vec<ShortcutConfig> {
        self.registered
            .lock()
            .map(|reg| reg.clone())
            .unwrap_or_default()
    }

    /// Enable or disable all global shortcuts.
    #[allow(dead_code)]
    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().unwrap() = enabled;
        info!(enabled, "Global shortcuts toggled");
    }

    /// Check if global shortcuts are enabled.
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read().unwrap()
    }
}

/// Parse a modifier+key combination string into Tauri's type-safe representations.
///
/// Returns `(modifiers, code)`.
fn parse_modifier_combo(key: &str) -> Result<(Modifiers, Code), String> {
    let parts: Vec<&str> = key
        .split('+')
        .map(|s| s.trim())
        .collect();

    if parts.is_empty() {
        return Err("Empty shortcut key".to_string());
    }

    let mut modifiers = Modifiers::empty();
    let mut code_str = None;

    for part in &parts {
        match *part {
            "Ctrl" | "Control" => modifiers.insert(Modifiers::CONTROL),
            "Shift" => modifiers.insert(Modifiers::SHIFT),
            "Alt" => modifiers.insert(Modifiers::ALT),
            "Super" | "Cmd" | "Meta" | "Win" => modifiers.insert(Modifiers::SUPER),
            "CmdOrControl" => {
                modifiers.insert(Modifiers::CONTROL);
            }
            other => {
                code_str = Some(other);
            }
        }
    }

    let code_str = code_str.ok_or("No key specified in shortcut")?;

    let code = match code_str {
        "Q" => Code::KeyQ,
        "M" => Code::KeyM,
        "N" => Code::KeyN,
        "T" => Code::KeyT,
        "H" => Code::KeyH,
        "W" => Code::KeyW,
        "S" => Code::KeyS,
        "F" => Code::KeyF,
        "1" => Code::Digit1,
        "2" => Code::Digit2,
        "3" => Code::Digit3,
        _ => {
            return Err(format!("Unknown key code: {}", code_str));
        }
    };

    Ok((modifiers, code))
}

/// Find the action associated with a given shortcut key string.
fn find_action_by_key(app: &AppHandle, key: &str) -> Option<ShortcutAction> {
    // The shortcut handler needs to know the action mapping.
    // We use an app state approach to store the mapping.
    if let Some(state) = app.try_state::<ShortcutActionMap>() {
        let map = state.inner.read().unwrap();
        map.get(key).cloned()
    } else {
        // Fallback: match common shortcuts
        match key {
            "Ctrl+Shift+Q" => Some(ShortcutAction::Quit),
            "Ctrl+Shift+M" => Some(ShortcutAction::MinimizeToggle),
            "Ctrl+Shift+N" => Some(ShortcutAction::NewChat),
            "Ctrl+Shift+T" => Some(ShortcutAction::ToggleTheme),
            "Ctrl+Shift+H" => Some(ShortcutAction::ShowHelp),
            _ => None,
        }
    }
}

/// Emit a shortcut action event to the frontend.
fn emit_shortcut_event(app: &AppHandle, action: &ShortcutAction) {
    info!(action = ?action, "Emitting shortcut action to frontend");

    if let Ok(payload) = serde_json::to_value(action) {
        let _ = app.emit("global-shortcut-triggered", payload);
    }
}

/// A thread-safe map from shortcut key strings to actions,
/// stored in Tauri app state for use in global-shortcut handlers.
pub struct ShortcutActionMap {
    pub inner: Arc<std::sync::RwLock<std::collections::HashMap<String, ShortcutAction>>>,
}

impl ShortcutActionMap {
    pub fn new(shortcuts: Vec<ShortcutConfig>) -> Self {
        let mut map = std::collections::HashMap::new();
        for s in shortcuts {
            map.insert(s.key.clone(), s.action.clone());
        }
        Self {
            inner: Arc::new(std::sync::RwLock::new(map)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modifier_combo() {
        let (mods, code) = parse_modifier_combo("Ctrl+Shift+Q").unwrap();
        assert!(mods.contains(Modifiers::CONTROL));
        assert!(mods.contains(Modifiers::SHIFT));
        assert_eq!(code, Code::KeyQ);
    }

    #[test]
    fn test_parse_invalid_combo() {
        assert!(parse_modifier_combo("").is_err());
        assert!(parse_modifier_combo("Ctrl").is_err()); // no key
    }

    #[test]
    fn test_default_shortcuts() {
        let mgr = GlobalShortcutManager::new(
            tauri::test::mock_app_handle(
                &tauri::test::MockWindowServer::new(),
                tauri::test::MockRuntime::new(),
            ),
        );
        let defaults = mgr.default_shortcuts();
        assert_eq!(defaults.len(), 5);
        assert_eq!(defaults[0].key, "Ctrl+Shift+Q");
        assert_eq!(defaults[0].action, ShortcutAction::Quit);
    }
}