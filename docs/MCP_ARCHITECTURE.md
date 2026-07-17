---
description: "MCP architecture for Wiki Labs AI Copilot — skill design, server architecture, protocol, lifecycle, transport, isolation, testing, and distribution."
icon: puzzle-piece
---

# Wiki Labs AI Copilot — MCP Architecture

## MCP Integration Strategy

The Model Context Protocol (MCP) provides a standardized way for the AI copilot to discover and invoke tools, resources, and prompts provided by domain-specific expertise modules ("skills"). MCP is not used for autonomous execution — it is a **tool discovery and invocation protocol** that gives the AI reasoning engine a structured way to access domain expertise.

### Design Philosophy

- **Skills provide knowledge and tools, NOT actions**. The AI reasons using skill-provided information and recommends actions, but the human engineer executes them.
- **Each skill is an independent MCP server** — a self-contained process with its own tool definitions, resource catalog, and prompt templates.
- **The core acts as an MCP client** — it discovers skills, forwards tool calls, and presents results to the AI provider.
- **Skills are pluggable** — new skills can be added without modifying the core application.

### MCP Usage Pattern

```
┌─────────────────────────────────────────────────────────────┐
│                     MCP Flow                                │
│                                                             │
│  1. Skill Discovery:                                        │
│     Core ──→ MCP servers ──→ tools/list, resources/list    │
│     Core collects all tool definitions into a catalog       │
│                                                             │
│  2. AI Reasoning:                                           │
│     AI provider receives the catalog + context              │
│     AI decides which tool to invoke                         │
│                                                             │
│  3. Tool Invocation:                                        │
│     AI ──→ Core ──→ MCP server (tools/call)                │
│     MCP server ──→ Native handler ──→ Result               │
│     Result flows back: MCP server → Core → AI              │
│                                                             │
│  4. Human Confirmation (CRITICAL):                          │
│     Before ANY tool that affects the engineer's             │
│     environment, the copilot presents the action for        │
│     confirmation. No tool is executed without human         │
│     approval.                                               │
│                                                             │
│  5. Advisor Presentation:                                   │
│     Results are formatted as recommendations in the chat    │
│     or real-time suggestion panel.                          │
└─────────────────────────────────────────────────────────────┘
```

## Skill Architecture Design

### Skill Package Structure

Each skill is a self-contained package with the following structure:

```
skills/<skill-id>/
├── SKILL.md                      # Skill manifest (YAML frontmatter + markdown body)
├── metadata.json                 # Structured metadata (version, tools, workflows)
├── mcp-server/                   # MCP server binary
│   └── wikilabs-<skill-id>       # Platform-specific binary
├── tools/                        # Tool definitions
│   ├── tool_name.md              # Tool description, parameters, examples
│   └── ...
├── resources/                    # Static resources
│   ├── reference.md              # Reference documentation
│   ├── checklists.json           # Verification checklists
│   └── ...
├── workflows/                    # Engineering workflows
│   ├── workflow_name.md          # Workflow steps, commands, checklists
│   └── ...
├── knowledge/                    # Bundled knowledge
│   ├── best_practices.md         # Best practices reference
│   └── common_patterns.md        # Common issue patterns
└── tests/                        # Skill tests
    ├── tool_tests.rs             # Unit tests for tool handlers
    └── integration_tests.rs      # Integration tests with MCP client
```

### Skill Manifest (SKILL.md)

```markdown
---
name: Red Hat OpenShift
description: OpenShift cluster management, troubleshooting, and administration
metadata:
  id: openshift
  version: "1.2.0"
  category: infrastructure
  icon: 🔵
  tags:
    - kubernetes
    - openshift
    - pods
    - routes
    - operators
  required_credential_services:
    - openshift_api
  auto_start: false
---

# OpenShift Skill

Provides tools and knowledge for Red Hat OpenShift cluster operations.

## Tools
- `list_pods`: List pods in a namespace
- `describe_pod`: Get detailed pod information
- `check_health`: Check cluster health status
- ...

## Workflows
- `pod_crash_loop`: Troubleshoot CrashLoopBackOff
- `cluster_upgrade`: Upgrade OpenShift cluster
- ...

## Knowledge
- Best practices for OpenShift administration
- Common issues and their resolutions
```

