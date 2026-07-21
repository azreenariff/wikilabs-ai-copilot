# SQL Server Core Architecture

## Overview

Microsoft SQL Server is a relational database management system (RDBMS) that provides a comprehensive platform for data storage, processing, and analytics. The SQL Server engine is built on a modular architecture consisting of several key subsystems that work together to provide ACID-compliant transaction processing, high-performance query execution, and reliable data storage.

The architecture can be divided into four major subsystems:

1. **Buffer Pool** — Memory management and data page caching
2. **Query Processor** — SQL parsing, optimization, and execution
3. **Storage Engine** — Data and log file management
4. **Lock Manager** — Concurrency control and transaction isolation

These subsystems interact closely to provide the performance and reliability that enterprise workloads require. Understanding these internals is essential for effective performance tuning and troubleshooting.

## Buffer Pool Architecture

### Structure

The buffer pool is the heart of SQL Server's memory architecture. It manages the caching of data pages from disk in memory to minimize disk I/O. The buffer pool consists of:

- **Data Pages** — 8 KB units of data read from data files (.mdf, .ndf)
- **Index Pages** — 8 KB units of index data structures
- **Text/Image Pages** — Pages for LOB (Large Object) data types
- **IAM Pages** — Index Allocation Map pages tracking extent allocation
- **PFS Pages** — Page Free Space pages tracking page utilization
- **GCM Pages** — Global Change Map pages for version store tracking
- **SGAM Pages** — Shared Global Allocation Map pages for mixed extent tracking

### Buffer Pool LRU List

The buffer pool maintains a Least Recently Used (LRU) list of all cached pages. When a page is accessed, it moves to the hot end of the LRU chain. Pages that are not accessed for a sufficient time are evicted to make room for new pages. The LRU list is divided into two chains:

- **Hot Chain** — Recently accessed pages, protected from immediate eviction
- **Cold Chain** — Older pages, subject to eviction by the lazy writer

### Buffer Pool Extension (BPE)

Starting with SQL Server 2014, SQL Server introduced Buffer Pool Extension, which allows SSDs to extend the buffer pool beyond RAM capacity. BPE stores cold pages on SSD storage, reducing memory pressure when available RAM is insufficient for the working set.

**Configuration:**

```sql
-- Enable and configure BPE (SQL Server 2014-2016)
ALTER SERVER CONFIGURATION
SET BUFFER POOL EXTENSION ON
(FILENAME = 'D:\SSDBPE\bpext.bpe', SIZE = 100 GB);

-- Disable BPE
ALTER SERVER CONFIGURATION
SET BUFFER POOL EXTENSION OFF;
```

**SQL Server 2017+ Note:** Buffer Pool Extension is deprecated in SQL Server 2017 and later. Instead, use the Database Tuning Advisor or Query Store for memory optimization recommendations, and consider increasing RAM or optimizing query patterns.

### Memory Clerks

SQL Server allocates memory through memory clerks, each responsible for a specific type of memory allocation. Key memory clerks include:

- **BUFFER clerk** — Buffer pool pages
- **CACHESTORE_OBJCP** — Object plan cache (compiled plans)
- **CACHESTORE_SQLCP** — Ad-hoc and prepared SQL plan cache
- **CACHESTORE_PHDR** — Stored procedure and trigger header cache
- **CACHESTORE_STMTCP** — Statement-level plan cache
- **MEMORYCLERK_SQLWORKSPACE** — Query workspace for sorting and hashing
- **MEMORYCLERK_SOSNODE** — SOS node memory allocation
- **CACHESTORE_XMLDB** — XML data cache
- **CACHESTORE_XPROC** — Extended stored procedures cache

Monitoring memory clerks helps identify memory pressure and potential leaks:

```sql
SELECT type,
       SUM(pages_kb) / 1024 AS memory_mb,
       COUNT(*) AS page_count
FROM sys.dm_os_memory_clerks
GROUP BY type
ORDER BY memory_mb DESC;
```

## Query Processor

The query processor transforms T-SQL statements into efficient execution plans. It consists of four main stages:

