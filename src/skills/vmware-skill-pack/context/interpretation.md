# VMware vSphere — Context Interpretation

## Purpose

This document defines how to interpret context for the VMware vSphere engineering skill pack.

## Context Sources

### 1. User Input
- Natural language descriptions of infrastructure issues
- Error messages and stack traces
- vSphere Client UI notifications
- Monitoring/alert notifications (Nagios, Checkmk, Prometheus)

### 2. Technical Signals
- vpxd/vpxa log entries
- vmkernel.log, syslog entries
- vSphere Client UI state (red status, warnings)
- Performance metrics (CPU ready, memory pressure, disk latency)
- HA/DRS alarms and events
- Storage errors (LUN loss, path failures)
- vMotion migration errors

### 3. Environmental Context
- vCenter Server version and type (VCSA vs Windows)
- ESXi host versions and hardware
- Cluster configuration (HA enabled, DRS mode)
- Storage topology (VMFS, NFS, vSAN)
- Network topology (vSS vs vDS, VLANs)

## Interpretation Framework

### Pattern: Symptom → Category → Workflow

```
User: "vCenter is down"
    │
    ├─→ Category: vCenter Service Failure
    ├─→ Likely Services: vpxd, vpxd-svcs, vsphere-client
    └─→ Workflow: vmware-vcenter-not-starting
```

```
User: "ESXi host disconnected from vCenter"
    │
    ├─→ Category: Host Disconnection
    ├─→ Possible Causes: Network, certificate, management service
    └─→ Workflow: vmware-host-disconnected
```

```
User: "VM is running very slow"
    │
    ├─→ Category: VM Performance Degradation
    ├─→ Possible Causes: CPU ready, memory, disk latency, snapshots
    └─→ Workflow: vmware-vm-slow
```

```
User: "Datastore almost full"
    │
    ├─→ Category: Storage Exhaustion
    ├─→ Possible Causes: Snapshots, orphaned files, capacity
    └─→ Workflow: vmware-datastore-almost-full
```

```
User: "HA cluster has red status"
    │
    ├─→ Category: HA Failure
    ├─→ Possible Causes: Network, heartbeat, admission control
    └─→ Workflow: vmware-cluster-ha-failure
```

```
User: "vMotion failed"
    │
    ├─→ Category: Migration Failure
    ├─→ Possible Causes: Compatibility, network, resources
    └─→ Workflow: vmware-vmotion-failed
```

### Confidence Scoring

| Confidence | Description | Example |
|------------|-------------|---------|
| High (0.9+) | Clear signal, specific error | "vpxd service failed to start" |
| Medium (0.7-0.9) | Multiple possible causes | "Host disconnected" |
| Low (0.5-0.7) | Vague description, needs more info | "Something wrong with cluster" |
| Very Low (<0.5) | Insufficient information | "?" |

## Context Resolution Steps

1. **Parse the user input** — extract key entities (VM names, hostnames, error codes)
2. **Match against detection rules** — find relevant rule(s)
3. **Score confidence** — based on clarity and specificity
4. **Select workflow** — choose the most appropriate workflow
5. **Request clarification** — if confidence is too low to proceed

## Multi-Context Scenarios

When multiple contexts are detected:

1. **Prioritize by severity** — host crash > HA failure > VM slow
2. **Check for root cause relationships** — datastore full → VM can't start
3. **Present options** — if unrelated issues, list both
4. **Work sequentially** — fix root causes first

## vSphere-Specific Context Patterns

### vCenter UI Patterns
- Red host icon → host disconnected or not responding
- Yellow host icon → host in maintenance mode or warnings
- Red VM icon → VM has critical alarm
- Yellow VM icon → VM has warning (high CPU ready, low disk space)
- Exclamation mark on cluster → HA/DRS issue

### Log Pattern Matching
- "Cannot contact SSO service" → vCenter/SSO connectivity issue
- "Heartbeat timeout" → HA/host management network issue
- "Admission control would prevent failover" → Resource reservation issue
- "Storage path unavailable" → Storage multipathing/FC/iSCSI issue
- "VMotion cannot complete" → Compatibility/network issue

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial context interpretation guide |