# EDB PostgreSQL State-Machine Troubleshooting Workflows

## Overview

This document defines state-machine troubleshooting workflows for PostgreSQL operational issues. Each workflow provides a structured, phase-gated approach to diagnosing and resolving database problems. Workflows follow the STANDARD WORKFLOW MODEL: State → Observation → Decision → State transition.

## Workflow 1: Server Startup Failure

### State Machine

```
[START] → Collect Evidence → Diagnose Root Cause → Apply Fix → Verify Startup → [COMPLETE]
```

### State 1: Collect Evidence
**Purpose**: Gather diagnostic information before making changes

**Evidence Collection Steps**:
1. Check PostgreSQL log file for errors
2. Verify data directory permissions (`ls -la <data_dir>`)
3. Verify postmaster.pid file status
4. Check system memory (free -h)
5. Check shared memory parameters (sysctl -a | grep shm)
6. Verify data directory is not full (`df -h <data_dir>`)
7. Check for conflicting processes on PostgreSQL port

**Confidence Requirement**: ≥ 0.85 before proceeding

**Transition**: 
- If logs contain clear error → go to State 2
- If evidence insufficient → go to State 1 (collect more evidence)

### State 2: Diagnose Root Cause
**Purpose**: Identify the specific cause of startup failure

**Diagnosis Paths**:
- "shared memory" error → Shared memory configuration issue
- "disk full" error → Disk space issue
- "FATAL: could not create shared memory" → Kernel parameter issue
- "PANIC: could not locate a valid checkpoint record" → Data corruption
- "LOG: data directory is not empty" → Already initialized
- "LOG: data directory has wrong ownership" → Permission issue
- "LOG: could not bind" → Port conflict

**Confidence Requirement**: ≥ 0.90 before proceeding

**Transition**: 
- Clear cause identified → go to State 3
- Multiple possible causes → go to State 1 (collect more evidence)

### State 3: Apply Fix
**Purpose**: Apply the targeted fix for the identified cause

**Fix Actions by Cause**:
- Shared memory → Increase kernel.shmmax and kernel.shmall
- Disk full → Clean WAL archives or expand disk
- Permission issue → chown postgres:postgres <data_dir>
- Port conflict → Kill conflicting process or change port
- Corrupted checkpoint → Use pg_resetwal (emergency only)
- Already initialized → Verify existing cluster is running

**Risk Assessment**:
- Kernel parameter change → Medium (requires sysctl reload)
- Permission fix → Low
- Disk cleanup → Low (with caution)
- pg_resetwal → Critical (can lose data)

**Transition**: 
- Fix applied → go to State 4
- Fix failed → go to State 2 (re-diagnose)

### State 4: Verify Startup
**Purpose**: Confirm PostgreSQL is running correctly

**Verification Steps**:
1. `pg_isready -h <host> -p <port>` → exit code 0
2. `pg_ctl status -D <data_dir>` → running
3. Check log for "database system is ready to accept connections"
4. Connect with `psql -c "SELECT 1;"` → returns 1
5. Verify `pg_stat_activity` shows background processes

**Confidence Requirement**: All checks pass

**Transition**:
- All checks pass → [COMPLETE]
- Any check fails → go to State 2 (re-diagnose)

---

## Workflow 2: Connection Exhaustion

### State Machine

```
[START] → Check Connection Count → Identify Connection Type → Resolve → Verify → [COMPLETE]
```

### State 1: Check Connection Count
**Purpose**: Determine if connection exhaustion is occurring

**Checks**:
1. `psql -c "SELECT count(*) FROM pg_stat_activity;"`
2. `psql -c "SHOW max_connections;"`
3. `psql -c "SELECT state, count(*) FROM pg_stat_activity GROUP BY state;"`
4. Check for idle-in-transaction sessions
5. Check PgBouncer pool status if using PgBouncer

**Confidence Requirement**: ≥ 0.90

**Transition**:
- Count ≥ 80% of max_connections → State 2
- Count < 80% → State 5 (not connection exhaustion)

### State 2: Identify Connection Type
**Purpose**: Determine what type of connections are consuming resources

**Classification**:
- `idle` → Connection pool leak
- `idle in transaction` → Application not committing/rolling back
- `active` → Normal application activity
- `active (statement timeout)` → Long-running queries

**Confidence Requirement**: ≥ 0.85

**Transition**:
- idle-in-transaction dominant → State 3a
- idle dominant → State 3b
- active dominant → State 3c
- Mixed → State 3d

### State 3: Resolution by Type
**Purpose**: Apply targeted resolution for the connection type

