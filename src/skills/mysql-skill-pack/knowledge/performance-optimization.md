# MySQL Performance Optimization

## Overview

MySQL performance optimization is a multi-dimensional discipline that encompasses query optimization, index design, buffer pool tuning, I/O optimization, and system-level configuration. This document provides a comprehensive guide to diagnosing and resolving performance issues in MySQL 8.0 and 8.4.

## Query Optimization

The most impactful performance improvement in MySQL typically comes from optimizing queries rather than tuning configuration parameters.

### Understanding Query Execution

**EXPLAIN Statement**: The primary tool for understanding how MySQL executes a query.

```
EXPLAIN SELECT o.id, o.status, c.name
FROM orders o
JOIN customers c ON o.customer_id = c.id
WHERE o.status = 'pending'
  AND o.total > 100
ORDER BY o.created_at DESC
LIMIT 20;
```

**Key EXPLAIN Output Fields**:
| Field | Meaning | Good Value | Problem Indicator |
|-------|---------|-----------|------------------|
| type | Access method | ALL, index, range, ref, eq_ref, const | ALL = full table scan |
| possible_keys | Candidate indexes | Non-empty | Empty = no index available |
| key | Actual index used | Non-empty | NULL = no index used |
| key_len | Index bytes used | Appropriate for query | Shorter than expected = partial usage |
| rows | Estimated rows examined | Low compared to total | Much higher than rows examined suggests missing index |
| Extra | Additional info | Empty or "Using index" | "Using filesort", "Using temporary" |

**Access Types (Best to Worst)**:
1. **const**: Single row from primary/unique key
2. **eq_ref**: Single row per join from unique key
3. **ref**: Multiple rows from non-unique index
4. **range**: Index range scan (BETWEEN, IN, >, <)
5. **index**: Full index scan
6. **ALL**: Full table scan (worst case)

### Query Patterns to Optimize

**Pattern 1: SELECT ***
```sql
-- BAD: Fetching unnecessary columns
SELECT * FROM orders WHERE customer_id = 42;

-- GOOD: Explicit columns, covering index if possible
SELECT id, status, total FROM orders WHERE customer_id = 42;
```

**Pattern 2: ORDER BY without index**
```sql
-- BAD: Requires filesort
SELECT * FROM orders ORDER BY created_at DESC;

-- GOOD: Indexed ORDER BY
CREATE INDEX idx_orders_created ON orders(created_at DESC);
```

**Pattern 3: Implicit type conversion**
```sql
-- BAD: Index unusable due to type mismatch
SELECT * FROM users WHERE phone_number = 1234567890;  -- phone_number is VARCHAR

-- GOOD: Match type
SELECT * FROM users WHERE phone_number = '1234567890';
```

**Pattern 4: Functions on indexed columns**
```sql
-- BAD: Function on column prevents index usage
SELECT * FROM orders WHERE YEAR(created_at) = 2024;

-- GOOD: Range scan that uses index
SELECT * FROM orders WHERE created_at >= '2024-01-01' AND created_at < '2024-02-01';
```

**Pattern 5: LIKE with leading wildcard**
```sql
-- BAD: No index usage
SELECT * FROM products WHERE name LIKE '%widget%';

-- GOOD: Use FULLTEXT index for text search
CREATE FULLTEXT INDEX ft_products_name ON products(name);
SELECT * FROM products WHERE MATCH(name) AGAINST('widget');
```

**Pattern 6: Pagination with OFFSET**
```sql
-- BAD: Scanning all preceding rows for large offsets
SELECT * FROM orders ORDER BY id LIMIT 10000 OFFSET 100000;

-- GOOD: Seek pagination (keyset pagination)
SELECT * FROM orders
WHERE id > 100000
ORDER BY id
LIMIT 10;
```

### EXPLAIN ANALYZE (MySQL 8.0.18+)

Extended EXPLAIN that shows actual execution metrics rather than estimates:

