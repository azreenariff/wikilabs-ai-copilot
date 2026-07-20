# Context Provider Specification

**Phase 10** — Wiki Labs AI Copilot

---

## Overview

The Context Provider Framework provides a generic interface for read-only integrations with external systems. Providers collect information but never modify anything.

## Design Principles

1. **Read-only** — Providers only retrieve data. No writes, no modifications.
2. **Generic** — Any external system can be a provider via the same trait.
3. **Extensible** — New providers are added without modifying existing code.
4. **Pluggable** — Providers can be enabled/disabled via configuration.

## Architecture

```
Decision Engine (requests context)
    ↓
┌─────────────────────────────────┐
│  Context Provider Framework     │
│                                 │
│  • ContextProvider trait        │
│  • MCPContextProvider           │
│  • StaticContextProvider        │
│  • ProviderManager              │
└─────────────────────────────────┘
    ↓
External Systems (OpenShift, VMware, etc.)
```

## Key Types

### ContextProvider Trait

The generic interface all providers implement:

```rust
pub trait ContextProvider: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> &[Capability];
    fn can_provide(&self, query: &ContextQuery) -> bool;
    fn provide(&self, query: &ContextQuery) -> ContextResponse;
    fn is_available(&self) -> bool;
}
```

### Capability

Each provider declares what it can do:

```rust
pub enum Capability {
    GetClusterStatus,
    GetNodes,
    GetPods,
    GetEvents,
    GetLogs,
    GetVMStatus,
    GetHostHealth,
    GetDatastoreStatus,
    GetAlerts,
    GetServices,
    GetSystemInfo,
    GetProcesses,
    GetDiskUsage,
    GetConfig,
}
```

### ContextQuery

```rust
pub struct ContextQuery {
    pub capability: Capability,
    pub namespace: Option<String>,
    pub labels: Vec<(String, String)>,
    pub time_range: Option<TimeRange>,
}
```

### ContextResponse

```rust
pub struct ContextResponse {
    pub data: Vec<u8>,          // Raw response bytes (JSON, text, etc.)
    pub mime_type: String,      // "application/json", "text/plain", etc.
    pub metadata: HashMap<String, String>, // Content-type hints, sources
}
```

## Provider Implementations

### MCPContextProvider

For MCP (Model Context Protocol) servers:

```rust
pub struct MCPContextProvider {
    name: String,
    server_url: String,
    capabilities: Vec<Capability>,
    client: MCPClient,
}

impl ContextProvider for MCPContextProvider {
    fn provide(&self, query: &ContextQuery) -> ContextResponse {
        match query.capability {
            Capability::GetPods => {
                // Calls: mcp_client.call("openshift", "get_pods", { namespace: "..." })
                // Returns JSON array of pod status objects
            }
            Capability::GetEvents => {
                // Calls: mcp_client.call("openshift", "get_events", { namespace: "..." })
                // Returns JSON array of event objects
            }
            // ...
        }
    }
}
```

### StaticContextProvider

For static/local context (fallback when no external system available):

```rust
pub struct StaticContextProvider {
    name: String,
    local_data: HashMap<String, String>,
}
```

## Provider Configuration

Providers are configured via TOML:

```toml
[context.providers.openshift]
type = "mcp"
enabled = true
url = "http://localhost:3100"
capabilities = ["GetNodes", "GetPods", "GetEvents", "GetLogs"]

[context.providers.vmware]
type = "mcp"
enabled = true
url = "http://localhost:3101"
capabilities = ["GetVMStatus", "GetHostHealth", "GetDatastoreStatus"]

[context.providers.nagios]
type = "mcp"
enabled = true
url = "http://localhost:3102"
capabilities = ["GetAlerts", "GetServices"]
```

## Validation Checklist

- ✅ ContextProvider trait defines the generic interface
- ✅ MCPContextProvider implements MCP protocol integration
- ✅ All operations are read-only
- ✅ Provider capability system prevents unsupported queries
- ✅ Availability checking before use
- ✅ Error handling for unavailable providers