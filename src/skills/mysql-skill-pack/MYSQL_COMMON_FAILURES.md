# MySQL Common Failures

## Purpose

This document catalogs common MySQL failure patterns, their symptoms, detection methods, root causes, and recommended remediation approaches.

---

## Failure Pattern 1: Too Many Connections (ERROR 1040)

### Symptom
Applications receive errors: `ERROR 1040 (08004): Too many connections`. New client connections are rejected.

### Affected Components
- Connection management
- `max_connections` setting
- Connection pool configuration

### Detection
- CLI output: `ERROR 1040 (08004): Too many connections`
- `SHOW STATUS LIKE 'Max_used_connections'` — compare with `max_connections`
- `SHOW STATUS LIKE 'Threads_connected'` — may be near `max_connections`
- Error log: `Can't create a new thread (errno 11)`

### Root Causes
1. Application not closing connections properly (connection leak)
2. `max_connections` set too low for workload
3. `wait_timeout` too high — idle connections held too long
4. Sudden traffic spike exceeding capacity
5. Long-running queries holding connections

### Remediation Guidance
- **Immediate**: Increase `max_connections` if not at OS limit
- **Immediate**: Kill idle connections in Sleep state with high `wait_timeout`
- **Short-term**: Enable connection pooling (ProxySQL, PgBouncer)
- **Long-term**: Fix application connection leaks, tune `wait_timeout`, increase `max_connections`

### Risk Assessment: High (production connectivity impact)

---

## Failure Pattern 2: Authentication Denied (ERROR 1045)

### Symptom
Client receives: `ERROR 1045 (28000): Access denied for user 'user'@'host' (using password: YES|NO)`.

### Affected Components
- User account
- Authentication plugin
- Host-based access control

### Detection
- CLI output: `ERROR 1045 (28000)`
- Error log: `Access denied for user`

### Root Causes
1. Wrong password
2. User doesn't exist for the specified host
3. Authentication plugin mismatch (mysql_native_password vs caching_sha2_password)
4. Account is locked or expired
5. Host-based ACL restriction

### Remediation Guidance
- **Immediate**: Verify user/host combination with `SELECT user, host FROM mysql.user`
- **Immediate**: Check account status with `SELECT Account_locked, Account_expiry FROM mysql.user`
- **Short-term**: Reset password or adjust authentication plugin
- **Long-term**: Use `caching_sha2_password` (MySQL 8.0 default) and verify client driver compatibility

### Risk Assessment: Medium (connection failure only, no data impact)

---

## Failure Pattern 3: InnoDB Deadlock

### Symptom
Transaction receives: `ERROR 1213 (40001): Deadlock found when trying to get lock; try restarting transaction`. Error log contains `LATEST DETECTED DEADLOCK`.

### Affected Components
- InnoDB transaction management
- Row-level locking
- Application transaction ordering

### Detection
- CLI output: `ERROR 1213 (40001)`
- Error log: `LATEST DETECTED DEADLOCK` section
- `SHOW ENGINE INNODB STATUS\G` — Transactions section
- `pt-deadlock-logger` captures if running

### Root Causes
1. Two transactions acquiring locks in different order
2. Large transactions holding locks for too long
3. Missing indexes causing lock escalation to table-level locks
4. UPDATE/DELETE without WHERE on primary key
5. Gap locks on gaps between index values

### Remediation Guidance
- **Immediate**: Application should retry the failed transaction (automatic or manual retry logic)
- **Short-term**: Reorder lock acquisition in application code
- **Short-term**: Add indexes to avoid gap locks and lock escalation
- **Long-term**: Reduce transaction scope, use optimistic concurrency control
- **Diagnostic**: Run `pt-deadlock-logger` to capture deadlock patterns over time

### Risk Assessment: Medium (transaction failure, application-level retry can resolve)

---

## Failure Pattern 4: Replication Lag

### Symptom
Replica falls behind master. `Seconds_Behind_Master` shows increasing lag on replica.

### Affected Components
- Replication IO thread
- Replication SQL thread
- Binary log delivery
- Network connectivity

### Detection
- CLI: `SHOW REPLICA STATUS\G` — `Seconds_Behind_Master` increasing
- Error log: Replication warnings
- Network monitoring: Latency between master and replica

### Root Causes
1. Write-heavy master with limited replica CPU/disk I/O
2. Large transactions on master that take long time on replica
3. Replication IO thread stopped (network issue, missing binlog)
4. Replication SQL thread stopped (data inconsistency, SQL error on replica)
5. Disk I/O bottleneck on replica

