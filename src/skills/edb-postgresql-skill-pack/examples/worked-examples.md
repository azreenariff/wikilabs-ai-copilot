# PostgreSQL Worked Examples

## Overview

This document provides detailed, step-by-step examples of common PostgreSQL operational tasks and troubleshooting scenarios. Each example follows the structured diagnostic reasoning approach from this skill pack.

---

## Worked Example 1: Diagnosing Slow Queries

### Scenario

An application reports that a specific query that normally completes in under 100ms is now taking 5-10 seconds. The query is:

```sql
SELECT u.id, u.name, u.email, o.order_date, o.total
FROM users u
JOIN orders o ON u.id = o.user_id
WHERE u.status = 'active'
  AND o.order_date > '2024-01-01'
ORDER BY o.order_date DESC
LIMIT 50;
```

### Step 1: Evidence Collection

```sql
-- Check if the query is in pg_stat_statements
SELECT query, calls, mean_exec_time, max_exec_time
FROM pg_stat_statements
WHERE query LIKE '%users u%JOIN orders o%';

-- Check current pg_stat_activity
SELECT pid, state, query_start, query
FROM pg_stat_activity
WHERE query LIKE '%users u%';

-- Check table statistics
SELECT relname, n_live_tup, last_analyze, last_autoanalyze
FROM pg_stat_user_tables
WHERE relname IN ('users', 'orders');

-- Check existing indexes
SELECT indexname, indexdef
FROM pg_indexes
WHERE tablename IN ('users', 'orders')
ORDER BY tablename, indexname;
```

**Findings**:
- Query mean_exec_time increased from 50ms to 8500ms
- last_autoanalyze for `orders` table was 3 days ago
- `orders` table has 2 million rows, `users` has 500,000 rows
- Existing indexes: `idx_orders_user_id` (on orders.user_id), `idx_users_status` (on users.status)

### Step 2: EXPLAIN ANALYZE

```sql
EXPLAIN (ANALYZE, BUFFERS, VERBOSE)
SELECT u.id, u.name, u.email, o.order_date, o.total
FROM users u
JOIN orders o ON u.id = o.user_id
WHERE u.status = 'active'
  AND o.order_date > '2024-01-01'
ORDER BY o.order_date DESC
LIMIT 50;
```

**Plan Output** (summarized):
```
Limit  (cost=0.00..245000.00 rows=50 width=80)
  -> Nested Loop  (cost=0.00..850000.00 rows=150000 width=80)
     -> Seq Scan on users u  (cost=0.00..15000.00 rows=200000 width=40)
         Filter: (status = 'active')
         Rows Removed by Filter: 300000
     -> Index Scan using orders_user_id_idx on orders o
         Index Cond: (user_id = u.id)
         Filter: (order_date > '2024-01-01')
         Rows Removed by Filter: 1850000
Buffers: shared hit=500000 read=250000
```

**Analysis**:
- Sequential scan on `users` table — filtering 300K rows out of 500K to find active users
- Nested loop join — for each of the 200K active users, looks up matching orders
- Each orders lookup filters out 1.85M rows (most orders are before 2024-01-01)
- Total: 750K buffer hits — very high I/O

### Step 3: Root Cause

The `orders` table has stale statistics (last analyzed 3 days ago) and the join pattern is inefficient:

1. **Stale statistics**: Query planner underestimates the cost of the nested loop
2. **No index on order_date**: Cannot use index for the date filter efficiently
3. **Large result set before LIMIT**: 150K rows estimated, many filtered out

### Step 4: Resolution

**Option A: Add composite index on orders**:
```sql
CREATE INDEX CONCURRENTLY idx_orders_date_user ON orders (order_date DESC, user_id);
```

This index supports:
- Date filtering with index scan (no full scan)
- user_id for the join (covering index)
- DESC matches the ORDER BY

**Option B: Analyze the orders table**:
```sql
ANALYZE orders;
```

**Option C: Add index on users.status**:
```sql
-- Check if idx_users_status already exists and covers the query
```

### Step 5: Verify

