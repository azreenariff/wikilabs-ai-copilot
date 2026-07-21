# EDB PostgreSQL Common Failure Patterns

## Overview

This document documents common enterprise failure patterns observed in PostgreSQL operational environments. Each pattern includes symptoms, detection methods, root cause analysis, and recommended actions.

## Failure Pattern 1: Transaction ID Wraparound

### Symptoms

- `WARNING: database must be vacuumed within 1091478515 transactions`
- Application experiencing slow queries or hangs
- Autovacuum cannot keep up with transaction generation
- `pg_stat_activity` shows long-running transactions holding locks
- Database enters "freeze age" warning

### Detection

```sql
-- Check oldest transaction age per database
SELECT datname, datfrozenxid, age(datfrozenxid) as transaction_age
FROM pg_database ORDER BY age(datfrozenxid) DESC;

-- Check table-level freeze age
SELECT relname, age(relfrozenxid) as table_age
FROM pg_class WHERE relkind = 'r' ORDER BY age(relfrozenxid) DESC LIMIT 10;

-- Check running transactions
SELECT pid, usename, query, age(now() - xact_start)
FROM pg_stat_activity WHERE state = 'active';
```

### Root Causes

1. Long-running transactions preventing autovacuum from advancing
2. Autovacuum disabled or misconfigured
3. Application not committing or rolling back transactions
4. High transaction rate overwhelming autovacuum workers
5. Idle-in-transaction sessions holding transaction IDs

### Recommended Actions

1. **Immediate**: Identify and terminate long-running/idle transactions
2. **Urgent**: Force immediate vacuum on affected tables
3. **Short-term**: Increase `autovacuum_vacuum_cost_limit`
4. **Long-term**: Fix application transaction management, add connection pooling
5. **Prevention**: Monitor `datfrozenxid` and `relfrozenxid` proactively

### Confidence: 0.95
### Risk: Critical — data loss possible if freeze age is reached

---

## Failure Pattern 2: Disk Space Exhaustion

### Symptoms

- `FATAL: could not extend file "base/...": No space left on device`
- `PANIC: could not write to file "pg_wal/...": No space left on device`
- PostgreSQL stops accepting connections
- `pg_isready` returns exit code 1
- Application errors: `ERROR: disk full`

### Detection

```bash
# Check system disk space
df -h /var/lib/pgsql
df -h /var/log

# Check PostgreSQL-specific disk usage
SELECT pg_size_pretty(pg_database_size('dbname'));
SELECT pg_size_pretty(pg_wal_size());

# Check pg_wal directory size
ls -la /var/lib/pgsql/data/pg_wal/ | wc -l

# Check archive directory
du -sh /archive/wal/*
```

### Root Causes

1. WAL archive directory not managed — WAL files accumulate indefinitely
2. pg_rewind or pg_basebackup output directory too small
3. Temporary file creation during large sorts or hash joins
4. Large pg_dump operations without space in output location
5. Table/index growth exceeding capacity planning
6. Inactive replication slots preventing WAL removal

### Recommended Actions

1. **Immediate**: Clean up WAL archive directory using `pg_archivecleanup`
2. **Immediate**: Remove inactive replication slots
3. **Immediate**: Clean up pg_basebackup/pg_rewind directories
4. **Short-term**: Implement WAL archive retention policy
5. **Short-term**: Add monitoring for disk usage alerts at 80%
6. **Long-term**: Plan capacity increases based on growth trends

### Confidence: 0.95
### Risk: Critical — database becomes unusable

---

## Failure Pattern 3: Deadlock Storms

### Symptoms

- `ERROR: deadlock detected` in application logs
- Intermittent query failures in application
- `pg_stat_activity` shows concurrent transactions waiting
- `pg_locks` shows multiple waiters
- Application retry logic amplifies the problem

### Detection

```sql
-- Find recent deadlocks
SELECT * FROM pg_stat_activity WHERE state = 'active';
SELECT * FROM pg_locks WHERE NOT granted;

-- Check deadlock rate in logs (search for 'deadlock detected')
-- Use pg_stat_progress_locks in PostgreSQL 12+
SELECT query, wait_event_type, wait_event FROM pg_stat_activity WHERE wait_event_type IS NOT NULL;
```

### Root Causes

1. Transactions accessing tables in different order
2. Application logic acquiring locks in non-deterministic order
3. Bulk operations competing with OLTP queries
4. Missing indexes causing table scans with broader lock scope
5. Dead transaction locks not being released promptly

### Recommended Actions

