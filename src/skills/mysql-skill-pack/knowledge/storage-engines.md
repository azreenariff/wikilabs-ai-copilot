# MySQL Storage Engines

## Overview

MySQL's pluggable storage engine architecture allows different storage backends to handle data with different characteristics. MySQL 8.0 supports six storage engines, each optimized for specific workloads. Understanding when and how to use each engine is critical for performance, reliability, and data integrity.

## InnoDB — The Default Storage Engine

InnoDB is the default and recommended storage engine for all MySQL production workloads. It provides ACID compliance, row-level locking, crash recovery, and foreign key support.

### Core Features

**Transaction Support**: Full ACID (Atomicity, Consistency, Isolation, Durability) compliance. Every transaction is either fully committed or fully rolled back, ensuring data integrity even during system failures.

**Row-Level Locking**: InnoDB uses row-level locks by default, allowing high concurrency. Multiple transactions can modify different rows in the same table simultaneously without blocking each other.

**Clustered Indexes**: InnoDB stores table data in a B-tree structure organized by the primary key (clustered index). This means the physical storage order matches the primary key order, making primary key lookups extremely efficient.

**Foreign Key Constraints**: InnoDB enforces referential integrity through foreign keys, preventing orphaned records and maintaining data relationships across tables.

**Crash Recovery**: The redo log (physical changes to data pages) and undo log (before-images for rollback) enable crash recovery. On startup, InnoDB replays redo log entries to restore unflushed changes and rolls back uncommitted transactions using undo log.

**Multi-Version Concurrency Control (MVCC)**: InnoDB provides snapshot isolation through undo log version chains. Each transaction sees a consistent snapshot of the data as of its start time, preventing dirty reads and reducing lock contention.

### InnoDB Internal Architecture

**Buffer Pool**: The buffer pool is InnoDB's memory area for caching data and index pages. It is the single most important performance parameter. Data pages and index pages are cached here, and I/O hits against the buffer pool are orders of magnitude faster than disk I/O.

- **Page size**: Default 16KB (configurable at compile time)
- **Size**: Set via `innodb_buffer_pool_size` — 60-80% of RAM on dedicated servers
- **Instances**: Multiple instances via `innodb_buffer_pool_instances` reduce locking contention
- **Hit ratio**: Target > 99% (`1 - Innodb_buffer_pool_reads / Innodb_buffer_pool_read_requests`)
- **LRU management**: Pages follow an LRU-like list with midpoint insertion for newly read pages

**Redo Log**: The redo log ensures durability and crash recovery through write-ahead logging (WAL). Physical changes are recorded in the redo log before being written to data pages on disk.

- **Circular buffer**: 2 or more log files, written circularly
- **Size**: `innodb_log_file_size` — typically 1-4 GB per file
- **Flush strategy**: `innodb_flush_log_at_trx_commit` (1=ACID, 2=fast with 1s risk, 0=fastest with crash risk)
- **Checkpointing**: Periodic flush of dirty pages from buffer pool to disk, triggered by redo log fill level

**Undo Log**: The undo log stores before-images for transaction rollback and MVCC snapshots.

- **Undo tablespaces**: Managed via `innodb_undo_tablespaces`
- **Number of undo log segments**: `innodb_undo_logs` (default 128)
- **Automatic truncation**: Undo logs are truncated when no active transaction needs them
- **Long-running transactions**: Can keep undo logs from being truncated, causing undo tablespace growth

**Change Buffer**: The change buffer caches changes to secondary index pages when the target page is not in the buffer pool. This avoids random I/O for secondary index updates.

- **Maximum size**: `innodb_change_buffer_max_size` (default 25% of buffer pool)
- **Applies to**: Non-unique secondary indexes only
- **Merge on read**: Changes are merged when the affected page is later read
- **Impact**: Significant performance improvement for INSERT-heavy workloads with secondary indexes

**Doublewrite Buffer**: A special buffer that receives data pages before they are written to their final location. Protects against partial page writes (torn pages) caused by crashes or OS-level failures.

- **Size**: 128 pages (2MB at 16KB page size)
- **Process**: InnoDB writes to doublewrite buffer first, then writes to final location
- **Recovery**: On crash recovery, InnoDB checks doublewrite buffer for complete pages before applying redo log

### Adaptive Hash Index