### Structured Metadata (metadata.json)

```json
{
  "id": "openshift",
  "name": "Red Hat OpenShift",
  "version": "1.2.0",
  "description": "OpenShift cluster management and troubleshooting",
  "category": "infrastructure",
  "icon": "🔵",
  "enabled": true,
  "auto_start": false,
  "tools": [
    {
      "name": "list_pods",
      "description": "List pods in a namespace",
      "inputSchema": {
        "type": "object",
        "properties": {
          "namespace": { "type": "string", "description": "Target namespace" },
          "label_selector": { "type": "string", "description": "Label selector filter" }
        },
        "required": ["namespace"]
      }
    },
    ...
  ],
  "resources": [
    {
      "uri": "ocp://best-practices",
      "name": "OpenShift Best Practices",
      "description": "Best practices for OpenShift administration",
      "mimeType": "text/markdown"
    }
  ],
  "workflows": [
    {
      "id": "pod_crash_loop",
      "name": "Troubleshoot CrashLoopBackOff",
      "steps": [...]
    }
  ],
  "required_credential_services": ["openshift_api"],
  "tags": ["kubernetes", "openshift", "pods", "routes"]
}
```

---

## MCP Server Per-Skill Design

### Server Process Model

Each skill runs as an **independent process** managed by the core:

```
┌─────────────────────────────────────────────────────┐
│              Rust Core (in-process)                  │
│                                                     │
│  ┌───────────────────────────────────────────────┐  │
│  │  MCP Skill Manager                            │  │
│  │                                               │  │
│  │  ┌──────────────┐ ┌──────────────┐           │  │
│  │  │  OpenShift   │ │    Linux     │  ...      │  │
│  │  │  MCP Server  │ │  MCP Server  │           │  │
│  │  │  (PID: 1234) │ │  (PID: 1235) │           │  │
│  │  └──────┬───────┘ └──────┬───────┘           │  │
│  │         │                │                     │  │
│  │         └────────┬───────┘                      │  │
│  │                  │ stdio (JSON-RPC)              │  │
│  └──────────────────┼─────────────────────────────┘  │
└─────────────────────┼───────────────────────────────┘
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
┌───────────┐ ┌───────────┐ ┌───────────┐
│   oc      │ │  ps, top  │ │  esxtop   │  ← External tools called by skill handlers
│   kubectl │ │  systemctl│ │  vim-cmd  │
└───────────┘ └───────────┘ └───────────┘
```

### MCP Server Implementation

Each MCP server is a Rust binary that:

1. **Implements the MCP protocol** (JSON-RPC 2.0 over stdio)
2. **Exposes tools** that can interact with the domain technology
3. **Provides resources** (static reference data, checklists)
4. **Provides prompts** (structured troubleshooting workflows)
5. **Authenticates** using credentials from the OS keychain
6. **Validates all inputs** before executing any external commands

### Example: OpenShift MCP Server

```rust
// openshift-mcp-server/src/main.rs (conceptual)

use mcproto::{Server, Request, Response};

#[tokio::main]
async fn main() {
    let server = Server::stdio();
    
    // Register tools
    server.register_tool("list_pods", |args| {
        let namespace = args["namespace"].as_str().unwrap();
        let token = credential_manager::get("openshift_api");
        // Execute: oc get pods -n {namespace}
        // Return formatted output
    }).await;
    
    server.register_tool("describe_pod", |args| {
        // Execute: oc describe pod {name} -n {namespace}
        // Return pod details
    }).await;
    
    server.register_resource("ocp://best-practices", || {
        include_str!("resources/best_practices.md")
    }).await;
    
    server.register_prompt("troubleshoot_crash_loop", |args| {
        // Return structured troubleshooting workflow
        Prompt {
            steps: vec![
                Step { name: "Check pod status", command: "oc get pods -n {namespace}" },
                Step { name: "Check pod logs", command: "oc logs {pod} -n {namespace} --tail=100" },
                // ...
            ]
        }
    }).await;
    
    server.serve().await;
}
```

---

## Communication Protocol Between Core and MCP Servers

### Transport Options

