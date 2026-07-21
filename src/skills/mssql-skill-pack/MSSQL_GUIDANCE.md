# SQL Server Engineering Guidance

## Overview

This document provides engineering guidance for working with Microsoft SQL Server in enterprise environments. It covers reasoning, safety, and recommendations for database administration tasks.

## Core Principles

### 1. Evidence-First Approach

Always collect evidence before making recommendations:
- **Wait statistics** — Primary diagnostic for performance issues
- **Query plans** — Understand actual execution paths
- **DMV snapshots** — Capture state at time of incident
- **Error logs** — Identify root causes
- **Performance counters** — Measure resource utilization

### 2. Least Privilege Access

- Provide minimum required permissions
- Use database roles instead of individual grants
- Separate development, staging, and production access
- Regular permission reviews

### 3. Maintenance Windows

- Perform schema changes during maintenance windows
- Schedule heavy operations during off-peak hours
- Never modify production without authorization
- Always notify stakeholders of changes

## Diagnostic Reasoning

### Step 1: Identify the Symptom

Common symptoms and their typical causes:

| Symptom | Likely Cause |
|---------|-------------|
| Slow queries | Missing indexes, bad plans, blocking |
| High CPU | Compute-intensive queries, parallelism issues |
| High I/O | Full table scans, missing indexes |
| High Memory | Large result sets, inefficient plans |
| Deadlocks | Lock contention, transaction scope |
| Blocking | Long-running transactions, missing indexes |
| Log full | Missing log backups, long transactions |

### Step 2: Gather Evidence

```sql
-- Priority 1: Wait statistics
SELECT TOP 10 wait_type, wait_time_ms, waiting_tasks_count
FROM sys.dm_os_wait_stats
ORDER BY wait_time_ms DESC;

-- Priority 2: Resource-heavy queries
SELECT TOP 10 qs.total_logical_reads, qs.execution_count
FROM sys.dm_exec_query_stats qs
ORDER BY qs.total_logical_reads DESC;

-- Priority 3: Blocking chains
SELECT blocking.session_id, blocked.session_id, blocking_wait.wait_type
FROM sys.dm_os_waiting_tasks blocking_wait
JOIN sys.dm_exec_sessions blocked ON blocking_wait.session_id = blocked.session_id
WHERE blocking_wait.blocking_session_id IS NOT NULL;
```

### Step 3: Root Cause Analysis

After gathering evidence, trace the symptom to its root cause:

- **Wait type CXPACKET** → Parallelism issue → Check MAXDOP settings
- **Wait type PAGEIOLATCH** → I/O bottleneck → Check storage, consider indexes
- **Wait type LCK_M_* → Lock contention → Check for missing indexes, long transactions
- **Wait type RESOURCE_SEMAPHORE → Memory pressure → Check memory grants, memory grants

### Step 4: Validate Solution

Before recommending any action:
- Verify the fix addresses the root cause
- Consider impact on other workloads
- Ensure rollback plan exists
- Test in non-production if possible

## Safety Guidelines

### Risk Assessment Matrix

| Change Type | Risk | Approval Required |
|------------|------|-------------------|
| Read-only queries | Low | None |
| Statistics update | Low | DBA |
| Index add/rebuild | Medium | DBA + change request |
| Schema change | High | DBA + change request + approval |
| Data modification | High | DBA + change request + approval |
| Configuration change | Medium | DBA + change request |
| Backup/restore | Low | DBA |
| AG failover | Critical | DBA + operations |

### Pre-Change Checklist

- [ ] Evidence collected and analyzed
- [ ] Change window scheduled
- [ ] Rollback plan documented
- [ ] Backup verified
- [ ] Stakeholders notified
- [ ] Monitoring in place
- [ ] Test completed (non-production)
- [ ] Performance baseline captured

### Post-Change Validation

- [ ] Change applied successfully
- [ ] No errors in ERRORLOG
- [ ] Performance metrics improved
- [ ] Application functioning normally
- [ ] Monitoring alerts clear
- [ ] Documentation updated

## Recommendation Confidence

Confidence scores reflect certainty in recommendations:

- **High (0.85-1.0)** — Well-documented pattern, clear evidence, low risk
- **Medium (0.60-0.84)** — Pattern recognized but additional validation needed
- **Low (0.35-0.59)** — Weak evidence, multiple possible causes, higher risk
- **Very Low (<0.35)** — Unclear pattern, escalate to senior engineer

### Confidence Factors

| Factor | Increases Confidence | Decreases Confidence |
|--------|---------------------|---------------------|
| Evidence | Consistent DMV data | Inconsistent/contradictory data |
| Pattern | Well-documented issue | Novel/unusual scenario |
| Testing | Validated in non-prod | No validation available |
| Risk | Low-risk change | High-risk change |
| History | Similar successful fix | First attempt |

## Version-Specific Guidance

### SQL Server 2017

- Enable Intelligent Query Processing features
- Consider cross-platform Always On
- Use Python/R integration cautiously with security
- Monitor tempDB with BPE (deprecated but functional)

### SQL Server 2019

- Enable smart memory grant to reduce over-grant
- Use batch mode on rowstore for analytical queries
- Consider Accelerated Database Recovery for fast recovery
- Monitor automatic tuning recommendations

### SQL Server 2022

- Use vectorized batch mode for analytic queries
- Enable Query Performance Advisor
- Use Intelligent Data Protection features
- Consider Azure SQL Managed Instance integration

## Common Patterns and Recommendations

### Pattern 1: High CXPACKET Waits
- **Symptoms:** Parallel query waits, inconsistent performance
- **Root Cause:** Excessive parallelism, resource contention
- **Recommendation:** Reduce MAXDOP to 8, check CPU utilization
- **Confidence:** High

### Pattern 2: High PAGEIOLATCH Waits
- **Symptoms:** Slow queries, disk I/O wait
- **Root Cause:** Insufficient I/O, missing indexes causing scans
- **Recommendation:** Check storage performance, add missing indexes
- **Confidence:** Medium

### Pattern 3: High RESOURCE_SEMAPHORE Waits
- **Symptoms:** Query waits for memory grant
- **Root Cause:** Insufficient memory, large sort/hash operations
- **Recommendation:** Check memory grants, optimize queries for memory
- **Confidence:** Medium

### Pattern 4: Blocking Chains
- **Symptoms:** Application timeouts, high LCK_M_* waits
- **Root Cause:** Long-running transactions, missing indexes
- **Recommendation:** Shorten transactions, add indexes, check isolation levels
- **Confidence:** Medium

### Pattern 5: tempDB Contention
- **Symptoms:** TempDB full errors, PAGEIOLATCH waits
- **Root Cause:** Insufficient tempDB files, version store pressure
- **Recommendation:** Add tempDB files, review transaction isolation
- **Confidence:** High

## Escalation Triggers

Escalate when:
- Production data at risk of loss
- Service disruption lasting > 15 minutes
- Security breach suspected
- Hardware failure detected
- Cannot resolve after 30 minutes of troubleshooting

## Documentation Requirements

All SQL Server engagements require documentation of:
- Problem description and impact
- Evidence collected (DMV queries, logs)
- Root cause analysis
- Resolution steps
- Prevention measures
- Lessons learned

## Conclusion

Effective SQL Server engineering follows evidence-based reasoning, conservative change management, and continuous improvement. Always validate findings with multiple sources, consider the broader system impact, and maintain clear documentation of all interventions.