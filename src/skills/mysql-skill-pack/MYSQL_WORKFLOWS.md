# MySQL Troubleshooting Workflows

## Purpose

This document provides state-machine troubleshooting workflows for MySQL. Each workflow follows the standardized evidence-collection → diagnosis → remediation → verification model. All workflows provide advisory guidance only — the engineer performs all actions.

## Workflow Model

Every workflow follows this state machine:

```
[evidence_collection] → [diagnosis] → [remediation] → [verification]
       ↑                        │
       └────────────────────────┘ (loop if unresolved)
```

### State Definitions

| State | Purpose | Exit Condition |
|-------|---------|----------------|
| evidence_collection | Gather all available diagnostic data | Sufficient evidence collected |
| diagnosis | Analyze evidence, form hypothesis | Root cause identified |
| remediation | Apply fix based on diagnosis | Fix applied |
| verification | Confirm resolution | Issue resolved OR return to diagnosis |

---

## Workflow 1: Connection Exhaustion

**Trigger**: `ERROR 1040 (08004): Too many connections`

### State 1: Evidence Collection

Collect:
- `Threads_connected` from `SHOW GLOBAL STATUS`
- `Max_used_connections` from `SHOW GLOBAL STATUS`
- `max_connections` from `SHOW GLOBAL VARIABLES`
- `Threads_running`, `Threads_created` from `SHOW GLOBAL STATUS`
- `Aborted_connects` from `SHOW GLOBAL STATUS`
- `SHOW FULL PROCESSLIST` — identify connection patterns (how many in Sleep state? how many active?)
- Error log for connection-related entries

**Confidence Threshold**: Need at least 3 of: Threads_connected count, max_connections value, processlist analysis, Aborted_connects trend

### State 2: Diagnosis

Determine if the cause is:
- **True capacity issue**: `Threads_connected` ≈ `max_connections` with high `Threads_running`
- **Connection leak**: High `Threads_connected` with most in `Sleep` state, `Threads_created` steadily increasing
- **Spike**: Sudden increase in connections from application traffic surge
- **Timeout misconfiguration**: `wait_timeout` too high holding idle connections

**Diagnosis Output**: Root cause category + confidence score

### State 3: Remediation

Based on diagnosis:
- **Capacity issue**: Advise increasing `max_connections` (not above OS file descriptor limit)
- **Connection leak**: Advise application connection pool tuning, fix connection close logic
- **Timeout misconfiguration**: Advise reducing `wait_timeout` for the workload profile
- **General**: Advise implementing connection pooling (ProxySQL, PgBouncer, application pool)

**Risk Assessment**: Medium — changing max_connections may impact memory usage

### State 4: Verification

Confirm:
- `Threads_connected` stays within safe range of `max_connections`
- No new `ERROR 1040` in application logs
- `Threads_running` returns to normal levels
- Connection pool utilization is healthy

---

## Workflow 2: Authentication Failure

**Trigger**: `ERROR 1045 (28000): Access denied`

### State 1: Evidence Collection

Collect:
- Exact error message (with/without password)
- User and host attempted
- `SELECT user, host, plugin, account_locked, password_expired FROM mysql.user WHERE user = 'target_user'`
- Authentication plugin in use
- Error log entries

**Confidence Threshold**: Exact error message + user verification

### State 2: Diagnosis

Identify:
- **Wrong password**: User exists, correct host, wrong credentials
- **Host mismatch**: User doesn't exist for the attempted host
- **Plugin mismatch**: `caching_sha2_password` vs `mysql_native_password` incompatibility
- **Account locked/expired**: `account_locked = 'Y'` or `password_expired = 'Y'`
- **Host ACL**: No entry matches the connecting host

### State 3: Remediation

- **Wrong password**: Reset password with `ALTER USER ... IDENTIFIED BY 'new_password'`
- **Host mismatch**: Create user for correct host or use wildcard `'%` with caution
- **Plugin mismatch**: Change authentication plugin or update client driver
- **Account locked**: `ALTER USER ... ACCOUNT UNLOCK`
- **Host ACL**: Grant access from correct host

### State 4: Verification

- Test connection with credentials
- Verify no new ERROR 1045 in logs
- Confirm correct authentication plugin

---

## Workflow 3: InnoDB Deadlock

**Trigger**: `ERROR 1213 (40001): Deadlock found`

### State 1: Evidence Collection

Collect:
- `SHOW ENGINE INNODB STATUS\G` — LATEST DETECTED DEADLOCK section
- `performance_schema.data_lock_waits` — current and historical lock waits
- `SHOW FULL PROCESSLIST` — identify potentially blocking queries
- Application transaction logs
- Table structures involved (index definitions)