### 1. Parser

The parser validates the syntactic correctness of the T-SQL statement. It checks for:
- Correct keyword usage
- Proper identifier quoting
- Valid clause ordering
- Syntax correctness

If the parser finds errors, it returns a syntax error message. Otherwise, it produces a parse tree representing the logical structure of the query.

### 2. Algebrizer

The algebrizer resolves all identifiers in the parse tree and produces a query tree representing the logical operations. During algebrization:
- Table and column names are resolved against system catalogs
- Data types are determined for all expressions
- Default values and schema names are resolved
- Views are expanded inline
- User-defined functions are resolved
- Permissions are checked

Common algebrization failures include:
- Missing table references
- Invalid column names
- Insufficient permissions
- Ambiguous column references
- Invalid function calls

### 3. Query Optimizer

The query optimizer is the most critical component for performance. It examines the query tree and generates multiple potential execution plans, selecting the one with the lowest estimated cost. The optimizer uses:

- **Statistical data** — Distribution statistics on columns used in predicates
- **Index information** — Available indexes, including covering indexes
- **Row estimates** — Cardinality estimates based on statistics
- **Cost model** — Estimated CPU and I/O cost of operators
- **Heuristics** — Rules that prune unlikely plan alternatives
- **Join algorithms** — Nested loop, merge join, hash join selection
- **Access methods** — Index seek, index scan, table scan selection

**SQL Server 2019 Improvements:**
- Adaptive joins (dynamic selection of nested loop vs. hash join)
- Batch mode on rowstore (columnar-like processing for rowstore tables)
- Smart memory grant (reduces memory grant overestimation)
- Query Store integration for runtime statistics

**SQL Server 2022 Improvements:**
- Vectorized batch mode for analytic queries
- Intelligent query processing (IQP) features
- Batch mode adaptive join improvements
- Table variable deferred compilation

### 4. Executor

The executor takes the selected execution plan and carries out the operations specified. Each operator in the plan executes as a row-by-row (or batch-by-batch) transformation. The executor:
- Reads data from disk or memory buffers
- Applies filters, joins, aggregations, and sorts
- Writes results to output destinations
- Reports performance metrics through DMVs

Common operators include:
- **Scan** — Table scan, clustered index scan, nonclustered index scan
- **Seek** — Clustered index seek, nonclustered index seek
- **Join** — Nested loop join, merge join, hash join
- **Aggregate** — SUM, COUNT, AVG, MIN, MAX
- **Sort** — Physical sort operator
- **Spool** — Eager/spool and lazy/spool for materialization
- **Compute Scalar** — Expression evaluation
- **Stream Aggregate** — Aggregation without sorting

### Query Store

Query Store (introduced in SQL Server 2016) captures query execution plans, runtime statistics, and regressions. It provides a historical record of query performance that the optimizer can use for plan forcing and performance trending.

**Key features:**
- Query plan capture (capture_mode = ALL, NONE, AUTO)
- Runtime statistics (execution count, elapsed time, CPU time)
- Plan forcing (manually pin preferred plans)
- Automatic plan regression detection
- Query performance trending

```sql
-- Enable Query Store
ALTER DATABASE [MyDatabase] SET QUERY_STORE = ON;
ALTER DATABASE [MyDatabase] SET QUERY_STORE (OPERATION_MODE = READ_WRITE);

-- View query performance
SELECT qsq.query_id,
       qsq.object_id,
       qsp.plan_id,
       qsrs.avg_duration,
       qsrs.avg_cpu_time,
       qsrs.avg_logical_io_reads
FROM sys.query_store_query qsq
JOIN sys.query_store_plan qsp ON qsq.query_id = qsp.query_id
JOIN sys.query_store_runtime_stats qsrs ON qsp.plan_id = qsrs.plan_id
ORDER BY qsrs.avg_duration DESC;
```

## Storage Engine

The storage engine handles physical data storage and retrieval. It consists of:

### File Manager

The file manager manages database files on disk:
- **Data files** (.mdf, .ndf) — Store tables, indexes, and data
- **Log files** (.ldf) — Store transaction log records
- **Full-text catalog files** (.ftc) — Store full-text index data

