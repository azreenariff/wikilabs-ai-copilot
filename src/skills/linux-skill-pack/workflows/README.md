# Linux Engineering — Workflow Catalog

## Purpose

This directory documents the workflow state machines for Linux engineering troubleshooting.

## Workflow State Machine

All Linux workflows follow a standard four-phase pattern:

```
Phase 1: Evidence Collection
    │
    ▼
Phase 2: Diagnosis
    │
    ▼
Phase 3: Remediation
    │
    ▼
Phase 4: Verification ───┐
    │                     │
    └── (if failed) ──────┘
```

## Available Workflows

| Workflow ID | Name | Description | File |
|-------------|------|-------------|------|
| service-not-starting | Service Not Starting | Diagnose services that fail to start | workflows/service.md |
| high-cpu-usage | High CPU Usage | Diagnose unexpected high CPU | workflows/cpu.md |
| disk-full | Disk Space Full | Resolve disk space exhaustion | workflows/storage.md |
| ssh-access-denied | SSH Access Denied | Fix SSH connection failures | workflows/ssh.md |
| network-connectivity | Network Connectivity Issue | Diagnose network problems | workflows/network.md |
| memory-exhaustion | Memory Exhaustion | Diagnose OOM and memory leaks | workflows/memory.md |
| boot-failure | Boot Failure | Fix boot problems | workflows/boot.md |
| package-installation | Package Installation Failure | Fix package issues | workflows/packages.md |
| system-slow | Performance Degradation | Diagnose general slowness | workflows/performance.md |

## Workflow Components

### States
Each workflow has four states:
- **evidence_collection**: Gather initial evidence
- **diagnosis**: Analyze evidence to find root cause
- **remediation**: Apply appropriate fix
- **verification**: Confirm the fix worked

### Transitions
Transitions between states are triggered by:
- Evidence collected (`evidence_collected`)
- Root cause identified (`root_cause_identified`)
- Remediation applied (`remediation_applied`)
- Resolution failed (`service_still_failing`, `cpu_still_high`, etc.)

### Evidence Requirements
Each workflow specifies required evidence:
- Command to run
- Expected output to check
- Success criteria

### Risk Classification
Each workflow is classified by risk:
- **Low**: Read-only, diagnostic commands
- **Medium**: May restart services or modify configuration
- **High**: May cause data loss or system instability

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial workflow catalog |