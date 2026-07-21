# Windows Engineering — Workflow Catalog

## Purpose

This directory documents the workflow state machines for Windows engineering troubleshooting.

## Workflow State Machine

All Windows workflows follow this state machine pattern:

```
[evidence_collection] → [diagnosis] → [remediation] → [verification]
```

### State Definitions

| State | Purpose | Terminal? |
|-------|---------|-----------|
| evidence_collection | Gather initial evidence and signals | No |
| diagnosis | Analyze evidence to identify root cause | No |
| remediation | Apply fix based on diagnosed cause | No |
| verification | Confirm resolution and restore service | Yes |

### State Transitions

| From | To | Condition |
|------|-----|-----------|
| evidence_collection | diagnosis | Evidence collected, sufficient signals |
| diagnosis | remediation | Root cause identified with confidence |
| remediation | verification | Fix applied, service restored |
| verification | evidence_collection | Resolution not confirmed, re-evaluate |
| diagnosis | evidence_collection | Insufficient evidence, gather more |
| remediation | diagnosis | Fix didn't work, re-diagnose |

## Workflow Inventory

| Workflow ID | Name | Risk | Primary Domain |
|-------------|------|------|----------------|
| windows-service-not-starting | Service Not Starting | Medium | Services |
| windows-event-log-error | Event Log Error | Low | Event Logging |
| windows-dns-resolution-failed | DNS Resolution Failure | Medium | Networking |
| windows-activedirectory-issue | AD Issue | High | Active Directory |
| windows-disk-full | Disk Space Critical | Medium | Storage |
| windows-powershell-error | PowerShell Error | Low | Scripting |
| windows-network-connectivity | Network Connectivity | Medium | Networking |
| windows-iis-down | IIS Down | Medium | Web Services |
| windows-update-failed | Windows Update Failure | Medium | Updates |
| windows-performance-slow | Performance Degradation | Medium | Performance |

## Workflow Design Principles

1. **Start with evidence** — Never jump to remediation
2. **Verify before and after** — Evidence collection before, verification after
3. **Document everything** — Commands run, output, decisions
4. **Escalate when uncertain** — Don't guess, get help
5. **Prevent recurrence** — Include lessons learned

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial workflow catalog |