```
EXPLAIN ANALYZE SELECT * FROM orders WHERE status = 'pending';
```

**Output shows**:
- Actual rows vs estimated rows
- Actual execution time
- Loop counts
- Exact execution flow

This is critical for diagnosing stale statistics and optimizer misestimates.

## Index Design

Indexes are the most important performance lever in MySQL. Good index design can reduce query times from seconds to milliseconds.

### Index Types in MySQL

| Type | Purpose | Best For |
|------|---------|----------|
| PRIMARY KEY | Clustered index on primary key | Table identity |
| UNIQUE | Enforces uniqueness | Email, username, etc. |
| INDEX (non-unique) | Speeds up lookups | Foreign keys, WHERE clauses |
| FULLTEXT | Natural language search | Text search |
| SPATIAL | Spatial data queries | Geospatial |
| HASH | Hash-based equality lookup | Memory engine only |
| INVISIBLE (8.0.13+) | Test removal without dropping | Safe index testing |

### Composite Index Design

**Leftmost Prefix Rule**: A composite index (a, b, c) can be used for queries filtering on a, a+b, a+b+c, but NOT b alone or c alone.

```
CREATE INDEX idx_abc ON table(a, b, c);

-- Uses index (leftmost): WHERE a = 1
-- Uses index (leftmost): WHERE a = 1 AND b = 2
-- Uses index (leftmost): WHERE a = 1 AND b = 2 AND c = 3
-- Uses index (range on b): WHERE a = 1 AND b > 10 AND c = 3
-- Does NOT use index: WHERE b = 2
-- Does NOT use index: WHERE c = 3
```

**Column Order Principles**:
1. **Equality columns first**: Columns used with `=`, `IN`, `<>` should come before range columns
2. **ORDER BY columns next**: Columns in ORDER BY if they follow equality conditions
3. **Range columns last**: Columns used with `>`, `<`, `BETWEEN` should come last
4. **Low cardinality last**: Columns like status (few distinct values) are least useful first in index

```
-- Good composite index order:
-- Equality on status, range on created_at, ORDER BY total
CREATE INDEX idx_status_created_total ON orders(status, created_at, total);
```

### Invisible Indexes (MySQL 8.0.13+)

Test index removal safely without actually dropping the index:

```
-- Make index invisible (optimizer ignores it)
ALTER TABLE orders ALTER INDEX idx_status INVISIBLE;

-- Monitor performance impact

-- If OK, drop it
ALTER TABLE orders DROP INDEX idx_status;

-- If not OK, make visible again
ALTER TABLE orders ALTER INDEX idx_status VISIBLE;
```

### Index Maintenance

**Analyze Table Statistics**:
```
ANALYZE TABLE orders;
```
Updates optimizer statistics. Run after large data changes.

**OPTIMIZE TABLE**:
```
OPTIMIZE TABLE orders;
```
Rebuilds table and indexes to reclaim space and defragment. Only for MyISAM or when significant space reclaimed is needed. For InnoDB, `ALTER TABLE ... FORCE` achieves similar results with less disruption.

**Monitor Index Usage**:
```
SELECT * FROM sys.schema_unused_indexes;
```
Identifies indexes that exist but are never used.

### Index Design Best Practices

1. **Index foreign key columns** — prevents table-level locks during joins
2. **Covering indexes** — include all columns needed by the query to avoid row lookups
3. **Avoid over-indexing** — each index slows writes (INSERT, UPDATE, DELETE)
4. **Use prefix indexes for long strings** — `INDEX(column(20))` for CHAR(255)
5. **Use DESC indexes for reverse-order queries** — MySQL 8.0+ supports DESC in indexes
6. **Monitor unused indexes** — remove them to reduce write overhead
7. **Consider partial indexes** — for specific query patterns, not general-purpose

### InnoDB Index Structure

