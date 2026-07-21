# SQL Server Context Interpretation

## Overview

This document describes how to interpret SQL Server outputs, logs, metrics, and diagnostic information. Understanding context interpretation is essential for accurate diagnosis and effective guidance.

## Interpreting DMV/DMF Output

### sys.dm_os_wait_stats

Wait statistics are the primary diagnostic tool for SQL Server performance issues.

**Interpretation guide:**

| Wait Type | Interpretation |
|-----------|----------------|
| CXPACKET | Parallelism contention — consider MAXDOP |
| SOS_SCHEDULER_YIELD | CPU pressure — check for CPU-intensive queries |
| PAGEIOLATCH_SH/SX/X | Disk I/O wait — check storage performance |
| LCK_M_* | Lock contention — check for blocking |
| RESOURCE_SEMAPHORE | Memory grant pressure — check memory grants |
| CMEMTHREAD | Memory allocation contention |
| WRITELOG | Log write wait — check log file performance |
| LOGGOBATCHWRITER | Log buffer wait — check log I/O |

**Key metrics:**
- `wait_time_ms` — Total wait time
- `waiting_tasks_count` — Number of waits
- `max_wait_time_ms` — Longest single wait
- `signal_wait_time_ms` — Time waiting for CPU after resource available

**Interpretation rules:**
1. **Normalize by sampling** — Divide wait_time by time since last collection
2. **Filter system waits** — Ignore CXPACKET for low-MAXDOP systems
3. **Look for dominant waits** — Focus on the top 3-5 wait types
4. **Correlate with workload** — Context determines whether a wait is normal

```sql
-- Sample wait statistics interpretation
-- Calculate wait percentages
SELECT
    wait_type,
    wait_time_ms,
    wait_time_ms * 100.0 / SUM(wait_time_ms) OVER () AS wait_percent,
    waiting_tasks_count,
    max_wait_time_ms
FROM sys.dm_os_wait_stats
WHERE wait_type NOT IN (
    'SLEEP_TASK', 'SLEEP_SYSTEMTHREAD', 'BROKER_TASK_START',
    'BROKER_RECEIVE_WAITFOR', 'REQUEST_FOR_DEADLOCK_SEARCH',
    'LAZYWRITER_SLEEP', 'SQLTRACE_BUFFER_FLUSH',
    'FT_IFTS_SCHEDULER_IDLE_WAIT', 'XE_DISPATCHER_WAIT',
    'XE_DISPATCHER_JOIN', 'BROKER_EVENT_HANDOFF',
    'CHECKPOINT_QUEUE', 'FT_IFTS_MAILBOX',
    'HADR_CLUSAPI_CALL', 'LOGMGR_QUEUE',
    'ONDEMAND_TASK_QUEUE'
)
ORDER BY wait_time_ms DESC;
```

### sys.dm_exec_query_stats

Query performance statistics from the plan cache.

**Interpretation guide:**

| Metric | Interpretation |
|--------|----------------|
| total_logical_reads | Total I/O cost — highest cost operator |
| total_worker_time | Total CPU time |
| total_elapsed_time | Total wall-clock time |
| execution_count | Number of executions |
| avg_elapsed_time | Average execution time |

**Key insights:**
- High logical reads with low execution count — scan-heavy queries
- High CPU with high execution count — compute-intensive queries
- Low reads but high elapsed time — I/O-bound queries
- High elapsed but low CPU — waiting for external resources

```sql
-- Top resource consumers
SELECT TOP 20
    qs.execution_count,
    qs.total_worker_time / qs.execution_count AS avg_cpu_ms,
    qs.total_elapsed_time / qs.execution_count AS avg_elapsed_ms,
    qs.total_logical_reads / qs.execution_count AS avg_logical_reads,
    SUBSTRING(st.text, (qs.statement_start_offset/2)+1,
        ((CASE qs.statement_end_offset
            WHEN -1 THEN DATALENGTH(st.text)
            ELSE qs.statement_end_offset
        END - qs.statement_start_offset)/2)+1) AS query_text
FROM sys.dm_exec_query_stats qs
CROSS APPLY sys.dm_exec_sql_text(qs.sql_handle) st
ORDER BY qs.total_logical_reads DESC;
```

### sys.dm_db_index_usage_stats

Index usage patterns for optimization.

**Interpretation guide:**

| Metric | Meaning |
|--------|---------|
| user_seeks | Times index was sought (good — selective) |
| user_scans | Times index was scanned (may need improvement) |
| user_lookups | Times index required key lookup (may need INCLUDE) |
| user_updates | Times index was updated (cost of maintaining index) |

**Key rules:**
- **Seeks > Scans** — Index is being used efficiently
- **Lookups > 0** — Consider adding columns to INCLUDE
- **Updates >> Seeks** — Index may not be worth maintaining
- **All zeros** — Unused index — consider dropping

### sys.dm_tran_locks

Current lock state for diagnosing blocking.

**Interpretation guide:**

| request_status | Meaning |
|----------------|---------|
| GRANT | Lock is held |
| WAIT | Lock is requested but not granted |
| CONVERT | Lock is being converted to different mode |

