# MySQL Diagnostic Reasoning Guide

## Purpose

This document provides the diagnostic reasoning framework for MySQL troubleshooting. It describes how to observe symptoms, form hypotheses, validate hypotheses, and recommend remediation — always as advisory guidance, never executing commands directly.

## Diagnostic Reasoning Model

The MySQL diagnostic reasoning follows a structured 4-phase model:

```
Observation → Hypothesis → Validation → Remediation
    ↓           ↓           ↓           ↓
 Evidence   Root Cause  Evidence    Advisory
 Collection  Analysis  Confirmation  Guidance
```

### Phase 1: Observation — Evidence Collection

Collect evidence before forming conclusions. Evidence sources include:

1. **Server Status**: `SHOW GLOBAL STATUS` — connection count, queries, cache hits
2. **Server Variables**: `SHOW GLOBAL VARIABLES` — configuration check
3. **Process List**: `SHOW FULL PROCESSLIST` — running queries, lock waits
4. **Error Log**: Last 100-200 lines — crash info, warnings, replication errors
5. **InnoDB Status**: `SHOW ENGINE INNODB STATUS\G` — locks, transactions, buffer pool
6. **Replication Status**: `SHOW REPLICA STATUS\G` — lag, thread status, errors
7. **Disk Space**: Filesystem usage — data dir, binlog, redo log
8. **Query Plan**: `EXPLAIN ANALYZE` for problematic queries — execution path

**Evidence Confidence**: Multiple independent evidence sources increase diagnostic confidence.

### Phase 2: Hypothesis — Root Cause Analysis

Form hypotheses based on evidence, prioritized by likelihood and impact:

#### Symptom → Hypothesis Mapping

| Symptom | Likely Cause | Confidence | Secondary Checks |
|---------|-------------|------------|-----------------|
| ERROR 1040: Too many connections | max_connections exhausted, connection leak | High | Check `Threads_connected`, `Aborted_connects` |
| ERROR 1045: Access denied | Wrong credentials, host mismatch, plugin mismatch | High | Check user/host in mysql.user, auth plugin |
| ERROR 2006: Server gone away | Crash, OOM, network drop, packet too large | Medium | Check error log, process status, `max_allowed_packet` |
| ERROR 1205: Lock wait timeout | Blocking transaction, missing index | High | Check `SHOW ENGINE INNODB STATUS`, `data_lock_waits` |
| ERROR 1213: Deadlock | Lock ordering conflict, large transactions | High | Check `LATEST DETECTED DEADLOCK` in status |
| Replication lag increasing | Write-heavy master, slow replica, network | Medium | Check replica threads, CPU, disk I/O |
| Slow query | Missing index, full table scan, stats stale | Medium | Check EXPLAIN, slow query log |
| Server won't start | Corrupt data, bad config, disk full | High | Check error log for crash details |

### Phase 3: Validation — Confirm Hypothesis

Validate each hypothesis with targeted evidence:

#### Validation Techniques

1. **Isolation Testing**: Isolate the variable (e.g., test on replica vs master)
2. **Historical Comparison**: Compare current metrics with known-good baselines
3. **Incremental Testing**: Test changes one at a time with monitoring between each
4. **Cross-Reference**: Correlate multiple evidence sources for confirmation
5. **Staging Verification**: Reproduce and validate in non-production

#### Confidence Scoring

After validation, assign confidence to the diagnosis:

- **High (0.85+)**: Multiple evidence sources confirm root cause
- **Medium (0.60-0.84)**: Strong evidence but some uncertainty remains
- **Low (below 0.60)**: Weak evidence, consider alternative hypotheses

### Phase 4: Remediation — Advisory Guidance

Recommend remediation based on validated root cause:

#### Remediation Priority

1. **Immediate Mitigation**: Stop the bleeding (kill blocking queries, increase limits, restart if needed)
2. **Short-term Fix**: Apply correct configuration, add indexes, fix application issues
3. **Long-term Prevention**: Architecture changes, monitoring, process improvements

