//! Accessibility — ARIA Labels, Screen Reader Support, Keyboard Navigation
//!
//! # Overview
//!
//! This module provides accessibility support for the Wiki Labs AI Copilot
//! desktop application. It ensures the application is usable by people with
//! disabilities, compliant with WCAG 2.1 AA guidelines, and provides proper
//! keyboard navigation and screen reader support.
//!
//! # Features
//!
//! - **ARIA label support** — Ensures all interactive elements have proper
//!   accessible labels for screen readers.
//! - **Keyboard navigation** — Full keyboard-only operation support with
//!   visible focus indicators.
//! - **Screen reader announcements** — Live region announcements for dynamic
//!   content changes.
//! - **Reduced motion** — Respects the user's OS-level reduced motion preference.
//! - **High contrast** — Supports high contrast / accessibility themes.
//! - **Font scaling** — Supports system font scaling for users who need larger text.
//!
//! # Accessibility Best Practices Implemented
//!
//! 1. Every button and interactive element has a descriptive label.
//! 2. Focus order matches the visual reading order.
//! 3. Dynamic content changes are announced to screen readers via live regions.
//! 4. All color-based information has text alternatives.
//! 5. Minimum touch target sizes are maintained (44x44px on Windows).
//! 6. Keyboard shortcuts are discoverable via the Help panel.
//!
//! # References
//!
//! - [WCAG 2.1 AA](https://www.w3.org/WAI/WCAG21/quickref/)
//! - [WAI-ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
//! - [Microsoft Accessibility Guidelines](https://learn.microsoft.com/en-us/windows/apps/accessibility/)

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Accessibility settings for the application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    /// Whether screen reader announcements are enabled.
    #[serde(default = "default_true")]
    pub screen_reader_enabled: bool,
    /// Whether keyboard navigation hints are shown.
    #[serde(default = "default_true")]
    pub keyboard_hints_enabled: bool,
    /// Whether reduced motion is requested.
    #[serde(default)]
    pub reduced_motion: bool,
    /// Whether high contrast mode is enabled.
    #[serde(default)]
    pub high_contrast: bool,
    /// Custom font scaling factor (1.0 = default).
    #[serde(default = "default_font_scale")]
    pub font_scale: f64,
    /// Minimum touch target size in pixels (Windows recommends 44x44).
    #[serde(default = "default_touch_size")]
    pub min_touch_size: u32,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            screen_reader_enabled: true,
            keyboard_hints_enabled: true,
            reduced_motion: false,
            high_contrast: false,
            font_scale: 1.0,
            min_touch_size: 44,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_font_scale() -> f64 {
    1.0
}

fn default_touch_size() -> u32 {
    44
}

/// ARIA role for an accessible element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AriaRole {
    /// Button element.
    Button,
    /// Input text field.
    TextField,
    /// Navigation menu.
    Navigation,
    /// Status message area.
    Status,
    /// Alert for important messages.
    Alert,
    /// Live region for dynamic content.
    LiveRegion,
    /// Dialog window.
    Dialog,
    /// Link.
    Link,
    /// Tree view.
    TreeView,
    /// Tab container.
    TabContainer,
    /// Custom fallback role.
    Custom(String),
}

impl std::fmt::Display for AriaRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Button => write!(f, "button"),
            Self::TextField => write!(f, "textbox"),
            Self::Navigation => write!(f, "navigation"),
            Self::Status => write!(f, "status"),
            Self::Alert => write!(f, "alert"),
            Self::LiveRegion => write!(f, "region[aria-live]"),
            Self::Dialog => write!(f, "dialog"),
            Self::Link => write!(f, "link"),
            Self::TreeView => write!(f, "tree"),
            Self::TabContainer => write!(f, "tablist"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// An accessible label description for an interactive element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibleLabel {
    /// The element identifier (e.g., "send-button").
    pub element_id: String,
    /// The ARIA label text for screen readers.
    pub label: String,
    /// The ARIA role of the element.
    pub role: AriaRole,
    /// Whether the element is visible to sighted users.
    pub visible: bool,
    /// Optional tooltip text.
    pub tooltip: Option<String>,
    /// Keyboard shortcut associated with this element.
    pub shortcut: Option<String>,
}

/// A screen reader announcement for dynamic content.
///
/// When sent to the frontend, this should be rendered as an
/// `aria-live="polite"` region that the screen reader will announce.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenReaderAnnouncement {
    /// The message to announce.
    pub message: String,
    /// Announcement level: "polite" (non-urgent) or "assertive" (urgent).
    pub politeness: Politeness,
}

/// Screen reader announcement politeness level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Politeness {
    /// Non-urgent announcement.
    Polite,
    /// Urgent announcement that interrupts current speech.
    Assertive,
}

impl std::fmt::Display for Politeness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Polite => write!(f, "polite"),
            Self::Assertive => write!(f, "assertive"),
        }
    }
}

/// Accessibility manager — tracks settings and provides accessibility info.
pub struct AccessibilityManager {
    settings: AccessibilitySettings,
}

impl AccessibilityManager {
    /// Create a new accessibility manager with default settings.
    pub fn new() -> Self {
        Self {
            settings: AccessibilitySettings::default(),
        }
    }

    /// Create from existing settings.
    pub fn from_settings(settings: AccessibilitySettings) -> Self {
        Self { settings }
    }