1. **Immediate**: Identify and terminate blocking sessions
2. **Short-term**: Review transaction ordering in application code
3. **Short-term**: Add appropriate indexes to reduce lock scope
4. **Short-term**: Set `lock_timeout` in application connections
5. **Long-term**: Refactor application to use consistent lock ordering
6. **Long-term**: Consider optimistic concurrency control

### Confidence: 0.90
### Risk: High — degrades application availability

---

## Failure Pattern 4: Replication Lag Crisis

### Symptoms

- `FATAL: the database system is shutting down`
- Application reporting stale data or connection errors
- `pg_stat_replication` shows `state = streaming` with growing lag
- Application reports `ERROR: canceling statement due to statement timeout`
- `replication_lag` exceeds SLA thresholds

### Detection

```sql
-- On primary
SELECT client_addr, state, sent_lsn, write_lsn, flush_lsn, replay_lsn,
       pg_wal_lsn_diff(sent_lsn, replay_lsn) as replay_lag_bytes
FROM pg_stat_replication;

-- On standby
SELECT pg_is_in_recovery(), pg_last_wal_receive_lsn(), pg_last_wal_replay_lsn();

-- Calculate lag
SELECT pg_wal_lsn_diff(
    pg_current_wal_lsn(), pg_last_wal_replay_lsn()
) as replay_lag;
```

### Root Causes

1. Network latency between primary and replica
2. Standby underperforming hardware (CPU, I/O)
3. Large transactions or bulk operations generating WAL
4. Checkpoint settings causing I/O spikes on standby
5. Long-running queries on standby preventing replay
6. Standby autovacuum competing for I/O

### Recommended Actions

1. **Immediate**: Identify and cancel long-running queries on standby
2. **Immediate**: Check network connectivity between nodes
3. **Short-term**: Tune `checkpoint_timeout` and `min_wal_size`
4. **Short-term**: Consider synchronous_commit = off for less critical replicas
5. **Short-term**: Increase standby resources (CPU, I/O)
6. **Long-term**: Evaluate logical replication for read scaling
7. **Long-term**: Consider synchronous replication for critical data

### Confidence: 0.90
### Risk: High — data consistency and availability at risk

---

## Failure Pattern 5: Connection Exhaustion

### Symptoms

- `ERROR: sorry, too many clients already`
- Application connection pool errors
- `pg_isready` returns connection refused
- Application threads blocked waiting for connections
- `pg_stat_activity` shows max_connections reached

### Detection

```sql
-- Check current connection count
SELECT count(*) FROM pg_stat_activity;

-- Check connection breakdown by state
SELECT state, count(*) FROM pg_stat_activity GROUP BY state ORDER BY count DESC;

-- Check idle-in-transaction sessions
SELECT count(*) FROM pg_stat_activity
WHERE state = 'idle in transaction' AND query_start < now() - interval '5 minutes';

-- Check max_connections setting
SHOW max_connections;
SHOW shared_preload_libraries;
```

### Root Causes

1. Connection pool exhaustion (PgBouncer or application pool)
2. Application connection leaks (connections not returned to pool)
3. Long-running idle-in-transaction sessions
4. `max_connections` set too low for workload
5. Connection storm from application restart or failover
6. Database restart causing connection pool to flood

### Recommended Actions

1. **Immediate**: Terminate idle-in-transaction sessions
2. **Immediate**: Verify PgBouncer configuration and pool sizes
3. **Immediate**: Check application connection pool settings
4. **Short-term**: Set `idle_in_transaction_session_timeout`
5. **Short-term**: Increase `max_connections` if capacity permits
6. **Short-term**: Optimize PgBouncer pool_mode and pool_size
7. **Long-term**: Fix application connection leaks
8. **Long-term**: Implement connection monitoring and alerting

### Confidence: 0.90
### Risk: High — all database users affected

---

## Failure Pattern 6: Checkpoint I/O Overload

### Symptoms

- Periodic query latency spikes
- `pg_stat_bgwriter.checkpoints_req` increasing rapidly
- `pg_stat_bgwriter.checkpoints_tim` lagging behind checkpoints requested
- I/O wait visible in `iostat` or `iotop`
- Autovacuum struggling to keep up
- Replication lag increasing during checkpoints

### Detection

```sql
-- Background writer statistics
SELECT checkpoints_timed, checkpoints_requested,
       buffers_checkpoint, buffers_write, buffers_clean,
       maxwritten_clean
FROM pg_stat_bgwriter;

-- Checkpoint configuration
SHOW checkpoint_completion_target;
SHOW checkpoint_timeout;
SHOW max_wal_size;
SHOW min_wal_size;

-- WAL generation rate
SELECT wal_records, wal_bytes FROM pg_stat_wal;
```