### Remediation Guidance
- **Immediate**: Check `Slave_IO_Running` and `Slave_SQL_Running` — both should be YES
- **Immediate**: Check `Last_SQL_Error` for specific error
- **Short-term**: Scale up replica resources (CPU, disk I/O)
- **Short-term**: Consider parallel replication (`slave_parallel_workers > 0`)
- **Long-term**: Evaluate Group Replication or InnoDB Cluster for better HA

### Risk Assessment: High (data staleness on read replicas, potential data loss if SQL thread fails)

---

## Failure Pattern 5: Lock Wait Timeout

### Symptom
Transaction receives: `ERROR 1205 (HY000): Lock wait timeout exceeded; try restarting transaction`.

### Affected Components
- InnoDB row-level locking
- Blocking transactions
- Application transaction timeout

### Detection
- CLI output: `ERROR 1205 (HY000)`
- `SHOW ENGINE INNODB STATUS\G` — TRANSACTIONS section
- `performance_schema.data_lock_waits` — blocking/blocked queries
- `SHOW FULL PROCESSLIST` — processes in `Lock` state

### Root Causes
1. Long-running transaction holding locks
2. Application not committing or rolling back transactions
3. Missing indexes causing table-level locks
4. Uncommitted transaction from crashed application session
5. High contention on hot rows

### Remediation Guidance
- **Immediate**: Identify blocking transaction from `SHOW ENGINE INNODB STATUS\G`
- **Immediate**: Kill blocking transaction if safe to do so
- **Short-term**: Increase `innodb_lock_wait_timeout` (default 50s) if timeout is too short
- **Long-term**: Fix application transaction management, add indexes, reduce transaction scope

### Risk Assessment: Medium-High (application transaction failure, may cascade)

---

## Failure Pattern 6: Server Gone Away (ERROR 2006)

### Symptom
Client receives: `ERROR 2006 (HY000): MySQL server has gone away`. Connection drops unexpectedly.

### Affected Components
- MySQL server process
- Network connectivity
- Packet size limits
- Server resources

### Detection
- CLI output: `ERROR 2006 (HY000)`
- Error log: Server crash, OOM kill, or normal shutdown
- Process monitoring: mysqld process not running

### Root Causes
1. mysqld process crashed (OOM, segfault, assertion)
2. Packet too large (`max_allowed_packet` exceeded)
3. Server restarted (update, crash, planned maintenance)
4. Network interruption between client and server
5. `wait_timeout` expired for long-running connections

### Remediation Guidance
- **Immediate**: Check error log for crash reason
- **Immediate**: Verify mysqld process is running
- **Immediate**: Check system logs for OOM killer
- **Short-term**: Increase `max_allowed_packet` if packet size issue
- **Long-term**: Fix OOM conditions, tune server resources

### Risk Assessment: High (service interruption, potential data loss if in-flight transactions)

---

## Failure Pattern 7: InnoDB Corruption

### Symptom
Server fails to start or queries fail with: `InnoDB: Starting crash recovery...`, `InnoDB: Corrupt page`, `InnoDB: Error: trying to access page`.

### Affected Components
- InnoDB storage engine
- Data dictionary
- Tablespace files
- Disk storage

### Detection
- Error log: `InnoDB: Starting crash recovery`, `InnoDB: Corrupt page`, `InnoDB: Data dictionary`
- CLI: `InnoDB status` shows corruption
- Tablespace files: Corrupted ibdata1 or .ibd files

### Root Causes
1. Hardware failure (disk bad sectors, RAID controller failure)
2. Power loss during write
3. OOM killer terminating mysqld mid-write
4. Filesystem corruption
5. InnoDB bug (rare)

### Remediation Guidance
- **Immediate**: Check error log for specific corruption details
- **Immediate**: Attempt crash recovery (may succeed automatically)
- **Short-term**: Restore from last good backup + binary logs
- **Short-term**: Use `innodb_force_recovery=1-6` to start server for data extraction
- **Long-term**: Investigate root cause (hardware, OOM, filesystem)
- **Emergency**: `innodb_force_recovery=4` allows reads only, `6` allows minimal writes

### Risk Assessment: Critical (potential data loss, requires restore)

---

## Failure Pattern 8: Disk Space Full

### Symptom
Server rejects writes with: `No space left on device` or `ENOSPC`. Error log shows disk space warnings.

### Affected Components
- Data directory filesystem
- Binary log files
- Redo log files
- Temporary tables directory
- Error log files