File organization:
- Each database has one primary file (.mdf) and can have multiple secondary files (.ndf)
- Files are organized into filegroups
- The primary filegroup contains system tables and is mandatory
- User-defined filegroups can be created for storage management
- Each file consists of 8 KB pages organized into 8 KB extents

### Page and Extent Architecture

- **Page** — 8 KB fundamental unit of data allocation
- **Extent** — 64 KB (8 consecutive pages) allocation unit
- **Mixed extent** — Shared by up to 8 objects (first 8 pages of new objects)
- **Uniform extent** — Dedicated to a single object (after initial growth)

Page header contains:
- Slot array (pointers to row data)
- Null bitmap
- Fixed-length column count
- Variable-length column count
- Row status flags

### Log Manager

The log manager handles transaction log operations:
- **Transaction log records** — LSN-based sequenced records
- **Log blocks** — Temporary log storage in memory
- **Log flushing** — Writing log records to disk (sync I/O)
- **Log truncation** — Reclaiming inactive virtual log files

The transaction log is essential for:
- ACID compliance (Atomicity, Consistency, Isolation, Durability)
- Crash recovery
- Transaction roll-forward and roll-back
- Backup and restore operations
- Replication and log shipping

### Transaction Manager

The transaction manager ensures ACID properties:
- **Atomicity** — All operations in a transaction succeed or all fail
- **Consistency** — Transaction moves database from one valid state to another
- **Isolation** — Concurrent transactions don't interfere with each other
- **Durability** — Committed changes survive system failure

The transaction manager uses the **two-phase commit protocol** for distributed transactions and the **redo/undo algorithm** for crash recovery:
- **Redo** — Reapply committed transactions after recovery
- **Undo** — Roll back uncommitted transactions after recovery

The recovery process uses LSN-based algorithms:
- **Analysis phase** — Identify active transactions at crash
- **Redo phase** — Reapply all committed changes
- **Undo phase** — Roll back uncommitted transactions

### Checkpoint Process

Checkpoints flush dirty pages (modified pages) from the buffer pool to disk:
- **Lazy writer** — Background thread that identifies pages to flush
- **Checkpoint** — Forced flush of all dirty pages above a threshold
- **Auto-checkpoint** — SQL Server 2012+ automatic checkpointing based on target recovery time

**Target Recovery Time (TRT):** The maximum time SQL Server aims to complete recovery after a crash. SQL Server 2012+ uses a dynamic TRT that adapts to database workload.

```sql
-- Check checkpoint activity
SELECT database_id,
       name,
       recovery_model_desc,
       log_reuse_wait_desc,
       log_write_wait_ms,
       checkpoint_lsn,
       recovery_lsn
FROM sys.databases;
```

### TempDB Storage

TempDB is a special system database used for:
- Temporal tables and table variables
- Internal worktables (sorting, hashing, spooling)
- Version store (snapshot isolation, indexes)
- Online index operations (sort results)
- Cursor temporary storage

TempDB is recreated on each SQL Server restart and is shared across all database sessions.

## Lock Manager

The lock manager controls concurrent access to database resources:
- **Lock escalation** — Conversion from row/page locks to table locks
- **Lock modes** — Shared (S), Update (U), Exclusive (X), Intent (IS, IX, IU)
- **Lock granularities** — Row, page, extent, table, database
- **Lock timeout** — configurable via LOCK_TIMEOUT session option
- **Lock hierarchy** — Parent locks must be held to obtain child locks

### Lock Modes

- **Shared (S)** — For read operations, allows concurrent readers
- **Update (U)** — For update operations, prevents deadlocks between readers and writers
- **Exclusive (X)** — For write operations, blocks all other access
- **Intent (IS, IX, IU)** — Indicates intention to acquire lower-level locks

### Deadlock Handling

SQL Server detects deadlocks by examining the lock wait-for graph. When a cycle is detected, one transaction is chosen as the deadlock victim based on:
- Cost of transaction rollback
- Transaction isolation level
- Priority (set via SET DEADLOCK_PRIORITY)

