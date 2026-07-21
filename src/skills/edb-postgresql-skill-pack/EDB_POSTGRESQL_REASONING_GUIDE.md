# EDB PostgreSQL Diagnostic Reasoning Guide

## Purpose

This guide provides the diagnostic reasoning framework for PostgreSQL issues. It uses decision trees and structured reasoning to guide investigation from symptoms to root causes.

## Diagnostic Framework

### Phase 1: Identify the Problem Class

When a PostgreSQL issue is reported, first classify it:

```
[SEVERITY] Issue Classification

Class: [Performance | Availability | Replication | Backup | Security | Capacity]
Impact: [Single query | Single database | Entire cluster | Replica-specific]
Duration: [Immediate | Intermittent | Gradual | Sudden]
Scope: [One table | One query | All queries | All databases]
```

### Phase 2: Evidence Collection Decision Tree

```
Start
├── Is server responding? (pg_isready)
│   ├── Yes → Check application symptoms
│   └── No → [Availability Issue] Go to Phase 3
├── Is replication affected? (pg_stat_replication)
│   ├── Yes → [Replication Issue] Go to Replication Tree
│   └── No → Continue
├── Is disk space adequate? (df, pg_size_pretty)
│   ├── Yes → Continue
│   └── No → [Capacity Issue] Go to Capacity Tree
├── Are locks involved? (pg_locks, pg_stat_activity)
│   ├── Yes → [Lock Issue] Go to Lock Tree
│   └── No → Continue
└── Performance issue? (pg_stat_statements, EXPLAIN ANALYZE)
    ├── Yes → [Performance Issue] Go to Performance Tree
    └── No → [Other] Collect more evidence
```

### Phase 3: Core Availability Decision Tree

```
Server not responding
├── Check PostgreSQL log file
│   ├── PANIC or FATAL found
│   │   ├── "disk full" → [Disk Space Issue] → Clean WAL archives
│   │   ├── "shared memory" → [Shared Memory Issue] → Increase shmmax
│   │   ├── "FATAL: could not create shared memory" → [Kernel Issue]
│   │   ├── "PANIC: could not locate a valid checkpoint record" → [WAL Corruption]
│   │   └── Other PANIC → [Data Corruption] → Check pg_controldata
│   ├── LOG: "could not accept new connection"
│   │   ├── "too many connections" → [Connection Exhaustion]
│   │   ├── "no more connections allowed" → [Max Connections]
│   │   └── "connection refused" → [Postmaster down]
│   └── No error → [Process Issue] → Check postmaster.pid
│
├── Check postmaster.pid file
│   ├── File exists but process dead → [Stale PID] → Remove postmaster.pid
│   ├── File exists and process alive → [Postmaster alive] → Check pg_isready
│   └── File missing → [No previous run] → Check if process exists
│
├── Check data directory permissions
│   └── Wrong owner → [Permission Issue] → chown postgres:postgres data_dir
│
└── Check port availability
    └── Port in use → [Port Conflict] → Check for another PostgreSQL instance
```

### Phase 4: Performance Decision Tree

```
Performance issue detected
├── Single query slow? (pg_stat_statements + EXPLAIN ANALYZE)
│   ├── Table scan (seq_scan high, idx_scan low)
│   │   ├── No index → [Missing Index] → CREATE INDEX
│   │   ├── Index not used (use_index = f) → [Index Selection Issue]
│   │   │   ├── Cost too low → [Statistics Stale] → ANALYZE
│   │   │   └── Index too broad → [Index Bloat] → VACUUM/REINDEX
│   │   └── Index exists but not matching WHERE → [Bad Index Design]
│   │       → Consider different index key columns
│   ├── High CPU on query
│   │   ├── Join is hash join → [Large Hash Join] → Check join selectivity
│   │   ├── Nested loop → [Small result set with high per-row cost]
│   │   └── Sort/Merge → [Memory pressure] → Increase work_mem
│   └── High I/O
│       ├── Buffer misses → [Cache miss] → Check shared_buffers
│       ├── Random I/O → [No index / Bad index] → Review indexes
│       └── Sequential I/O → [Table scan / Large scan] → Review queries
│
├── All queries slow?
│   ├── Check CPU usage (top, htop)
│   │   ├── CPU saturated → [CPU bottleneck] → Scale up or optimize queries
│   │   └── CPU low → [Not CPU-bound] → Check I/O
│   ├── Check I/O wait (iostat, vmstat)
│   │   ├── High I/O wait → [I/O bottleneck] → Check storage, checkpointing
│   │   └── Low I/O wait → [Not I/O-bound] → Check locks or memory
│   ├── Check lock waits (pg_locks)
│   │   ├── Many waits → [Lock contention] → Review transactions
│   │   └── Few waits → [Not lock-bound]
│   └── Check memory (free, vmstat)
│       ├── High swap → [Memory pressure] → Increase RAM or reduce work_mem
│       └── Low swap → [Memory adequate]
│
└── Query plan changed?
    ├── Different plan than before
    │   ├── Statistics stale → [Statistics Issue] → ANALYZE table
    │   ├── Parameter sensitive → [Parameter Sniffing] → Use plan guides
    │   └── Config changed → [Config Change] → Review recent changes
    └── Same plan but slower
        ├── Data distribution changed → [Data Skew] → Check data changes
        ├── System load increased → [System Load] → Check resources
        └── Cache warmed → [Cache Effect] → Check buffer usage
```