#### Risk Assessment

Every remediation recommendation includes:
- **Risk Level**: Low/Medium/High/Critical
- **Impact**: What changes, what stays the same
- **Rollback**: How to revert if needed
- **Verification**: How to confirm success

---

## Decision Trees

### Decision Tree 1: Connection Failures

```
User reports connection failure
│
├─ ERROR 1040 (Too many connections)?
│   ├─ YES → Check Threads_connected vs max_connections
│   │   ├─ Near max_connections → Increase max_connections or add connection pooling
│   │   └─ Well below max → Check for connection leak in application
│   │
│   └─ NO → Continue
│
├─ ERROR 1045 (Access denied)?
│   ├─ YES → Verify user/host combination in mysql.user
│   │   ├─ User exists → Check password, account locked status
│   │   └─ User doesn't exist → Check authentication plugin compatibility
│   │
│   └─ NO → Continue
│
├─ ERROR 2006 (Server gone away)?
│   ├─ YES → Check mysqld process running?
│   │   ├─ No → Check error log for crash reason
│   │   └─ Yes → Check network, max_allowed_packet, wait_timeout
│   │
│   └─ NO → Continue
│
└─ No specific error code?
    ├─ Check error log for authentication/connection errors
    └─ Check application logs for connection pool errors
```

### Decision Tree 2: Performance Degradation

```
User reports slow queries
│
├─ All queries slow or specific queries?
│   ├─ All queries → Check server resources (CPU, memory, disk I/O)
│   │   ├─ High CPU → Check innodb_thread_concurrency, thread_pool_*
│   │   ├─ High I/O → Check innodb_buffer_pool_size, disk type
│   │   └─ High Memory → Check buffer pool, sort buffer, join buffer
│   │
│   └─ Specific queries → Continue to EXPLAIN analysis
│
├─ EXPLAIN shows full table scan (type=ALL)?
│   ├─ YES → Add appropriate indexes
│   │   ├─ Check columns in WHERE, JOIN, ORDER BY
│   │   └─ Verify index selectivity
│   │
│   └─ NO → Continue
│
├─ EXPLAIN shows Using filesort or Using temporary?
│   ├─ YES → Optimize query to use index for sorting
│   │   ├─ Add composite index covering ORDER BY columns
│   │   └─ Consider reducing GROUP BY scope
│   │
│   └─ NO → Continue
│
├─ Tables have stale statistics?
│   ├─ YES → Run ANALYZE TABLE
│   │
│   └─ NO → Continue
│
└─ Check slow query log for patterns
    └─ Analyze with pt-query-digest for trends
```

### Decision Tree 3: Replication Issues

```
Replication issue reported
│
├─ SHOW REPLICA STATUS shows Slave_IO_Running = NO?
│   ├─ YES → Check network connectivity to master
│   │   ├─ Network issue → Fix network, restart IO thread
│   │   └─ Network OK → Check binlog file exists on master
│   │       ├─ File missing → Find closest available binlog, use CHANGE REPLICATION SOURCE
│   │       └─ File exists → Restart IO thread
│   │
│   └─ NO → Continue
│
├─ SHOW REPLICA STATUS shows Slave_SQL_Running = NO?
│   ├─ YES → Check Last_SQL_Error for specific error
│   │   ├─ Data inconsistency → Fix data, use pt-table-sync
│   │   ├─ SQL syntax error → Fix statement, use sql_slave_skip_counter (if safe)
│   │   └─ Duplicate key → Resolve duplicate, restart SQL thread
│   │
│   └─ NO → Continue
│
├─ Seconds_Behind_Master increasing?
│   ├─ YES → Check replica CPU, disk I/O, network latency
│   │   ├─ Replica resources → Scale up replica
│   │   ├─ Master write-heavy → Consider parallel replication
│   │   └─ Network → Check bandwidth, latency
│   │
│   └─ NO → Monitoring
│
└─ Check relay log for corruption
    └─ If corrupt → Purge relay logs, restart IO thread
```

