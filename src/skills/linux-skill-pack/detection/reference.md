# Linux Engineering — Detection Reference

## Purpose

This directory contains detection rule specifications and matching criteria for the Linux engineering skill pack.

## Detection Rule Catalog

| Rule ID | Name | Confidence | Primary Workflow |
|---------|------|------------|-----------------|
| linux-service-failure | Service Failure | 0.90 | service-not-starting |
| linux-high-cpu | High CPU Usage | 0.85 | high-cpu-usage |
| linux-disk-full | Disk Space Full | 0.92 | disk-full |
| linux-ssh-failure | SSH Access Denied | 0.88 | ssh-access-denied |
| linux-network-issue | Network Connectivity | 0.87 | network-connectivity |
| linux-memory-issue | Memory Exhaustion | 0.89 | memory-exhaustion |
| linux-boot-failure | Boot Failure | 0.82 | boot-failure |
| linux-package-failure | Package Failure | 0.86 | package-installation |
| linux-performance-degraded | Performance Degradation | 0.75 | system-slow |
| linux-linux-context | General Linux Context | 0.60 | (fallback) |

## Detection Flow

```
User Input
    │
    ├─→ Extract keywords and phrases
    │
    ├─→ Match against detection rule patterns
    │
    ├─→ Score confidence (0.0 - 1.0)
    │
    ├─→ If confidence >= 0.70 → Select workflow
    │
    ├─→ If confidence < 0.70 → Request clarification
    │
    └─→ If no match → Use fallback rule
```

## Matching Criteria

### Keyword Matching
Rules match against user-provided keywords and phrases:
- Exact matches: "service failed", "disk full"
- Pattern matches: "cpu.*high", "memory.*exhaust"
- Context keywords: "systemctl", "journalctl", "df"

### Confidence Factors
Confidence is affected by:
- **Specificity**: More specific = higher confidence
- **Clarity**: Clear error message = higher confidence
- **Completeness**: More evidence = higher confidence
- **Ambiguity**: Vague description = lower confidence

## Rule Maintenance

Rules should be reviewed and updated:
- When new issue types are discovered
- When confidence scores prove inaccurate
- When distribution changes affect detection
- Quarterly review cycle

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial detection reference |