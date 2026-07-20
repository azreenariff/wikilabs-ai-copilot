/// Features 4 & 5 — Read-Only Context Provider + MCP Integration Foundation
///
/// Feature 4: Generic framework for read-only context providers.
/// Providers ONLY collect information. They do not modify anything.
///
/// Feature 5: MCP integration as a context provider mechanism.
/// MCP servers are used for information retrieval and context enrichment only.
///
/// Examples:
/// - OpenShift MCP: get_cluster_status(), get_nodes(), get_pods(), get_events()
/// - VMware MCP: get_vm_status(), get_host_health()
/// - Nagios MCP: get_alerts(), get_services()

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A read-only context value retrieved from a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextValue {
    /// Key identifying the context data.
    pub key: String,
    /// Human-readable label.
    pub label: String,
    /// The actual data value.
    pub value: String,
    /// Confidence in this data.
    pub confidence: f64,
    /// Timestamp when data was retrieved.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// A collection of context values from a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// Provider that generated this snapshot.
    pub provider: String,
    /// Context values retrieved.
    pub values: Vec<ContextValue>,
    /// Whether the snapshot was fresh or cached.
    pub is_fresh: bool,
}

impl ContextSnapshot {
    /// Create a new context snapshot.
    pub fn new(provider: &str) -> Self {
        Self {
            provider: provider.to_string(),
            values: Vec::new(),
            is_fresh: true,
        }
    }

    /// Add a context value.
    pub fn add_value(&mut self, key: &str, label: &str, value: &str, confidence: f64) {
        self.values.push(ContextValue {
            key: key.to_string(),
            label: label.to_string(),
            value: value.to_string(),
            confidence,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Get a value by key.
    pub fn get_value(&self, key: &str) -> Option<&ContextValue> {
        self.values.iter().find(|v| v.key == key)
    }

    /// Get the number of values.
    pub fn value_count(&self) -> usize {
        self.values.len()
    }

    /// Format as a summary string.
    pub fn summary(&self) -> String {
        let mut output = format!("Context from '{}':\n", self.provider);
        for val in &self.values {
            output.push_str(&format!("  • {}: {} (confidence: {:.0}%)\n", val.label, val.value, val.confidence * 100.0));
        }
        output
    }
}

/// Trait for read-only context providers.
///
/// Any provider implementing this trait may only read information.
/// Write operations are strictly forbidden.
#[async_trait]
pub trait ReadOnlyContextProvider: Send + Sync {
    /// Get the provider name.
    fn name(&self) -> &str;

    /// Get the technology domain this provider serves.
    fn technology_domain(&self) -> &str;

    /// Retrieve context values from the provider.
    async fn retrieve_context(&self) -> Result<ContextSnapshot, String>;

    /// Check if the provider is available and connected.
    async fn is_available(&self) -> bool;

    /// Get the last time context was retrieved.
    fn last_retrieved(&self) -> Option<chrono::DateTime<chrono::Utc>>;
}

/// MCP context provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpProviderConfig {
    /// MCP server name.
    pub name: String,
    /// MCP server transport type.
    pub transport: McpTransportType,
    /// Server address or path.
    pub address: Option<String>,
    /// Whether this provider is enabled.
    pub enabled: bool,
}

/// MCP transport type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum McpTransportType {
    /// Standard streams (stdio).
    Stdio,
    /// Server-Sent Events (SSE).
    Sse,
}

/// An MCP context provider that wraps MCP servers for read-only access.
pub struct McpContextProvider {
    config: McpProviderConfig,
    last_retrieved: Option<chrono::DateTime<chrono::Utc>>,
}

impl McpProviderConfig {
    /// Create a new MCP provider config.
    pub fn new(name: &str, transport: McpTransportType) -> Self {
        Self {
            name: name.to_string(),
            transport,
            address: None,
            enabled: true,
        }
    }

    /// Set the server address.
    pub fn with_address(mut self, address: &str) -> Self {
        self.address = Some(address.to_string());
        self
    }

    /// Disable this provider.
    pub fn with_disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

impl McpContextProvider {
    /// Create a new MCP context provider.
    pub fn new(config: McpProviderConfig) -> Self {
        Self {
            config,
            last_retrieved: None,
        }
    }