### Detection
- CLI output: `No space left on device`, `ENOSPC`
- `df -h` — filesystem at 100%
- Error log: disk space warnings
- `SHOW ENGINE INNODB STATUS` — tablespace full messages

### Root Causes
1. Binary logs not expiring (missing `binlog_expire_logs_seconds`)
2. Large queries creating temporary tables
3. Uncontrolled data growth
4. Error logs growing unbounded
5. Backup files accumulating

### Remediation Guidance
- **Immediate**: Free space — purge old binary logs (`PURGE BINARY LOGS`)
- **Immediate**: Remove old backup files if redundant
- **Immediate**: Truncate large log files if safe
- **Short-term**: Expand filesystem or add disk
- **Long-term**: Configure `binlog_expire_logs_seconds`, implement log rotation, set up monitoring

### Risk Assessment: Critical (server becomes read-only or unresponsive)

---

## Failure Pattern 9: Slow Query Degradation

### Symptom
Queries that were fast suddenly become slow. Application experiences degraded response times.

### Affected Components
- Query optimizer
- Index usage
- Data distribution
- Statistics

### Detection
- Slow query log shows queries exceeding `long_query_time`
- `EXPLAIN` shows full table scans (`type=ALL`)
- `sys.schema_unused_indexes` shows indexes not being used
- `performance_schema.events_statements_summary_by_digest` shows increased latency

### Root Causes
1. Outdated table statistics (ran `ANALYZE TABLE` too long ago)
2. Indexes dropped or altered
3. Data distribution changed significantly
4. Missing indexes for new query patterns
5. Plan regression from optimizer changes

### Remediation Guidance
- **Immediate**: Run `ANALYZE TABLE` for affected tables
- **Immediate**: Check `EXPLAIN` output for changes in execution plan
- **Short-term**: Add missing indexes based on slow query log analysis
- **Short-term**: Use `OPTIMIZE TABLE` for heavily fragmented tables
- **Long-term**: Monitor query performance trends, maintain statistics regularly

### Risk Assessment: Medium-High (application performance impact)

---

## Failure Pattern 10: Configuration Parameter Misconfiguration

### Symptom
Unexpected behavior: slow performance, connection failures, or server not starting.

### Detection
- `SHOW GLOBAL VARIABLES` shows unexpected values
- Error log shows configuration warnings
- `mysqld --verbose --help` reveals mismatched defaults
- `mysqld --validate-config` fails (MySQL 8.0+)

### Root Causes
1. Incorrect values in my.cnf (wrong data types, out-of-range values)
2. Conflicting configuration file locations
3. Parameters set at wrong level (SESSION vs GLOBAL)
4. Deprecated parameters still in use

### Remediation Guidance
- **Immediate**: Review error log for configuration-related warnings
- **Immediate**: Validate config with `mysqld --validate-config` (MySQL 8.0.29+)
- **Short-term**: Use `SET PERSIST` to test changes before committing to my.cnf
- **Long-term**: Document all configuration changes with justification

### Risk Assessment: Medium (may cause degraded performance or startup failure)

---

## Quick Reference: Failure Pattern Summary

| Pattern | Error Code | Risk | Key Detection |
|---------|-----------|------|---------------|
| Too Many Connections | 1040 | High | `SHOW STATUS LIKE 'Max_used_connections'` |
| Authentication Denied | 1045 | Medium | CLI: `ERROR 1045` |
| Deadlock | 1213 | Medium | `SHOW ENGINE INNODB STATUS\G` |
| Replication Lag | N/A | High | `SHOW REPLICA STATUS\G` |
| Lock Wait Timeout | 1205 | Medium-High | `SHOW FULL PROCESSLIST` |
| Server Gone Away | 2006 | High | Error log, process status |
| InnoDB Corruption | N/A | Critical | Error log, crash recovery |
| Disk Space Full | N/A | Critical | `df -h`, ENOSPC errors |
| Slow Query | N/A | Medium-High | Slow query log, EXPLAIN |
| Config Misconfiguration | N/A | Medium | Error log, `SHOW VARIABLES` |

---

## References

- [MySQL 8.0 Error Codes](https://dev.mysql.com/doc/refman/8.0/en/server-error-codes.html)
- [MySQL 8.0 InnoDB Error Handling](https://dev.mysql.com/doc/refman/8.0/en/innodb-error-handling.html)
- [MySQL 8.0 Troubleshooting Guide](https://dev.mysql.com/doc/refman/8.0/en/troubleshooting-innodb.html)