# MySQL Workflows

## Overview

This directory contains detailed state-machine troubleshooting workflows for MySQL. Each workflow follows the STANDARD WORKFLOW MODEL:

**Observation** → **Interpretation** → **Possible Causes** → **Evidence Required** → **Investigation Order** → **Recommended Actions** → **Expected Findings** → **Possible Conclusions** → **Recommended Next Step** → **Expected Outcome** → **Risk Warnings** → **Documentation References**

## Workflow Index

| # | Workflow ID | Symptom | Severity | Est. Time | File |
|---|-----------|---------|----------|-----------|------|
| 1 | WF-001 | High Connection Count | High | 15-30 min | high-connection-count.md |
| 2 | WF-002 | Replication Lag | Medium | 10-20 min | replication-lag.md |
| 3 | WF-003 | Slow Query Performance | Medium | 20-45 min | slow-query.md |
| 4 | WF-004 | Lock Contention / Deadlock | Critical | 10-20 min | lock-contention.md |
| 5 | WF-005 | Replication Failure | Critical | 15-45 min | replication-failure.md |
| 6 | WF-006 | Buffer Pool Pressure | Medium | 10-15 min | buffer-pool-pressure.md |
| 7 | WF-007 | Disk Space Exhaustion | Critical | 10-20 min | disk-space-exhaustion.md |
| 8 | WF-008 | High CPU Usage | Medium | 15-30 min | high-cpu-usage.md |
| 9 | WF-009 | Backup Failure | High | 15-30 min | backup-failure.md |
| 10 | WF-010 | Binary Log Issues | Medium | 10-20 min | binary-log-issues.md |

## How to Use Workflows

1. **Identify the observed symptom** from the workflow index above
2. **Open the corresponding workflow** file
3. **Follow the structured investigation path** step by step
4. **Collect evidence** at each step — never skip evidence collection
5. **Apply recommendations** with appropriate confidence scoring
6. **Document findings** and outcomes for future reference

## Workflow Structure

Each workflow follows this standardized structure:

1. **Scenario**: Real-world problem context
2. **Observation**: Symptoms and indicators to look for
3. **Interpretation**: What the observations mean
4. **Possible Causes**: Ranked by likelihood
5. **Evidence Required**: What to check and why
6. **Investigation Order**: Prioritized steps
7. **Recommended Actions**: Specific remediation steps
8. **Expected Findings**: What to expect at each step
9. **Possible Conclusions**: Outcomes at each decision point
10. **Recommended Next Step**: Based on findings
11. **Expected Outcome**: Post-resolution state
12. **Risk Warnings**: Safety considerations
13. **Documentation References**: Links to MySQL documentation

## Workflow Selection Guide

### By Severity

| Severity | Action |
|----------|--------|
| Critical (server down, data loss risk) | WF-004, WF-005, WF-007 |
| High (service degraded) | WF-001, WF-009 |
| Medium (performance issues) | WF-002, WF-003, WF-006, WF-008, WF-010 |

### By Symptom Category

| Category | Workflows |
|----------|-----------|
| Connection / Access | WF-001, WF-004, WF-005 |
| Replication | WF-002, WF-005, WF-010 |
| Performance | WF-003, WF-006, WF-008 |
| Storage | WF-007 |
| Data Integrity | WF-004, WF-005 |
| Maintenance | WF-009 |

## References

- STANDARD WORKFLOW MODEL: See MySQL_WORKFLOWS.md
- MySQL Workflows: https://dev.mysql.com/doc/refman/8.0/en/