**Confidence Threshold**: Deadlock detail from INNODB STATUS + table schema

### State 2: Diagnosis

Analyze deadlock graph:
- Which transactions were involved?
- What locks were held vs requested?
- Lock ordering conflict?
- Missing indexes causing lock escalation?
- Gap locks on gap between index values?

**Diagnosis Categories**:
- Lock ordering conflict — transactions acquire locks in different order
- Large transaction — holds locks too long, overlapping with others
- Missing index — causes table-level locks instead of row-level
- Gap lock contention — concurrent inserts creating lock conflicts

### State 3: Remediation

- **Application-level**: Reorder lock acquisition in application code
- **Schema-level**: Add indexes to reduce lock scope
- **Transaction-level**: Reduce transaction scope, commit sooner
- **Monitoring**: Run `pt-deadlock-logger` for continuous deadlock capture

### State 4: Verification

- Confirm no new deadlocks for monitoring period (15-30 minutes)
- Check application logs for ERROR 1213
- Verify `Innodb_deadlocks` counter in status is stable

---

## Workflow 4: Replication Lag Recovery

**Trigger**: `Seconds_Behind_Master` exceeds threshold on replica

### State 1: Evidence Collection

Collect:
- `SHOW REPLICA STATUS\G` — all fields
- `Slave_IO_Running`, `Slave_SQL_Running` status
- `Seconds_Behind_Master` trend over time
- `Last_SQL_Error` if SQL thread stopped
- `Read_Master_Log_Pos`, `Exec_Master_Log_Pos` — gap indicates lag
- Replica resource utilization (CPU, disk I/O, memory)
- Master write load during lag period

**Confidence Threshold**: Full replica status + resource metrics

### State 2: Diagnosis

Determine root cause:
- **IO thread stopped**: `Slave_IO_Running = No` — network, binlog missing
- **SQL thread stopped**: `Slave_SQL_Running = No` — data inconsistency, error
- **Both running but lagging**: Write-heavy master, replica resource bottleneck
- **Large transaction**: Single large transaction on master causing replica lag spike

### State 3: Remediation

- **IO thread stopped**: Fix network, restart IO thread (`START REPLICA IO_THREAD`)
- **SQL thread stopped**: Fix error (data inconsistency, SQL issue), restart SQL thread
- **Both running but lagging**: Scale up replica, enable parallel replication (`slave_parallel_workers`)
- **Large transaction**: Plan for replication window, consider splitting large transactions

### State 4: Verification

- `Seconds_Behind_Master` decreases and stabilizes
- `Slave_IO_Running = Yes` and `Slave_SQL_Running = Yes`
- No new errors in replica error log
- Replication lag stays within acceptable threshold

---

## Workflow 5: Slow Query Investigation

**Trigger**: Query exceeds `long_query_time` threshold

### State 1: Evidence Collection

Collect:
- `EXPLAIN` output for the query
- `EXPLAIN ANALYZE` (MySQL 8.0.18+) for actual execution metrics
- Slow query log entry
- Table structures (indexes, partitions)
- Data volume in involved tables
- Table statistics freshness (`ANALYZE TABLE` timing)

**Confidence Threshold**: EXPLAIN + EXPLAIN ANALYZE + table statistics

### State 2: Diagnosis

Identify performance bottleneck:
- **Full table scan**: `type=ALL` in EXPLAIN — missing index
- **Filesort**: `Using filesort` in Extra — no index for ORDER BY
- **Temporary table**: `Using temporary` — GROUP BY or DISTINCT requiring temp table
- **Index condition pushdown failure**: Statistics outdated
- **Row estimation error**: Statistics stale, leading to poor join order
- **Large result set**: No LIMIT clause or excessive matching rows

### State 3: Remediation

- **Missing index**: Add appropriate index based on EXPLAIN output
- **Filesort**: Add index covering ORDER BY columns
- **Temporary table**: Optimize GROUP BY, add composite index
- **Stale statistics**: Run `ANALYZE TABLE`
- **Large results**: Add LIMIT, pagination

### State 4: Verification

- Re-run EXPLAIN — verify index usage improved
- Measure query time — should be within acceptable threshold
- Monitor `Handler_read_*` counters for reduced full scans
- Slow query log shows query no longer appears

---

## Workflow 6: Server Won't Start

**Trigger**: mysqld fails to start, `systemctl start mysqld` fails

### State 1: Evidence Collection

Collect:
- Error log (`/var/log/mysql/error.log` or `mysqld.err`) — last 50 lines
- System logs (`journalctl -u mysqld`)
- Disk space (`df -h` on data directory filesystem)
- File permissions on data directory
- Configuration file (`my.cnf`) — recent changes
- `mysqld --validate-config` (MySQL 8.0.29+)