The deadlock victim receives error 1205 and the transaction is rolled back. Deadlock information is captured in:
- ERRORLOG (deadlock graph)
- System_health Extended Events session
- Dedicated Admin Connection (DAC) DMVs

```sql
-- Query deadlock information from system_health
SELECT XEvent.value('(data/value)[1]', 'varchar(max)') AS deadlock_graph
FROM (
    SELECT XEvent = CONVERT(XML, event_data)
    FROM sys.fn_xe_file_target_read_file(
        'system_health*.xel', NULL, NULL, NULL)
) AS t
WHERE XEvent.value('(event/@name)[1]', 'varchar(50)') = 'xml_deadlock_report';
```

## Thread Architecture

SQL Server uses a sophisticated thread model to manage concurrent work:

### Schedulers

- Each scheduler represents a logical CPU
- Schedulers are managed by the SQL Server Resource Governor
- One scheduler per CPU core for user tasks

### Worker Threads

- **Cooperative scheduling** — Worker threads yield control voluntarily
- **Pre-emptive scheduling** — Used for I/O operations and external calls
- **Maximum worker threads** — Configurable via max worker threads option
- **Thread stack size** — Default 2 MB, can be configured

### Thread Pool

SQL Server manages a thread pool that adjusts to workload:
- **Worker threads** — Handle query execution, background tasks
- **Task threads** — Light-weight tasks (context switching)
- **I/O completion threads** — Handle asynchronous I/O completion
- **Buffer pool threads** — Background buffer pool management

### Resource Governor

Resource Governor controls resource consumption by workload groups:
- **Workload groups** — Groups of requests with similar resource requirements
- **Resource pools** — Collections of system resources allocated to groups
- **Classifier function** — Determines which group a request belongs to
- **Max resource usage** — CPU and memory limits per pool

## Version Control Architecture

SQL Server maintains version chains for consistent reads:
- **Version Store** — TempDB-stored versions of modified rows for snapshot isolation
- **Version Generator** — Creates and manages row versions
- **Version Cleaner** — Removes obsolete versions from the version store
- **GC Threshold** — Determines when versions become obsolete

The version store is critical for:
- Snapshot isolation reads (READ_COMMITTED_SNAPSHOT)
- Snapshot isolation transactions (SNAPSHOT isolation level)
- Online index operations
- Trigger row versioning

## SQL Server 2017/2019/2022 Architecture Enhancements

### SQL Server 2017
- **Intelligent Query Processing (IQP)** features
- **Always On** enhancements (cross-platform availability groups)
- **Memory-optimized tables** (In-Memory OLTP)
- **Python and R** integration in-database
- **Graph database** support

### SQL Server 2019
- **Batch mode on rowstore** — Columnar-like query processing for rowstore tables
- **Smart memory grant** — Reduced memory grant overestimation
- **Accelerated database recovery (ADR)** — Instant file initialization + transaction snapshot
- **Automatic tuning** — Automatic index management, force plan, verify plan
- **Big Data Clusters** — HDFS, Spark, and SQL Server integration

### SQL Server 2022
- **Vectorized batch mode** — Enhanced columnar processing for analytic workloads
- **Improvements to Intelligent Query Processing** — Adaptive joins, batch mode adaptive join, batch mode memory grant feedback
- **Intelligent performance insights** — Query performance advisor
- **Enhanced security** — Intelligent data protection, column-level encryption improvements
- **JSON improvements** — JSON_INDEXES function, JSON path expressions
- **Hash join improvements** — Better cardinality estimation for hash joins

## Conclusion

Understanding SQL Server architecture is fundamental to effective database administration and performance tuning. The buffer pool, query processor, storage engine, and lock manager work in concert to deliver ACID-compliant transaction processing. Key monitoring points include memory clerk usage, wait statistics, query plan cache, and lock statistics. The continuous evolution from SQL Server 2017 through 2022 has introduced increasingly sophisticated performance optimizations, with Intelligent Query Processing and batch mode processing being the most impactful for modern workloads.