# Windows Engineering — Detection Reference

## Purpose

This document defines the detection rules for the Windows engineering skill pack.

## Detection Rule Catalog

### Service Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| service-stopped | Service state != Running | Service stopped unexpectedly | Critical |
| service-failed | Service exit code != 0 | Service terminated with error | High |
| service-dependent-failed | Dependency service stopped | Required service not running | High |
| service-access-denied | Service start blocked | Permission or policy issue | High |

### Event Log Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| event-error-count | >10 errors in 1 hour | High error rate | Medium |
| event-critical | Critical event in System | Critical system event | Critical |
| event-application-failure | App crash event (ID 1000) | Application crash detected | High |
| event-service-failure | Event 7023/7024 | Service failure logged | High |
| event-disk-warning | Event 2019/2020 | Low disk space warning | Medium |

### DNS Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| dns-service-down | DNS service stopped | DNS Server not running | High |
| dns-resolution-failed | nslookup fails | Name resolution failing | High |
| dns-cache-corrupt | DNS cache issues | DNS cache corruption | Medium |
| dns-zone-error | Zone transfer failure | AD-integrated zone issue | High |

### AD Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| ad-replication-failed | Repadmin shows errors | AD replication broken | Critical |
| ad-login-failed | User login fails | Authentication failure | Critical |
| ad-dc-unreachable | Cannot reach DC | Domain Controller offline | Critical |
| ad-fsmo-offline | FSMO role holder down | Critical role unavailable | Critical |
| ad-gpo-error | GPO processing failed | Group Policy error | Medium |

### Network Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| network-disconnected | Adapter status Down | Network adapter down | High |
| ip-conflict | IP address conflict | Duplicate IP detected | High |
| gateway-unreachable | Cannot reach gateway | Default gateway issue | High |
| firewall-block | Port blocked | Firewall preventing access | Medium |

### Storage Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| disk-nearly-full | Volume > 90% used | Critical disk space | Critical |
| disk-writing-error | Write operations fail | Disk I/O error | Critical |
| volume-offline | Volume not mounted | NTFS volume offline | High |
| ntfs-corruption | CheckDisk needed | Filesystem corruption | High |

### Performance Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| high-cpu | CPU > 90% sustained | CPU overutilization | Medium |
| high-memory | Available RAM < 10% | Memory pressure | High |
| high-disk-io | Disk queue > 2 | Disk bottleneck | Medium |
| high-paging | Page file usage > 90% | Memory swapping | High |

### Security Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| failed-logon-spike | >10 failed logons/hr | Potential brute force | High |
| admin-account-locked | Admin account locked | Account lockout event | Medium |
| privilege-escalation | Admin rights change | Privilege change detected | Critical |
| windows-defender-off | AV disabled | Endpoint protection off | High |

## Detection Priority

Failures are prioritized by impact:

1. **Critical** — Immediate action required, potential data loss or service outage
2. **High** — Urgent attention needed, service degraded
3. **Medium** — Investigation required, performance impact
4. **Low** — Monitoring recommended, no immediate action
5. **Info** — Observational, no action needed

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial detection rule catalog |