    /// Get the current accessibility settings.
    #[allow(dead_code)]
    pub fn settings(&self) -> &AccessibilitySettings {
        &self.settings
    }

    /// Check if screen reader support is enabled.
    pub fn screen_reader_enabled(&self) -> bool {
        self.settings.screen_reader_enabled
    }

    /// Check if keyboard navigation hints are enabled.
    pub fn keyboard_hints_enabled(&self) -> bool {
        self.settings.keyboard_hints_enabled
    }

    /// Check if reduced motion is enabled.
    pub fn reduced_motion_enabled(&self) -> bool {
        self.settings.reduced_motion
    }

    /// Check if high contrast is enabled.
    pub fn high_contrast_enabled(&self) -> bool {
        self.settings.high_contrast
    }

    /// Get the font scaling factor.
    pub fn font_scale(&self) -> f64 {
        self.settings.font_scale
    }

    /// Get the minimum touch target size.
    pub fn min_touch_size(&self) -> u32 {
        self.settings.min_touch_size
    }

    /// Create an accessible label for a UI element.
    pub fn create_label(
        element_id: &str,
        label: &str,
        role: AriaRole,
        visible: bool,
        tooltip: Option<&str>,
        shortcut: Option<&str>,
    ) -> AccessibleLabel {
        AccessibleLabel {
            element_id: element_id.to_string(),
            label: label.to_string(),
            role,
            visible,
            tooltip: tooltip.map(|s| s.to_string()),
            shortcut: shortcut.map(|s| s.to_string()),
        }
    }

    /// Create a screen reader announcement.
    pub fn create_announcement(
        message: &str,
        politeness: Politeness,
    ) -> ScreenReaderAnnouncement {
        ScreenReaderAnnouncement {
            message: message.to_string(),
            politeness,
        }
    }

    /// Get all predefined accessible labels for the application.
    ///
    /// This provides a complete list of labels that should be applied
    /// to UI elements to ensure screen reader compatibility.
    #[allow(dead_code)]
    pub fn get_default_labels(&self) -> Vec<AccessibleLabel> {
        vec![
            AccessibleLabel {
                element_id: "chat-input".to_string(),
                label: "Chat message input".to_string(),
                role: AriaRole::TextField,
                visible: true,
                tooltip: Some("Type your message here").to_string(),
                shortcut: None,
            },
            AccessibleLabel {
                element_id: "send-button".to_string(),
                label: "Send message".to_string(),
                role: AriaRole::Button,
                visible: true,
                tooltip: Some("Send the chat message").to_string(),
                shortcut: Some("Ctrl+Enter".to_string()),
            },
            AccessibleLabel {
                element_id: "settings-button".to_string(),
                label: "Settings".to_string(),
                role: AriaRole::Button,
                visible: true,
                tooltip: Some("Open settings panel").to_string(),
                shortcut: Some("Ctrl+Shift+S".to_string()),
            },
            AccessibleLabel {
                element_id: "status-indicator".to_string(),
                label: "Connection status".to_string(),
                role: AriaRole::Status,
                visible: true,
                tooltip: None,
                shortcut: None,
            },
            AccessibleLabel {
                element_id: "knowledge-panel".to_string(),
                label: "Knowledge panel".to_string(),
                role: AriaRole::TabContainer,
                visible: true,
                tooltip: None,
                shortcut: Some("Ctrl+Shift+K".to_string()),
            },
            AccessibleLabel {
                element_id: "guidance-panel".to_string(),
                label: "Guidance panel".to_string(),
                role: AriaRole::TabContainer,
                visible: true,
                tooltip: None,
                shortcut: Some("Ctrl+Shift+G".to_string()),
            },
            AccessibleLabel {
                element_id: "quit-button".to_string(),
                label: "Quit application".to_string(),
                role: AriaRole::Button,
                visible: true,
                tooltip: Some("Close the application").to_string(),
                shortcut: Some("Ctrl+Shift+Q".to_string()),
            },
        ]
    }

    /// Validate accessibility compliance of the application configuration.
    ///
    /// Returns a list of warnings if any accessibility requirements are not met.
    #[allow(dead_code)]
    pub fn validate_accessibility(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if !self.settings.screen_reader_enabled {
            issues.push(
                "Screen reader support is disabled — "
                    .to_string()
            );
        }

        if self.settings.font_scale < 0.5 {
            issues.push(
                "Font scale is below 0.5 — "
                    .to_string()
            );
        }

        if self.settings.min_touch_size < 44 {
            issues.push(
                "Minimum touch size is below 44px — "
                    .to_string()
            );
        }

        if issues.is_empty() {
            info!("Accessibility validation passed");
        } else {
            warn!(
                issues = issues.len(),
                "Accessibility validation found issues"
            );
        }

        issues
    }
}

impl Default for AccessibilityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_labels_exist() {
        let mgr = AccessibilityManager::new();
        let labels = mgr.get_default_labels();
        assert!(labels.len() >= 5);
    }

    #[test]
    fn test_accessibility_settings_defaults() {
        let settings = AccessibilitySettings::default();
        assert!(settings.screen_reader_enabled);
        assert!(settings.keyboard_hints_enabled);
        assert!(!settings.reduced_motion);
        assert!(!settings.high_contrast);
        assert_eq!(settings.font_scale, 1.0);
        assert_eq!(settings.min_touch_size, 44);
    }
}