| Transport | Use Case | Pros | Cons |
|-----------|---------|------|------|
| **stdio** (default) | Local skills | Simple, secure (same user), low overhead | No remote access, single process |
| **HTTP** | Remote skills (enterprise) | Can reach on-prem inference servers | Requires TLS, network setup |
| **WebSocket** | Streaming tools | Real-time data feeds | More complex, connection management |

### Protocol (JSON-RPC 2.0 over MCP)

#### Connection Lifecycle

```
MCP Client (Core)              MCP Server (Skill)
     │                               │
     │── Initialize ────────────────►│  { protocolVersion: "2024-11-05" }
     │                               │
     │◄─ Initialized ───────────────│  { serverInfo: { name, version } }
     │                               │
     │── tools/list ────────────────►│
     │◄─ [{ name, description,      │  Tool definitions
     │      inputSchema }]          │
     │                               │
     │── resources/list ───────────►│
     │◄─ [{ uri, name, mimeType }]  │  Resource definitions
     │                               │
     │── prompts/list ─────────────►│
     │◄─ [{ name, description }]    │  Prompt definitions
     │                               │
     │── (ongoing)                  │
     │── tools/call ───────────────►│  { name, arguments }
     │◄─ { content: [...] }         │  Result
     │                               │
     │── (shutdown) ───────────────►│
     │                               │  (process exits)
```

#### Tool Call Protocol

```json
// Request: Core → MCP Server
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "method": "tools/call",
  "params": {
    "name": "list_pods",
    "arguments": {
      "namespace": "production",
      "label_selector": "app=web"
    }
  }
}

// Response: MCP Server → Core
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "NAME          READY   STATUS    RESTARTS\nweb-01        1/1     Running   0\nweb-02        1/1     Running   0\nweb-03        0/1     CrashLoop 3"
      }
    ]
  }
}

// Error Response
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "error": {
    "code": -32603,
    "message": "Internal error: oc command failed: Unauthorized"
  }
}
```

#### Tool Call Timeout

- **Default timeout**: 30 seconds per tool call
- **Configurable**: Per-tool timeout can be set in skill metadata
- **Behavior on timeout**: Core cancels the call and returns timeout error to AI

---

## Skill Lifecycle

### Skill States

```
          ┌──────────┐
          │ Uninstalled│
          └────┬─────┘
               │ Install
          ┌────▼─────┐
          │ Installed │
          └────┬─────┘
               │ Enable
          ┌────▼─────┐
          │ Disabled  │◄── Disable
          └────┬─────┘    │
               │ Start    │ Crash
          ┌────▼─────┐    │
          │  Running  │────┘
          └──────────┘
```

### Lifecycle States

| State | Description | Process | Tool Access |
|-------|-------------|---------|-------------|
| **Uninstalled** | Skill package not present on disk | Not present | No |
| **Installed** | Skill package present but not enabled | Not spawned | No |
| **Disabled** | Skill enabled in settings but not running | Not spawned | No |
| **Running** | MCP server process is active | Spawned and connected | Yes |

### Lifecycle Transitions

| Transition | Trigger | Action |
|------------|---------|--------|
| Uninstalled → Installed | `skill install <skill-id>` | Download and extract skill package |
| Installed → Disabled | `skill enable <skill-id>` | Mark as enabled, create config |
| Disabled → Running | First tool call or auto-start | Spawn MCP server process |
| Running → Disabled | `skill disable <skill-id>` | Gracefully shutdown MCP server |
| Running → Error | Process crash, health check fail | Auto-restart with backoff |
| Any → Uninstalled | `skill uninstall <skill-id>` | Remove package, stop process |

### Auto-Restart Policy

```
Crash detected → Wait 1s → Restart
Crash detected → Wait 2s → Restart
Crash detected → Wait 4s → Restart
Crash detected → Wait 8s → Restart
Crash detected → Wait 16s → Restart
Crash detected → Wait 30s (max) → Restart
```

If restart fails 5 times consecutively, mark skill as Error and notify user.

---

## Skill Discovery and Registration

### Discovery Process

1. **Scan skill directory** (`~/.local/share/wikilabs/skills/`) for installed skill packages
2. **Parse SKILL.md** and metadata.json for each skill
3. **Validate**: Check version, checksum, and required files
4. **Register** with the skill registry in the core
5. **Spawn** MCP servers for auto-start skills or on first tool call