- **Clustered index**: The table data IS the primary key B-tree
- **Secondary indexes**: Store the primary key value as the leaf pointer
- **Covering index**: If all queried columns are in the index, no row lookup needed
- **Cardinality**: Number of distinct values in index — higher is better for selectivity

## Buffer Pool Optimization

The buffer pool is the single most important InnoDB performance parameter.

### Sizing the Buffer Pool

**Guideline**: Set `innodb_buffer_pool_size` to 60-80% of available RAM on a dedicated database server.

```
# Check current size
SHOW VARIABLES LIKE 'innodb_buffer_pool_size';

# Check usage
SHOW STATUS LIKE 'Innodb_buffer_pool%';

# Calculate hit ratio
-- Hit ratio = 1 - (Innodb_buffer_pool_reads / Innodb_buffer_pool_read_requests)
-- Target: > 99%
```

**Multiple Instances**: For high-concurrency systems, use multiple buffer pool instances:
```
innodb_buffer_pool_size = 16G
innodb_buffer_pool_instances = 8
```
Each instance has its own LRU list and lock, reducing contention.

### Buffer Pool Monitoring

**Key Metrics**:
| Metric | Meaning | Healthy Range |
|--------|---------|--------------|
| Innodb_buffer_pool_pages_free | Unused pages | > 0 (not zero) |
| Innodb_buffer_pool_pages_dirty | Modified pages needing flush | Should stabilize |
| Innodb_buffer_pool_pages_total | Total allocated pages | Matches configured size |
| Innodb_buffer_pool_read_requests | Total read attempts | Increasing normally |
| Innodb_buffer_pool_reads | Physical disk reads | < 1% of read_requests |
| Innodb_buffer_pool_read_aheads | Pre-read pages | Depends on workload |

**Performance Schema views**:
```
SELECT * FROM sys.memory_global_total;
SELECT * FROM sys.memory_global_current;
SELECT * FROM sys.memory_by_thread_by_current_bytes;
```

### Dirty Page Management

**Write-Back Strategy**:
- `innodb_flush_method=O_DIRECT`: Bypass OS cache
- `innodb_flush_log_at_trx_commit=1`: Full ACID (flush every commit)
- `innodb_flush_log_at_trx_commit=2`: Fast with 1s crash risk

**I/O Capacity Tuning**:
```
innodb_io_capacity = 200    # HDD: 200, SSD: 2000
innodb_io_capacity_max = 400   # HDD: 400, SSD: 4000
```
Controls how aggressively InnoDB flushes dirty pages. Higher values mean more aggressive flushing.

## Slow Query Diagnosis

### Slow Query Log

**Enable**:
```
slow_query_log = ON
slow_query_log_file = /var/log/mysql/slow.log
long_query_time = 1.0   # Log queries exceeding 1 second
```

**Parse with pt-query-digest**:
```
pt-query-digest /var/log/mysql/slow.log
```
Provides query statistics, classification, and recommendations.

### Performance Schema

Performance Schema instruments internal server operations for detailed analysis.

**Key tables**:
| Table | Purpose |
|-------|---------|
| events_statements_summary_by_digest | Aggregated statement statistics |
| events_waits_summary_by_index_usage | Wait events by index usage |
| events_waits_summary_by_table | Wait events by table |
| hosts | Host-level statistics |
| users | User-level statistics |

**Common queries**:
```
-- Top 10 statements by total time
SELECT digest,
  ROUND(SUM/TIMER_WAIT/1000000000000,2) AS total_time_s,
  ROUND(AVG/TIMER_WAIT/1000000000000,2) AS avg_time_s,
  COUNT_STAR AS exec_count
FROM performance_schema.events_statements_summary_by_digest
ORDER BY total_time_s DESC
LIMIT 10;

-- Top tables by lock wait time
SELECT OBJECT_SCHEMA, OBJECT_NAME, COUNT_STAR,
  SUM/TIMER_WAIT/1000000000000 AS total_latency_s
FROM performance_schema.table_io_waits_summary_by_table
WHERE COUNT_STAR > 0
ORDER BY total_latency_s DESC
LIMIT 10;
```