After creating the index:
```sql
EXPLAIN (ANALYZE, BUFFERS)
SELECT u.id, u.name, u.email, o.order_date, o.total
FROM users u
JOIN orders o ON u.id = o.user_id
WHERE u.status = 'active'
  AND o.order_date > '2024-01-01'
ORDER BY o.order_date DESC
LIMIT 50;
```

**New Plan**:
```
Limit  (cost=0.43..15.50 rows=50 width=80)
  -> Nested Loop  (cost=0.43..4750.00 rows=150000 width=80)
     -> Index Scan using idx_orders_date_user on orders o
          Index Cond: (order_date > '2024-01-01')
          Rows Removed by Filter: 150000
     -> Index Scan using idx_users_id on users u
          Index Cond: (id = o.user_id)
          Filter: (status = 'active')
Buffers: shared hit=500 read=1000
```

**Results**:
- Execution time: 12ms (down from 8500ms)
- Buffer usage: 1500 (down from 750000)
- Plan changed from Seq Scan to Index Scan

### Step 6: Prevention

1. Monitor `last_autoanalyze` for critical tables
2. Add `pg_cron` job for regular ANALYZE on large tables
3. Set `default_statistics_target = 200` for frequently queried tables
4. Monitor pg_stat_statements for performance regressions
5. Review query plans after schema changes

### Confidence: 0.95

---

## Worked Example 2: Resolving Replication Lag

### Scenario

The monitoring dashboard shows that a standby server's replication lag has grown from < 1MB to 500MB over the past hour. The application is experiencing stale data reads from the standby.

### Step 1: Evidence Collection

```sql
-- On primary: Check replication status
SELECT client_addr, client_port, state, sync_state,
       sent_lsn, write_lsn, flush_lsn, replay_lsn,
       pg_wal_lsn_diff(sent_lsn, replay_lsn) as replay_lag_bytes,
       pg_size_pretty(pg_wal_lsn_diff(sent_lsn, replay_lsn)) as replay_lag_pretty
FROM pg_stat_replication;

-- On primary: Check replication slots
SELECT slot_name, slot_type, active, restart_lsn,
       pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn)) as retained_wal
FROM pg_replication_slots;

-- On primary: Check WAL generation
SELECT pg_size_pretty(pg_wal_size());

-- On standby: Check if server is accepting queries
-- pg_isready -h standby-host
```

**Findings**:
- Standby: standby-01, replay lag: 500MB, state: streaming
- Replication slot: pg_repl_slot_01, active: true
- pg_wal_size: 2.5GB (normal range: 1-2GB)
- Standby is accepting connections
- Other standbys are not affected (only standby-01 has high lag)

### Step 2: Investigate Standby

```sql
-- On standby: Check what's preventing WAL replay
SELECT pid, usename, state, query, query_start
FROM pg_stat_activity
WHERE state = 'active' AND pid != pg_backend_pid();

-- On standby: Check background writer
SELECT checkpoints_timed, checkpoints_requested,
       buffers_checkpoint, buffers_clean
FROM pg_stat_bgwriter;

-- On standby: Check for long-running queries
SELECT pid, usename, query, now() - query_start as duration
FROM pg_stat_activity
WHERE state = 'active' AND query_start < now() - interval '1 minute';

-- On standby: Check I/O usage
-- Run: iostat -x 1
```

**Findings**:
- One long-running analytical query on standby:
  - `SELECT count(*), avg(total) FROM orders WHERE order_date > '2023-01-01' GROUP BY user_id;`
  - Duration: 45 minutes and still running
  - Query is doing a full table scan on orders table
- I/O wait is high (80%+ wait time on standby)
- Checkpoints are frequent (requested > timed)

### Step 3: Root Cause

The long-running analytical query on the standby is blocking WAL replay:
1. PostgreSQL replay must wait for concurrent queries to complete
2. The query is doing a full table scan causing I/O contention
3. WAL accumulation because replay cannot keep up with I/O

### Step 4: Resolution