### Phase 5: Replication Decision Tree

```
Replication issue detected
├── Replica not connecting? (pg_stat_replication shows no rows for this replica)
│   ├── pg_isready on replica: No → [Replica Down] → pg_ctl start replica
│   ├── pg_isready on replica: Yes
│   │   ├── Connection error in logs → [Auth/Network] → Check pg_hba.conf, network
│   │   └── No connection error → [Slot Issue] → Check replication slot status
│   └── Slot inactive → [Inactive Slot] → Check pg_replication_slots
│
├── Replication lag high? (pg_wal_lsn_diff on primary)
│   ├── Lag increasing → [Lag Growing]
│   │   ├── Replica I/O slow → [Replica I/O] → Check replica I/O capacity
│   │   ├── Replica CPU high → [Replica CPU] → Check replica CPU
│   │   ├── Checkpoint on replica → [Checkpoint I/O] → Tune checkpoint_timeout
│   │   └── Long queries on replica → [Query Blocking] → Cancel long queries on standby
│   └── Lag steady → [Lag Maintained] → Check if within SLA
│
├── Replication slot inactive?
│   ├── Slot type physical
│   │   ├── Replica removed → [Orphaned Slot] → DROP slot
│   │   └── Replica down → [Down Replica] → Fix replica, slot will resume
│   └── Slot type logical
│       ├── Consumer stopped → [Logical Consumer Down] → Restart consumer
│       └── Publication dropped → [Publication Issue] → Check PUBLICATION status
│
├── WAL position diverged? (pg_wal_lsn_diff between primary and replica)
│   ├── Replica behind primary by many LSNs
│   │   ├── WAL archived → [Archive Available] → WAL should replay
│   │   └── WAL not archived → [WAL Loss] → May need pg_basebackup re-sync
│   └── Replica ahead of primary → [Split Brain] → Check replication topology
│
└── Promotion attempted? (pg_ctl promote)
    ├── Promotion failed → [Promotion Issue] → Check logs for errors
    ├── Promotion succeeded → [Promotion Successful] → Verify new primary
    └── Old primary came back → [Split Brain Risk] → Check if data diverged
```

### Phase 6: Capacity Decision Tree

```
Capacity issue suspected
├── Disk space low? (df, du)
│   ├── pg_wal directory large → [WAL Accumulation]
│   │   ├── Archive not working → [Archive Failure] → Fix archive_command
│   │   ├── Slots preventing cleanup → [Active Slots] → Review replication slots
│   │   └── Archive not needed → [Unnecessary Archive] → Disable if not needed
│   ├── Database growing → [Database Growth] → Plan capacity increase
│   ├── Tablespace full → [Tablespace Full] → Add more space to tablespace
│   └── Temporary files → [Temp Files] → Increase work_mem, optimize queries
│
├── Connection count high? (pg_stat_activity vs max_connections)
│   ├── Idle-in-transaction → [Leaked Transactions] → Set idle_in_transaction_session_timeout
│   ├── Idle → [Idle Connections] → Check PgBouncer pool config
│   └── Active → [Load Increase] → Scale or optimize
│
├── Table size growing? (pg_relation_size, pg_total_relation_size)
│   ├── Expected growth → [Normal Growth] → Plan for it
│   ├── Unexpected growth → [Anomaly] → Investigate data patterns
│   └── Table bloat → [Bloat] → VACUUM FULL or pg_repack
│
└── Memory pressure? (free, shared_buffers vs RAM)
    ├── Swap usage → [Swap] → Increase RAM or reduce shared_buffers
    ├── Buffer cache misses → [Cache Misses] → Increase shared_buffers
    └── work_mem high → [work_mem Issue] → Reduce work_mem
```

### Phase 7: Lock Decision Tree

