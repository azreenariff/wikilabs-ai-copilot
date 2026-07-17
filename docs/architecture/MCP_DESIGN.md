# MCP Design — Wiki Labs AI Copilot

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│          MCP Skill Runtime (single in-process module)         │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐   │
│  │  Skill Module Trait                                     │   │
│  │  pub trait SkillModule: Send + Sync {                  │   │
│  │      fn id(&self) -> &str;                              │   │
│  │      fn tools(&self) -> Vec<ToolDefinition>;           │   │
│  │      async fn call_tool(&self, ...) -> ToolResult;     │   │
│  │  }                                                       │   │
│  └────────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐   │
│  │  Skill Registry: HashMap<String, Box<dyn SkillModule>>  │   │
│  │  - Tools aggregated into global tool catalog            │   │
│  │  - Namespace: "openshift__list_pods"                    │   │
│  │  - Lazy loading: loaded on first tool call              │   │
│  └────────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐   │
│  │  [NEW] Context Bus: Skills publish context events       │   │
│  │  - OpenShift: Detected CrashLoopBackOff on pod web-01   │   │
│  │  - Linux: Pod web-01 is running on node node-3         │   │
│  │  - VMware: Node node-3 is a VM on ESXi host esxi-01    │   │
│  └────────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐   │
│  │  MCP Server Bridge: Exposes single MCP server          │   │
│  │  - Implements draft MCP spec (2024-11-05)              │   │
│  │  - Internal skill registry abstracts MCP protocol       │   │
│  └────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

## Module Loading Strategy

| Phase | Mechanism | Use Case |
|-------|-----------|----------|
| Phase 1 | Compile-time cargo workspace modules | Bundled skills only |
| Phase 2 | Dynamic linking (.so/.dll/.dylib) | Third-party skills |
| Phase 3 | WebAssembly (Wasmtime) | Sandboxed community skills |