    /// Simulate retrieving context from an MCP server.
    /// In production, this would call the MCP server via stdio/SSE.
    pub async fn mock_retrieve(&self, domain: &str) -> ContextSnapshot {
        let mut snapshot = ContextSnapshot::new(&self.config.name);

        match domain.to_lowercase().as_str() {
            "openshift" | "kubernetes" => {
                snapshot.add_value("cluster_status", "Cluster Status", "Healthy", 0.95);
                snapshot.add_value("node_count", "Node Count", "3", 0.95);
                snapshot.add_value("pod_count", "Active Pods", "42", 0.90);
                snapshot.add_value("event_count", "Recent Events", "5 warnings, 2 errors", 0.85);
            }
            "linux" | "system" => {
                snapshot.add_value("hostname", "Hostname", "prod-worker-01", 0.99);
                snapshot.add_value("cpu_usage", "CPU Usage", "67%", 0.95);
                snapshot.add_value("memory_usage", "Memory Usage", "78%", 0.95);
                snapshot.add_value("disk_usage", "Disk Usage", "82%", 0.90);
                snapshot.add_value("uptime", "Uptime", "45 days", 0.99);
            }
            "vmware" | "vsphere" => {
                snapshot.add_value("host_status", "Host Status", "Connected", 0.95);
                snapshot.add_value("vm_count", "VM Count", "24", 0.95);
                snapshot.add_value("esxi_version", "ESXi Version", "8.0 U3", 0.99);
                snapshot.add_value("datastore_free", "Datastore Free", "1.2 TB", 0.90);
            }
            "nagios" => {
                snapshot.add_value("alerts", "Active Alerts", "3 warnings, 1 critical", 0.90);
                snapshot.add_value("services", "Total Services", "156", 0.95);
                snapshot.add_value("hosts", "Total Hosts", "42", 0.95);
                snapshot.add_value("uptime_percent", "System Uptime", "99.7%", 0.99);
            }
            "checkmk" => {
                snapshot.add_value("site_status", "Site Status", "Online", 0.95);
                snapshot.add_value("check_results", "Check Results", "12 critical, 5 warnings", 0.90);
                snapshot.add_value("hosts", "Hosts Monitored", "85", 0.95);
            }
            _ => {
                snapshot.add_value("status", "Provider Status", "Available", 0.99);
            }
        }

        snapshot
    }
}

#[async_trait]
impl ReadOnlyContextProvider for McpContextProvider {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn technology_domain(&self) -> &str {
        &self.config.name
    }

    async fn retrieve_context(&self) -> Result<ContextSnapshot, String> {
        if !self.config.enabled {
            return Err(format!("MCP provider '{}' is disabled", self.config.name));
        }

        match &self.config.transport {
            McpTransportType::Stdio => {
                Ok(self.mock_retrieve(&self.config.name).await)
            }
            McpTransportType::Sse => {
                Ok(self.mock_retrieve(&self.config.name).await)
            }
        }
    }

    async fn is_available(&self) -> bool {
        self.config.enabled
    }

    fn last_retrieved(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.last_retrieved
    }
}

/// The read-only context provider registry.
///
/// Manages all registered providers and provides unified access to context data.
pub struct ReadOnlyContextRegistry {
    providers: HashMap<String, Box<dyn ReadOnlyContextProvider>>,
}

impl Default for ReadOnlyContextRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadOnlyContextRegistry {
    /// Create a new context registry.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a read-only context provider.
    pub fn register<T: ReadOnlyContextProvider + 'static>(&mut self, provider: T) {
        self.providers.insert(provider.name().to_string(), Box::new(provider));
    }

    /// Get a provider by name.
    pub fn get(&self, name: &str) -> Option<&dyn ReadOnlyContextProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    /// Get all registered providers.
    pub fn all(&self) -> Vec<&dyn ReadOnlyContextProvider> {
        self.providers
            .values()
            .map(|p| p.as_ref())
            .collect()
    }