```
Lock issue detected
├── Session waiting on lock? (pg_locks + pg_blocking_pids)
│   ├── Waiting for AccessExclusiveLock
│   │   ├── ALTER TABLE → [DDL Lock] → Wait for DDL to complete or cancel DDL
│   │   ├── VACUUM FULL → [Full Vacuum Lock] → Cancel VACUUM FULL, use regular VACUUM
│   │   └── TRUNCATE → [Truncate Lock] → Cancel TRUNCATE or wait
│   ├── Waiting for RowExclusiveLock
│   │   ├── UPDATE/DELETE → [Row Lock] → Check which rows are locked
│   │   └── INSERT → [Insert Lock] → Check for duplicate key conflicts
│   └── Waiting for ShareLock
│       ├── SELECT FOR UPDATE → [For Update Lock] → Cancel conflicting query
│       └── CREATE INDEX CONCURRENTLY → [Index Lock] → Wait or cancel index build
│
├── Deadlock detected? (pg_stat_activity + logs)
│   ├── Two-way deadlock → [Simple Deadlock] → PostgreSQL will auto-resolve
│   ├── N-way deadlock → [Complex Deadlock] → More complex resolution
│   └── Frequent deadlocks → [Deadlock Storm] → Review transaction ordering
│
└── Lock not releasing?
    ├── Transaction not committed → [Long Transaction] → Cancel or commit transaction
    ├── Application crash → [Crashed App] → Wait for PostgreSQL to detect
    ├── Network disconnect → [Net Issue] → PostgreSQL should detect
    └── Idle-in-transaction → [Idle Session] → Set idle_in_transaction_session_timeout
```

## Common Diagnostic Queries

### Quick Health Check

```sql
-- One-line health summary
SELECT
    (SELECT count(*) FROM pg_stat_activity) as active_connections,
    (SELECT count(*) FROM pg_stat_activity WHERE state = 'idle in transaction') as idle_in_txn,
    (SELECT pg_is_in_recovery()) as is_standby,
    (SELECT pg_size_pretty(pg_database_size('template1'))) as template1_size;
```

### Connection Health

```sql
SELECT state, count(*) as cnt,
       min(query_start) as oldest,
       max(query_start) as newest
FROM pg_stat_activity
WHERE datname = current_database()
GROUP BY state
ORDER BY cnt DESC;
```

### Lock Analysis

```sql
SELECT
    waiting.pid as waiting_pid,
    waiting.usename as waiting_user,
    waiting.query as waiting_query,
    blocking.pid as blocking_pid,
    blocking.usename as blocking_user,
    blocking.query as blocking_query
FROM pg_locks waiting
JOIN pg_locks blocking ON waiting.locktype = blocking.locktype
    AND waiting.relation = blocking.relation
    AND waiting.pid != blocking.pid
WHERE NOT waiting.granted AND blocking.granted;
```

### Replication Lag

```sql
SELECT
    client_addr,
    state,
    sent_lsn,
    write_lsn,
    flush_lsn,
    replay_lsn,
    pg_wal_lsn_diff(sent_lsn, replay_lsn) as replay_lag_bytes,
    pg_wal_lsn_diff(sent_lsn, write_lsn) as write_lag_bytes,
    pg_wal_lsn_diff(write_lsn, flush_lsn) as flush_lag_bytes
FROM pg_stat_replication;
```

### Table Health

```sql
SELECT
    relname,
    n_live_tup,
    n_dead_tup,
    last_vacuum,
    last_autovacuum,
    last_analyze,
    last_autoanalyze,
    pg_size_pretty(pg_total_relation_size(relid)) as total_size
FROM pg_stat_user_tables
ORDER BY n_dead_tup DESC NULLS LAST
LIMIT 20;
```

## Evidence Collection Checklist

Before any diagnosis:

- [ ] PostgreSQL log file reviewed for errors
- [ ] `pg_isready` status checked
- [ ] `pg_stat_activity` examined for active sessions
- [ ] Disk space verified (system + data directory)
- [ ] Replication status checked (if applicable)
- [ ] Lock status examined (if performance issue)
- [ ] Configuration values checked (`SHOW ALL`)
- [ ] Backup status verified (if backup-related issue)
- [ ] Recent changes reviewed (config, DDL, deployment)
- [ ] External factors assessed (network, storage, OS)

## Confidence Scoring

After diagnosis, assign a confidence level:

| Confidence | Meaning | Action |
|------------|---------|--------|
| 0.95+ | Near-certain — evidence directly confirms root cause | Recommend immediate action |
| 0.90-0.94 | High — evidence strongly supports diagnosis | Recommend action with high confidence |
| 0.85-0.89 | Medium-High — strong evidence, minor ambiguity | Recommend action with caution |
| 0.80-0.84 | Medium — evidence supports but could be alternative | Suggest verification steps |
| 0.70-0.79 | Low-Medium — suggestive, need more evidence | Recommend additional investigation |
| < 0.70 | Low — weak evidence, multiple possible causes | Recommend systematic investigation |

## Escalation Rules

### When to Escalate

1. **Data corruption detected** — Immediate escalation to senior DBA
2. **Production system completely down** — Immediate escalation
3. **Data loss possible** — Immediate escalation
4. **Root cause unknown after evidence collection** — Escalate for additional investigation
5. **Multi-day outage** — Escalate to management
6. **Customer-facing impact** — Escalate to incident management

### When NOT to Escalate

1. Single query performance issue (optimize query)
2. Configuration tuning recommendation (apply if approved)
3. Index creation suggestion (plan during maintenance)
4. Backup verification (test in staging)
5. Monitoring setup assistance (configure and verify)