# MSSQL Common Failure Patterns

## Overview

This reference documents common Microsoft SQL Server failure patterns, their symptoms, diagnosis methods, and recommended actions.

## Common Failures

### Memory Pressure

**Symptoms**:
- Slow query performance
- High page life expectancy (PLE) drops
- Buffer cache hit ratio decreasing

**Diagnosis**:
```sql
-- Check memory usage
SELECT * FROM sys.dm_os_memory_clerks ORDER BY pages_allocated_bytes DESC;

-- Check page life expectancy
SELECT * FROM sys.dm_os_performance_counters
WHERE counter_name = 'Page life expectancy';

-- Check memory grants
SELECT * FROM sys.dm_exec_query_memory_grants;
```

**Remediation**:
1. Review and optimize memory-intensive queries
2. Set max server memory appropriately
3. Remove unnecessary SQL Server features
4. Check for memory leaks in extensions
5. Consider upgrading hardware

### Deadlock Issues

**Symptoms**:
- Application timeout errors
- Deadlock errors in application logs
- Transaction rollbacks

**Diagnosis**:
```sql
-- Find deadlocks
SELECT * FROM sys.dm_os_waiting_tasks
WHERE wait_type = 'LCK_M_X';

-- Check deadlock graphs
SELECT CAST(event_data AS XML) AS deadlock_graph
FROM sys.fn_xe_file_target_read_file('system_health*.xel', null, null, null)
WHERE object_name = 'xml_deadlock_report';
```

**Remediation**:
1. Optimize transaction ordering
2. Reduce transaction scope
3. Add appropriate indexes to avoid table scans
4. Consider isolation level changes
5. Implement retry logic in applications

### Disk Space Exhaustion

**Symptoms**:
- Database write failures
- Transaction log growth
- Slow I/O performance

**Diagnosis**:
```sql
-- Check database sizes
SELECT name, size/128.0 AS size_mb
FROM sys.master_files;

-- Check transaction log usage
DBCC SQLPERF(LOGSPACE);

-- Check disk space
EXEC xp_fixeddrives;
```

**Remediation**:
1. Shrink unused databases
2. Clean up transaction log
3. Expand disk capacity
4. Implement log shipping
5. Review backup retention policies

### Blocking Chains

**Symptoms**:
- Slow query performance
- Application timeouts
- High resource utilization

**Diagnosis**:
```sql
-- Find blocking chains
SELECT blocking_session_id, session_id, wait_type, wait_time
FROM sys.dm_exec_requests
WHERE blocking_session_id > 0;

-- Check long-running queries
SELECT session_id, status, command, cpu_time, total_elapsed_time
FROM sys.dm_exec_requests
WHERE total_elapsed_time > 60000;
```

**Remediation**:
1. Identify and terminate blocking sessions
2. Optimize blocking queries
3. Review transaction isolation levels
4. Add appropriate indexes
5. Implement row versioning

### Replication Issues

**Symptoms**:
- Replication lag
- Subscriber disconnects
- Data sync failures

**Diagnosis**:
```sql
-- Check replication status
SELECT * FROM msdb.dbo.msysreplicationservers;

-- Check distribution database
EXEC sp_replstatus;

-- Check replication agents
EXEC sp_helpreplicationagents;
```

**Remediation**:
1. Check network connectivity
2. Verify distribution database health
3. Restart replication agents
4. Check for conflicting changes
5. Review replication configuration

## References

- SQL Server Troubleshooting: https://learn.microsoft.com/en-us/sql/
- SQL Server Common Issues: https://learn.microsoft.com/en-us/sql/
- SQL Server Support: https://learn.microsoft.com/en-us/sql/