**3a — idle-in-transaction**:
```
1. Set idle_in_transaction_session_timeout = 30s (advisory)
2. Terminate longest idle-in-transaction sessions
3. Alert application team for transaction management fix
```

**3b — idle connections**:
```
1. Check PgBouncer pool config
2. Verify application is returning connections to pool
3. Adjust pool_size and max_client_conn in PgBouncer
```

**3c — active connections**:
```
1. Identify which queries are running
2. Check if queries are performing as expected
3. Consider increasing max_connections if legitimate growth
4. Scale PgBouncer if using pooler
```

**3d — mixed**:
```
1. Address the most impactful category first
2. Set timeouts to prevent accumulation
3. Apply the most appropriate resolution from above
```

### State 4: Verify Resolution
**Purpose**: Confirm connection count is manageable

**Verification**:
1. `psql -c "SELECT count(*) FROM pg_stat_activity;"` → < 80% of max_connections
2. No new idle-in-transaction sessions for 5 minutes
3. Application functioning normally
4. No error messages about connection exhaustion

**Confidence Requirement**: All checks pass

**Transition**:
- All pass → [COMPLETE]
- Any fail → State 2 (re-evaluate)

---

## Workflow 3: Replication Lag

### State Machine

```
[START] → Measure Lag → Identify Cause → Resolve → Monitor → [COMPLETE]
```

### State 1: Measure Lag
**Purpose**: Quantify replication lag

**Measurement**:
```sql
-- On primary
SELECT client_addr, state, sent_lsn, replay_lsn,
       pg_wal_lsn_diff(sent_lsn, replay_lsn) as replay_lag
FROM pg_stat_replication;

-- On standby
SELECT pg_last_wal_receive_lsn(), pg_last_wal_replay_lsn(),
       pg_wal_lsn_diff(pg_current_wal_lsn(), pg_last_wal_replay_lsn());
```

**Confidence Requirement**: ≥ 0.90

**Transition**:
- Lag > 30s → State 2 (critical)
- Lag 1-30s → State 2 (monitoring)
- Lag < 1s → [COMPLETE] (acceptable)
- Replication disconnected → State 2 (disconnected)

### State 2: Identify Cause
**Purpose**: Determine what is causing the lag

**Causes to Check**:
1. Network latency between nodes
2. Replica I/O capacity (iostat, df)
3. Replica CPU utilization
4. Checkpoint configuration
5. Long-running queries on standby (if hot standby)
6. Large transactions generating WAL
7. Standby autovacuum contention
8. Network packet loss

**Confidence Requirement**: ≥ 0.85

**Transition**:
- Single clear cause → State 3
- Multiple potential causes → State 2 (narrow down)
- Cause unclear → State 1 (re-measure)

### State 3: Resolve
**Purpose**: Apply targeted fix

**Resolutions by Cause**:
- Network latency → Check network path, reduce distance, optimize TCP
- Replica I/O → Upgrade storage, reduce WAL generation
- Replica CPU → Upgrade CPU, reduce workload on replica
- Checkpoint I/O → Tune checkpoint_timeout, checkpoint_completion_target
- Long queries on standby → Cancel or timeout standby queries
- Large transactions → Break into smaller transactions
- Autovacuum contention → Tune autovacuum on standby

### State 4: Monitor
**Purpose**: Monitor lag recovery

**Monitoring**:
1. Re-measure lag (State 1 checks)
2. Monitor every 30s for 10 minutes
3. Verify lag stays below SLA threshold
4. Check for recurring patterns

**Confidence Requirement**: Lag < 10s for 5 consecutive measurements

**Transition**:
- Lag < SLA → [COMPLETE]
- Lag increasing → State 2 (re-diagnose)

---

## Workflow 4: Disk Space Exhaustion

### State Machine

```
[START] → Check All Disk Locations → Identify Consumer → Clean Up → Prevent → [COMPLETE]
```

### State 1: Check All Disk Locations
**Purpose**: Identify which disk is near capacity

**Checks**:
```bash
df -h
du -sh /var/lib/pgsql
du -sh /var/lib/pgsql/pg_wal
du -sh /archive/wal  # WAL archive
du -sh /backup       # Backup location
```

**Confidence Requirement**: ≥ 0.90

**Transition**:
- Any disk > 85% → State 2
- All disks < 85% → [COMPLETE]

### State 2: Identify Consumer
**Purpose**: Determine what is consuming the most disk space

**Checks**:
```bash
# PostgreSQL data
ls -lhS /var/lib/pgsql/data/base/* | head -20

# WAL files
ls -lhS /var/lib/pgsql/data/pg_wal/ | head -20

# Archive
ls -lhS /archive/wal/ | tail -20

# Check replication slot retention
psql -c "SELECT slot_name, pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn) FROM pg_replication_slots;"
```