| request_mode | Meaning |
|--------------|---------|
| S | Shared — read lock |
| U | Update — update in progress |
| X | Exclusive — write lock |
| IS | Intent Shared — hierarchy lock |
| IX | Intent Exclusive — hierarchy lock |

## Interpreting Error Messages

### Severity Levels

| Severity | Description | Action |
|----------|-------------|--------|
| 0-10 | Informational | No action required |
| 11-16 | User-error | Fix query or permissions |
| 17-19 | Resource errors | Check resources (memory, disk) |
| 20-25 | Fatal/severe | Contact DBA, check hardware |

### Common Error Patterns

**Error 1205 (Deadlock):**
- **Meaning** — Transaction chosen as deadlock victim
- **Action** — Review query patterns, add indexes, reduce transaction scope
- **Detection** — Extended Events, ERRORLOG, sys.dm_os_ring_buffers

**Error 845 (Database mirroring timeout):**
- **Meaning** — Database cannot keep up with transaction log
- **Action** — Check disk I/O, reduce workload, check network (for mirroring)

**Error 9002 (Transaction log full):**
- **Meaning** — Transaction log has run out of space
- **Action** — Shrink log, add log file, backup log, fix root cause

**Error 17804 (Always On health check failed):**
- **Meaning** — Replica health check failed
- **Action** — Check network, check replica status, verify endpoint

**Error 2601/2627 (Unique key violation):**
- **Meaning** — Duplicate key inserted
- **Action** — Check application logic, review constraints

**Error 208 (Invalid object name):**
- **Meaning** — Table/view does not exist
- **Action** — Check schema qualification, verify object exists

## Interpreting Performance Counter Output

### Buffer Cache Hit Ratio

```sql
-- Buffer cache hit ratio > 90% is good
SELECT
    (SELECT cntr_value FROM sys.dm_os_performance_counters
     WHERE counter_name = 'Buffer cache hit ratio') AS cache_hit_ratio,
    (SELECT cntr_value FROM sys.dm_os_performance_counters
     WHERE counter_name = 'Buffer cache hit ratio base') AS cache_hit_ratio_base;
```

**Interpretation:**
- **> 95%** — Excellent — most pages in memory
- **90-95%** — Good — acceptable performance
- **80-90%** — Monitor — increasing I/O
- **< 80%** — Problem — insufficient memory or excessive I/O

### Page Life Expectancy

```sql
SELECT cntr_value AS page_life_expectancy_seconds
FROM sys.dm_os_performance_counters
WHERE counter_name = 'Page life expectancy'
  AND instance_name = '_Total';
```

**Interpretation:**
- **> 300 seconds** — Good — pages stay in buffer pool
- **100-300 seconds** — Monitor — working set growing
- **< 100 seconds** — Problem — memory pressure or excessive I/O

**Trend matters more than absolute value** — a declining PLE indicates growing memory pressure.

### Lazy Writer Activity

```sql
SELECT cntr_value AS lazy_writes_per_second
FROM sys.dm_os_performance_counters
WHERE counter_name = 'Lazy writes/sec';
```

**Interpretation:**
- **0-5/sec** — Normal — occasional page flushing
- **5-15/sec** — Monitor — moderate memory pressure
- **> 15/sec** — Problem — significant memory pressure

## Interpreting Always On Metrics

### Synchronization Health

```sql
-- Check synchronization state
SELECT ar.replica_server_name,
       drd.database_name,
       drs.synchronization_state_desc,
       drs.synchronization_health_desc,
       drs.last_hardened_lsn,
       drs.last_sent_lsn,
       drs.last_sent_time,
       drs.last_redone_time
FROM sys.dm_hadr_database_replica_states drs
JOIN sys.dm_hadr_availability_replica_states ar
    ON drs.replica_id = ar.replica_id;
```

**Synchronization states:**
- **SYNCHRONIZED** — Replica is caught up
- **SYNCHRONIZING** — Replica is catching up
- **NOT SYNCHRONIZED** — Replica is behind

**Health states:**
- **HEALTHY** — All databases synchronized
- **NOT HEALTHY** — One or more databases not synchronized

### Log Send Queue

```sql
-- Check log send queue size
SELECT ar.replica_server_name,
       drd.database_name,
       drs.log_send_queue_size / (1024.0 * 1024) AS log_send_queue_mb,
       drs.redo_queue_size / (1024.0 * 1024) AS redo_queue_mb,
       drs.log_send_rate / (1024.0 * 1024) AS log_send_rate_mb_sec,
       drs.redo_rate / (1024.0 * 1024) AS redo_rate_mb_sec
FROM sys.dm_hadr_database_replica_states drs
JOIN sys.dm_hadr_availability_replica_states ar
    ON drs.replica_id = ar.replica_id;
```

**Interpretation:**
- **Log send queue growing** — Primary is generating log faster than sending
- **Redo queue growing** — Secondary is applying log slower than receiving
- **Both small** — Healthy synchronization

## Interpreting Query Store Metrics

### Query Performance Regression

