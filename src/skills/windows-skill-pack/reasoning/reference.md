# Windows Engineering — Reasoning Reference

## Purpose

This document defines the diagnostic reasoning framework for Windows engineering troubleshooting.

## Reasoning Model

The Windows engineering skill pack uses a **hierarchical diagnostic reasoning** model:

### Level 1: Symptom Classification

Classify into categories:

1. **Service Issues** — Windows service failures, dependencies
2. **Event Log Issues** — Application, system, or security events
3. **Network Issues** — Connectivity, DNS, firewall
4. **Storage Issues** — Disk space, volume, filesystem
5. **AD Issues** — Domain, replication, authentication
6. **Performance Issues** — CPU, memory, disk, network throughput
7. **Security Issues** — Authentication, permissions, compliance

### Level 2: Root Cause Analysis

```
Symptom Detected
    │
    ├─→ Is it a service issue?
    │   ├─→ Check status: Get-Service
    │   ├─→ Check dependencies: sc queryex
    │   ├─→ Check logs: Event Viewer
    │   └─→ Restart: Start-Service
    │
    ├─→ Is it a network issue?
    │   ├─→ Check adapter: Get-NetAdapter
    │   ├─→ Check IP: Get-NetIPAddress
    │   ├─→ Test connectivity: Test-NetConnection
    │   └─→ Check DNS: Resolve-DnsName
    │
    ├─→ Is it a storage issue?
    │   ├─→ Check volumes: Get-Volume
    │   ├─→ Check disk space: Get-CimInstance Win32_LogicalDisk
    │   ├─→ Find large files: Get-ChildItem
    │   └─→ Clean up: Remove-Item
    │
    ├─→ Is it a performance issue?
    │   ├─→ Check CPU: Get-Counter Processor
    │   ├─→ Check memory: Get-Counter Memory
    │   ├─→ Check disk: Get-Counter PhysicalDisk
    │   └─→ Identify processes: Get-Process
    │
    └─→ Is it an AD issue?
        ├─→ Check DC: Get-ADDomainController
        ├─→ Check replication: Repadmin /showrepl
        ├─→ Check DNS: nslookup
        └─→ Test connection: Test-ComputerSecureChannel
```

### Level 3: Decision Trees

#### Service Decision Tree
```
Service not starting?
    │
    ├─→ Service stopped → Start-Service
    │       ↓
    │       Still stopped? → Check Event Log
    │           ↓
    │           Dependency failed? → Start dependency
    │           Access denied? → Fix permissions
    │           Missing file? → Repair installation
    │
    └─→ Service failed → Check Error Code
            ↓
            1067 → Process terminated
            1053 → Timeout
            1068 → Dependency failed
            70008 → Access denied
```

#### Network Decision Tree
```
Network issue?
    │
    ├─→ Cannot resolve → DNS issue
    │   ├─→ nslookup fails → Restart DNS service
    │   ├─→ Wrong IP → Check DNS records
    │   └─→ Timeout → Check firewall rules
    │
    ├─→ Cannot connect → Connectivity issue
    │   ├─→ Ping fails → Check adapter/firewall
    │   ├─→ Port blocked → Check Windows Firewall
    │   └─→ Timeout → Check routing/gateway
    │
    └─→ Slow performance → Resource issue
        ├─→ High CPU → Kill/process optimization
        ├─→ High memory → Add RAM/migrate
        └─→ High disk I/O → Move workloads
```

## Confidence Scoring

| Confidence | Description | Example |
|------------|-------------|---------|
| High (0.9+) | Clear error, specific cause | "Event ID 1000: Application crash" |
| Medium (0.7-0.9) | Multiple possible causes | "Service won't start" |
| Low (0.5-0.7) | Vague symptoms, needs more info | "Windows is slow" |
| Very Low (<0.5) | Insufficient data | "?" |

## Escalation Criteria

Escalate to Microsoft Support when:

1. **Data loss risk** — Potential for data corruption
2. **Cannot diagnose** — Root cause unclear after evidence collection
3. **Bug suspected** — Issue matches known Windows bug patterns
4. **Hardware failure** — Physical hardware failure suspected
5. **Out of scope** — Issue beyond skill pack scope

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial reasoning framework |