**Immediate Actions**:
```sql
-- On standby: Cancel the long-running query
SELECT pg_cancel_backend(<pid>);
-- If still stuck, terminate:
SELECT pg_terminate_backend(<pid>);
```

**Short-term Actions**:
1. Monitor replay lag after query cancellation
2. Verify lag decreases: `SELECT pg_wal_lsn_diff(sent_lsn, replay_lsn) FROM pg_stat_replication;`
3. Consider setting statement_timeout for analytical queries:
   ```sql
   ALTER ROLE analyst_user SET statement_timeout = '300000';
   ```

**Medium-term Actions**:
1. Schedule analytical queries during low-traffic periods
2. Move analytical workload to a dedicated reporting database
3. Create materialized views for common analytical queries
4. Consider pg_stat_statements to identify long-running queries

**Long-term Actions**:
1. Implement read/write routing for analytical queries
2. Consider a separate read replica for reporting
3. Use partitioning to improve analytical query performance
4. Consider logical replication for reporting database

### Step 5: Verify

After cancellation:
```sql
-- On primary: Monitor replay lag
SELECT client_addr,
       pg_wal_lsn_diff(sent_lsn, replay_lsn) as replay_lag_bytes
FROM pg_stat_replication
WHERE client_addr = 'standby-ip';
```

**Expected**: Replay lag should decrease steadily as WAL is replayed.

### Step 6: Prevention

1. Set `statement_timeout` for analytical users
2. Use `pg_stat_activity` to monitor long-running queries on standby
3. Move analytical queries to dedicated reporting infrastructure
4. Consider `autovacuum` tuning on standby to reduce I/O contention
5. Monitor I/O on standby continuously

### Confidence: 0.90

---

## Worked Example 3: Recovering from Disk Space Exhaustion

### Scenario

The production PostgreSQL server stops accepting new connections with the error: `FATAL: could not extend file "base/16384/12345": No space left on device`.

### Step 1: Evidence Collection

```bash
# Check system disk space
df -h

# Check PostgreSQL data directory
df -h /var/lib/pgsql/data

# Check data directory contents by size
du -sh /var/lib/pgsql/data/* | sort -rh | head -20

# Check pg_wal directory
ls -la /var/lib/pgsql/data/pg_wal/ | head -20
du -sh /var/lib/pgsql/data/pg_wal/

# Check archive directory
du -sh /archive/wal/

# Check PostgreSQL logs
tail -100 /var/log/postgresql/postgresql-*.log | grep -i "space\|disk\|full\|FATAL\|PANIC"
```

**Findings**:
- Root disk at 97% usage
- pg_wal directory: 45GB of WAL files (abnormally large — typically 1-2GB)
- Archive directory: 200GB of WAL files
- No replication slots were consuming WAL
- Logs show: `FATAL: could not extend file "base/...": No space left on device`

### Step 2: Identify the Problem

```sql
-- If server can still accept some connections:
SELECT slot_name, slot_type, active,
       pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn)) as retained_wal
FROM pg_replication_slots;

-- Check if WAL archiver is failing:
SELECT archived_count, failed_count, last_failed_wal, last_failed_time
FROM pg_stat_archiver;
```

**Findings**:
- Two inactive replication slots consuming 40GB of WAL
- Archive command failing (failed_count > 0) — archive disk also full
- Archive timeout: 0 (not forced)

### Step 3: Immediate Resolution

**Step 3a: Clean up inactive replication slots**:
```sql
-- Identify inactive slots
SELECT slot_name, active FROM pg_replication_slots WHERE NOT active;

-- Drop inactive slots (AFTER confirming standbys are disconnected)
SELECT pg_drop_replication_slot('inactive_slot_1');
SELECT pg_drop_replication_slot('inactive_slot_2');
```

