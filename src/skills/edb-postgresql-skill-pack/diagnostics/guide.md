# EDB PostgreSQL Diagnostic Procedures

## Overview

Systematic diagnostic procedures for common EDB PostgreSQL issues.

## Performance Issues

### Slow Queries

**Symptoms**:
- Queries taking longer than expected
- High CPU or I/O usage
- Application timeouts

**Diagnostic Steps**:
```sql
-- Enable pg_stat_statements
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- Find slow queries
SELECT query, calls, total_exec_time, mean_exec_time, rows
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;

-- Analyze specific query
EXPLAIN ANALYZE SELECT * FROM large_table WHERE id = 12345;

-- Check index usage
SELECT schemaname, tablename, indexname, idx_scan, idx_tup_read
FROM pg_stat_user_indexes
ORDER BY idx_scan ASC;
```

**Evidence Required**:
- Query execution plan
- pg_stat_statements data
- Index usage statistics

**Remediation**:
1. Optimize query structure
2. Add or modify indexes
3. Update table statistics
4. Tune memory settings
5. Consider query rewriting

### Resource Usage

**Symptoms**:
- High CPU utilization
- Memory pressure
- Disk I/O bottlenecks

**Diagnostic Steps**:
```sql
-- Check resource usage
SELECT * FROM pg_stat_activity WHERE state = 'active';

-- Check table sizes
SELECT relname, pg_size_pretty(pg_total_relation_size(oid))
FROM pg_class WHERE relkind = 'r' ORDER BY pg_total_relation_size(oid) DESC;

-- Check checkpoint settings
SHOW checkpoint_completion_target;
SHOW checkpoint_timeout;
```

**Evidence Required**:
- System resource metrics
- pg_stat_activity output
- Checkpoint configuration

**Remediation**:
1. Tune checkpoint settings
2. Optimize queries
3. Add resources if needed
4. Review configuration parameters
5. Implement partitioning

## Connection Issues

### Connection Failures

**Symptoms**:
- Applications unable to connect
- "too many connections" errors
- Connection timeouts

**Diagnostic Steps**:
```sql
-- Check connection count
SELECT count(*) FROM pg_stat_activity;

-- Check max_connections
SHOW max_connections;

-- Check connection settings
SHOW ssl;
SHOW listen_addresses;
```

**Evidence Required**:
- pg_stat_activity data
- max_connections setting
- pg_hba.conf configuration

**Remediation**:
1. Increase max_connections if needed
2. Implement connection pooling
3. Review application connection management
4. Fix pg_hba.conf issues
5. Kill unnecessary connections

## Replication Issues

### Replication Lag

**Symptoms**:
- Standby falling behind primary
- Increased replication delay
- Failover risk

**Diagnostic Steps**:
```sql
-- Check replication status
SELECT * FROM pg_stat_replication;

-- Check lag
SELECT pg_wal_lsn_diff(pg_current_wal_lsn(), client_flush_lsn) AS lag_bytes;

-- Check standby health
SELECT pg_is_in_recovery();
```

**Evidence Required**:
- Replication status
- Lag measurements
- Standby health status

**Remediation**:
1. Check network connectivity
2. Verify disk space on standby
3. Check standby configuration
4. Review WAL settings
5. Restart replication if needed

## References

- EDB PostgreSQL Diagnostics: https://www.enterprisedb.com/docs/
- PostgreSQL Diagnostics: https://www.postgresql.org/docs/current/
- EDB PostgreSQL Support: https://www.enterprisedb.com/support/