### Skill Registry

The core maintains an in-memory skill registry:

```rust
struct SkillRegistry {
    skills: HashMap<String, SkillEntry>,
    enabled: HashSet<String>,
    auto_start: HashSet<String>,
}

struct SkillEntry {
    id: String,
    name: String,
    version: String,
    tools: Vec<ToolDefinition>,
    resources: Vec<ResourceDefinition>,
    prompts: Vec<PromptDefinition>,
    status: SkillStatus,
    process: Option<ProcessHandle>,
}
```

### Tool Catalog Aggregation

The core aggregates all tool definitions from all enabled skills into a **global tool catalog** that is presented to the AI provider:

```json
{
  "tools": [
    {
      "id": "openshift__list_pods",
      "name": "openshift list_pods",
      "skill_id": "openshift",
      "skill_name": "Red Hat OpenShift",
      "description": "List pods in a namespace",
      "inputSchema": { ... }
    },
    {
      "id": "linux__check_disk_usage",
      "name": "linux check_disk_usage",
      "skill_id": "linux",
      "skill_name": "Linux Systems",
      "description": "Check disk usage and identify large files",
      "inputSchema": { ... }
    },
    ...
  ]
}
```

---

## Capability Definitions

### Tool Capabilities

Each tool is classified by its capability type:

| Capability | Description | Example | Human Confirmation Required? |
|-----------|-------------|---------|------------------------------|
| **Query** | Read-only information retrieval | `list_pods`, `check_disk_usage` | No (informational only) |
| **Diagnose** | Analyze current state | `analyze_vmstat`, `describe_pod` | No (analysis only) |
| **Command** | Execute a command | `oc delete pod`, `systemctl restart` | **Yes** |
| **Configuration** | Modify configuration | Update deployment, change settings | **Yes** |
| **Network** | Access network resources | `curl`, `ping` | **Yes** |

### Resource Capabilities

Resources are static reference data provided by skills:

| Resource | Description | Example |
|----------|-------------|---------|
| **Reference** | Best practices documentation | `ocp://best-practices` |
| **Checklist** | Verification checklists | `ocp://upgrade-checklist` |
| **Pattern** | Common issue patterns | `vmware://cpu-ready-pattern` |
| **Template** | Command templates | `linux://systemctl-template` |

### Prompt Capabilities

Prompts are structured troubleshooting workflows:

| Prompt | Description | Example |
|--------|-------------|---------|
| **Workflow** | Step-by-step troubleshooting | `pod_crash_loop` |
| **Investigation** | Systematic investigation guide | `high_cpu_investigation` |
| **Planning** | Change planning template | `cluster_upgrade_plan` |

---

## Tool Definitions — Initial Skills

### OpenShift Skill Tools

| Tool | Capability | Description |
|------|-----------|-------------|
| `list_pods` | Query | List pods in a namespace with status |
| `describe_pod` | Diagnose | Get detailed pod information, events, containers |
| `list_nodes` | Query | List cluster nodes and their status |
| `describe_node` | Diagnose | Get node details, capacity, conditions |
| `check_cluster_health` | Query | Overall cluster health status |
| `list_events` | Query | Recent cluster events |
| `list_services` | Query | Services in a namespace |
| `describe_service` | Diagnose | Service details, endpoints, ports |
| `list_deployments` | Query | Deployments in a namespace |
| `describe_deployment` | Diagnose | Deployment details, replica status |
| `list_pvcs` | Query | Persistent volume claims |
| `describe_route` | Diagnose | Route details and status |
| `get_logs` | Query | Pod or container logs |
| `exec_command` | Command | Execute command in a container |

### Linux Skill Tools

| Tool | Capability | Description |
|------|-----------|-------------|
| `check_disk_usage` | Query | Disk usage by mount point |
| `find_large_files` | Query | Find files larger than threshold |
| `check_memory` | Query | Memory usage (free, used, cached, swap) |
| `check_cpu` | Query | CPU usage, load average, top processes |
| `check_network` | Query | Network interfaces, connections, errors |
| `check_service_status` | Query | Systemd service status |
| `analyze_vmstat` | Diagnose | Analyze vmstat output for issues |
| `analyze_iostat` | Diagnose | Analyze iostat output for I/O issues |
| `check_journald` | Query | Recent journal entries filtered by service |
| `find_recent_errors` | Query | Errors from system logs in last N hours |
| `list_running_processes` | Query | Top processes by resource usage |
| `check_dns` | Query | DNS resolution test |
| `check_ssh` | Query | SSH service status and recent auth attempts |
| `execute_command` | Command | Execute a shell command |