**Step 3b: Clean up WAL archive**:
```bash
# Find the oldest WAL still needed (after dropping slots, this should be recent)
psql -c "SELECT pg_walfile_name(pg_current_wal_lsn());"

# Clean old WAL files — keep only the last 100 files
cd /archive/wal/
ls -1 | sort | head -n -100 | xargs rm -f

# Clean pg_wal directory (only after confirming archiving works)
pg_archivecleanup /archive/wal/ $(psql -t -c "SELECT pg_walfile_name(pg_current_wal_lsn());")

# Or use find (remove files older than 24 hours):
find /archive/wal/ -type f -mtime +1 -delete
```

**Step 3c: Check disk space**:
```bash
df -h /var/lib/pgsql/data
df -h /archive
```

### Step 4: Verify Recovery

```bash
# Check if PostgreSQL is accepting connections
pg_isready -h localhost -p 5432

# If still not working, restart PostgreSQL:
pg_ctl restart -D /var/lib/pgsql/data

# Verify connections work:
psql -c "SELECT 1;"
```

### Step 5: Prevent Recurrence

**Implement WAL archive retention**:
```bash
# Create automated cleanup script
cat > /usr/local/bin/clean_wal_archive.sh << 'EOF'
#!/bin/bash
# Clean WAL files older than 7 days
find /archive/wal/ -type f -mtime +7 -delete
# Clean pg_wal if archive is working
if [ -n "$(pg_isready -t 5)" ]; then
    OLDEST_SLOT=$(psql -t -c "SELECT COALESCE(min(restart_lsn), pg_current_wal_lsn()) FROM pg_replication_slots;")
    WAL_FILE=$(psql -t -c "SELECT pg_walfile_name('${OLDEST_SLOT}');")
    pg_archivecleanup /archive/wal/ "$WAL_FILE"
fi
EOF
chmod +x /usr/local/bin/clean_wal_archive.sh
```

**Schedule automated cleanup**:
```bash
# Add to crontab (run every 6 hours)
0 */6 * * * /usr/local/bin/clean_wal_archive.sh >> /var/log/wal_cleanup.log 2>&1
```

**Set up monitoring**:
```sql
-- Alert on archive failures
SELECT archived_count, failed_count
FROM pg_stat_archiver;

-- Alert on replication slot retention
SELECT slot_name,
       pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn))
FROM pg_replication_slots;
```

**Configuration changes**:
```ini
# postgresql.conf
archive_timeout = 300           # Force WAL switch every 5 minutes
```

### Step 6: Capacity Planning

- Set up disk usage alerts at 80%
- Implement WAL archive retention policy (7 days recommended minimum)
- Monitor disk usage trends
- Plan for storage capacity increases based on growth

### Confidence: 0.95

---

## Worked Example 4: Fixing Autovacuum Performance Issue

### Scenario

Over the past week, the `transactions` table has been showing steadily increasing dead tuple count. The table has 50 million live tuples and has reached 10 million dead tuples. The table is used for both OLTP writes and reporting reads.

### Step 1: Evidence Collection

```sql
-- Check table vacuum status
SELECT relname, n_live_tup, n_dead_tup,
       last_autovacuum, last_autoanalyze,
       vacuum_count, autovacuum_count
FROM pg_stat_user_tables
WHERE relname = 'transactions';

-- Check autovacuum configuration
SHOW autovacuum_max_workers;
SHOW autovacuum_vacuum_scale_factor;
SHOW autovacuum_vacuum_threshold;
SHOW autovacuum_vacuum_cost_limit;
SHOW autovacuum_vacuum_cost_delay;

-- Check if autovacuum is actually running
SELECT * FROM pg_stat_progress_vacuum;

-- Check if there are long-running transactions
SELECT pid, usename, state,
       age(now() - xact_start) as xact_age,
       left(query, 100) as query
FROM pg_stat_activity
WHERE state = 'active' AND query_start < now() - interval '1 hour';
```

**Findings**:
- transactions: 50M live, 10M dead tuples
- last_autovacuum: 5 days ago
- autovacuum_max_workers: 3 (default)
- autovacuum_vacuum_scale_factor: 0.2 (default — means vacuum triggers at 20% dead)
- autovacuum_vacuum_threshold: 50 (default)
- One long-running transaction (3 hours old) blocking vacuum
- No autovacuum workers running

