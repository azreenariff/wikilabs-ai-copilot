# Linux Engineering — Reasoning Reference

## Purpose

This document defines the diagnostic reasoning framework for Linux engineering troubleshooting.

## Reasoning Model

The Linux engineering skill pack uses a structured diagnostic reasoning approach:

```
Problem Statement
    │
    ├─→ 1. Classify the issue type
    │     (service, performance, security, network, storage)
    │
    ├─→ 2. Determine urgency and scope
    │     (outage, degradation, prevention, audit)
    │
    ├─→ 3. Gather evidence
    │     (systematic data collection)
    │
    ├─→ 4. Analyze evidence
    │     (hypothesis generation and testing)
    │
    ├─→ 5. Identify root cause
    │     (most likely explanation supported by evidence)
    │
    ├─→ 6. Plan remediation
    │     (fix with minimal risk)
    │
    └─→ 7. Verify resolution
          (confirm the fix worked)
```

## Classification Rules

### Issue Type Classification

| Symptom Pattern | Likely Type | Primary Workflow |
|----------------|-------------|-----------------|
| "service failed", "cannot start" | Service Failure | service-not-starting |
| "slow", "high load", "lagging" | Performance | system-slow |
| "access denied", "authentication fail" | Security | ssh-access-denied |
| "cannot connect", "timeout", "unreachable" | Network | network-connectivity |
| "disk full", "no space left" | Storage | disk-full |
| "high CPU", "spike" | Performance | high-cpu-usage |
| "out of memory", "OOM killed" | Memory | memory-exhaustion |
| "cannot boot", "kernel panic" | Boot | boot-failure |
| "cannot install", "dependency error" | Package | package-installation |

### Urgency Classification

| Level | Description | Response Time |
|-------|-------------|---------------|
| Critical | Complete outage, data loss risk | Immediate |
| High | Service degradation, user impact | < 30 min |
| Medium | Non-critical issue, workaround exists | < 4 hours |
| Low | Cosmetic, optimization, prevention | < 24 hours |

## Evidence Collection Strategy

### Rule: Collect Before Analyzing
Always collect sufficient evidence before forming hypotheses. Premature diagnosis leads to wrong fixes.

### Rule: Use the 5 Whys
When a root cause is found, ask "why" 5 times to find the underlying cause:

```
1. Why did nginx fail? → Port 80 already in use
2. Why is port 80 in use? → Apache is running
3. Why is Apache running? → It was enabled after migration
4. Why was it enabled? → Not disabled during nginx installation
5. Why wasn't it disabled? → No check was done
Root cause: Process not verified before service startup
```

### Rule: Consider Multiple Hypotheses
For ambiguous cases, generate multiple hypotheses and test each:

```
User: "Server is slow"
    │
    ├─→ Hypothesis 1: CPU bottleneck
    │   └─→ Test: ps aux --sort=-%cpu | head -5
    │
    ├─→ Hypothesis 2: Memory pressure
    │   └─→ Test: free -m
    │
    ├─→ Hypothesis 3: Disk I/O wait
    │   └─→ Test: iostat -x 1 3
    │
    └─→ Hypothesis 4: Network issue
        └─→ Test: ping, traceroute, ss
```

## Diagnostic Decision Tree

```
Linux Issue
    │
    ├─→ Is it a service issue?
    │   ├─→ Yes → Check: systemctl status, journalctl
    │   │           ↓
    │   │       Is it config error? → Fix config
    │   │       Is it dependency? → Fix dependency
    │   │       Is it resource? → Free resource
    │   │
    │   └─→ No
    │
    ├─→ Is it a performance issue?
    │   ├─→ Yes → Check: CPU, memory, disk I/O, network
    │   │           ↓
    │   │       High CPU? → Identify process, optimize/kill
    │   │       High memory? → Find leak, increase RAM
    │   │       High I/O? → Identify process, optimize
    │   │
    │   └─→ No
    │
    ├─→ Is it a security issue?
    │   ├─→ Yes → Check: auth logs, firewall, permissions
    │   │           ↓
    │   │       Auth failure? → Fix credentials/keys
    │   │       Permission issue? → Fix ACLs/SELinux
    │   │       Firewall issue? → Fix rules
    │   │
    │   └─→ No
    │
    ├─→ Is it a network issue?
    │   ├─→ Yes → Check: interfaces, routes, DNS, firewalls
    │   │           ↓
    │   │       Interface down? → Bring up
    │   │       No route? → Add route
    │   │       DNS fail? → Fix DNS config
    │   │
    │   └─→ No
    │
    └─→ Is it a storage issue?
        ├─→ Yes → Check: disk space, inode usage, mount status
        │           ↓
        │       Disk full? → Clean up/extend
        │       Inode full? → Remove small files
        │       Not mounted? → Fix fstab/mount
        │
        └─→ No → Unknown issue → Escalate
```

## Risk Assessment

Before applying any remediation:

| Factor | Low Risk | High Risk |
|--------|----------|-----------|
| Reversibility | Easy to undo | Difficult to undo |
| Impact | Single service | All services |
| Data | No data at risk | Data could be lost |
| Users | Minimal impact | Many users affected |
| Time | Quick to revert | Long downtime |

**Safe operations:** Reading logs, checking status, viewing configs
**Caution operations:** Restarting services, modifying configs, killing processes
**High risk operations:** Rebooting, filesystem changes, kernel parameters, rm commands

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial reasoning reference |