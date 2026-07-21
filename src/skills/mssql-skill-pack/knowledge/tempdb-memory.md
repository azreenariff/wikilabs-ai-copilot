# TempDB and Memory Management

## Overview

TempDB and memory management are two of the most critical subsystems in SQL Server performance. TempDB is the shared workspace for sorting, hashing, versioning, and intermediate results — making it a potential bottleneck for concurrent workloads. Memory management determines how efficiently data, indexes, and query plans are cached in RAM. Understanding both subsystems is essential for enterprise SQL Server performance tuning.

## TempDB Architecture

### What Is TempDB?

TempDB is a system database that provides a shared workspace for:
- Internal work tables (sorting, hashing, spooling)
- Version store for snapshot isolation
- Table variables and temporary tables (#temp)
- Intermediate query results
- Online index operation results
- Cursor temporary storage
- Row versioning for triggers and MERGE operations

TempDB is recreated each time SQL Server starts, meaning any data in it is lost on restart. This design enables fast startup and eliminates the need for crash recovery.

### TempDB File Configuration

Proper TempDB configuration is critical for performance:

**File count recommendations:**
- Create one data file per logical processor core, up to 8 files
- For more than 8 cores, start with 8 files and increase if contention persists
- All files should be the same size to ensure even distribution

**File growth settings:**
- Use AUTO-GROWTH in megabytes (not percentage)
- Set reasonable growth increments (e.g., 64 MB or 512 MB)
- Avoid percentage growth — causes unpredictable file sizes
- Pre-allocate sufficient space to minimize growth events

**File placement:**
- Place TempDB files on high-performance SSD storage
- Distribute files across different physical disks
- Ensure files are on separate controllers if possible

```sql
-- Check current TempDB configuration
SELECT name,
       physical_name,
       size_mb = size * 8 / 1024,
       growth_mb = growth * 8 / 1024,
       is_percent_growth,
       state_desc
FROM sys.master_files
WHERE database_id = 2;  -- TempDB database_id

-- Example: Add TempDB files (run during maintenance)
-- This script creates one file per core, all same size
DECLARE @numFiles INT = 8;
DECLARE @maxFiles INT = 8;

SELECT @numFiles = CASE
    WHEN @@CPU_BOOST = 0 THEN
        CASE WHEN @@CPU_COUNT > 8 THEN 8 ELSE @@CPU_COUNT END
    ELSE
        CASE WHEN @@CPU_COUNT > 8 THEN 8 ELSE @@CPU_COUNT END
END;

DECLARE @i INT = 1;
DECLARE @sql NVARCHAR(MAX) = '';

WHILE @i <= @numFiles
BEGIN
    SET @sql += 'ALTER DATABASE tempdb ADD FILE (NAME = N''tempdev' + CAST(@i AS VARCHAR) +
                 ''', FILENAME = N''D:\TempDB\tempdb' + CAST(@i AS VARCHAR) + '.ndf'', SIZE = 1024MB,
                 FILEGROWTH = 256MB);' + CHAR(13);
    SET @i += 1;
END

PRINT @sql;
-- EXEC sp_executesql @sql;
```

**SQL Server 2016+ automatic file creation:**
- SQL Server automatically creates 8 TempDB data files
- Files are created with equal size and AUTO-GROWTH
- Automatic file creation is triggered when there is contention
- Can be controlled via startup trace flag 1117 (deprecated, now default)

### TempDB Allocation Mechanisms

TempDB uses two types of allocation pages:

- **SGAM (Shared Global Allocation Map)** — Tracks mixed extent allocation
- **PFS (Page Free Space)** — Tracks page utilization
- **CM (Change Map)** — Tracks page modifications for version store
- **GAM (Global Allocation Map)** — Tracks extent allocation

**Allocation paths:**
- **Uniform extents** — All 8 pages in an extent belong to the same object (preferred)
- **Mixed extents** — Up to 8 pages from different objects share an extent (initial allocation)

**Trace flags for TempDB optimization:**
- **Trace flag 1117** — Extent allocation (default in SQL Server 2016+)
- **Trace flag 1118** — Uniform extent allocation for temp tables (default in SQL Server 2016+)

```sql
-- Check if trace flags are active
DBCC TRACEOFF(1117, 1118, -1);  -- View without applying
DBCC TRACESTATUS(1117, 1118);

-- Note: TF 1117 and 1118 are defaults in SQL Server 2016+
-- No need to set manually unless disabled
```

## TempDB Contention

### PFS Page Contention

PFS page contention occurs when many sessions try to allocate pages simultaneously:

**Symptoms:**
- High PAGEIOLATCH_SH wait type
- Slow temporary table creation
- Slow query performance for batch operations

**Mitigation:**
- Multiple TempDB data files with equal size
- Trace flag 1118 (default in SQL Server 2016+)
- Even file growth increments

### Version Store Contention

The version store uses TempDB to maintain row versions for:
- Snapshot isolation transactions
- READ_COMMITTED_SNAPSHOT
- Online index operations
- Trigger and MERGE row versioning

**Symptoms:**
- High VERSION_STORE memory clerk usage
- tempdb_full errors
- Slow query performance with long-running transactions

**Mitigation:**
- Monitor version store usage
- Keep transactions short
- Use READ_COMMITTED_SNAPSHOT for better concurrency
- Monitor tempdb space usage

```sql
-- Check version store usage
SELECT SUM(version_store_reserved_page_count) AS version_store_pages,
       SUM(version_store_reserved_page_count) * 8 / 1024 AS version_store_mb
FROM sys.dm_tran_version_snapshot_space_usage;

-- Check session space usage
SELECT session_id,
       user_objects_alloc_page_count,
       user_objects_dealloc_page_count,
       internal_objects_alloc_page_count,
       internal_objects_dealloc_page_count
FROM sys.dm_db_session_space_usage
ORDER BY (user_objects_alloc_page_count + internal_objects_alloc_page_count) DESC;

-- Check task space usage
SELECT session_id,
       task_alloc_page_count,
       task_dealloc_page_count
FROM sys.dm_db_task_space_usage
ORDER BY (task_alloc_page_count + task_dealloc_page_count) DESC;

-- Check file space usage
SELECT file_id,
       file_snapshot_id,
       user_objects_alloc_page_count,
       user_objects_dealloc_page_count,
       internal_objects_alloc_page_count,
       internal_objects_dealloc_page_count,
       unallocated_extent_page_count,
       version_store_reserved_page_count
FROM sys.dm_db_file_space_usage;
```

### Page Latch Contention

Page latch contention occurs when multiple threads try to access the same page simultaneously:

**Types of page latch waits:**
- **PAGEIOLATCH_SH/SX/X** — I/O latch waits
- **PAGELATCH_SH/SX/X** — In-memory latch waits
- **PAGEUPD** — Page update contention

**Diagnostic queries:**

```sql
-- Check page latch waits
SELECT wait_type,
       waiting_tasks_count,
       wait_time_ms,
       max_wait_time_ms,
       signal_wait_time_ms
FROM sys.dm_os_wait_stats
WHERE wait_type LIKE 'PAGEIOLATCH%'
    OR wait_type LIKE 'PAGELATCH%'
ORDER BY wait_time_ms DESC;

-- Check latch class contention
SELECT CONVERT(VARCHAR(25), resource_description) AS resource,
       wait_type,
       wait_duration_ms,
       session_id
FROM sys.dm_os_waiting_tasks
WHERE wait_type LIKE 'PAGELATCH_%'
ORDER BY wait_duration_ms DESC;
```

**Mitigation strategies:**
- Multiple TempDB data files (one per core, max 8+)
- Equal-size files with even file growth
- Even distribution across physical disks
- Consider FILESTREAM for large objects

## TempDB Best Practices

### 1. File Count

| CPU Cores | Recommended Files |
|-----------|-------------------|
| 1-8 | 1 per core |
| 8-16 | 8 files |
| 16-32 | 8-16 files (add if contention) |
| 32+ | 16+ files (add if contention) |

### 2. File Size and Growth

- All files should be **equal size**
- Use **MB growth** (not percentage)
- Pre-allocate to avoid growth events
- Recommended growth: **64 MB to 512 MB** per increment

### 3. File Placement

- Place on **fast SSD storage**
- Distribute across **different physical disks**
- Avoid placing on same disk as primary database files
- Use separate controller for each file if possible

### 4. Monitoring

- Monitor **tempdb space usage** regularly
- Check **page latch contention** using DMVs
- Monitor **version store size**
- Track **tempdb growth rate** over time

## Memory Management

### SQL Server Memory Architecture

SQL Server memory is managed through the **buffer pool** and memory clerks:

**Memory types:**
- **Buffer pool** — Caches data pages (largest consumer)
- **Plan cache** — Stores compiled execution plans
- **Lock memory** — Stores lock information
- **Procedure cache** — Stores stored procedure metadata
- **Extended procedures** — Memory for extended stored procedures
- **CLR memory** — Memory for Common Language Runtime
- **OLE DB providers** — Memory for OLE DB data sources

### Max Server Memory

The `max server memory` setting limits the total memory SQL Server can consume. This is critical in shared environments.

**Configuration:**
```sql
-- View current max server memory
EXEC sp_configure 'max server memory (MB)';

-- Set max server memory (in MB)
EXEC sp_configure 'max server memory (MB)', 61440;  -- 60 GB
RECONFIGURE;

-- Leave 2-4 GB for OS on each instance
-- Formula: (Total RAM - OS overhead) / Number of SQL Server instances
```

**Guidelines:**
- **Dedicated server:** Leave 4 GB for OS minimum
- **Shared server:** Leave more for other applications
- **Virtual machines:** Consider hypervisor overhead
- **Always test** memory settings under production load

### Memory Clerks

Memory clerks are the building blocks of SQL Server memory allocation:

```sql
-- Check memory clerk usage
SELECT type,
       name,
       pages_kb / 1024 AS memory_mb,
       single_pages_kb / 1024 AS single_pages_mb,
       multi_pages_kb / 1024 AS multi_pages_mb,
       virtual_memory_committed_kb / 1024 AS vm_committed_mb,
       virtual_memory_reserved_kb / 1024 AS vm_reserved_mb
FROM sys.dm_os_memory_clerks
ORDER BY pages_kb DESC;
```

**Key memory clerks:**

| Clerk | Purpose |
|-------|---------|
| CACHESTORE_OBJCP | Compiled plans cache |
| CACHESTORE_SQLCP | SQL plans cache (ad-hoc, prepared) |
| CACHESTORE_PHDR | Procedure headers |
| BUFFER | Buffer pool pages |
| MEMORYCLERK_SQLWORKSPACE | Query workspace (sort/hash) |
| CACHESTORE_XMLDB | XML data cache |
| MEMORYCLERK_SOSNODE | SOS allocation node |
| CACHESTORE_XPROC | Extended stored procedures |

### Memory Pressure

Memory pressure occurs when SQL Server cannot get enough memory for its operations:

**Types of memory pressure:**
- **Internal memory pressure** — SQL Server needs memory but doesn't have enough
- **External memory pressure** — OS cannot give SQL Server the memory it requests

**Symptoms:**
- High PAGEIOLATCH waits
- Slow query performance
- Frequent checkpoint activity
- High lazy writer activity
- Buffer cache hit ratio below 90%

**Monitoring memory pressure:**

```sql
-- Buffer cache hit ratio
SELECT cntr_value / 1000.0 AS buffer_cache_hit_ratio_percent
FROM sys.dm_os_performance_counters
WHERE counter_name = 'Buffer cache hit ratio'
  AND instance_name = '_Total';

-- Lazy writer activity
SELECT cntr_value AS lazy_writes_per_second
FROM sys.dm_os_performance_counters
WHERE counter_name = 'Lazy writes/sec'
  AND instance_name = '_Total';

-- Checkpoint activity
SELECT cntr_value AS checkpoints_per_second
FROM sys.dm_os_performance_counters
WHERE counter_name = 'Checkpoints/sec'
  AND instance_name = '_Total';

-- Memory grant wait info
SELECT waiting_tasks_count,
       wait_time_ms,
       max_wait_time_ms
FROM sys.dm_os_wait_stats
WHERE wait_type = 'RESOURCE_SEMAPHORE';
```

### Memory Grants

Memory grants allocate workspace for sorting, hashing, and spooling operations. Insufficient or excessive memory grants cause performance problems.

**Memory grant process:**
1. Query optimizer estimates workspace memory requirement
2. Request is queued in memory grant pool
3. Grant is awarded when sufficient memory is available
4. Query executes with granted memory
5. Memory is released when query completes

**Problems:**
- **Memory grant wait** — Query waits for memory grant (RESOURCE_SEMAPHORE wait)
- **Memory grant spill** — Query spills to tempDB when grant is insufficient
- **Over-grant** — Memory grant much larger than actual need

```sql
-- Check memory grants
SELECT TOP 20
       q.text AS query_text,
       s.session_id,
       s.request_time,
       s.granted_memory_kb,
       s.used_memory_kb,
       s.max_used_memory_kb,
       s.required_memory_kb,
       s.grant_time
FROM sys.dm_exec_query_memory_grants s
CROSS APPLY sys.dm_exec_sql_text(s.sql_handle) q
ORDER BY s.wait_time_ms DESC;

-- Check memory grant spills
SELECT session_id,
       request_time,
       grant_time,
       requested_memory_kb,
       granted_memory_kb,
       used_memory_kb,
       query_cost,
       timeout_sec
FROM sys.dm_exec_query_memory_grants
ORDER BY granted_memory_kb DESC;
```

**Mitigation strategies:**
- **Smart memory grant** (SQL Server 2019+) — Reduced over-grant by 40%
- **Query Store** — Force better plans with lower memory requirements
- **Index optimization** — Reduce sort/hash requirements
- **Memory grants feedback** (SQL Server 2017+) — Runtime adjustment of memory grants

### Lock Memory

Lock memory stores lock information and can grow significantly with many concurrent transactions.

```sql
-- Check lock memory usage
SELECT type,
       pages_kb / 1024 AS lock_memory_mb
FROM sys.dm_os_memory_clerks
WHERE type = 'MEMORYCLERK_LOCKS';

-- Check lock count
SELECT request_session_id,
       resource_type,
       resource_database_id,
       resource_associated_entity_id,
       request_mode,
       request_status
FROM sys.dm_tran_locks
ORDER BY request_session_id;
```

## Memory Optimization Techniques

### 1. Buffer Pool Extension (SQL Server 2014-2016)

```sql
-- Enable BPE (deprecated in SQL Server 2017+)
ALTER SERVER CONFIGURATION
SET BUFFER POOL EXTENSION ON
(FILENAME = 'D:\SSDBPE\bpext.bpe', SIZE = 100 GB);
```

### 2. Large Page Support

Large pages (2 MB) reduce TLB (Translation Lookaside Buffer) misses:

**Requirements:**
- SQL Server running as service account with "Lock Pages in Memory" permission
- Windows server configured for large pages
- SQL Server Enterprise Edition (SQL Server 2017+)

```sql
-- Enable large pages (requires admin)
-- 1. Grant "Lock Pages in Memory" to SQL Server service account
-- 2. Enable large pages in SQL Server configuration manager
-- 3. Restart SQL Server service
```

### 3. Affinity Mask

Configure SQL Server to use specific processors:

```sql
-- Set processor affinity
EXEC sp_configure 'affinity mask', 15;  -- Use CPUs 0-3
RECONFIGURE;

-- Set I/O affinity
EXEC sp_configure 'affinity I/O mask', 240;  -- Use CPUs 4-7 for I/O
RECONFIGURE;
```

### 4. Resource Governor

Control memory usage per workload:

```sql
-- Configure Resource Governor for memory control
ALTER WORKLOAD GROUP wg_default
WITH (MAX_MEMORY_PERCENT = 70);

ALTER RESOURCE GOVERNOR RECONFIGURE;
```

## TempDB vs Memory Relationship

### Memory Pressure Effects on TempDB

When memory pressure occurs:
1. Buffer pool pages are flushed to disk
2. Lazy writer moves cold pages to disk
3. More I/O is needed for data access
4. Queries need more workspace in tempDB
5. TempDB becomes a bottleneck

**Chain reaction:**
Memory pressure → Buffer pool eviction → Increased tempDB usage → TempDB contention → Slower queries → More memory pressure

### Query Workload Impact on TempDB

Workload types that heavily use tempDB:
- **Sorting large result sets** — ORDER BY on large tables
- **Hash joins** — Large dimension joins
- **Hash aggregation** — GROUP BY on large datasets
- **Table variables** — Table variable declarations
- **Temporary tables** — #temp table operations
- **Snapshot isolation** — Row versioning in version store
- **Online index operations** — Sort results in tempDB

## Version-Specific Features

### SQL Server 2017
- **Intelligent Query Processing** — Batch mode execution for tempDB operations
- **Memory grant feedback** — Runtime memory grant adjustment
- **Batch mode on rowstore** — Improved tempDB efficiency for analytical queries
- **Buffer Pool Extension deprecated** — Replace with proper RAM sizing

### SQL Server 2019
- **Smart memory grant** — Reduced over-grant by up to 40%
- **Accelerated Database Recovery** — Reduced tempDB version store pressure
- **Automatic tuning** — Automatic index recommendations for tempDB optimization
- **Batch mode on rowstore** — Columnar processing for tempDB workloads

### SQL Server 2022
- **Vectorized batch mode** — Enhanced tempDB efficiency for analytic queries
- **Intelligent query processing** — Better memory grant estimation
- **Batch mode memory grant feedback** — Runtime adjustment of memory grants
- **Enhanced TempDB monitoring** — Better DMV coverage

## Troubleshooting TempDB and Memory

### Common Issues

1. **tempdb_full**
   - Check tempDB size and growth settings
   - Identify queries using excessive tempDB space
   - Check for version store bloat
   - Monitor for long-running transactions

2. **Page latch contention**
   - Increase TempDB data file count
   - Ensure equal-size files with even growth
   - Distribute files across physical disks
   - Consider hardware upgrades for I/O

3. **Memory grant waits**
   - Check for queries with excessive memory grants
   - Review query plans for sort/hash operators
   - Consider query rewrite to reduce sort requirements
   - Enable memory grant feedback (SQL Server 2017+)

4. **Version store bloat**
   - Monitor long-running transactions
   - Check for open cursors
   - Verify READ_COMMITTED_SNAPSHOT setting
   - Review trigger complexity

5. **Buffer cache hit ratio degradation**
   - Increase max server memory
   - Check for excessive tempDB spills
   - Review query plans for full table scans
   - Consider adding indexes

### Diagnostic Queries

```sql
-- Comprehensive tempDB and memory diagnostic
-- TempDB space usage by session
SELECT
    s.session_id,
    s.request_id,
    s.user_objects_alloc_page_count,
    s.internal_objects_alloc_page_count,
    (s.user_objects_alloc_page_count + s.internal_objects_alloc_page_count) * 8 / 1024 AS total_mb
FROM sys.dm_db_session_space_usage s
JOIN sys.dm_exec_sessions se ON s.session_id = se.session_id
WHERE s.session_id > 50  -- Internal sessions
ORDER BY (s.user_objects_alloc_page_count + s.internal_objects_alloc_page_count) DESC;

-- Memory pressure overview
SELECT
    (SELECT cntr_value FROM sys.dm_os_performance_counters WHERE counter_name = 'Buffer cache hit ratio') AS buffer_cache_hit_ratio,
    (SELECT cntr_value FROM sys.dm_os_performance_counters WHERE counter_name = 'Lazy writes/sec') AS lazy_writes,
    (SELECT cntr_value FROM sys.dm_os_performance_counters WHERE counter_name = 'Checkpoints/sec') AS checkpoints,
    (SELECT cntr_value FROM sys.dm_os_performance_counters WHERE counter_name = 'Page life expectancy') AS page_life_expectancy;
```

## Conclusion

TempDB and memory management are deeply intertwined in SQL Server performance. Proper TempDB configuration (multiple equal-size files, fast storage, even growth) prevents allocation contention. Memory management requires careful tuning of max server memory, monitoring of memory clerks, and attention to memory grant behavior. SQL Server 2017+ introduced significant improvements including smart memory grant, memory grant feedback, and batch mode on rowstore that address many common performance issues. Regular monitoring of tempDB space usage, version store size, page latch waits, and memory grant statistics is essential for maintaining optimal performance in enterprise SQL Server environments.