# MySQL Diagnostic Procedures Guide

## Purpose

This guide defines the standard diagnostic procedures for MySQL troubleshooting. Each procedure follows a structured approach: identify the symptom, collect evidence, diagnose the root cause, and recommend resolution.

## Diagnostic Framework

All MySQL diagnostics follow the STANDARD WORKFLOW MODEL:

1. **Observe**: Identify symptoms and indicators
2. **Interpret**: Analyze the observed data
3. **Hypothesize**: Generate possible causes
4. **Collect Evidence**: Gather confirming data
5. **Diagnose**: Narrow to root cause
6. **Recommend**: Propose resolution with risk assessment
7. **Verify**: Confirm the fix works

## Diagnostic Procedure Inventory

### DP-01: Connection Failure Diagnosis

**Symptom**: Application cannot connect to MySQL server

**Evidence Collection**:
1. Check `Threads_connected` vs `max_connections`
2. Review error log for `Access denied` entries
3. Verify network connectivity (ping, telnet to port 3306)
4. Check `show grants` for the connecting user
5. Verify DNS resolution for the host specification

**Decision Points**:
- If `max_connections` reached: Check `wait_timeout`, increase `max_connections`, or add connection pool
- If `Access denied`: Check credentials, user host specification, authentication plugin compatibility
- If network failure: Check firewall, security groups, MySQL bind-address
- If DNS failure: Use IP address instead of hostname

### DP-02: Performance Degradation Diagnosis

**Symptom**: Queries take longer than expected, high latency, increased error rates

**Evidence Collection**:
1. Enable and review slow query log
2. Check `SHOW GLOBAL STATUS` for key metrics:
   - `Threads_running`: High value indicates contention
   - `Innodb_buffer_pool_reads / Innodb_buffer_pool_read_requests`: Buffer pool hit ratio
   - `Handler_read_rnd`: High value indicates full table scans
   - `Connections` vs `Threads_connected`: Connection churn
3. Run `EXPLAIN` on identified slow queries
4. Check `SHOW ENGINE INNODB STATUS` for lock contention
5. Review `sys.schema_unused_indexes` for index opportunities

**Decision Points**:
- If low buffer pool hit ratio: Increase `innodb_buffer_pool_size`
- If high filesort: Add or optimize indexes for ORDER BY
- If full table scans: Add missing indexes
- If lock contention: Reduce transaction scope, add indexes on WHERE columns
- If high connection churn: Increase `thread_cache_size`, optimize connection management

### DP-03: Replication Lag Diagnosis

**Symptom**: `Seconds_Behind_Source` on replica is increasing or high

**Evidence Collection**:
1. Check `SHOW REPLICA STATUS` — both IO and SQL threads running?
2. Check source: `SHOW BINARY LOGS`, `SHOW MASTER STATUS`
3. Check replica: `SHOW SLAVE STATUS` — `Seconds_Behind_Source`, `Last_Error`
4. Check replica I/O: Is relay log growing?
5. Check replica SQL: Are large transactions present?
6. Review replica hardware resources (CPU, I/O, memory)

**Decision Points**:
- If IO thread stopped: Check network connectivity, binlog availability
- If SQL thread stopped: Check `Last_Error`, fix data inconsistency
- If lag increasing but both threads running:
  - Large transactions on source: Split into smaller transactions
  - Replica overload: Scale replica hardware or add read replicas
  - Network latency: Check network path between source and replica

### DP-04: Disk Space Emergency Diagnosis

**Symptom**: MySQL error about disk space, server fails to start, or disk > 90% full

**Evidence Collection**:
1. `df -h /path/to/mysql/data`
2. `du -sh /path/to/mysql/data/*` — largest consumers
3. Check binary logs: `SHOW BINARY LOGS`
4. Check InnoDB files: `.ibd` files in data directory
5. Check error log for disk-related errors

**Decision Points**:
- Binary logs: Purge old logs, configure `binlog_expire_logs_seconds`
- Large `.ibd` files: Consider table partitioning, archive old data
- General log enabled: Disable in production
- Temporary files: Check `tmpdir` for large temp files

### DP-05: InnoDB Corruption Diagnosis

**Symptom**: Error log shows page corruption, crashes, or data integrity errors

**Evidence Collection**:
1. Check error log for InnoDB-specific errors
2. `CHECK TABLE` on affected tables
3. `SHOW ENGINE INNODB STATUS` for corruption details
4. Check disk health (SMART data, filesystem errors)
5. Review recent changes (crash, hardware replacement, filesystem resize)

**Decision Points**:
- Minor corruption: Try `innodb_force_recovery` levels 1-2
- Data-level corruption: `ALTER TABLE` to rebuild affected tables
- Page-level corruption: Restore from backup, replay binlogs
- Filesystem corruption: Check and repair filesystem, check disk

### DP-06: Lock Contention Diagnosis

**Symptom**: High lock wait times, `innodb_lock_wait_timeout` errors, deadlocks

**Evidence Collection**:
1. `SHOW ENGINE INNODB STATUS\G` — LATEST DETECTED DEADLOCK section
2. `SELECT * FROM performance_schema.data_lock_waits` — current waits
3. `SELECT * FROM performance_schema.data_locks` — current locks
4. `SHOW PROCESSLIST` — identify long-running queries
5. Review application transaction patterns

