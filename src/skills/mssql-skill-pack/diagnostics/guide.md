# MSSQL Diagnostic Procedures

## Overview

Systematic diagnostic procedures for common Microsoft SQL Server issues.

## Performance Issues

### Slow Queries

**Symptoms**:
- Queries taking longer than expected
- High CPU or I/O usage
- Application timeouts

**Diagnostic Steps**:
```sql
-- Find slow queries using DMVs
SELECT TOP 10
    qs.execution_count,
    qs.total_elapsed_time,
    qs.total_worker_time,
    qs.total_logical_reads,
    qs.total_logical_writes,
    SUBSTRING(st.text, (qs.statement_start_offset/2)+1,
        ((CASE qs.statement_end_offset
            WHEN -1 THEN DATALENGTH(st.text)
            ELSE qs.statement_end_offset
        END - qs.statement_start_offset)/2)+1) AS query_text
FROM sys.dm_exec_query_stats qs
CROSS APPLY sys.dm_exec_sql_text(qs.sql_handle) st
ORDER BY qs.total_elapsed_time DESC;

-- Get execution plans
DBCC FREEPROCCACHE; -- Use with caution in production
SET STATISTICS IO ON;
SET STATISTICS TIME ON;
```

**Evidence Required**:
- Query execution plans
- Performance metrics
- Wait statistics

**Remediation**:
1. Optimize query structure
2. Add appropriate indexes
3. Update statistics
4. Consider query hints
5. Review server configuration

### Resource Usage

**Symptoms**:
- High CPU utilization
- Memory pressure
- Disk I/O bottlenecks

**Diagnostic Steps**:
```sql
-- Check resource usage
SELECT * FROM sys.dm_os_ring_buffers
WHERE ring_buffer_type = 'RING_BUFFER_RESOURCE';

-- Check wait types
SELECT top 10 wait_type, waiting_tasks_count, wait_time_ms
FROM sys.dm_os_wait_stats
ORDER BY wait_time_ms DESC;

-- Check buffer pool usage
SELECT * FROM sys.dm_os_buffer_descriptors
ORDER BY database_id, file_id;
```

**Evidence Required**:
- Wait statistics
- Resource usage metrics
- Buffer pool analysis

**Remediation**:
1. Optimize resource-intensive queries
2. Add indexes to reduce I/O
3. Tune memory settings
4. Consider hardware upgrades
5. Review workload patterns

## Connection Issues

### Connection Failures

**Symptoms**:
- Applications unable to connect
- "too many connections" errors
- Connection timeouts

**Diagnostic Steps**:
```sql
-- Check connection count
SELECT COUNT(*) AS connection_count
FROM sys.dm_exec_connections;

-- Check connection details
SELECT session_id, login_name, host_name, program_name
FROM sys.dm_exec_sessions
WHERE is_user_process = 1;

-- Check configuration
SELECT * FROM sys.configurations
WHERE name LIKE '%connection%';
```

**Evidence Required**:
- Connection count
- Connection details
- Configuration settings

**Remediation**:
1. Review application connection management
2. Implement connection pooling
3. Check for connection leaks
4. Optimize long-running transactions
5. Review server configuration

## Replication Issues

### Replication Lag

**Symptoms**:
- Subscriber falling behind publisher
- Increased replication delay
- Data inconsistency

**Diagnostic Steps**:
```sql
-- Check replication status
EXEC sp_replstatus;

-- Check distribution database
EXEC sp_replmonitorsubscriptionpendingcmds;

-- Check replication agents
EXEC sp_helpreplicationagents;
```

**Evidence Required**:
- Replication status
- Lag measurements
- Agent status

**Remediation**:
1. Check network connectivity
2. Verify distribution database health
3. Restart replication agents
4. Review replication configuration
5. Consider adding subscribers

## References

- SQL Server Diagnostics: https://learn.microsoft.com/en-us/sql/
- SQL Server Troubleshooting: https://learn.microsoft.com/en-us/sql/
- SQL Server DMVs: https://learn.microsoft.com/en-us/sql/