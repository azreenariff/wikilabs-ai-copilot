# MCP Context Integration

**Phase 10** — Wiki Labs AI Copilot

---

## Overview

MCP (Model Context Protocol) integration allows the AI Copilot to connect to external systems — OpenShift, VMware, Nagios, etc. — as read-only context providers. MCP servers expose structured tools that return data in JSON format, which the AI Copilot uses to enrich its understanding of the engineer's environment.

## Design Principles

1. **Read-only** — MCP servers are used for information retrieval only.
2. **Tool-based** — MCP servers expose named tools (e.g., `get_pods()`, `get_events()`).
3. **Typed responses** — MCP servers return structured JSON, not raw strings.
4. **Pluggable** — Any MCP-compliant server can be integrated.

## Architecture

```
┌──────────────────────────────────────────────────┐
│                    AI Copilot                     │
│                                                   │
│  ┌─────────────────────────────────────────┐     │
│  │  Context Provider Framework             │     │
│  │                                         │     │
│  │  ┌───────────────────────────────────┐  │     │
│  │  │  MCPContextProvider               │  │     │
│  │  │                                   │  │     │
│  │  │  • name: "OpenShift"              │  │     │
│  │  │  • server_url: http://localhost   │  │     │
│  │  │  • tools: [get_pods, get_events]  │  │     │
│  │  │  • capabilities: [...]            │  │     │
│  │  └───────────────────────────────────┘  │     │
│  └─────────────────────────────────────────┘     │
│                       ↓                          │
│              Decision Engine                     │
└──────────────────────────────────────────────────┘
                       ↓
┌──────────────────────────────────────────────────┐
│              MCP Server                           │
│                                                   │
│  Tool: get_pods(namespace: str) → JSON array      │
│  Tool: get_events(namespace: str) → JSON array    │
│  Tool: get_nodes() → JSON object                  │
│  Tool: get_cluster_status() → JSON object         │
└──────────────────────────────────────────────────┘
```

## MCP Tool Examples

### OpenShift MCP Server

| Tool | Parameters | Returns |
|------|-----------|---------|
| `get_cluster_status()` | none | Cluster health, node count, pod count |
| `get_nodes()` | none | Node names, roles, status, resources |
| `get_pods(namespace)` | namespace (optional) | Pod names, status, restart count, age |
| `get_events(namespace)` | namespace (optional) | Event timestamps, types, reasons, messages |
| `get_logs(pod, namespace)` | pod name, namespace | Container log output (last N lines) |

### VMware MCP Server

| Tool | Parameters | Returns |
|------|-----------|---------|
| `get_vm_status(vm_name)` | vm_name | VM power state, CPU, memory, storage |
| `get_host_health()` | none | Host status, resource utilization |
| `get_datastore_status()` | none | Datastore names, capacity, free space |

### Nagios MCP Server

| Tool | Parameters | Returns |
|------|-----------|---------|
| `get_alerts()` | none | Current alert count, severity distribution |
| `get_services(host)` | host | Service status (OK, WARNING, CRITICAL) |
| `get_host_status(host)` | host | Host status, last check, downtime |

## Integration Flow

```
1. Engineer opens OpenShift dashboard
   ↓
2. Desktop Application detects: "OpenShift" technology
   ↓
3. Decision Engine evaluates: "OpenShift investigation likely"
   ↓
4. AI Copilot queries MCP server: get_cluster_status()
   ↓
5. MCP server returns: {"status": "warning", "nodes_down": 1}
   ↓
6. Decision Engine updates confidence to 0.92
   ↓
7. AI Copilot generates recommendation: "Check pod events on affected nodes"
```

## Configuration

```toml
[context.providers.openshift]
type = "mcp"
enabled = true
url = "http://localhost:3100"
tools = ["get_cluster_status", "get_nodes", "get_pods", "get_events", "get_logs"]
timeout_seconds = 10

[context.providers.vmware]
type = "mcp"
enabled = false
url = "http://localhost:3101"
tools = ["get_vm_status", "get_host_health", "get_datastore_status"]
timeout_seconds = 15
```

## Validation Checklist

- ✅ MCP context provider implements generic ContextProvider trait
- ✅ All MCP tools are read-only
- ✅ Tool names map to MCP server tool definitions
- ✅ Timeout and retry configuration available
- ✅ Provider availability checking implemented
- ✅ Error handling for unreachable MCP servers