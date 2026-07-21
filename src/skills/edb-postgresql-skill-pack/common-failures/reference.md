# EDB PostgreSQL Common Failure Patterns

## Overview

This reference documents common EDB PostgreSQL failure patterns, their symptoms, diagnosis methods, and recommended actions.

## Common Failures

### Connection Pool Exhaustion

**Symptoms**:
- New connections rejected with "too many connections"
- Applications experience timeouts
- pg_stat_activity shows high connection count

**Diagnosis**:
```sql
-- Check active connections
SELECT count(*) FROM pg_stat_activity;

-- Check connection settings
SHOW max_connections;
SHOW superuser_reserved_connections;

-- Check connection states
SELECT state, count(*) FROM pg_stat_activity GROUP BY state;
```

**Remediation**:
1. Increase max_connections (with memory impact consideration)
2. Implement connection pooling (PgBouncer, pgpool-II)
3. Review application connection management
4. Kill idle long-running connections
5. Tune superuser_reserved_connections

### Disk Space Exhaustion

**Symptoms**:
- Database writes failing
- Replication lag increasing
- WAL archive filling up

**Diagnosis**:
```sql
-- Check database sizes
SELECT datname, pg_size_pretty(pg_database_size(datname)) FROM pg_database ORDER BY pg_database_size(datname) DESC;

-- Check WAL directory size
SELECT pg_size_pretty(pg_wal) AS wal_size;

-- Check pg_walarchiver status
SELECT * FROM pg_stat_archiver;

-- Check disk usage
df -h
```

**Remediation**:
1. Clean up old WAL segments
2. Archive WAL files to offsite storage
3. Vacuum databases to reclaim space
4. Increase disk capacity
5. Implement automated cleanup policies

### Replication Failure

**Symptoms**:
- Replication lag increasing
- Standby showing disconnected
- Failover failing

**Diagnosis**:
```sql
-- Check replication status
SELECT * FROM pg_stat_replication;

-- Check replication lag
SELECT pg_wal_lsn_diff(pg_current_wal_lsn(), client_restart_lsn) FROM pg_stat_replication;

-- Check standby status
SELECT * FROM pg_stat_activity WHERE datname = 'replication';
```

**Remediation**:
1. Check network connectivity between primary and standby
2. Verify replication user permissions
3. Check disk space on standby
4. Review pg_hba.conf for replication entries
5. Restart replication if needed

### Lock Contention

**Symptoms**:
- Queries hanging or slow
- Long running transactions
- Application timeout errors

**Diagnosis**:
```sql
-- Check for lock conflicts
SELECT blocked_locks.pid AS blocked_pid,
       blocking_locks.pid AS blocking_pid,
       blocked_activity.usename AS blocked_user,
       blocking_activity.usename AS blocking_user,
       blocked_activity.query AS blocked_query,
       blocking_activity.query AS blocking_query
FROM pg_catalog.pg_locks blocked_locks
JOIN pg_catalog.pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
JOIN pg_catalog.pg_locks blocking_locks ON blocking_locks.locktype = blocked_locks.locktype
  AND blocking_locks.database IS NOT DISTINCT FROM blocked_locks.database
  AND blocking_locks.relation IS NOT DISTINCT FROM blocked_locks.relation
  AND blocking_locks.pid != blocked_locks.pid
JOIN pg_catalog.pg_stat_activity blocking_activity ON blocking_activity.pid = blocking_locks.pid
WHERE NOT blocked_locks.granted;

-- Check long running transactions
SELECT pid, now() - pg_stat_activity.query_start AS duration, query
FROM pg_stat_activity WHERE now() - query_start > interval '5 minutes';
```

**Remediation**:
1. Identify and terminate blocking queries if appropriate
2. Review application transaction management
3. Optimize slow queries
4. Consider row-level locking strategies
5. Tune lock_timeout settings

### Performance Degradation

**Symptoms**:
- Slow query execution
- High CPU usage
- Increased I/O wait

**Diagnosis**:
```sql
-- Check pg_stat_statements for slow queries
SELECT query, calls, total_exec_time, mean_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;

-- Check table statistics
SELECT relname, seq_scan, idx_scan, n_tup_ins, n_tup_upd, n_tup_del
FROM pg_stat_user_tables ORDER BY seq_scan DESC;

-- Check for missing indexes
SELECT * FROM pg_stat_user_indexes WHERE idx_scan = 0;
```

**Remediation**:
1. Optimize slow queries with EXPLAIN ANALYZE
2. Add missing indexes
3. Update table statistics (ANALYZE)
4. Tune shared_buffers and work_mem
5. Consider partitioning large tables

## References

- EDB PostgreSQL Troubleshooting: https://www.enterprisedb.com/docs/
- PostgreSQL Troubleshooting: https://www.postgresql.org/docs/current/
- EDB PostgreSQL Support: https://www.enterprisedb.com/support/