### Sys Schema Views

The sys schema provides human-friendly views over Performance Schema:

| View | Purpose |
|------|---------|
| schema_table_statistics_with_buffer | Table I/O and lock stats |
| host_by_current_memory_usage | Memory usage by host |
| user_by_current_memory_usage | Memory usage by user |
| statement_analysis | SQL statement performance analysis |
| host_summary | Host-level aggregated stats |
| statement_type_stats | Statement type breakdown |
| schema_index_statistics | Index usage statistics |
| schema_redundant_indexes | Redundant index detection |

## Connection and Thread Management

### Connection Tuning

**max_connections**: Set to handle peak concurrent connections plus overhead.
```
# Typical values: 200-500 for web apps
# High-connection apps may need 1000+
```

**wait_timeout**: Reduce idle connection timeout to free resources.
```
# For web apps: 300-600 seconds
# Default is 28800 (8 hours) — too high
```

**thread_cache_size**: Cache idle threads for reuse.
```
# Monitor Threads_created vs Connections
# If Threads_created >> Connections, increase thread_cache_size
```

### Thread Pool (MySQL 8.0)

Thread pool plugin manages connection multiplexing:

**Benefits**:
- Reduces memory per connection
- Prevents thread creation spikes
- Queues connections during overload
- Prioritizes queries

**Configuration**:
```
plugin_load_add="thread_pool.so"
thread_pool_size = 8   # Number of worker threads
thread_pool_stall_limit = 500  # ms before considering stalled
```

## I/O Optimization

### Disk I/O Monitoring

**Linux tools**:
- `iostat -x 1`: CPU and I/O statistics
- `vmstat 1`: Virtual memory statistics
- `iotop`: Per-process I/O monitoring

**MySQL indicators**:
- `Innodb_buffer_pool_reads`: Physical reads (high = buffer pool too small)
- `Innodb_data_reads`: Data read I/O
- `Innodb_data_writes`: Data write I/O
- `Handler_read_rnd`: Random reads (high = full table scans)

### Storage Recommendations

| Workload | Storage Type | Notes |
|----------|-------------|-------|
| General OLTP | SSD/NVMe | Low latency critical |
| High-write workload | NVMe with RAID 10 | Write amplification protection |
| Read-heavy | SSD with large buffer pool | Buffer pool absorbs reads |
| Time-series | HDD with large sequential writes | Sequential I/O friendly |
| Archival | HDD | Cost-effective for sequential access |

### Filesystem Tuning

```
# Recommended mount options
noatime,notail,commit=60

# Disable access time updates
mount -o remount,noatime /data

# XFS: Optimize for MySQL
mkfs.xfs -f -i size=512 /dev/sdX
```

## CPU and Memory Optimization

### CPU Monitoring

**Key indicators**:
- High `Threads_running` relative to CPU cores
- `Innodb_adaptive_hash_index` contention
- `Innodb_buffer_pool_wait_free` — waits for free buffer pool pages

**Tuning**:
- `innodb_thread_concurrency`: Limit concurrent threads (default 0 = unlimited)
- `innodb_read_io_threads` / `innodb_write_io_threads`: Set to 4-8 for SSD
- `innodb_page_cleaners`: Flush thread count (default 4, increase for high dirty page rate)

### Memory Management

**Total memory budget**:
```
innodb_buffer_pool_size: 60-80% of RAM (dedicated server)
OS and other processes: 15-20%
Query buffers, thread stacks: 5-10%
```

**Per-connection memory** (each connection uses):
- `join_buffer_size`: Up to 4MB per join
- `sort_buffer_size`: Up to 8MB per sort
- `read_buffer_size`: Per sequential scan
- `read_rnd_buffer_size`: Per random read
- Thread stack: ~2MB

