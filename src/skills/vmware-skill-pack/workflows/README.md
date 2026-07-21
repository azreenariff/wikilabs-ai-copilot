# VMware vSphere — Workflow Catalog

## Purpose

This directory documents the workflow state machines for VMware vSphere engineering troubleshooting.

## Workflow State Machine

All VMware workflows follow this state machine pattern:

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
| vmware-vcenter-not-starting | vCenter Service Failure | Medium | vCenter |
| vmware-host-disconnected | Host Disconnection | Medium | Host |
| vmware-vm-slow | VM Performance Degradation | Low | VM |
| vmware-datastore-almost-full | Datastore Space Exhaustion | Medium | Storage |
| vmware-vm-missing-disk | Virtual Disk Missing | High | Storage |
| vmware-cluster-ha-failure | HA Cluster Failure | High | Cluster |
| vmware-vmotion-failed | vMotion Failure | Medium | Network |
| vmware-cluster-drs-issue | DRS Issue | Medium | Cluster |
| vmware-esxi-crash | ESXi Host Crash | High | Host |
| vmware-backup-failure | VM Backup Failure | Medium | Storage |

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