InnoDB monitors index lookups and, when it detects that certain index values are accessed very frequently, automatically creates a hash index in memory atop the existing B-tree indexes. This provides O(1) lookups for highly selective queries on frequently accessed values.

- **Automatic**: Created and managed by InnoDB without configuration
- **Monitoring**: `innodb_adaptive_hash_index` parameter (default ON)
- **Trade-off**: Hash index maintenance adds CPU overhead; monitor with `performance_schema`
- **Dropped automatically**: When access patterns change, InnoDB drops the hash index

### InnoDB Tuning Parameters

| Parameter | Purpose | Typical Value | Impact |
|-----------|---------|--------------|--------|
| innodb_buffer_pool_size | Buffer pool size | 60-80% of RAM | Largest performance impact |
| innodb_buffer_pool_instances | Number of instances | 8+ (high concurrency) | Reduces lock contention |
| innodb_log_file_size | Redo log file size | 1-4 GB | Write performance, crash recovery time |
| innodb_flush_log_at_trx_commit | Flush strategy | 1 or 2 | Durability vs performance |
| innodb_flush_method | I/O method | O_DIRECT | Avoids double buffering |
| innodb_io_capacity | I/O capacity for flushing | 200 (HDD), 2000 (SSD) | Checkpoint aggressiveness |
| innodb_io_capacity_max | Max I/O capacity | 2x io_capacity | Burst flush capacity |
| innodb_thread_concurrency | Max concurrent threads | 0 (unlimited) | Adjust if CPU-bound |
| innodb_read_io_threads | Read I/O threads | 4 | Read parallelism |
| innodb_write_io_threads | Write I/O threads | 4 | Write parallelism |
| innodb_open_files | Max open .ibd files | 2000+ | Large table count support |

### InnoDB Failure Modes

**Deadlock**: Two or more transactions holding locks that each other needs. InnoDB detects the cycle and rolls back one transaction. Mitigate by ordering lock acquisition, adding indexes, reducing transaction scope.

**Lock Wait Timeout**: A transaction waits longer than `innodb_lock_wait_timeout` (default 50s) for a lock. Check for blocking queries, missing indexes causing table locks, or long-running transactions.

**Buffer Pool Exhaustion**: `Innodb_buffer_pool_pages_free` near zero indicates insufficient buffer pool size. Monitor hit ratio and increase `innodb_buffer_pool_size` if needed.

**Data Dictionary Corruption**: MySQL 8.0's transactional data dictionary prevents this better than MySQL 5.7. If detected (`innodb_force_recovery`), attempt to start at recovery level 1-6 and extract data, then restore from backup.

---

## MyISAM

MyISAM is a legacy storage engine that does not support transactions, row-level locking, or foreign keys. It uses table-level locking and is generally not recommended for new applications.

### Characteristics

**No Transaction Support**: Each statement is auto-committed. No ROLLBACK capability. Crash recovery is limited to table repair.

**Table-Level Locking**: All operations (reads and writes) acquire a lock on the entire table. This severely limits concurrency — a single write blocks all reads and other writes.

**Fast Reads**: For read-heavy, write-rare workloads without transaction requirements, MyISAM can be faster than InnoDB due to simpler locking and no undo/redo overhead.

**Storage Structure**:
- `.MYD` file: Data file
- `.MYI` file: Index file
- `.frm` file: Table definition (shared with InnoDB)

**Features**:
- Full-text search indexes (native support before MySQL 5.6)
- Compressed read-only tables (myisampack)
- Automatic key cache for indexes

### When MyISAM Might Still Be Used

1. **Legacy applications**: Existing MyISAM tables not yet converted
2. **Read-heavy log tables**: Tables with massive inserts and no updates/deletes
3. **Full-text search**: Before MySQL 5.6, MyISAM was the only engine with full-text indexes

### Limitations

- No foreign key constraints
- No crash-safe recovery for data pages
- Table-level locking limits concurrency
- No MVCC
- `ALTER TABLE` operations require full table rebuild (slow for large tables)
- Not supported for system tables in MySQL 8.0

### Migration to InnoDB

To convert MyISAM tables to InnoDB:
```
ALTER TABLE table_name ENGINE=InnoDB;
```

Or use Percona's `pt-online-schema-change` for large tables with minimal downtime.

---

## Archive Engine

The Archive storage engine is designed for high-compression, append-only data storage. It is ideal for logging, data warehousing, and archival workloads.

### Characteristics