### Root Causes

1. `max_wal_size` too small, causing frequent checkpoints
2. `checkpoint_completion_target` too low (old default of 0.5)
3. Write-heavy workload generating WAL faster than background writer can flush
4. Insufficient I/O capacity on storage subsystem
5. `wal_buffers` too small

### Recommended Actions

1. **Immediate**: Monitor checkpoint frequency
2. **Short-term**: Increase `max_wal_size` to 4-8GB (or more for heavy workloads)
3. **Short-term**: Set `checkpoint_completion_target = 0.9`
4. **Short-term**: Set `checkpoint_timeout` to 15-30 minutes
5. **Short-term**: Increase `wal_buffers` to 64MB
6. **Long-term**: Upgrade storage subsystem I/O capacity
7. **Long-term**: Implement I/O scheduling policies (cgroups)

### Confidence: 0.85
### Risk: Medium — performance degradation

---

## Failure Pattern 7: Autovacuum Failure

### Symptoms

- `n_dead_tup` growing on critical tables
- `pg_stat_progress_vacuum` not showing activity
- Autovacuum worker processes consuming excessive CPU or I/O
- `autovacuum_freeze_max_age` approaching limits
- Application performance degradation (increased sequential scans)

### Detection

```sql
-- Table-level vacuum status
SELECT relname, n_live_tup, n_dead_tup,
       last_autovacuum, last_autoanalyze,
       vacuum_count, analyze_count
FROM pg_stat_user_tables
ORDER BY n_dead_tup DESC LIMIT 20;

-- Autovacuum configuration
SHOW autovacuum;
SHOW autovacuum_max_workers;
SHOW autovacuum_vacuum_scale_factor;
SHOW autovacuum_vacuum_threshold;

-- Check for autovacuum failures in logs
-- Search for 'autovacuum' in PostgreSQL log files
```

### Root Causes

1. `autovacuum_vacuum_cost_delay` too low, causing I/O throttling
2. `autovacuum_vacuum_cost_limit` too low for workload
3. Large single transaction preventing autovacuum advancement
4. Table with extremely high write rate (millions of inserts/hour)
5. Autovacuum disabled in `postgresql.conf`
6. Storage I/O saturated by other processes

### Recommended Actions

1. **Immediate**: Run manual VACUUM on tables with highest `n_dead_tup`
2. **Immediate**: Check and fix autovacuum configuration
3. **Short-term**: Set table-level autovacuum overrides for critical tables
4. **Short-term**: Increase `autovacuum_vacuum_cost_limit` to 2000-4000
5. **Short-term**: Set `autovacuum_vacuum_scale_factor` to 0.05 for write-heavy tables
6. **Long-term**: Monitor autovacuum metrics and alert on threshold breaches
7. **Long-term**: Review application transaction patterns

### Confidence: 0.90
### Risk: Medium to High — affects query performance and xid wraparound

---

## Failure Pattern 8: WAL Archive Exhaustion

### Symptoms

- `FATAL: could not write to file "pg_wal/...": No space left on device`
- `ERROR: archive_mode is enabled but archival failed`
- WAL files accumulating in archive directory
- `pg_stat_archiver.failed_count` increasing
- Replication lag on replicas with active slots

### Detection

```sql
-- Archiver statistics
SELECT archived_count, last_archived_wal, last_archived_time,
       failed_count, last_failed_wal, last_failed_time
FROM pg_stat_archiver;

-- WAL directory usage
SELECT pg_size_pretty(pg_wal_size());

-- Replication slots consuming WAL
SELECT slot_name, slot_type, active, restart_lsn,
       pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn) as retained_wal_bytes
FROM pg_replication_slots;
```

### Root Causes

1. Archive retention policy not implemented or too generous
2. Archive command failing silently
3. Replication slots not cleaned up after replica removal
4. Archive storage not monitored or alerting not configured
5. WAL level set to `replica` when not needed (should be `logical` only if using logical replication)

### Recommended Actions

1. **Immediate**: Implement WAL archive cleanup using `pg_archivecleanup`
2. **Immediate**: Clean up inactive replication slots
3. **Immediate**: Verify `archive_command` syntax and error handling
4. **Short-term**: Implement automated archive retention policies
5. **Short-term**: Set up monitoring for archive directory disk usage
6. **Long-term**: Review WAL level configuration
7. **Long-term**: Implement archive monitoring with alerting

### Confidence: 0.90
### Risk: High — prevents WAL archiving and can cause disk space issues

