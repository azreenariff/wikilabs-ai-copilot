# Context Interpretation Guide

## Purpose

This guide explains how to interpret PostgreSQL outputs, logs, metrics, and diagnostic information. It covers reading and understanding the various data sources a DBA or engineer encounters when managing PostgreSQL systems.

## Reading PostgreSQL Log Files

### Log File Location

The default log directory is configured by `log_directory` in postgresql.conf:
```ini
log_directory = 'log'    # Relative to data directory
# or
log_directory = '/var/log/postgresql'   # Absolute path
```

### Log Level Hierarchy

| Level | Description | Use Case |
|-------|-------------|----------|
| LOG | Informational messages | Normal operation tracking |
| INFO | Information with detail | Detailed operation info |
| NOTICE | User notices | Warnings to users |
| WARNING | Warning messages | Potential issues |
| ERROR | Error messages | Recoverable errors |
| FATAL | Fatal errors | Connection-level failures |
| PANIC | Emergency condition | All connections terminated |

### Interpreting Key Log Patterns

**Startup Messages**:
```
LOG:  database system was shut down at 2024-01-15 10:00:00 UTC
LOG:  database system is ready to accept connections
```
→ Clean shutdown followed by successful startup

```
LOG:  entering emergency recovery mode
LOG:  redo starts at 0/1A00028
LOG:  redo done at 0/1A00F58
```
→ Crash recovery performed — data was recovered from WAL

**Connection Messages**:
```
LOG:  connection received: host=10.0.1.50 port=5432
LOG:  connection authorized: user=app_user database=mydb SSL enabled
FATAL:  sorry, too many clients already
```
→ Connection received but rejected due to max_connections limit

**Query Messages**:
```
LOG:  checkpoint starting: time
LOG:  checkpoint complete: wrote 245 buffers (1.5%); 0 WAL file(s) added
LOG:  duration: 1234.567 ms  statement: SELECT ...
```
→ Checkpoint completed normally; query took 1.2 seconds

**Error Messages**:
```
ERROR:  deadlock detected
DETAIL:  Process 1234 waits for ShareLock on transaction 5678; blocked by process 5679.
HINT:  See server log for query details.
```
→ Deadlock detected between processes 1234 and 5679

```
FATAL:  could not extend file "base/16384/12345": No space left on device
HINT:  Check free disk space.
```
→ Disk space exhaustion — critical

**Replication Messages**:
```
LOG:  started streaming replication from primary at 0/5000000
LOG:  redo at 0/5000000, shutdown false
LOG:  consistent recovery state reached at 0/5001234
```
→ Standby successfully received and replayed WAL

```
WARNING:  replication slot "standby1" is inactive and consuming WAL
```
→ Standby disconnected but slot retained — WAL accumulating

## Interpreting pg_stat_activity

The pg_stat_activity view shows all current sessions:

```sql
SELECT pid, usename, datname, client_addr, state,
       query_start, wait_event_type, wait_event,
       query, backend_start, xact_start
FROM pg_stat_activity
ORDER BY query_start;
```

**State Values**:
| State | Meaning | Action |
|-------|---------|--------|
| active | Query executing | Normal |
| idle | Waiting for next query | Check for idle-in-transaction |
| idle in transaction | Transaction open, no query | Potentially problematic |
| idle in transaction (aborted) | Transaction failed, not yet rolled back | Must rollback |
| disabled | Session state tracking disabled | Monitor |

**Wait Events**:
| Wait Event | Meaning |
|-----------|---------|
| Lock | Waiting for a lock |
| BufferPin | Waiting for a buffer to be released |
| ClientRead | Waiting for client to receive data |
| LWLock | Waiting for lightweight lock |
| IPC | Waiting for inter-process communication |
| Timeout | Waiting for a timeout |

**Interpreting Wait Events**:
- `Lock` waits → Check pg_locks for blocking sessions
- `BufferPin` waits → Buffer manager contention
- `ClientRead` waits → Network or client application slow
- `LWLock` waits → Internal PostgreSQL contention