**Confidence Threshold**: Error log entry with specific error + supporting evidence

### State 2: Diagnosis

Identify start failure reason:
- **Data corruption**: InnoDB crash recovery failed, corrupt data dictionary
- **Configuration error**: Invalid parameter, deprecated option, wrong type
- **Disk full**: No space on data directory or redo log partition
- **Permission issue**: Data directory not owned by mysql user
- **Port conflict**: Another process using port 3306
- **OOM**: Kernel killed mysqld process

### State 3: Remediation

- **Data corruption**: Try `innodb_force_recovery=1` through `6` to extract data, then restore from backup
- **Configuration error**: Fix invalid parameters in my.cnf, validate with `mysqld --validate-config`
- **Disk full**: Free disk space (purge binlogs, old backups), then retry start
- **Permission issue**: `chown -R mysql:mysql /var/lib/mysql`
- **Port conflict**: Kill conflicting process or change port
- **OOM**: Increase available memory, reduce `innodb_buffer_pool_size`

### State 4: Verification

- mysqld starts successfully
- `systemctl status mysqld` shows active
- `mysqladmin ping` responds
- Error log shows clean startup (no errors)

---

## Workflow 7: Replication Break Fix

**Trigger**: Replication stops with SQL thread error, `Slave_SQL_Running = No`

### State 1: Evidence Collection

Collect:
- `SHOW REPLICA STATUS\G` — `Last_SQL_Error` message
- `Last_SQL_Error_Timer`
- `Exec_Master_Log_Pos` vs `Read_Master_Log_Pos` gap
- Error log around time of failure
- Table/schema involved (from error message)
- Binary log contents at failure position (`mysqlbinlog`)

**Confidence Threshold**: Last_SQL_Error message + binary log analysis

### State 2: Diagnosis

Analyze SQL thread error:
- **Data inconsistency**: Replica data differs from master (row missing, duplicate)
- **SQL syntax error**: Replicated statement not compatible with replica version
- **Constraint violation**: UNIQUE, FOREIGN KEY constraint on replica
- **Schema mismatch**: Different table structure between master and replica
- **Privilege issue**: User lacks permissions on replica

### State 3: Remediation

- **Data inconsistency**: Fix data manually or use `pt-table-sync` to synchronize
- **SQL syntax error**: Skip the error with `SET GLOBAL sql_slave_skip_counter = 1` (evaluate safety first!)
- **Constraint violation**: Resolve constraint conflict on replica
- **Schema mismatch**: Apply missing schema change to replica
- **Privilege issue**: Grant required privileges on replica

**Critical**: For `sql_slave_skip_counter`, evaluate if skipping the transaction is safe. Check what the transaction was doing.

### State 4: Verification

- `Slave_SQL_Running = Yes` after restart
- `Seconds_Behind_Master` decreases
- No new errors in replica error log
- Data consistency verified with row counts or checksums

---

## Workflow 8: Disk Space Emergency

**Trigger**: Disk usage > 90% on data directory filesystem

### State 1: Evidence Collection

Collect:
- `df -h` — filesystem usage by mount point
- `du -sh /var/lib/mysql/*` — space by directory
- `du -sh /var/lib/mysql/*.ibd` — largest tables
- Binary log files: `ls -lh /var/lib/mysql/mysql-bin.*`
- Error log for disk space warnings
- `SHOW BINARY LOGS`

**Confidence Threshold**: Disk usage + directory breakdown + binlog sizes

### State 2: Diagnosis

Identify space consumer:
- **Binary logs**: Accumulated without expiration — `binlog_expire_logs_seconds` not set
- **Large tables**: Growing data volume, especially `.ibd` files
- **Temporary tables**: Large queries creating temp tables on disk
- **Error logs**: Log rotation not configured
- **Backup files**: Unrotated backup files consuming space

### State 3: Remediation

- **Binary logs**: `PURGE BINARY LOGS BEFORE 'N days ago'` — verify no replica needs old logs first
- **Large tables**: Archive old data, add partitioning, consider compression
- **Temp tables**: Optimize queries causing disk temp tables
- **Error logs**: Configure log rotation (`log_error_verbosity`, cron rotation)
- **Backup files**: Remove redundant backups, move to external storage

### State 4: Verification

- Disk usage drops below 80%
- mysqld operational (not in read-only mode)
- No new disk space warnings in error log
- Binary log retention policy confirmed

---

## Workflow 9: Configuration Validation

**Trigger**: Configuration change suspected, server behaving unexpectedly

### State 1: Evidence Collection

Collect:
- `SHOW GLOBAL VARIABLES` — all relevant variables
- `SHOW PERSISTED VARIABLES` (MySQL 8.0.16+)
- `cat mysqld-auto.cnf` — persisted variable file
- Current my.cnf contents
- Error log for configuration warnings
- `mysqld --validate-config` (MySQL 8.0.29+)
- Application behavior before/after change

