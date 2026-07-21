# Linux Engineering — Common Failures Reference

## Purpose

This directory documents common failure modes for Linux systems.

## Categories

| Category | Description | File |
|----------|-------------|------|
| Service Failures | Services failing to start/restart | common-failures/service.md |
| Performance Failures | CPU, memory, I/O bottlenecks | common-failures/performance.md |
| Network Failures | Connectivity, DNS, routing issues | common-failures/network.md |
| Storage Failures | Disk full, filesystem corruption | common-failures/storage.md |
| Security Failures | Auth failures, permission issues | common-failures/security.md |

## Failure Classification

### By Symptom

| Symptom | Likely Cause | Severity |
|---------|-------------|----------|
| Service crash | Config error, OOM, dependency | High |
| High load | CPU-bound process, I/O wait | Medium-High |
| Slow response | Resource contention | Medium |
| Connection refused | Service down, firewall | High |
| Permission denied | Wrong ACLs, SELinux | Medium |
| Disk I/O errors | Failing disk, controller | Critical |
| Kernel panic | Hardware failure, bad module | Critical |

### By Impact

| Impact Level | Affected Systems | Response |
|-------------|-----------------|----------|
| P1 - Critical | All production systems | Immediate |
| P2 - High | Service degraded | < 30 min |
| P3 - Medium | Single system | < 4 hours |
| P4 - Low | Cosmetic, non-urgent | < 24 hours |

## Root Cause Patterns

### Pattern 1: Cascading Failure
```
Root Cause: Disk full
    ↓
Service cannot write logs
    ↓
Service crashes
    ↓
Users affected
```

### Pattern 2: Configuration Drift
```
Root Cause: Manual config change
    ↓
Change not documented
    ↓
System fails after update
    ↓
No one knows why
```

### Pattern 3: Resource Exhaustion
```
Root Cause: Memory leak
    ↓
Memory fills up
    ↓
OOM killer activates
    ↓
Critical processes killed
    ↓
Service outage
```

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial common failures reference |