**Append-Only**: Only INSERT and SELECT are supported. No UPDATE or DELETE operations. This design enables maximum compression.

**ZLIB Compression**: Uses ZLIB compression for storage. Compression ratios vary by data type but typically achieve 4:1 to 10:1 compression for log data.

**No Indexing**: The Archive engine does not support indexes (except for an implicit row pointer). All queries require full table scans. This is acceptable for archival workloads where data is written sequentially and read by range.

**High Throughput Writes**: Because of no indexing and single-writer design, Archive tables can handle very high write throughput.

### Configuration

- `innodb_archive_compression`: Controls compression level (1-9, where 9 is maximum)
- Default compression level is 6

### Use Cases

1. **Application logging**: Store application logs with high compression
2. **Time-series data**: Store sensor readings, metrics, or event logs
3. **Data warehousing**: Archive historical data before loading aggregated results
4. **Audit trails**: Store immutable audit records

### Limitations

- No UPDATE or DELETE — once data is written, it stays
- No indexes — only full table scans
- No foreign keys
- No transaction support
- Not suitable for OLTP workloads

---

## Memory Engine

The Memory (HEAP) storage engine stores all data in RAM, providing extremely fast access for temporary data or caching layers.

### Characteristics

**In-Memory Storage**: All data resides in memory. No disk I/O for reads or writes. Provides microsecond-level access times.

**Hash Index**: By default, Memory tables use hash indexes for O(1) lookups. The `USING BTREE` modifier enables B-tree indexes for range queries.

**Data Volatility**: All data is lost on server restart. Memory tables are suitable for caching, session storage, and temporary work tables.

**Table-Level Locking**: Despite in-memory operation, Memory tables use table-level locks. Concurrent writes are serialized.

**Storage Structure**:
- `.MYD` file: Memory pointer (not actual data)
- `.MYI` file: Index pointer
- `.frm` file: Table definition

### Configuration

| Parameter | Purpose | Default | Notes |
|-----------|---------|---------|-------|
| max_heap_table_size | Max size of individual Memory table | Same as tmp_table_size | Per-table limit |
| tmp_table_size | Max size of internal temp tables | 16MB | Also limits MEMORY tables |

### Use Cases

1. **Session storage**: Cache user session data for fast access
2. **Lookup tables**: Frequently accessed reference data
3. **Temporary processing**: Intermediate results for complex queries
4. **Counters and accumulators**: Real-time metrics aggregation

### Limitations

- Data lost on restart — no durability
- Table-level locking limits write concurrency
- Memory consumption grows with data — watch for OOM
- max_heap_table_size and tmp_table_size constraints
- Not suitable for critical data

---

## Blackhole Engine

The Blackhole storage engine accepts and stores no data. INSERT and UPDATE operations are processed but the data is discarded. SELECT always returns empty sets.

### Characteristics

**Data Discard**: All data-modifying operations succeed but nothing is stored. This makes Blackhole tables perfect for replication forwarding.

**Replication Forwarding**: When replicated, Blackhole tables cause the replicated statement to be executed on the replica — effectively acting as a relay.

### Use Cases

1. **Replication relay**: Forward statements to downstream replicas without storing them locally
2. **Testing**: Simulate table operations for application testing
3. **Schema validation**: Test DDL/DML without affecting data

### Limitations

- No data storage
- No SELECT results
- Purely a logical construct

---

## Federated Engine

The Federated engine allows MySQL to access tables on remote MySQL servers as if they were local. It creates a local table definition that maps to a remote table.

### Characteristics

**Remote Table Access**: Each table definition includes connection parameters to the remote server. Operations are executed remotely and results returned.

**No Local Storage**: Data resides entirely on the remote server. The local server only maintains the table definition.

**Network Dependency**: Availability depends on network connectivity to the remote server. Latency affects all operations.

### Configuration

The federated connection string is stored in the table's data directory file, containing:
- Host, port, user, password
- Database and table name on the remote server

### Use Cases

1. **Unified query interface**: Query multiple MySQL servers from a single client
2. **Data federation**: Combine data from geographically distributed MySQL instances

### Limitations

- Network dependency — offline = unavailable
- No local caching of data
- Limited feature support (no foreign keys, triggers, etc.)
- High latency for remote queries
- Security: credentials stored in connection parameters
- Performance: every query is a network round-trip

---

## Storage Engine Comparison