**Confidence Requirement**: ≥ 0.85

**Transition**:
- WAL archive dominant → State 3a
- Replication slot retention → State 3b
- Large tables → State 3c
- Temporary files → State 3d

### State 3: Clean Up
**Purpose**: Free disk space safely

**3a — WAL archive**: Use pg_archivecleanup with the oldest needed WAL
**3b — Replication slots**: Drop inactive slots (after confirming replica can proceed)
**3c — Large tables**: VACUUM FULL or table cleanup
**3d — Temp files**: Cancel queries generating temp files, increase work_mem

### State 4: Prevent
**Purpose**: Implement monitoring to prevent recurrence

**Actions**:
1. Set up disk usage alerts at 80%
2. Configure WAL archive retention policy
3. Configure replication slot monitoring
4. Document what was cleaned up and why

**Confidence Requirement**: All preventive measures in place

**Transition**:
- Prevention implemented → [COMPLETE]
- Prevention incomplete → State 4 (complete remaining measures)

---

## Workflow 5: Slow Query Diagnosis

### State Machine

```
[START] → Identify Query → Capture Plan → Analyze Plan → Optimize → Verify → [COMPLETE]
```

### State 1: Identify Query
**Purpose**: Pinpoint the slow query

**Identification**:
1. `psql -c "SELECT * FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;"`
2. `psql -c "SELECT pid, usename, state, query FROM pg_stat_activity WHERE state = 'active';"`
3. Application logs for slow query markers
4. pg_stat_statements tracking

**Confidence Requirement**: ≥ 0.90

**Transition**:
- Specific query identified → State 2
- Multiple candidates → State 1 (narrow down)

### State 2: Capture Plan
**Purpose**: Get the EXPLAIN ANALYZE plan

**Plan Capture**:
```sql
EXPLAIN (ANALYZE, BUFFERS, VERBOSE) <query>;
```

**Confidence Requirement**: Plan captured

**Transition**:
- Plan captured → State 3
- Plan incomplete → State 2 (try again with more options)

### State 3: Analyze Plan
**Purpose**: Diagnose the plan for performance issues

**Analysis Checklist**:
- [ ] Seq scan on large table? → Need index
- [ ] Nested loop with large outer? → Check join selectivity
- [ ] Hash join with large input? → Memory issue
- [ ] Sort with high cost? → Check work_mem
- [ ] Materialize not used? → Consider materialized CTE
- [ ] Bitmap scan not used? → Check index availability
- [ ] Filter rows not indexed? → Consider partial index
- [ ] Statistics stale? → Run ANALYZE

**Confidence Requirement**: ≥ 0.85

**Transition**:
- Clear optimization opportunity → State 4
- No clear issue → State 2 (collect more evidence)

### State 4: Optimize
**Purpose**: Apply optimization

**Actions by Issue**:
- Missing index → CREATE INDEX CONCURRENTLY
- Stale statistics → ANALYZE table
- work_mem too low → Increase work_mem or optimize query
- Bad join order → Review query structure
- Missing WHERE clause → Add appropriate filter
- Unnecessary subquery → Flatten query

### State 5: Verify
**Purpose**: Confirm optimization worked

**Verification**:
1. Run EXPLAIN ANALYZE again → compare execution times
2. Check pg_stat_statements for updated metrics
3. Verify application performance improves
4. Monitor for 15 minutes for stability

**Confidence Requirement**: Execution time reduced by > 50%

**Transition**:
- Performance improved → [COMPLETE]
- No improvement → State 3 (re-analyze)

---

## Workflow 6: Index Bloat Resolution

### State Machine

```
[START] → Measure Bloat → Identify Target Indexes → Choose Method → Rebuild → Verify → [COMPLETE]
```

### State 1: Measure Bloat
**Purpose**: Quantify index bloat

**Measurement**:
```sql
SELECT
    schemaname,
    relname,
    indexrelname,
    pg_size_pretty(pg_relation_size(indexrelid)) as index_size,
    idx_scan,
    idx_tup_read
FROM pg_stat_user_indexes
ORDER BY pg_relation_size(indexrelid) DESC;
```

**Confidence Requirement**: ≥ 0.85

**Transition**:
- Bloat > 30% → State 2
- Bloat < 30% → [COMPLETE] (acceptable)

### State 2: Identify Target Indexes
**Purpose**: Select which indexes to rebuild

**Selection Criteria**:
1. Indexes with highest bloat percentage
2. Indexes actively used (idx_scan > 0)
3. Indexes on critical tables
4. Indexes causing performance impact