## Interpreting pg_stat_database

Database-level statistics:

```sql
SELECT datname,
       xact_commit, xact_rollback,
       blks_read, blks_hit,
       tup_returned, tup_fetched, tup_inserted, tup_updated, tup_deleted,
       deadlocks, conflicts,
       temp_files, temp_bytes
FROM pg_stat_database
WHERE datname NOT IN ('template0', 'template1');
```

**Cache Hit Ratio Calculation**:
```sql
blks_hit / (blks_hit + blks_read)
```
- Ratio > 0.99: Excellent cache performance
- Ratio > 0.95: Good
- Ratio < 0.95: May need tuning (check shared_buffers, workload pattern)

**Transaction Ratio**:
```sql
xact_commit / (xact_commit + xact_rollback)
```
- High rollback ratio → Check application error handling
- Normal rollback is expected (failed transactions)

**Temp File Activity**:
- `temp_files > 0` indicates operations exceeded work_mem
- `temp_bytes` shows total temp file I/O

## Interpreting pg_stat_user_tables

Table-level statistics:

```sql
SELECT relname,
       n_live_tup, n_dead_tup,
       n_tup_ins, n_tup_upd, n_tup_del,
       last_vacuum, last_autovacuum, last_analyze, last_autoanalyze,
       vacuum_count, autovacuum_count, analyze_count, autoanalyze_count,
       seq_scan, idx_scan, idx_tup_fetch,
       seq_tup_read
FROM pg_stat_user_tables
ORDER BY n_dead_tup DESC;
```

**Key Interpretations**:
- `n_dead_tup` high → Autovacuum may need tuning
- `seq_scan` high + `idx_scan` low → May need indexes
- `last_autovacuum` old + `n_dead_tup` high → Autovacuum not keeping up
- `idx_tup_fetch` vs `seq_tup_read` → Index effectiveness

**Dead Tuple Thresholds**:
- `n_dead_tup < n_live_tup × 0.1`: Normal
- `n_dead_tup > n_live_tup × 0.1`: Consider autovacuum tuning
- `n_dead_tup > 100,000`: Immediate attention

## Interpreting pg_stat_replication

Replication status on primary:

```sql
SELECT client_addr, client_port,
       sent_lsn, write_lsn, flush_lsn, replay_lsn,
       state, sync_state,
       pg_wal_lsn_diff(sent_lsn, replay_lsn) as replay_lag_bytes
FROM pg_stat_replication;
```

**State Values**:
| State | Meaning |
|-------|---------|
| streaming | Actively sending WAL |
| catchup | Catching up after reconnect |
| backup | In backup mode |
| startup | Starting up replication |

**Sync State**:
| Sync State | Meaning |
|-----------|---------|
| async | Asynchronous replication |
| sync | Synchronous standby (confirmed) |
| potential | Potential synchronous standby |

**Lag Interpretation**:
- `replay_lag_bytes < 1MB`: Good
- `replay_lag_bytes 1MB - 100MB`: Monitor
- `replay_lag_bytes > 100MB`: Investigate immediately

## Interpreting pg_stat_bgwriter

Background writer statistics:

```sql
SELECT checkpoints_timed, checkpoints_requested,
       checkpoints_req, buffers_checkpoint,
       buffers_clean, maxwritten_clean,
       buffers_backend, buffers_backend_fsync,
       buffers_alloc
FROM pg_stat_bgwriter;
```

**Key Metrics**:
- `checkpoints_timed` vs `checkpoints_requested`:
  - If timed >> requested: Good (checkpoints triggered by timeout)
  - If requested >> timed: Bad (frequent checkpoints due to high write rate)
- `maxwritten_clean > 0`: Background writer couldn't flush buffers fast enough
  → Increase `bgwriter_lru_maxpages` or `bgwriter_lru_multiplier`
- `buffers_backend_fsync`: High indicates backends doing their own fsync
  → May need more background writer activity

## Interpreting pg_stat_archiver

WAL archiving statistics:

```sql
SELECT archived_count, last_archived_wal, last_archived_time,
       failed_count, last_failed_wal, last_failed_time,
       stats_reset
FROM pg_stat_archiver;
```

**Interpretation**:
- `failed_count = 0`: Archive working correctly
- `failed_count > 0`: Archive failure — investigate archive_command
- `last_archived_time` old: Archiving may be stalled
- `last_failed_wal`: Identifies which WAL file failed to archive

## Interpreting EXPLAIN Plans

**Key Plan Node Types**:
| Node | Meaning |
|------|---------|
| Seq Scan | Sequential table scan |
| Index Scan | Scan using an index |
| Index Only Scan | Scan using index without accessing table |
| Bitmap Heap Scan | Scan using bitmap from multiple indexes |
| Nested Loop Join | Nested loop join |
| Hash Join | Hash join |
| Merge Join | Merge join |
| Sort | Sort operation |
| Limit | LIMIT operation |
| Aggregate | Aggregation (GROUP BY) |

**Cost Interpretation**:
- `cost=X.Y..Y.Y`: Startup cost .. total cost
- `rows`: Estimated number of output rows
- `width`: Estimated average row width in bytes

**Performance Indicators**:
- Seq Scan on large table → May need index
- High estimated rows vs actual → Statistics may be stale (run ANALYZE)
- Nested loop with large outer → Check for better join strategy
- High total cost → Query may need optimization

## Interpreting pg_stat_progress_* Views

Progress views show long-running operation status:

```sql
-- Vacuum progress
SELECT * FROM pg_stat_progress_vacuum;

-- Create index progress
SELECT * FROM pg_stat_progress_create_index;

-- Copy progress
SELECT * FROM pg_stat_progress_copy;
```

**VACUUM Progress Phases**:
1. Removing dead tuples
2. Processing visibility map
3. Processing indexes

**Index Build Progress**:
1. Acquiring lock
2. Building index entries
3. Sorting index entries
4. Building leaf pages
5. Finalizing

## Log Interpretation by Severity

### ERROR Level Logs
Common causes:
- Syntax errors in SQL
- Constraint violations
- Permission denied
- Deadlock detected
- Disk I/O errors

### FATAL Level Logs
Common causes:
- Connection limit exceeded
- Invalid authentication
- SSL handshake failure
- Shutdown request
- Configuration errors

### PANIC Level Logs
Common causes:
- Data corruption detected
- Invalid WAL record
- Critical shared memory error
- System call failure

### WARNING Level Logs
Common causes:
- Replication slot inactive
- Long-running transaction
- Autovacuum delay
- Lock wait timeout approaching
- WAL archive slow

## Metrics Interpretation Guidelines

### Connection Monitoring
```
Connection usage = count(pg_stat_activity) / max_connections × 100
```
- < 50%: Healthy
- 50-80%: Monitor closely
- > 80%: Investigation needed
- > 95%: Critical — possible connection exhaustion

### Cache Hit Ratio
```
Cache hit ratio = blks_hit / (blks_hit + blks_read)
```
- > 0.99: Excellent (OLTP)
- > 0.95: Good
- 0.85-0.95: Monitor (may need tuning)
- < 0.85: Investigate (shared_buffers may be too low)

### Replication Lag
```
lag = pg_wal_lsn_diff(sent_lsn, replay_lsn)
```
- < 1MB: Normal
- 1MB-10MB: Monitor
- 10MB-100MB: Investigation needed
- > 100MB: Critical — immediate attention

### Disk Usage
```
Usage = used_space / total_space × 100
```
- < 80%: Healthy
- 80-90%: Monitor and plan capacity
- 90-95%: Investigation needed
- > 95%: Critical — immediate cleanup needed

## References

- [PostgreSQL Performance Monitoring](https://www.postgresql.org/docs/current/monitoring-stats.html)
- [PostgreSQL Log Messages](https://www.postgresql.org/docs/current/erver-admin-logging.html)
- [PostgreSQL System Catalogs](https://www.postgresql.org/docs/current/catalogs.html)
- [EDB Documentation](https://www.enterprisedb.com/docs/)