| Feature | InnoDB | MyISAM | Archive | Memory | Blackhole | Federated |
|---------|--------|--------|---------|--------|-----------|-----------|
| Transactions | Yes | No | No | No | No | No |
| Row-level locking | Yes | No | No | No | N/A | No |
| Table-level locking | Yes | Yes | Yes | Yes | N/A | N/A |
| Foreign keys | Yes | No | No | No | No | Yes (remote) |
| Crash recovery | Yes | Limited | No | No | N/A | N/A |
| MVCC | Yes | No | No | No | No | No |
| Indexes | B-tree | B-tree | None | Hash/B-tree | N/A | B-tree |
| Full-text search | Yes | Yes | No | No | No | No |
| Compressed tables | N/A | Yes (myisampack) | Yes (ZLIB) | No | N/A | N/A |
| Append-only | No | No | Yes | No | Yes (logical) | No |
| In-memory data | No | No | No | Yes | No | No |
| Remote access | No | No | No | No | No | Yes |
| Default engine | Yes (8.0+) | Legacy | Niche | Niche | Niche | Niche |

---

## Engine Selection Guide

### General Rule

**Use InnoDB for everything** unless you have a specific reason to use another engine. InnoDB provides the best balance of performance, reliability, and features for production workloads.

### Engine Selection Decision Tree

```
Does the workload require transactions?
├─ YES → InnoDB
└─ NO
   ├─ High compression needed + append-only?
   │   ├─ YES → Archive
   │   └─ NO → Continue
   ├─ Sub-millisecond latency + volatile data?
   │   ├─ YES → Memory
   │   └─ NO → Continue
   ├─ Replication relay?
   │   ├─ YES → Blackhole
   │   └─ NO → Continue
   ├─ Remote table access?
   │   ├─ YES → Federated (prefer ProxySQL for production)
   │   └─ NO → Continue
   └─ Legacy MyISAM table?
       └─ Plan migration to InnoDB
```

### Performance Considerations

| Factor | Recommendation |
|--------|---------------|
| Read-heavy, write-rare, no transactions | Consider MyISAM only if legacy |
| Write-heavy, high concurrency | InnoDB with proper buffer pool sizing |
| Mixed read/write, transactions required | InnoDB — always |
| Time-series, archival data | Archive engine |
| Session data, cache tables | Memory engine |
| Large tables with secondary indexes | InnoDB with change buffer optimization |

### Best Practices

1. **Verify engine**: Check `SHOW TABLE STATUS` or `information_schema.TABLES` to confirm all production tables use InnoDB
2. **Convert MyISAM**: Plan migration of remaining MyISAM tables to InnoDB
3. **Monitor InnoDB**: Use `SHOW ENGINE INNODB STATUS` and Performance Schema for deep monitoring
4. **Buffer pool sizing**: Set `innodb_buffer_pool_size` to 60-80% of available RAM on dedicated servers
5. **Test ALTER TABLE**: When converting from MyISAM to InnoDB, test on non-production first for large tables
6. **Monitor undo log**: Long-running transactions can cause undo tablespace growth

---

## References

- [MySQL 8.0 Reference Manual: Storage Engines](https://dev.mysql.com/doc/refman/8.0/en/storage-engines.html)
- [MySQL 8.0 Reference Manual: InnoDB Storage Engine](https://dev.mysql.com/doc/refman/8.0/en/innodb-storage-engine.html)
- [MySQL 8.0 Reference Manual: InnoDB Architecture](https://dev.mysql.com/doc/refman/8.0/en/innodb-architecture.html)
- [MySQL 8.0 Reference Manual: InnoDB Buffer Pool](https://dev.mysql.com/doc/refman/8.0/en/innodb-buffer-pool.html)
- [MySQL 8.0 Reference Manual: InnoDB Redo Log](https://dev.mysql.com/doc/refman/8.0/en/innodb-redo-log.html)
- [MySQL 8.0 Reference Manual: InnoDB Undo Log](https://dev.mysql.com/doc/refman/8.0/en/innodb-undo-tablespaces.html)
- [MySQL 8.0 Reference Manual: InnoDB Change Buffer](https://dev.mysql.com/doc/refman/8.0/en/innodb-change-buffer.html)
- [MySQL 8.0 Reference Manual: InnoDB Adaptive Hash Index](https://dev.mysql.com/doc/refman/8.0/en/innodb-adaptive-hash.html)