---

## Failure Pattern 9: Index Bloat

### Symptoms

- Slow queries on large tables
- High disk usage for index-heavy tables
- `pg_stat_user_tables` shows low `idx_scan` but high index size
- `pg_freespace` shows significant empty pages in indexes
- Index size far exceeds table size

### Detection

```sql
-- Index bloat estimation
SELECT
    nspname,
    relname AS table_name,
    indexrelname AS index_name,
    pg_size_pretty(pg_relation_size(indexrelid)) AS index_size,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
ORDER BY pg_relation_size(indexrelid) DESC LIMIT 20;

-- Estimate bloat percentage
SELECT
    schemaname,
    tablename,
    indexname,
    pg_size_pretty(pg_relation_size(indexname::regclass)) AS index_size
FROM pg_indexes
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY pg_relation_size(indexname::regclass) DESC;
```

### Root Causes

1. Frequent UPDATE and DELETE operations on indexed columns
2. VACUUM not keeping up with dead tuples in indexes
3. Large B-tree indexes with random insert patterns
4. `maintenance_work_mem` too small for efficient VACUUM
5. No index reuse — indexes not scanned frequently

### Recommended Actions

1. **Immediate**: Identify bloatiest indexes
2. **Short-term**: Run `VACUUM (VERBOSE, ANALYZE)` on affected tables
3. **Short-term**: Use `REINDEX CONCURRENTLY` for online rebuild (PostgreSQL 15+)
4. **Short-term**: Increase `maintenance_work_mem` for faster VACUUM
5. **Short-term**: Remove unused indexes
6. **Long-term**: Monitor index usage and prune unused indexes
7. **Long-term**: Consider partitioning to reduce index sizes

### Confidence: 0.85
### Risk: Medium — performance degradation, wasted storage

---

## Failure Pattern 10: Shared Memory / Kernel Configuration Issues

### Symptoms

- PostgreSQL fails to start
- `ERROR: could not create shared memory segment: Invalid argument`
- `PANIC: could not allocate shared memory`
- `LOG: shared_buffers is larger than available kernel shared memory`
- `FATAL: the postmaster has exited`
- System logs show OOM killer or shared memory allocation failures

### Detection

```bash
# Check shared memory limits
cat /proc/sys/kernel/shmmax
cat /proc/sys/kernel/shmall
sysctl -a | grep shm

# Check PostgreSQL shared_buffers configuration
psql -c "SHOW shared_buffers;"
psql -c "SHOW effective_cache_size;"

# Check available system memory
free -h
cat /proc/meminfo

# Check PostgreSQL logs
grep -i "shared memory\|shmmax\|shared_buffers" /var/log/postgresql/*.log
```

### Root Causes

1. `shmmax` kernel parameter too low for `shared_buffers` setting
2. `shmall` too low for total shared memory usage
3. 32-bit PostgreSQL on systems needing > 4GB shared memory
4. Container/docker shared memory limits
5. Insufficient physical RAM on the system

### Recommended Actions

1. **Immediate**: Increase `kernel.shmmax` to at least 2x `shared_buffers`
2. **Immediate**: Increase `kernel.shmall` appropriately
3. **Short-term**: Adjust `shared_buffers` to fit available memory
4. **Short-term**: Verify container shared memory limits (shm_size)
5. **Long-term**: Plan hardware upgrades for larger `shared_buffers` needs
6. **Long-term**: Consider 64-bit PostgreSQL installations

### Configuration Changes

```bash
# /etc/sysctl.conf additions:
kernel.shmmax = <at least 2x shared_buffers>
kernel.shmall = <total shared memory pages>

# Apply:
sysctl -p
```

### Confidence: 0.90
### Risk: Critical — PostgreSQL cannot start

---

## Failure Pattern Summary

| Pattern | Detection Confidence | Risk | Frequency |
|---------|---------------------|------|-----------|
| Transaction ID Wraparound | 0.95 | Critical | Low (but devastating) |
| Disk Space Exhaustion | 0.95 | Critical | Medium |
| Deadlock Storms | 0.90 | High | Medium |
| Replication Lag Crisis | 0.90 | High | Medium |
| Connection Exhaustion | 0.90 | High | Medium |
| Checkpoint I/O Overload | 0.85 | Medium | Medium |
| Autovacuum Failure | 0.90 | Medium-High | Medium |
| WAL Archive Exhaustion | 0.90 | High | Medium |
| Index Bloat | 0.85 | Medium | High |
| Shared Memory Issues | 0.90 | Critical | Low |