**Confidence Threshold**: GLOBAL VARIABLES vs my.cnf + persisted variables

### State 2: Diagnosis

Identify configuration issues:
- **Parameter value out of range**: Value exceeds min/max
- **Wrong data type**: String where number expected
- **Conflicting sources**: Different values in my.cnf vs persisted vs command line
- **Deprecated parameter**: Parameter removed in current version
- **Session vs global mismatch**: Variable changed at SESSION level not GLOBAL

### State 3: Remediation

- **Out of range**: Adjust value to valid range
- **Wrong type**: Fix data type in my.cnf
- **Conflicting sources**: Use `RESET PERSIST` to clear, re-apply with `SET PERSIST`
- **Deprecated**: Remove from my.cnf, use replacement parameter
- **Scope issue**: Use `SET GLOBAL` not `SET SESSION` for server-wide changes

### State 4: Verification

- `SHOW GLOBAL VARIABLES` shows expected values
- `mysqld --validate-config` passes
- Server behavior matches expectations
- Application performance stable

---

## Workflow 10: Backup and Restore Validation

**Trigger**: Backup failure, restore needed, or backup integrity concern

### State 1: Evidence Collection

Collect:
- Backup log output
- Backup file size and completeness
- `SHOW GLOBAL STATUS` at backup time (if available)
- Binary log position at backup (`--master-data=2` output)
- Replication status at backup time
- Error log for errors during backup
- Backup method used (mysqldump, xtrabackup, etc.)

**Confidence Threshold**: Backup log + file verification + replication status

### State 2: Diagnosis

Identify backup issues:
- **Incomplete backup**: Missing databases or tables
- **Inconsistent backup**: Binary log position mismatch, incomplete transaction
- **Permission error**: Backup user lacks privileges
- **Disk space**: Backup destination full
- **Timeout**: Backup exceeded `max_allowed_packet` or timeout

### State 3: Remediation

- **Incomplete**: Re-run backup with corrected parameters
- **Inconsistent**: Use `--single-transaction` for InnoDB consistency, verify `--master-data`
- **Permission**: Grant SELECT, RELOAD, LOCK TABLES, SHOW VIEW, EVENT, TRIGGER to backup user
- **Disk space**: Free space on backup destination, or change `--target-dir`
- **Timeout**: Increase `max_allowed_packet`, split backup with `--databases`

### State 4: Verification

- Backup file is valid and complete
- Restore test confirms backup integrity
- Binary log position matches expected recovery point
- Replication can resume from backup point if needed

---

## Workflow Summary Matrix

| # | Workflow | Trigger | Risk Level | Time to Resolution |
|---|----------|---------|-----------|-------------------|
| 1 | Connection Exhaustion | ERROR 1040 | High | 15-60 min |
| 2 | Authentication Failure | ERROR 1045 | Medium | 5-30 min |
| 3 | InnoDB Deadlock | ERROR 1213 | Medium | 15-45 min |
| 4 | Replication Lag | Seconds_Behind_Master | High | 30-120 min |
| 5 | Slow Query | long_query_time exceeded | Medium | 30-60 min |
| 6 | Server Won't Start | mysqld not running | Critical | 15-60 min |
| 7 | Replication Break | Slave_SQL_Running = No | Critical | 30-120 min |
| 8 | Disk Space Emergency | Disk > 90% | Critical | 15-60 min |
| 9 | Configuration Validation | Unexpected behavior | Medium | 15-30 min |
| 10 | Backup/Restore Validation | Backup failure | High | 30-60 min |

---

## Workflow Execution Rules

1. Always collect complete evidence before diagnosis
2. Assign confidence score after each diagnostic step
3. Never skip from evidence directly to remediation
4. Loop back to diagnosis if remediation doesn't resolve the issue
5. Document each step of the workflow for audit trail
6. Verify resolution before closing the workflow
7. Escalate to P1 (critical) if data loss or complete outage is occurring

---

## References

- [MySQL 8.0 Troubleshooting Guide](https://dev.mysql.com/doc/refman/8.0/en/troubleshooting-innodb.html)
- [MySQL 8.0 Replication](https://dev.mysql.com/doc/refman/8.0/en/replication.html)
- [MySQL 8.0 Performance Schema](https://dev.mysql.com/doc/refman/8.0/en/performance-schema.html)
- [MySQL 8.0 Backup and Recovery](https://dev.mysql.com/doc/refman/8.0/en/backup-recovery.html)
- [Percona Toolkit Documentation](https://www.percona.com/doc/percona-toolkit/LATEST/index.html)