**Confidence Requirement**: ≥ 0.90

**Transition**:
- Targets identified → State 3
- No clear targets → State 1 (re-measure)

### State 3: Choose Method
**Purpose**: Select the appropriate rebuild method

**Methods**:
- PostgreSQL 15+: REINDEX CONCURRENTLY TABLE table_name
- All versions: CREATE INDEX CONCURRENTLY (drop + recreate)
- PostgreSQL 12+: pg_repack for zero-downtime
- Offline maintenance window: REINDEX DATABASE

### State 4: Rebuild
**Purpose**: Execute the rebuild

**Actions**:
1. Choose method based on State 3
2. Schedule during low-traffic period
3. Execute rebuild
4. Monitor progress with pg_stat_progress_create_index

### State 5: Verify
**Purpose**: Confirm bloat reduced

**Verification**:
1. Re-measure index sizes (State 1 checks)
2. Verify index usage still correct
3. Confirm no performance degradation
4. Update monitoring baselines

**Confidence Requirement**: Bloat reduced to < 15%

**Transition**:
- Bloat < 15% → [COMPLETE]
- Bloat still high → State 2 (re-evaluate)

---

## Workflow 7: Backup and Restore Verification

### State Machine

```
[START] → Verify Backup Completeness → Test Restore → Verify Data → Document → [COMPLETE]
```

### State 1: Verify Backup Completeness
**Purpose**: Ensure backup was complete and consistent

**Checks**:
1. Backup file exists and has expected size
2. WAL archive complete up to backup point
3. pg_stat_archiver shows no failures during backup
4. Backup format is valid (can list contents)
5. pg_basebackup completed without errors

**Confidence Requirement**: ≥ 0.90

**Transition**:
- All checks pass → State 2
- Any check fails → State 1 (fix backup)

### State 2: Test Restore
**Purpose**: Verify backup can be restored to a test environment

**Actions**:
1. Set up test PostgreSQL instance
2. Restore backup (pg_restore or pg_basebackup)
3. Apply WAL archive up to backup point
4. Verify PostgreSQL starts successfully
5. Connect and verify basic functionality

**Confidence Requirement**: PostgreSQL starts on test instance

**Transition**:
- Test instance running → State 3
- Test instance failed → State 1 (fix restore process)

### State 3: Verify Data
**Purpose**: Confirm restored data is correct

**Checks**:
1. All databases present
2. Table counts match expected values
3. Sample data rows verify correctly
4. Indexes rebuild correctly
5. Extensions are installed and functional
6. Replication can be set up from restored instance

**Confidence Requirement**: All checks pass

**Transition**:
- All pass → State 4
- Any fail → State 1 (fix backup or restore process)

### State 4: Document
**Purpose**: Document the verification results

**Documentation**:
1. Backup verification date
2. Restore test results
3. Any issues found and resolved
4. Restore time measured
5. Updated backup runbook if needed

**Confidence Requirement**: Documentation complete

**Transition**:
- [COMPLETE] — backup and restore verified

---

## Workflow 8: Autovacuum Performance Issue

### State Machine

```
[START] → Check Autovacuum Status → Identify Target Tables → Tune Autovacuum → Verify → [COMPLETE]
```

### State 1: Check Autovacuum Status
**Purpose**: Determine if autovacuum is causing performance issues

**Checks**:
```sql
-- Table-level dead tuple accumulation
SELECT relname, n_dead_tup, last_autovacuum, last_autoanalyze
FROM pg_stat_user_tables ORDER BY n_dead_tup DESC LIMIT 20;

-- Autovacuum configuration
SHOW autovacuum_max_workers;
SHOW autovacuum_vacuum_scale_factor;
SHOW autovacuum_vacuum_cost_delay;
SHOW autovacuum_vacuum_cost_limit;

-- Running vacuum operations
SELECT * FROM pg_stat_progress_vacuum;

-- Autovacuum activity in logs
```

**Confidence Requirement**: ≥ 0.85

**Transition**:
- Dead tuples accumulating → State 2
- Autovacuum not running → State 2
- Autovacuum too aggressive → State 2
- All normal → [COMPLETE]

### State 2: Identify Target Tables
**Purpose**: Pinpoint tables causing the issue

**Identification**:
1. Tables with n_dead_tup > threshold
2. Tables with low n_live_tup / n_dead_tup ratio
3. Tables with recent large data changes
4. Tables not being vacuumed by autovacuum

**Confidence Requirement**: ≥ 0.90

**Transition**:
- Targets identified → State 3
- No clear targets → State 1 (collect more evidence)