### Step 2: Root Cause

Two issues:
1. **Long-running transaction**: Preventing autovacuum from advancing transaction IDs
2. **Insufficient autovacuum workers**: Only 3 workers for a write-heavy system

The long-running transaction is likely from a reporting query that never committed.

### Step 3: Resolution

**Immediate — Terminate blocking transaction**:
```sql
-- Identify the blocking transaction
SELECT pid, usename, datname, state, xact_start, query_start,
       left(query, 100) as query
FROM pg_stat_activity
WHERE state = 'active' AND xact_start < now() - interval '1 hour'
ORDER BY xact_start;

-- Terminate the blocking transaction
SELECT pg_terminate_backend(<pid>);
```

**Short-term — Increase autovacuum workers**:
```ini
# postgresql.conf
autovacuum_max_workers = 4
autovacuum_vacuum_cost_limit = 4000
autovacuum_vacuum_scale_factor = 0.05
```
```sql
-- Table-level override for transactions table
ALTER TABLE transactions SET (autovacuum_vacuum_scale_factor = 0.01);
ALTER TABLE transactions SET (autovacuum_vacuum_threshold = 1000);
SELECT pg_reload_conf();
```

**Force immediate vacuum on transactions**:
```sql
-- Run VACUUM (doesn't block reads)
VACUUM transactions;
```

### Step 4: Verify

```sql
-- Check dead tuple count after vacuum
SELECT relname, n_live_tup, n_dead_tup, last_autovacuum
FROM pg_stat_user_tables
WHERE relname = 'transactions';

-- Check autovacuum is running
SELECT * FROM pg_stat_progress_vacuum;

-- Monitor for 30 minutes
SELECT relname, n_dead_tup, last_autovacuum
FROM pg_stat_user_tables
WHERE relname = 'transactions';
```

**Expected**: Dead tuple count should decrease and stay low.

### Step 5: Prevention

1. **Monitor n_dead_tup** weekly for all large tables
2. **Set table-level overrides** for critical tables
3. **Increase autovacuum_max_workers** for write-heavy systems
4. **Increase autovacuum_vacuum_cost_limit** for faster vacuum
5. **Set idle_in_transaction_session_timeout** to catch leaked transactions:
   ```ini
   idle_in_transaction_session_timeout = 30000  -- 30 seconds
   ```
6. **Add monitoring alerts** for:
   - `n_dead_tup > 100,000`
   - `last_autovacuum > 24 hours ago`
   - Long-running idle-in-transaction sessions

### Confidence: 0.90

---

## Common Troubleshooting Patterns

### Pattern 1: Application Reports Slow Queries

```
1. Check pg_stat_statements for top queries
2. Run EXPLAIN ANALYZE on slow queries
3. Check for missing indexes
4. Check for stale statistics (run ANALYZE)
5. Check for lock contention
6. Check for high I/O wait
```

### Pattern 2: Server Not Accepting Connections

```
1. Check pg_isready
2. Check PostgreSQL log
3. Check disk space
4. Check if postmaster is running
5. Check max_connections
6. Check port availability
```

### Pattern 3: High CPU Usage

```
1. Check pg_stat_activity for long queries
2. Check EXPLAIN plans for expensive operations
3. Check autovacuum activity
4. Check for checkpoint I/O
5. Check for missing indexes
```

### Pattern 4: High I/O Wait

```
1. Check checkpoint frequency (pg_stat_bgwriter)
2. Check for autovacuum activity
3. Check for long-running queries doing full scans
4. Check for pg_dump/pg_basebackup operations
5. Check storage performance
```

### Pattern 5: Replication Issues

```
1. Check pg_stat_replication
2. Check replication slots
3. Check WAL archiving
4. Check network connectivity
5. Check standby disk space
6. Check standby CPU and I/O
```

## References

- [PostgreSQL Performance Monitoring](https://www.postgresql.org/docs/current/monitoring-stats.html)
- [PostgreSQL Troubleshooting](https://www.postgresql.org/docs/current/monitoring-stats.html)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)