### VMware vSphere Skill Tools

| Tool | Capability | Description |
|------|-----------|-------------|
| `list_vms` | Query | List virtual machines and their status |
| `describe_vm` | Diagnose | VM details: resources, network, disks |
| `list_hosts` | Query | List ESXi hosts |
| `describe_host` | Diagnose | Host details: resources, health, events |
| `check_vm_performance` | Diagnose | VM performance metrics (CPU, memory, disk, network) |
| `list_datastores` | Query | Datastore usage and status |
| `check_datastore_health` | Diagnose | Datastore health, I/O latency |
| `list_networks` | Query | Virtual networks |
| `check_vmware_tools` | Query | VMware Tools status for VMs |
| `get_vm_events` | Query | VM and host events |
| `check_host_health` | Diagnose | Host health summary |
| `execute_esxcli` | Command | Execute esxcli command on a host |

### Ansible Skill Tools

| Tool | Capability | Description |
|------|-----------|-------------|
| `inventory_list` | Query | List inventory hosts and groups |
| `playbook_syntax_check` | Query | Validate playbook syntax |
| `playbook_diff` | Query | Dry-run playbook (check mode) |
| `host_facts` | Query | Gather facts for a host |
| `module_docs` | Query | Get documentation for an Ansible module |
| `role_structure` | Query | Analyze role directory structure |
| `execute_playbook` | Command | Run an Ansible playbook |

### Nagios XI Skill Tools

| Tool | Capability | Description |
|------|-----------|-------------|
| `list_services` | Query | List monitored services and their status |
| `list_hosts` | Query | List monitored hosts |
| `get_host_status` | Diagnose | Detailed host status and performance data |
| `get_service_status` | Diagnose | Detailed service status and performance data |
| `list_acknowledgements` | Query | Current problem acknowledgements |
| `list_notifications` | Query | Recent notifications |
| `get_perf_data` | Query | Performance data for a service/host |
| `check_command_defs` | Query | Check command definitions |
| `trigger_check` | Command | Force a check on a host/service |

### Checkmk Skill Tools

| Tool | Capability | Description |
|------|-----------|-------------|
| `list_sites` | Query | List Checkmk sites |
| `get_host_list` | Query | List monitored hosts |
| `get_host_status` | Diagnose | Host status, service status, downtime |
| `get_service_list` | Query | Services for a host |
| `get_service_status` | Diagnose | Service status and performance data |
| `get_metrics` | Query | Metric values for a service |
| `get_alerts` | Query | Current alerts and outages |
| `execute_check` | Command | Force a check on a host/service |

### MySQL Skill Tools

| Tool | Capability | Description |
|------|-----------|-------------|
| `check_connection` | Query | Test database connection |
| `list_databases` | Query | List databases |
| `list_tables` | Query | List tables in a database |
| `check_replication` | Diagnose | Master/slave replication status |
| `check_connections` | Query | Current connections and max usage |
| `check_slow_queries` | Query | Slow query log analysis |
| `check_buffer_pool` | Diagnose | InnoDB buffer pool usage |
| `check_locks` | Diagnose | Current locks and waiters |
| `execute_query` | Command | Execute a SQL query |

### EDB PostgreSQL Skill Tools

| Tool | Capability | Description |
|------|-----------|-------------|
| `check_connection` | Query | Test database connection |
| `list_databases` | Query | List databases |
| `list_tables` | Query | List tables |
| `check_replication` | Diagnose | Streaming replication status |
| `check_connections` | Query | Connection count and limits |
| `check_wal` | Diagnose | WAL segment usage and archiving |
| `check_vacuum` | Diagnose | VACUUM status and bloat |
| `check_locks` | Diagnose | Current locks |
| `execute_query` | Command | Execute a SQL query |