### State 3: Tune Autovacuum
**Purpose**: Apply autovacuum tuning

**Actions**:
1. Set table-level autovacuum overrides for problematic tables
2. Increase autovacuum_vacuum_cost_limit (2000-4000)
3. Decrease autovacuum_vacuum_scale_factor for write-heavy tables (0.05-0.1)
4. Set table-level autovacuum_vacuum_scale_factor if needed
5. Set autovacuum_vacuum_threshold to lower value for large tables

### State 4: Verify
**Purpose**: Confirm tuning resolved the issue

**Verification**:
1. Re-check dead tuple counts (State 1 checks)
2. Verify autovacuum is running on target tables
3. Monitor for 24 hours for stability
4. Confirm no performance degradation from vacuum activity

**Confidence Requirement**: Dead tuples < threshold for 24 hours

**Transition**:
- Stable → [COMPLETE]
- Issues persist → State 3 (tune further)

---

## Workflow 9: SSL/TLS Configuration Issue

### State Machine

```
[START] → Check SSL Status → Verify Certificates → Fix Certificates → Test Connection → [COMPLETE]
```

### State 1: Check SSL Status
**Purpose**: Determine current SSL configuration

**Checks**:
```sql
SHOW ssl;
SHOW ssl_cert_file;
SHOW ssl_key_file;
SHOW ssl_ca_file;
```

**Confidence Requirement**: ≥ 0.90

**Transition**:
- SSL enabled but issues → State 2
- SSL disabled → State 2 (if required)
- SSL not needed → [COMPLETE]

### State 2: Verify Certificates
**Purpose**: Check certificate validity

**Checks**:
1. Certificate file exists and is readable
2. Certificate not expired
3. Certificate matches private key
4. Certificate chain is complete
5. Private key permissions are correct (owner: postgres, mode: 600)

**Confidence Requirement**: ≥ 0.85

**Transition**:
- Certificate issues → State 3
- No certificate issues → State 1 (re-check SSL config)

### State 3: Fix Certificates
**Purpose**: Resolve certificate issues

**Actions**:
1. Renew expired certificates
2. Fix file permissions
3. Reinstall certificate chain
4. Update ssl_cert_file paths if needed
5. Ensure CA bundle is complete

### State 4: Test Connection
**Purpose**: Verify SSL connections work

**Verification**:
1. `psql -c "SHOW ssl;"` → on
2. Connect via SSL: `psql "sslmode=require host=..."` → success
3. Check certificate details in pg_stat_ssl
4. Verify application SSL connections work

**Confidence Requirement**: All SSL connections work

**Transition**:
- All pass → [COMPLETE]
- Any fail → State 2 (re-diagnose)

---

## Workflow 10: Configuration Change Impact

### State Machine

```
[START] → Document Current Config → Apply Change → Verify Impact → Monitor → [COMPLETE]
```

### State 1: Document Current Config
**Purpose**: Record current configuration for rollback

**Actions**:
1. Save current postgresql.conf
2. Record current SHOW ALL output
3. Document any running sessions that may be affected
4. Note replication status
5. Note backup status

**Confidence Requirement**: ≥ 0.95

**Transition**:
- Current state documented → State 2
- Documentation incomplete → State 1 (complete documentation)

### State 2: Apply Change
**Purpose**: Apply the configuration change

**Actions**:
1. Use ALTER SYSTEM SET (preferred) or edit postgresql.conf
2. Apply with ALTER SYSTEM SET or ALTER SYSTEM RESET
3. Note which parameters need reload vs restart
4. Apply pg_reload_conf() or restart as needed

**Confidence Requirement**: Change applied without errors

**Transition**:
- Change applied → State 3
- Change failed → State 1 (restore from documentation)

### State 3: Verify Impact
**Purpose**: Confirm change had intended effect

**Verification**:
1. Verify parameter changed: `SHOW parameter_name`
2. Check PostgreSQL log for errors
3. Check pg_stat_activity for query performance
4. Check replication status
5. Check backup status

**Confidence Requirement**: ≥ 0.85

**Transition**:
- Intended effect + no adverse effects → State 4
- Intended effect + adverse effects → State 3 (monitor)
- No intended effect → State 2 (adjust change)

### State 4: Monitor
**Purpose**: Monitor for adverse effects

**Monitoring**:
1. Monitor pg_stat_activity for 15 minutes
2. Check query performance metrics
3. Check replication status
4. Check autovacuum activity
5. Check disk space (checkpoints, temp files)

**Confidence Requirement**: 15 minutes of stable operation

**Transition**:
- Stable → [COMPLETE]
- Adverse effects → State 2 (revert and re-diagnose)