**With many connections, per-connection buffers can dominate**:
```
# Reduce per-connection buffers if using many connections
join_buffer_size = 256K   # Default 256K — usually sufficient
sort_buffer_size = 256K   # Default 2MB — reduce for high connection count
read_buffer_size = 128K   # Default 128K — usually fine
read_rnd_buffer_size = 256K # Default 256K — usually fine
```

## Version-Specific Optimizations

### MySQL 8.0 Performance Features

| Feature | Benefit | MySQL Version |
|---------|---------|---------------|
| Hash joins | Faster large table joins | 8.0.18+ |
| Parallel query execution | Parallel scan and aggregation | 8.0.30+ |
| Invisible indexes | Safe index testing | 8.0.13+ |
| Instant DDL | Non-blocking ALTER TABLE | 8.0.12+ |
| Window functions | Analytical queries in single pass | 8.0+ |
| CTEs | Recursive and common table expressions | 8.0+ |
| Optimized optimizer_switch | Fine-grained control | 8.0+ |

### MySQL 8.4 Performance Improvements

| Feature | Benefit |
|---------|---------|
| JSON data dictionary | Faster metadata operations |
| Enhanced Instant DDL | More ALTER TABLE operations are instant |
| Optimized buffer pool page life cycle | Better page eviction |
| Improved cost model | Better query plans |

## Optimization Workflow

### Step-by-Step Approach

1. **Establish baseline**: Measure current performance
2. **Identify bottlenecks**: Use slow query log, Performance Schema, sys views
3. **Optimize queries**: Fix SELECT *, functions on indexed columns, missing indexes
4. **Design indexes**: Add composite indexes for frequent query patterns
5. **Tune buffer pool**: Set to 60-80% of available RAM
6. **Tune I/O**: Configure io_capacity for storage type
7. **Verify improvements**: Compare against baseline
8. **Monitor continuously**: Set up ongoing performance monitoring

### Optimization Checklist

- [ ] Slow query log enabled with appropriate threshold
- [ ] EXPLAIN used for all slow queries
- [ ] Primary keys on all tables
- [ ] Foreign key indexes present
- [ ] Composite indexes match query patterns (leftmost prefix)
- [ ] No functions on indexed columns in WHERE clauses
- [ ] SELECT * replaced with explicit columns
- [ ] LIKE patterns avoid leading wildcards
- [ ] Pagination uses keyset, not OFFSET
- [ ] Buffer pool sized appropriately
- [ ] innodb_flush_method = O_DIRECT
- [ ] innodb_io_capacity matches storage type
- [ ] Thread pool enabled for high connection count
- [ ] Per-connection buffers tuned for connection count
- [ ] Table statistics current (ANALYZE TABLE)
- [ ] Unused indexes identified and removed

## References

- [MySQL 8.0 Reference Manual: Optimization](https://dev.mysql.com/doc/refman/8.0/en/optimization.html)
- [MySQL 8.0 Reference Manual: Query Optimization](https://dev.mysql.com/doc/refman/8.0/en/optimizing-queries.html)
- [MySQL 8.0 Reference Manual: EXPLAIN Output](https://dev.mysql.com/doc/refman/8.0/en/explain-output.html)
- [MySQL 8.0 Reference Manual: Indexes](https://dev.mysql.com/doc/refman/8.0/en/indexes.html)
- [MySQL 8.0 Reference Manual: Buffer Pool](https://dev.mysql.com/doc/refman/8.0/en/innodb-buffer-pool.html)
- [MySQL 8.0 Reference Manual: Slow Query Log](https://dev.mysql.com/doc/refman/8.0/en/slow-query-log.html)
- [MySQL 8.0 Reference Manual: Performance Schema](https://dev.mysql.com/doc/refman/8.0/en/performance-schema.html)
- [MySQL 8.0 Reference Manual: Sys Schema](https://dev.mysql.com/doc/refman/8.0/en/sys-schema.html)