**Decision Points**:
- Deadlocks: Ensure consistent lock ordering, add indexes, reduce transaction scope
- Lock wait timeouts: Add missing indexes, split large transactions, check for table scans
- General contention: Scale out reads, use partitioning, review schema design

### DP-07: Authentication Failure Diagnosis

**Symptom**: `ERROR 1045 (28000): Access denied for user` or connection dropped during auth

**Evidence Collection**:
1. Check error log for authentication errors
2. `SELECT user, host, plugin, account_locked FROM mysql.user WHERE user='username'`
3. Verify password matches (test with known good credential)
4. Check authentication plugin compatibility with client driver
5. Verify user host specification allows the connection source

**Decision Points**:
- Wrong password: Reset password via `ALTER USER`
- Authentication plugin mismatch: Create user with `mysql_native_password` for legacy clients, or upgrade client driver
- Host mismatch: Update user's host specification
- Account locked: `ALTER USER ... ACCOUNT UNLOCK`
- Expired password: `ALTER USER ... PASSWORD EXPIRE NEVER` or reset password

## Diagnostic Tools Reference

### Command-Line Tools

| Tool | Purpose | Key Usage |
|------|---------|----------|
| `mysql` | Client for diagnostics | `SHOW STATUS`, `SHOW PROCESSLIST`, `EXPLAIN` |
| `mysqladmin` | Server administration | `status`, `processlist`, `variables` |
| `mysqlbinlog` | Binary log analysis | `--start-datetime`, `--stop-datetime` |
| `innochecksum` | Data page verification | Check InnoDB data files |
| `myisamchk` | MyISAM table repair | `--check`, `--recover` |

### System Tools

| Tool | Purpose | Key Usage |
|------|---------|----------|
| `iostat` | Disk I/O monitoring | `iostat -x 1` |
| `vmstat` | System performance | `vmstat 1` |
| `top` | Process resource usage | Sort by CPU or memory |
| `netstat` | Network connections | `netstat -anp \| grep 3306` |
| `ss` | Socket statistics | `ss -tnp \| grep 3306` |

### MySQL Diagnostic Statements

| Statement | Purpose |
|-----------|---------|
| `SHOW ENGINE INNODB STATUS` | Comprehensive InnoDB diagnostics |
| `SHOW GLOBAL STATUS` | Runtime statistics |
| `SHOW GLOBAL VARIABLES` | Configuration parameters |
| `SHOW PROCESSLIST` | Active connections and queries |
| `SHOW SLAVE STATUS` | Replication status |
| `SHOW OPEN TABLES` | Open table cache usage |
| `SHOW BINARY LOGS` | Binary log inventory |
| `CHECK TABLE` | Table integrity check |
| `ANALYZE TABLE` | Update table statistics |

### Performance Schema Queries

```sql
-- Current wait events
SELECT * FROM performance_schema.events_waits_current
WHERE EVENT_NAME LIKE '%lock%' OR EVENT_NAME LIKE '%io%';

-- Long-running statements
SELECT * FROM performance_schema.events_statements_history
WHERE TIMER_WAIT > 1000000000000  -- 1 second
ORDER BY TIMER_WAIT DESC
LIMIT 10;

-- Current transactions
SELECT * FROM performance_schema.events_transactions_current;

-- Table I/O statistics
SELECT * FROM sys.schema_table_statistics;
```

## Diagnostic Checklist

Before diagnosing any MySQL issue:

- [ ] Check error log first — look for ERROR-level entries
- [ ] Verify disk space is adequate (> 15% free)
- [ ] Check if mysqld is running (`pgrep mysqld`, `systemctl status mysqld`)
- [ ] Check system memory (`free -h`)
- [ ] Check CPU utilization (`top`, `mpstat`)
- [ ] Check network connectivity to MySQL server
- [ ] Verify MySQL version and patch level
- [ ] Check for recent changes (config, schema, data volume)
- [ ] Review monitoring alerts for correlated issues

## Diagnostic Report Template

```
Diagnostic Report: [Issue Summary]
Date: [YYYY-MM-DD]
MySQL Version: [version]
Server: [hostname/IP]

## Symptoms
[Describe observed symptoms]

## Evidence Collected
1. [Evidence item 1 with analysis]
2. [Evidence item 2 with analysis]
3. [Evidence item 3 with analysis]

## Root Cause
[Identified root cause]

## Impact
[Scope and severity of impact]

## Resolution
[Step-by-step resolution]

## Verification
[How to verify the fix]

## Prevention
[Steps to prevent recurrence]

## Confidence
[Overall confidence in diagnosis: 0.XX]
```

## References

- [MySQL 8.0 Reference Manual: Error Log](https://dev.mysql.com/doc/refman/8.0/en/error-log.html)
- [MySQL 8.0 Reference Manual: Performance Schema Tables](https://dev.mysql.com/doc/refman/8.0/en/performance-schema-tables.html)
- [MySQL 8.0 Reference Manual: InnoDB Diagnostics](https://dev.mysql.com/doc/refman/8.0/en/innodb-monitor.html)
- [MySQL 8.0 Reference Manual: Replication Diagnostics](https://dev.mysql.com/doc/refman/8.0/en/replication-solutions-replication-errors.html)