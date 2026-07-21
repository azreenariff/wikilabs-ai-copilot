# Windows Engineering — Context Interpretation

## Purpose

This document defines how to interpret context for the Windows engineering skill pack.

## Context Sources

### 1. User Input
- Natural language descriptions of Windows Server issues
- Event IDs and error messages from Event Viewer
- PowerShell error output
- Windows error dialogs and notifications

### 2. Technical Signals
- Event Log entries (System, Application, Security)
- Windows Performance Monitor metrics
- PowerShell execution errors
- Event Viewer error/critical entries
- Windows Update failure reports
- Service control manager logs

### 3. Environmental Context
- Windows Server version (2016, 2019, 2022, 2025)
- Server role (Domain Controller, File Server, Web Server, etc.)
- Active Directory domain membership
- Network configuration (DHCP, static IP, DNS)
- Installed roles and features

## Interpretation Framework

### Pattern: Symptom → Category → Workflow

```
User: "Windows service won't start"
    │
    ├─→ Category: Service Failure
    ├─→ Likely Services: Depends on reported service
    └─→ Workflow: windows-service-not-starting
```

```
User: "DNS not resolving"
    │
    ├─→ Category: DNS Resolution Failure
    ├─→ Possible Causes: DNS service down, cache issue, firewall
    └─→ Workflow: windows-dns-resolution-failed
```

```
User: "Active Directory replication is broken"
    │
    ├─→ Category: AD Issue
    ├─→ Possible Causes: Network, DNS, replication error
    └─→ Workflow: windows-activedirectory-issue
```

```
User: "Disk C: is full"
    │
    ├─→ Category: Storage Exhaustion
    ├─→ Possible Causes: Log files, temp files, updates
    └─→ Workflow: windows-disk-full
```

```
User: "Event log shows error ID 1001"
    │
    ├─→ Category: Event Log Error
    ├─→ Possible Causes: Application crash, service failure
    └─→ Workflow: windows-event-log-error
```

```
User: "IIS not serving websites"
    │
    ├─→ Category: IIS Failure
    ├─→ Possible Causes: W3SVC service down, app pool crash
    └─→ Workflow: windows-iis-down
```

```
User: "Windows Update failed to install patches"
    │
    ├─→ Category: Update Failure
    ├─→ Possible Causes: Corrupt update cache, missing dependencies
    └─→ Workflow: windows-update-failed
```

```
User: "Server is running very slow"
    │
    ├─→ Category: Performance Degradation
    ├─→ Possible Causes: High CPU, memory pressure, disk I/O
    └─→ Workflow: windows-performance-slow
```

### Confidence Scoring

| Confidence | Description | Example |
|------------|-------------|---------|
| High (0.9+) | Clear signal, specific error | "Service failed with error 1067" |
| Medium (0.7-0.9) | Multiple possible causes | "Windows service not starting" |
| Low (0.5-0.7) | Vague description, needs more info | "Server is acting weird" |
| Very Low (<0.5) | Insufficient information | "?" |

## Windows-Specific Context Patterns

### Event ID Patterns
- Event ID 1000 → Application crash (EXCEPTION_ACCESS_VIOLATION)
- Event ID 1001 → Application error dump
- Event ID 7023 → Service terminated with error
- Event ID 7024 → Service failed to start
- Event ID 1006 → Active Directory replication error
- Event ID 1058 → Windows Update install failure
- Event ID 41 → Unexpected kernel power event (BSOD)
- Event ID 6008 → Unexpected shutdown
- Event ID 1002 → Application hang

### Service Error Codes
- Error 1067 → Process terminated unexpectedly
- Error 1053 → Service timeout
- Error 1068 → Service dependency failed
- Error 70008 → Service failed due to access denied
- Error 2 → File not found
- Error 1058 → Disabled by policy

### PowerShell Patterns
- "Access is denied" → Permission issue or execution policy
- "Module not found" → Module not installed or loaded
- "Cannot bind argument" → Incorrect parameter format
- "The running command stopped because..." → Timeout or resource limit
- "ExecutionPolicy" → Scripts blocked by execution policy

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial context interpretation guide |