```sql
-- Detect regression
SELECT qsq.query_id,
       qsq.object_id,
       qsp.plan_id,
       qsrs.avg_duration,
       qsrs.avg_cpu_time,
       qsrs.avg_logical_io_reads,
       qsrs.execution_count,
       qsrs.last_execution_time
FROM sys.query_store_runtime_stats qsrs
JOIN sys.query_store_plan qsp ON qsrs.plan_id = qsp.plan_id
JOIN sys.query_store_query qsq ON qsp.query_id = qsq.query_id
WHERE qsrs.last_execution_time > DATEADD(HOUR, -24, GETDATE())
ORDER BY qsrs.avg_duration DESC;
```

**Interpretation:**
- **Duration increasing** — Query is slowing down
- **CPU increasing** — More CPU-intensive
- **IO increasing** — More I/O intensive
- **Execution count high with poor performance** — High-impact query

## Interpreting Execution Plans

### Visual Plan Indicators

| Visual Cue | Meaning |
|------------|---------|
| Red arrow (cost > 30%) | Expensive operator — investigate |
| Yellow arrow (cost 10-30%) | Moderate cost — consider optimizing |
| Green arrow (cost < 10%) | Low cost — not a concern |
| Warning triangle | Potential issue (e.g., implicit conversion) |
| Missing index suggestion | Index recommendation |

### Operator-Level Analysis

| Operator | When to investigate |
|----------|---------------------|
| Table Scan | Large table with no index or wide range predicate |
| Key Lookup | Index not covering — consider INCLUDE |
| Sort | Large sort — consider index or memory grant |
| Hash Match | Large hash — consider index or memory grant |
| Nested Loops | Small row count — typically efficient |
| Merge Join | Sorted inputs — typically efficient |
| Compute Scalar | Expression evaluation — check for implicit conversions |
| Stream Aggregate | Aggregation — check for sort dependency |

### Estimated vs Actual Rows

| Condition | Meaning |
|-----------|---------|
| Estimated ≈ Actual | Statistics accurate |
| Estimated >> Actual | Overestimation — may cause inefficient plan |
| Estimated << Actual | Underestimation — may cause memory grant issues |

**When estimated differs significantly from actual:**
- Update statistics with FULLSCAN
- Check for parameter sniffing
- Consider query rewrites
- Use OPTION (RECOMPILE) for dynamic queries

## Interpreting Lock and Blocking Output

### Identifying Blocking Chains

```sql
-- Find blocking chains
SELECT
    blocking.session_id AS blocking_session,
    blocked.session_id AS blocked_session,
    blocking_wait.wait_type,
    blocking_wait.wait_duration_ms,
    blocked_query.text AS blocked_query,
    blocking_query.text AS blocking_query
FROM sys.dm_os_waiting_tasks blocking_wait
JOIN sys.dm_exec_sessions blocked ON blocking_wait.session_id = blocked.session_id
JOIN sys.dm_exec_requests blocking ON blocking_wait.blocking_session_id = blocking.session_id
CROSS APPLY sys.dm_exec_sql_text(blocking_wait.session_id) blocked_query
CROSS APPLY sys.dm_exec_sql_text(blocking_wait.blocking_session_id) blocking_query
WHERE blocking_wait.blocking_session_id IS NOT NULL
ORDER BY blocking_wait.wait_duration_ms DESC;
```

**Interpretation:**
- **Short blocking (< 5 seconds)** — Normal for high-concurrency systems
- **Medium blocking (5-60 seconds)** — Investigate
- **Long blocking (> 60 seconds)** — Problem — likely root cause

### Lock Escalation Detection

```sql
-- Check for lock escalation
SELECT
    session_id,
    resource_type,
    request_mode,
    request_status,
    request_owner_type
FROM sys.dm_tran_locks
WHERE resource_type = 'OBJECT'
ORDER BY session_id;
```

**Interpretation:**
- **OBJECT locks** — Lock escalation has occurred
- **PAGE locks** — May escalate to OBJECT
- **KEY/ROW locks** — Fine-grained locking (no escalation)

## Interpreting TempDB Metrics

### TempDB Space Usage

```sql
-- Check TempDB allocation
SELECT
    SUM(user_objects_alloc_page_count) * 8 / 1024 AS user_objects_mb,
    SUM(internal_objects_alloc_page_count) * 8 / 1024 AS internal_objects_mb,
    SUM(version_store_reserved_page_count) * 8 / 1024 AS version_store_mb,
    SUM(unallocated_extent_page_count) * 8 / 1024 AS free_mb
FROM sys.dm_db_file_space_usage;
```

**Interpretation:**
- **High user_objects** — Temp tables, table variables
- **High internal_objects** — Sort, hash, spool operations
- **High version_store** — Long-running transactions with snapshot isolation
- **Low free** — TempDB may be running out of space

## Conclusion

Interpreting SQL Server output requires understanding both the individual metrics and the broader context of the workload. Always correlate findings with business impact, consider trends over time rather than absolute values, and validate findings with multiple diagnostic sources. The key is to build a mental model of how each component interacts and how symptoms manifest across different DMVs and metrics.