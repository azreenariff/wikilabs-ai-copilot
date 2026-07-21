# Linux Engineering — Context Interpretation

## Purpose

This document defines how to interpret context for the Linux engineering skill pack.

## Context Sources

### 1. User Input
- Natural language descriptions of issues
- Error messages and stack traces
- Command outputs
- Monitoring/alert notifications

### 2. Technical Signals
- Service status outputs
- Log entries (journalctl, /var/log/*)
- Performance metrics (CPU, memory, disk, network)
- Security events (auth failures, SELinux denials)

### 3. Environmental Context
- Distribution and version
- Hardware specifications
- Running services
- Network topology

## Interpretation Framework

### Pattern: Symptom → Category → Workflow

```
User: "The website is down"
    │
    ├─→ Category: Service Failure
    ├─→ Likely Services: nginx, httpd, haproxy
    └─→ Workflow: service-not-starting
```

```
User: "Server is very slow"
    │
    ├─→ Category: Performance Degradation
    ├─→ Possible Causes: CPU, memory, disk I/O, network
    └─→ Workflow: system-slow
```

```
User: "Cannot SSH into the server"
    │
    ├─→ Category: SSH Access Denied
    ├─→ Possible Causes: auth failure, firewall, service down
    └─→ Workflow: ssh-access-denied
```

### Confidence Scoring

| Confidence | Description | Example |
|------------|-------------|---------|
| High (0.9+) | Clear signal, single root cause | "nginx.service: Failed with exit code 1" |
| Medium (0.7-0.9) | Multiple possible causes | "Server is slow" |
| Low (0.5-0.7) | Vague description, needs more info | "Something isn't working" |
| Very Low (<0.5) | Insufficient information | "?" |

## Context Resolution Steps

1. **Parse the user input** — extract key entities (service names, errors, hostnames)
2. **Match against detection rules** — find relevant rule(s)
3. **Score confidence** — based on clarity and specificity
4. **Select workflow** — choose the most appropriate workflow
5. **Request clarification** — if confidence is too low to proceed

## Multi-Context Scenarios

When multiple contexts are detected:

1. **Prioritize by severity** — service outage > performance issue
2. **Check for root cause relationships** — disk full → service fails
3. **Present options** — if unrelated issues, list both
4. **Work sequentially** — fix root causes first

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial context interpretation guide |