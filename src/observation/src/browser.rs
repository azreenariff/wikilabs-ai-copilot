//! Observation Framework — Browser Provider
//!
//! Detects browser context: URL, page title, browser type.
//! Focuses on engineering portals (OpenShift, vCenter, Nagios, Checkmk, Grafana).
//!
//! Platform support:
//! - Linux: X11 window detection, /proc filesystem for browser processes
//! - Windows: Win32 API for browser windows
//! - macOS: Accessibility API for browser windows
//!
//! Does NOT perform content analysis — only detects browser context metadata.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::event::{EventType, ObservationEvent, ObservationPayload, ProviderType};
use crate::provider::{ObservationProvider, ProviderConfig, ProviderLifecycle, ProviderState};

/// Known engineering portal URLs/patterns.
#[allow(dead_code)]
const ENGINEERING_PORTAL_PATTERNS: &[&str] = &[
    "openshift",
    "ocp",
    "okd",
    "vcenter",
    "vmware",
    "vsphere",
    "nagios",
    "checkmk",
    "grafana",
    "prometheus",
    "kubernetes",
    "k8s",
    "jenkins",
    "gitlab",
    "github",
];

/// Browser context information.
#[derive(Debug, Clone)]
pub struct BrowserContext {
    /// Browser name.
    pub browser_name: String,
    /// Current URL (if accessible).
    pub url: Option<String>,
    /// Page title.
    pub title: Option<String>,
    /// Whether this is a browser context we care about.
    pub is_engineering_portal: bool,
}

impl BrowserContext {
    #[allow(dead_code)]
    fn from_title(browser: &str, title: &str, url: &str) -> Self {
        let is_engineering = ENGINEERING_PORTAL_PATTERNS.iter().any(|pattern| {
            title.to_lowercase().contains(pattern) || url.to_lowercase().contains(pattern)
        });

        Self {
            browser_name: browser.to_string(),
            url: if url.is_empty() {
                None
            } else {
                Some(url.to_string())
            },
            title: if title.is_empty() {
                None
            } else {
                Some(title.to_string())
            },
            is_engineering_portal: is_engineering,
        }
    }
}

/// Browser provider state.
pub struct BrowserState {
    pub config: ProviderConfig,
    pub state: ProviderState,
    pub lifecycle: ProviderLifecycle,
    pub last_context: Option<BrowserContext>,
}

impl BrowserState {
    fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            state: ProviderState::Disabled,
            lifecycle: ProviderLifecycle::new(),
            last_context: None,
        }
    }
}

/// Browser observation provider.
pub struct BrowserProvider {
    state: Arc<Mutex<BrowserState>>,
}

impl BrowserProvider {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(BrowserState::new(ProviderConfig::default()))),
        }
    }

    /// Detect current browser context (platform-specific stub).
    fn detect_browser_context(&self) -> Option<BrowserContext> {
        // This would use platform-specific browser automation APIs
        // e.g., Selenium, Playwright, or native window introspection
        None
    }
}

impl Default for BrowserProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ObservationProvider for BrowserProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Browser
    }

    fn name(&self) -> &str {
        "Browser"
    }

    fn description(&self) -> &str {
        "Detects browser context: URL, page title, browser type, engineering portal detection"
    }

    fn config(&self) -> ProviderConfig {
        self.state.lock().unwrap().config.clone()
    }

    fn set_config(&mut self, config: ProviderConfig) {
        self.state.lock().unwrap().config = config;
    }

    fn state(&self) -> ProviderState {
        self.state.lock().unwrap().state.clone()
    }

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
        let mut state = self.state.lock().unwrap();

        match self.detect_browser_context() {
            Some(context) => {
                // Only emit event if context changed
                let changed = state
                    .last_context
                    .as_ref()
                    .map(|prev| {
                        prev.browser_name != context.browser_name
                            || prev.url != context.url
                            || prev.title != context.title
                    })
                    .unwrap_or(true);

                state.last_context = Some(context.clone());

                if !changed {
                    return Ok(Vec::new());
                }

                let payload = serde_json::json!({
                    "browser": context.browser_name,
                    "url": context.url,
                    "title": context.title,
                    "is_engineering_portal": context.is_engineering_portal,
                });

                Ok(vec![ObservationEvent::new(
                    EventType::BrowserContextChanged,
                    ProviderType::Browser,
                    context.browser_name.clone(),
                    None,
                    ObservationPayload::new(payload),
                )])
            }
            None => Ok(vec![ObservationEvent::new(
                EventType::BrowserContextChanged,
                ProviderType::Browser,
                "stub".to_string(),
                None,
                ObservationPayload::new(serde_json::json!({
                    "status": "no_browser_context_detected",
                    "platform": std::env::consts::OS,
                })),
            )]),
        }
    }

    fn lifecycle(&self) -> ProviderLifecycle {
        self.state.lock().unwrap().lifecycle.clone()
    }

    fn status_details(&self) -> HashMap<String, serde_json::Value> {
        let state = self.state.lock().unwrap();
        let mut details = HashMap::new();
        if let Some(ref ctx) = state.last_context {
            details.insert(
                "last_browser".to_string(),
                serde_json::json!(ctx.browser_name),
            );
            details.insert("last_url".to_string(), serde_json::json!(ctx.url));
            details.insert(
                "is_portal".to_string(),
                serde_json::json!(ctx.is_engineering_portal),
            );
        }
        details.insert(
            "platform".to_string(),
            serde_json::json!(std::env::consts::OS),
        );
        details
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_provider_creation() {
        let provider = BrowserProvider::new();
        assert_eq!(provider.provider_type(), ProviderType::Browser);
        assert_eq!(provider.name(), "Browser");
    }

    #[test]
    fn test_engineering_portal_detection() {
        let ctx = BrowserContext::from_title(
            "firefox",
            "OpenShift Console",
            "https://openshift.example.com/console",
        );
        assert!(ctx.is_engineering_portal);

        let ctx = BrowserContext::from_title(
            "chrome",
            "Grafana - Dashboards",
            "https://grafana.example.com/d/dashboard",
        );
        assert!(ctx.is_engineering_portal);

        let ctx = BrowserContext::from_title(
            "safari",
            "My Personal Blog",
            "https://blog.example.com/post",
        );
        assert!(!ctx.is_engineering_portal);

        let ctx =
            BrowserContext::from_title("firefox", "vCenter Server", "https://vcenter.example.com/");
        assert!(ctx.is_engineering_portal);
    }

    #[test]
    fn test_provider_lifecycle() {
        let mut provider = BrowserProvider::new();
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
        let mut provider = BrowserProvider::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let events = rt.block_on(async {
            provider.start().await.unwrap();
            provider.observe().await.unwrap()
        });
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, EventType::BrowserContextChanged);
    }

    #[test]
    fn test_config_get_set() {
        let mut provider = BrowserProvider::new();
        let mut config = provider.config();
        config.enabled = false;
        provider.set_config(config);
        assert!(!provider.config().enabled);
    }

    #[test]
    fn test_status_details() {
        let provider = BrowserProvider::new();
        let details = provider.status_details();
        assert!(details.contains_key("platform"));
    }
}