### Microsoft SQL Server Skill Tools

| Tool | Capability | Description |
|------|-----------|-------------|
| `check_connection` | Query | Test database connection |
| `list_databases` | Query | List databases |
| `list_tables` | Query | List tables |
| `check_jobs` | Query | SQL Agent job status |
| `check_backups` | Diagnose | Recent backup status |
| `check_wait_stats` | Diagnose | Server wait statistics |
| `check_locks` | Diagnose | Current locks and blockers |
| `execute_query` | Command | Execute a T-SQL query |

### Windows Skill Tools

| Tool | Capability | Description |
|------|-----------|-------------|
| `check_services` | Query | Windows service status |
| `check_event_logs` | Query | Windows Event Log entries |
| `check_disk` | Query | Disk usage and health |
| `check_performance` | Query | Windows Performance Counter data |
| `check_updates` | Query | Windows Update status |
| `check_users` | Query | Active user sessions |
| `check_registry` | Query | Registry key values |
| `check_windows_firewall` | Query | Firewall rules and status |
| `execute_command` | Command | Execute a PowerShell or CMD command |

---

## MCP Transport Layer

### Default Transport: stdio

All local skills use stdio as the default transport:

```
┌──────────────┐                    ┌─────────────────┐
│  Rust Core    │                    │ MCP Server      │
│              │  stdin (write)     │                 │
│  ────────────┼───────────────────►│                 │
│              │  stdout (read)     │                 │
│  ◄───────────┼───────────────────┤                 │
│  TCP socket  │                    │                 │
│  for PID mgmt│                    │                 │
└──────────────┘                    └─────────────────┘
```

**Protocol**: JSON-RPC 2.0 messages are sent over stdin/stdout, separated by newlines. Each message is a JSON object with a `Content-Length` header (like HTTP/1.1) for reliable framing.

### Alternative Transport: HTTP

For remote skills (e.g., enterprise on-prem MCP servers):

```
┌──────────────┐  HTTPS/TLS   ┌─────────────────┐
│  Rust Core    │─────────────►│  Remote MCP     │
│              │              │  Server          │
│  ────────────│ HTTP POST   │  (port 8080)     │
│  ────────────│◄────────────│                  │
└──────────────┘              └─────────────────┘
```

**Protocol**: HTTP POST with JSON body, MCP protocol in the request/response body. TLS is required.

### Alternative Transport: WebSocket

For streaming tools (real-time data feeds):

```
┌──────────────┐  WSS/TLS    ┌─────────────────┐
│  Rust Core    │────────────►│  Remote MCP     │
│              │◄────────────│  Server          │
└──────────────┘              └─────────────────┘
```

**Protocol**: WebSocket upgrade → MCP JSON-RPC over WebSocket frames.

---

## Skill Isolation and Sandboxing

### Process Isolation

- Each MCP server runs as an **independent OS process**
- Crash of one server does not affect others or the core
- Each server runs with the **same user privileges** as the main application
- No inter-process communication between MCP servers (they only communicate through the core)

### Credential Isolation

- Each skill only has access to its own required credentials
- Credentials are provided via environment variables or stdin injection — never passed as command-line arguments
- Credential access is logged in the audit trail

### Network Isolation

- MCP servers have no network access by default
- If a skill requires network access (e.g., API calls), it must be explicitly permitted in skill metadata
- Network access is logged and audited

### Resource Isolation

- MCP server processes share the same memory space as the core (same user, same machine)
- No CPU or memory limits are enforced at the process level (the skill itself manages its resource usage)
- The core monitors MCP server health via heartbeat checks

### Permission Model

| Permission | Who Grants | Scope |
|-----------|-----------|-------|
| Observation permissions | User (OS-level) | Global or per-source |
| Skill enable/disable | User (in-app settings) | Per-skill |
| Tool execution confirmation | User (in-chat) | Per-tool invocation |
| Credential access | System (keychain) | Per-service |
| Network access (if required) | User (skill metadata) | Per-skill |

---

## Skill Testing Framework

### Test Categories

| Category | Description | When |
|----------|-------------|------|
| **Unit Tests** | Test individual tool handlers in isolation | Every PR |
| **Integration Tests** | Test tool calls through MCP protocol | Every PR |
| **Workflow Tests** | Test troubleshooting workflows end-to-end | Every release |
| **Security Tests** | Test credential handling, input validation | Every release |
| **Performance Tests** | Test tool response times | Every release |