    /// Check if a provider is registered.
    pub fn has(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// Retrieve context from all available providers.
    pub async fn retrieve_all_context(&self) -> Vec<ContextSnapshot> {
        let mut snapshots = Vec::new();
        for provider in self.providers.values() {
            if provider.is_available().await {
                if let Ok(snapshot) = provider.retrieve_context().await {
                    snapshots.push(snapshot);
                }
            }
        }
        snapshots
    }

    /// Get a unified context summary from all providers.
    pub async fn unified_summary(&self) -> String {
        let mut output = "Unified Context Summary\n".to_string();
        for snapshot in self.retrieve_all_context().await {
            output.push_str(&snapshot.summary());
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_snapshot_creation() {
        let snapshot = ContextSnapshot::new("test-provider");
        assert_eq!(snapshot.provider, "test-provider");
        assert_eq!(snapshot.value_count(), 0);
    }

    #[test]
    fn test_context_snapshot_add_values() {
        let mut snapshot = ContextSnapshot::new("openshift");
        snapshot.add_value("cluster_status", "Cluster Status", "Healthy", 0.95);
        snapshot.add_value("node_count", "Node Count", "3", 0.99);

        assert_eq!(snapshot.value_count(), 2);
        assert!(snapshot.get_value("cluster_status").is_some());
        assert!(snapshot.get_value("node_count").is_some());
        assert!(snapshot.get_value("missing").is_none());
    }

    #[test]
    fn test_context_snapshot_summary() {
        let mut snapshot = ContextSnapshot::new("linux");
        snapshot.add_value("cpu", "CPU Usage", "67%", 0.95);
        snapshot.add_value("mem", "Memory", "78%", 0.90);

        let summary = snapshot.summary();
        assert!(summary.contains("linux"));
        assert!(summary.contains("CPU Usage"));
        assert!(summary.contains("67%"));
        assert!(summary.contains("Memory"));
    }

    #[test]
    fn test_mcp_provider_openshift() {
        let provider = McpContextProvider::new(McpProviderConfig::new("openshift", McpTransportType::Stdio));
        let snapshot = futures::executor::block_on(provider.mock_retrieve("openshift"));

        assert_eq!(snapshot.provider, "openshift");
        assert!(snapshot.value_count() >= 4);
        assert!(snapshot.get_value("cluster_status").is_some());
        assert!(snapshot.get_value("node_count").is_some());
    }

    #[test]
    fn test_mcp_provider_linux() {
        let provider = McpContextProvider::new(McpProviderConfig::new("linux", McpTransportType::Stdio));
        let snapshot = futures::executor::block_on(provider.mock_retrieve("linux"));

        assert_eq!(snapshot.provider, "linux");
        assert!(snapshot.get_value("hostname").is_some());
        assert!(snapshot.get_value("cpu_usage").is_some());
        assert!(snapshot.get_value("disk_usage").is_some());
    }

    #[test]
    fn test_mcp_provider_vmware() {
        let provider = McpContextProvider::new(McpProviderConfig::new("vmware", McpTransportType::Sse));
        let snapshot = futures::executor::block_on(provider.mock_retrieve("vmware"));

        assert!(snapshot.get_value("host_status").is_some());
        assert!(snapshot.get_value("vm_count").is_some());
    }

    #[test]
    fn test_mcp_provider_nagios() {
        let provider = McpContextProvider::new(McpProviderConfig::new("nagios", McpTransportType::Stdio));
        let snapshot = futures::executor::block_on(provider.mock_retrieve("nagios"));

        assert!(snapshot.get_value("alerts").is_some());
        assert!(snapshot.get_value("services").is_some());
    }

    #[test]
    fn test_mcp_provider_disabled() {
        let config = McpProviderConfig::new("disabled-mcp", McpTransportType::Stdio)
            .with_disabled();
        let provider = McpContextProvider::new(config);

        assert!(!futures::executor::block_on(provider.is_available()));
    }

    #[test]
    fn test_context_registry_register() {
        let mut registry = ReadOnlyContextRegistry::new();
        let provider = McpContextProvider::new(McpProviderConfig::new("openshift", McpTransportType::Stdio));
        registry.register(provider);

        assert!(registry.has("openshift"));
        assert!(!registry.has("nonexistent"));
    }

    #[test]
    fn test_context_registry_all_providers() {
        let mut registry = ReadOnlyContextRegistry::new();
        registry.register(McpContextProvider::new(McpProviderConfig::new("linux", McpTransportType::Stdio)));
        registry.register(McpContextProvider::new(McpProviderConfig::new("nagios", McpTransportType::Sse)));

        let all = registry.all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_context_registry_unified_summary() {
        let mut registry = ReadOnlyContextRegistry::new();
        registry.register(McpContextProvider::new(McpProviderConfig::new("openshift", McpTransportType::Stdio)));

        let summary = futures::executor::block_on(registry.unified_summary());
        assert!(summary.contains("Unified Context Summary"));
        assert!(summary.contains("openshift"));
    }

    #[test]
    fn test_context_registry_retrieve_all() {
        let mut registry = ReadOnlyContextRegistry::new();
        registry.register(McpContextProvider::new(McpProviderConfig::new("linux", McpTransportType::Stdio)));

        let snapshots = futures::executor::block_on(registry.retrieve_all_context());
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].provider, "linux");
    }

    #[test]
    fn test_readonly_context_provider_trait() {
        let provider = McpContextProvider::new(McpProviderConfig::new("test", McpTransportType::Stdio));
        assert_eq!(provider.name(), "test");
        assert_eq!(provider.technology_domain(), "test");
    }
}