### Decision Tree 4: InnoDB Issues

```
InnoDB issue reported
│
├─ Error log shows crash recovery?
│   ├─ YES → Check error log for recovery outcome
│   │   ├─ Recovery succeeded → Monitor for recurrence
│   │   └─ Recovery failed → Check disk, try innodb_force_recovery
│   │
│   └─ NO → Continue
│
├─ ERROR 1213 (Deadlock)?
│   ├─ YES → Check LATEST DETECTED DEADLOCK in INNODB STATUS
│   │   ├─ Recurring deadlock → Fix application lock ordering
│   │   ├─ Missing index → Add indexes to reduce lock scope
│   │   └─ Large transaction → Reduce transaction scope
│   │
│   └─ NO → Continue
│
├─ ERROR 1205 (Lock wait timeout)?
│   ├─ YES → Check data_lock_waits for blocking PID
│   │   ├─ Blocking query identified → Kill or wait for completion
│   │   ├─ Long-running transaction → Check application commit logic
│   │   └─ Missing index causing table lock → Add index
│   │
│   └─ NO → Continue
│
├─ Buffer pool pressure?
│   ├─ YES → Check Innodb_buffer_pool_read_requests/reads ratio
│   │   ├─ Hit ratio < 99% → Increase innodb_buffer_pool_size
│   │   └─ Hit ratio OK → Check for full table scans
│   │
│   └─ NO → Continue
│
└─ Corruption suspected?
    └─ Check error log, attempt restore from backup + binlogs
```

---

## Root Cause Classification

### Application-Level Issues

| Indicator | Evidence | Likely Cause |
|-----------|----------|-------------|
| Error only during specific queries | EXPLAIN shows full scan | Missing index, poor query |
| Connection pool exhaustion | High Threads_connected, low usage | Connection leak |
| Random failures | Intermittent errors in logs | Network, timeout, resource |

### Configuration Issues

| Indicator | Evidence | Likely Cause |
|-----------|----------|-------------|
| All queries slow after change | SHOW VARIABLES shows new values | Bad configuration |
| Server won't start | Error log on startup | Config error, corruption |
| Unexpected behavior | Mismatched GLOBAL vs SESSION | Wrong variable scope |

### Infrastructure Issues

| Indicator | Evidence | Likely Cause |
|-----------|----------|-------------|
| Disk space alerts | df -h shows 100% | No space left |
| OOM events | /var/log/messages, kernel logs | Memory exhaustion |
| Slow replica | High I/O wait on replica | Disk bottleneck |

### Data Issues

| Indicator | Evidence | Likely Cause |
|-----------|----------|-------------|
| Duplicate key error | ERROR 1062 | Data integrity |
| Corrupt pages | InnoDB corruption messages | Hardware, crash |
| Missing tables | ERROR 1146 | Accidental DROP |

---

## Diagnostic Confidence Guidelines

### High Confidence (0.85+)
- Multiple independent evidence sources point to same root cause
- Error code directly indicates the problem
- Clear error message in error log with actionable details
- Reproducible in staging environment

### Medium Confidence (0.60-0.84)
- Strong evidence but some alternative explanations possible
- Symptom could have multiple root causes
- Requires additional investigation to confirm
- Partial evidence from logs and metrics

### Low Confidence (below 0.60)
- Weak or ambiguous evidence
- Multiple plausible root causes
- Information insufficient for confident diagnosis
- Requires additional data collection

---

## References

- [MySQL 8.0 Troubleshooting Guide](https://dev.mysql.com/doc/refman/8.0/en/troubleshooting-innodb.html)
- [MySQL 8.0 Error Handling](https://dev.mysql.com/doc/refman/8.0/en/innodb-error-handling.html)
- [MySQL 8.0 Performance Schema](https://dev.mysql.com/doc/refman/8.0/en/performance-schema.html)
- [MySQL 8.0 InnoDB Locking](https://dev.mysql.com/doc/refman/8.0/en/innodb-locking.html)