### Test Implementation

```rust
// openshift-skill/tests/tool_tests.rs (conceptual)

#[tokio::test]
async fn test_list_pods_success() {
    // Mock oc command output
    let mock_output = "NAME          READY   STATUS\nweb-01        1/1     Running\nweb-02        0/1     CrashLoopBackOff";
    
    // Call tool
    let result = call_tool("list_pods", json!({
        "namespace": "production",
        "label_selector": "app=web"
    })).await;
    
    // Assert
    assert_eq!(result.content.len(), 1);
    assert!(result.content[0].text.contains("web-01"));
    assert!(result.content[0].text.contains("web-02"));
}

#[tokio::test]
async fn test_list_pods_missing_namespace() {
    // Missing required parameter
    let result = call_tool("list_pods", json!({}))
        .await;
    
    assert!(result.is_error());
    assert!(result.error.message.contains("namespace"));
}

#[tokio::test]
async fn test_describe_pod_command_failure() {
    // Command fails (pod not found)
    let result = call_tool("describe_pod", json!({
        "namespace": "production",
        "name": "nonexistent"
    })).await;
    
    assert!(result.is_error());
    assert!(result.error.message.contains("not found"));
}
```

### Test Data

Test data is stored in the skill's `tests/` directory:
- Mock command output files (`.txt`)
- Expected tool response files (`.json`)
- Test configuration files (`.yaml`)

---

## Skill Packaging and Distribution

### Package Format

Skills are distributed as **versioned packages**:

```
wikilabs-skill-openshift-1.2.0-win-x64.zip
wikilabs-skill-openshift-1.2.0-macos-arm64.zip
wikilabs-skill-openshift-1.2.0-linux-x64.zip
wikilabs-skill-openshift-1.2.0-linux-arm64.zip
```

Each package contains:
- MCP server binary (platform-specific)
- SKILL.md and metadata.json
- Tool definitions, resources, workflows
- Test suite (optional, for development)
- Signature file (`.sig`) for verification

### Distribution Channels

| Channel | Description | Use Case |
|---------|-------------|---------|
| **Update Server** | HTTPS endpoint with versioned packages | Production skill distribution |
| **Local File** | User imports a `.zip` or directory | Custom/internal skills |
| **Git Repository** | Skills cloned from a git repo | Open-source skills |

### Installation Flow

```
1. User selects a skill to install
2. Application downloads the skill package from the update server
3. Signature is verified (`.sig` file against public key)
4. Package is extracted to `~/.local/share/wikilabs/skills/<skill-id>/`
5. SKILL.md and metadata.json are parsed and validated
6. Skill is registered in the skill registry
7. Skill is marked as "Installed" (disabled by default)
8. User can enable it from the skill management UI
```

### Update Flow

```
1. Periodically (e.g., daily), application checks for skill updates
2. For each installed skill, compares local version with server version
3. If a newer version is available, notifies the user
4. On user approval, downloads and installs the new package
5. Running MCP servers are gracefully stopped and restarted with the new binary
6. Skill metadata and tool definitions are re-registered
```

### Skill Signing

Skills are signed using **Ed25519 signatures**:

```
Signature Process:
  1. Skill author creates the package
  2. Author signs the package with their private key
  3. Signature is stored in the `.sig` file
  4. On installation, the core verifies the signature against the
     trusted public key stored in the application

Verification:
  - Package hash is computed (SHA-256)
  - Signature is verified against the hash
  - If verification fails, installation is rejected
```

## References

- [ARCHITECTURE.md](ARCHITECTURE.md) — System architecture overview
- [COMPONENT_DESIGN.md](COMPONENT_DESIGN.md) — MCP Skill Manager component
- [DATA_MODEL.md](DATA_MODEL.md) — Skill metadata and configuration data model
- [SECURITY_ARCHITECTURE.md](SECURITY_ARCHITECTURE.md) — Security considerations for MCP
- [TECHNOLOGY_SELECTION.md](TECHNOLOGY_SELECTION.md) — Technology choices for MCP transport