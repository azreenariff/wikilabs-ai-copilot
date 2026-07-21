# Indexes and Statistics

## Overview

Indexes and statistics are fundamental to SQL Server query performance. Indexes provide efficient data access paths, while statistics provide the query optimizer with the cardinality estimation data it needs to choose optimal execution plans. Misconfigured or outdated indexes and statistics are among the most common causes of SQL Server performance problems.

This document covers index types, statistics management, execution plan analysis, and query optimization techniques for SQL Server 2017, 2019, and 2022.

## Index Fundamentals

### How Indexes Work

An index is a data structure (typically a B-tree) that provides a fast lookup path to table data. Without an index, SQL Server must perform a table scan — reading every row in the table sequentially. With an index, SQL Server can navigate directly to the required rows using the B-tree search.

**B-tree structure:**
- **Root page** — Top-level page with pointers to intermediate pages
- **Intermediate pages** — Multi-level pages with pointers to lower pages
- **Leaf pages** — Bottom-level pages containing the actual index data
- **Leaf level for clustered index** — Contains the actual table data pages
- **Leaf level for nonclustered index** — Contains index key values and row locators

**Search performance:**
- With n pages in the B-tree, lookup time is O(log n)
- A B-tree with 3 levels can search 1 million+ rows efficiently
- Depth is determined by data volume and key size

### Clustered Index

A clustered index determines the physical order of data in a table. A table can have only one clustered index.

**Characteristics:**
- Data rows are stored in leaf pages in sorted order
- Leaf level contains the actual table data
- Only one clustered index per table
- Primary key is often the clustered index (but not required)
- All nonclustered indexes include clustered index key as row locator

```sql
-- Create clustered index
CREATE CLUSTERED INDEX CX_Orders_OrderDate
ON Orders (OrderDate);

-- Drop clustered index
DROP INDEX CX_Orders_OrderDate ON Orders;
```

### Nonclustered Index

A nonclustered index is a separate structure from the data rows. It contains index key values and a row locator pointing to the actual data.

**Characteristics:**
- Can have multiple nonclustered indexes per table
- Leaf level contains index key values and row locator
- Row locator depends on table structure:
  - Heap table: file-page-row ID (RID)
  - Clustered table: clustered index key
- Nonclustered indexes do not affect physical data ordering

```sql
-- Create nonclustered index
CREATE NONCLUSTERED INDEX IX_Orders_CustomerID
ON Orders (CustomerID)
INCLUDE (OrderAmount, OrderStatus);

-- Filtered index (reduces index size for selective data)
CREATE NONCLUSTERED INDEX IX_Orders_Active
ON Orders (OrderDate)
WHERE OrderStatus = 'Active';
```

### Covering Indexes

A covering index contains all columns required by a query, allowing the query to be satisfied entirely from the index without accessing the table.

**Benefits:**
- Eliminates key lookups
- Reduces I/O
- Improves query performance significantly

```sql
-- Covering index for a specific query
-- Query: SELECT CustomerID, OrderAmount FROM Orders WHERE CustomerID = 123
CREATE NONCLUSTERED INDEX IX_Orders_CustAmount
ON Orders (CustomerID)
INCLUDE (OrderAmount);
```

### Columnstore Indexes

Columnstore indexes store data in columnar format rather than row-based format. This provides significant benefits for analytical and reporting workloads.

**Characteristics:**
- **Clustered columnstore index** — Replaces the clustered index entirely
- **Nonclustered columnstore index** — Additional index on top of rowstore
- **Batch mode execution** — Columnstore enables batch processing for 3-7x performance
- **Compression** — Typically 10x compression vs rowstore
- **Best for** — Data warehouse tables, large fact tables, aggregate queries

**Rowgroup and segment:**
- **Rowgroup** — A batch of rows stored in columnar format (typically 1 million rows)
- **Segment** — A subset of a rowgroup that can be eliminated during queries
- **DelRowGroups** — Background process that closes and compacts rowgroups

```sql
-- Create clustered columnstore index
CREATE CLUSTERED COLUMNSTORE INDEX CCI_Orders
ON Orders;

-- Create nonclustered columnstore index
CREATE NONCLUSTERED COLUMNSTORE INDEX NCCI_Orders
ON Orders
(
    OrderDate, CustomerID, OrderAmount, OrderStatus
);

-- Index rebuild to force rowgroup compaction
ALTER INDEX ALL ON Orders REBUILD;
```

**SQL Server 2019+ Columnstore improvements:**
- Batch mode on rowstore — Columnar-like processing for rowstore tables
- Columnstore index rebuild in online mode
- Delta rowgroup merge improvements
- Batch mode adaptive join with columnstore

### Filtered Indexes

Filtered indexes are optimized for queries that filter on a specific subset of data.

**Characteristics:**
- Include a WHERE clause in the index definition
- Smaller index size than full indexes
- Faster updates for non-filtered data
- Automatic maintenance — SQL Server maintains them automatically

```sql
-- Filtered index for active records
CREATE NONCLUSTERED INDEX IX_Orders_Active
ON Orders (OrderDate)
WHERE OrderStatus = 'Active';

-- Filtered index for NULL values
CREATE NONCLUSTERED INDEX IX_Orders_NullDate
ON Orders (ShipDate)
WHERE ShipDate IS NULL;
```

### Index Fill Factor

Fill factor determines the percentage of space on each leaf-level page to fill with data, leaving room for expansion.

**Fill factor values:**
- **100 (default)** — Pages fully packed, no room for growth
- **70-90** — Recommended for tables with frequent updates
- **0 or 100** — Same behavior for new indexes

```sql
-- Create index with fill factor
CREATE NONCLUSTERED INDEX IX_Orders_CustomerID
ON Orders (CustomerID)
WITH (FILLFACTOR = 80);

-- Set default fill factor at server level
EXEC sp_configure 'fill factor percentage', 80;
RECONFIGURE;
```

## Statistics

### What Are Statistics?

Statistics are objects that contain distribution information about the data in one or more columns. The query optimizer uses statistics to estimate the number of rows (cardinality) that will result from query predicates, which directly affects execution plan selection.

**Statistics components:**
- **Histogram** — Distribution of values in the first key column
- **Density vector** — Correlation between columns in the index
- **Density** — Number of distinct values / total rows
- **Last updated** — Timestamp of last statistics update
- **Sample size** — Number of rows sampled

```sql
-- View statistics
SELECT name,
       last_updated,
       rows_sampled,
       rows,
       steps,
       unfiltered_rows
FROM sys.stats
WHERE object_id = OBJECT_ID('Orders');

-- Update statistics
UPDATE STATISTICS Orders;

-- Update statistics with full scan
UPDATE STATISTICS Orders WITH FULLSCAN;

-- Update statistics for a specific column
UPDATE STATISTICS Orders (Stats_OrderAmount);
```

### Statistics and Query Performance

The query optimizer relies on statistics for cardinality estimation:

**Without statistics:**
- Optimizer assumes uniform distribution
- Selectivity estimated as 1 / (number of distinct values)
- Plans may be suboptimal for skewed data

**With accurate statistics:**
- Optimizer knows value distribution
- Can choose appropriate join algorithms
- Can choose appropriate access methods (seek vs scan)
- Can estimate memory grants accurately

### Auto-Create Statistics

SQL Server automatically creates statistics on columns used in predicates:

```sql
-- Auto-create stats enabled (default)
-- When query uses: SELECT * FROM Orders WHERE CustomerID = 123
-- SQL Server creates: _WA_Sys_00000001_12345678
-- If no existing stats on CustomerID column
```

**Auto-created statistics naming:**
- Format: `_WA_Sys_<hash>_<table_id>`
- Created when a column in a predicate has no existing statistics

### Auto-Update Statistics

SQL Server automatically updates statistics when they become stale:

**Staleness thresholds:**
- **For tables with < 500 rows** — Update when any row changes
- **For tables with >= 500 rows** — Update when:
  - 500 + 20% of rows have changed, OR
  - For large tables, threshold is based on row count

```sql
-- Control auto-update statistics
ALTER DATABASE [MyDatabase] SET AUTO_UPDATE_STATISTICS ON;
ALTER DATABASE [MyDatabase] SET AUTO_UPDATE_STATISTICS_ASYNC OFF;

-- Async statistics update (does not block query)
ALTER DATABASE [MyDatabase] SET AUTO_UPDATE_STATISTICS_ASYNC ON;
```

**Async statistics update:**
- Query proceeds with old statistics
- Statistics update in background
- Prevents plan recompilation blocking
- Risk: Suboptimal plans until update completes

### Statistics with Full Scan

For best accuracy, use FULLSCAN on critical tables:

```sql
-- Full scan statistics update (most accurate)
UPDATE STATISTICS Orders WITH FULLSCAN;

-- Sampled statistics update (default behavior)
UPDATE STATISTICS Orders WITH SAMPLE 20 PERCENT;

-- Sampled statistics with fixed row count
UPDATE STATISTICS Orders WITH SAMPLE 1000000 ROWS;
```

**FULLSCAN trade-off:**
- **Pros** — Most accurate cardinality estimates
- **Cons** — More I/O, longer update time
- **Recommendation** — Use FULLSCAN for critical tables during maintenance windows

## Index Maintenance

### Index Fragmentation

Fragmentation occurs when index pages are not in logical order, causing additional I/O during scans:

**Fragmentation types:**
- **Internal fragmentation** — Pages not fully utilized (fill factor related)
- **External fragmentation** — Logical order doesn't match physical order
- **Page splits** — New rows inserted in middle of page, causing split

**Measuring fragmentation:**

```sql
-- Check index fragmentation
SELECT OBJECT_NAME(s.object_id) AS table_name,
       i.name AS index_name,
       s.avg_fragmentation_in_percent,
       s.avg_page_space_used_in_percent,
       s.page_count,
       s.record_count,
       s.fragment_count
FROM sys.dm_db_index_physical_stats(
    DB_ID('MyDatabase'), NULL, NULL, NULL, 'DETAILED') s
JOIN sys.indexes i ON s.object_id = i.object_id AND s.index_id = i.index_id
WHERE s.avg_fragmentation_in_percent > 10
ORDER BY s.avg_fragmentation_in_percent DESC;
```

**Fragmentation thresholds:**
- **< 5%:** No action needed
- **5-30%:** Reorganize index
- **> 30%:** Rebuild index

### Index Reorganize

Reorganize physically reorders index pages to match logical order, defragmenting at leaf level.

```sql
-- Reorganize specific index
ALTER INDEX IX_Orders_CustomerID ON Orders REORGANIZE;

-- Reorganize all indexes on a table
ALTER INDEX ALL ON Orders REORGANIZE;

-- Reorganize all indexes on all tables in database
EXEC sp_MSforeachtable
    'ALTER INDEX ALL ON ? REORGANIZE';
```

### Index Rebuild

Rebuild drops and recreates the index, providing the most complete defragmentation.

```sql
-- Rebuild specific index
ALTER INDEX IX_Orders_CustomerID ON Orders REBUILD;

-- Rebuild all indexes on a table
ALTER INDEX ALL ON Orders REBUILD;

-- Rebuild with online option (Enterprise Edition)
ALTER INDEX ALL ON Orders REBUILD WITH (ONLINE = ON);

-- Rebuild and reorganize
ALTER INDEX ALL ON Orders REBUILD;
ALTER INDEX ALL ON Orders REORGANIZE;
```

**Online rebuild (Enterprise Edition):**
- Table remains available during rebuild
- Requires tempDB space for sorting
- Reduces lock duration
- Additional I/O for tempDB

### Ola Hallengren Maintenance Solution

Ola Hallengren's maintenance scripts are the industry standard for SQL Server index maintenance:

```sql
-- Index optimization using Ola Hallengren
EXECUTE dbo.IndexOptimize
    @Databases = 'MyDatabase',
    @FragmentationLow = NULL,
    @FragmentationMedium = 'INDEX_REORGANIZE,INDEX_REBUILD_ONLINE,INDEX_REBUILD_OFFLINE',
    @FragmentationHigh = 'INDEX_REBUILD_ONLINE,INDEX_REBUILD_OFFLINE',
    @FragmentationLevel1 = 5,
    @FragmentationLevel2 = 30,
    @UpdateStatistics = 'ALL',
    @OnlyModifiedStatistics = 'Y',
    @LogToTable = 'Y';
```

## Query Store and Index Recommendations

Query Store provides historical data for index optimization:

```sql
-- Identify queries with high resource usage
SELECT qsqt.query_sql_text,
       qsp.plan_id,
       qsrs.avg_cpu_time,
       qsrs.avg_logical_io_reads,
       qsrs.avg_duration,
       qsrs.execution_count
FROM sys.query_store_runtime_stats qsrs
JOIN sys.query_store_plan qsp ON qsrs.plan_id = qsp.plan_id
JOIN sys.query_store_query qsq ON qsp.query_id = qsq.query_id
JOIN sys.query_store_query_text qsqt ON qsq.query_text_id = qsqt.query_text_id
ORDER BY qsrs.avg_logical_io_reads DESC;

-- Find missing index recommendations
SELECT dmigs.avg_total_user_cost,
       dmigs.avg_user_impact,
       dmigs.last_user_seek,
       dmigs.statement,
       dmigs.equality_columns,
       dmigs.inequality_columns,
       dmigs.included_columns
FROM sys.dm_db_missing_index_groups mig
JOIN sys.dm_db_missing_index_group_stats mgis ON mig.group_handle = mgis.group_handle
JOIN sys.dm_db_missing_index_details mid ON mig.index_handle = mid.index_handle
ORDER BY mgis.avg_user_impact DESC;
```

**Missing index recommendations:**
- `avg_user_impact` — Percentage improvement estimated
- `avg_total_user_cost` — Average cost reduction
- `equality_columns` — Columns used in equality predicates
- `inequality_columns` — Columns used in range predicates
- `included_columns` — Columns that would make it covering

## Index Design Best Practices

### 1. Follow These Rules for Index Design

- **Clustered index on monotonically increasing column** — Sequential inserts reduce fragmentation
- **Nonclustered indexes for frequently queried columns** — Based on WHERE, JOIN, ORDER BY clauses
- **Include columns** — Add frequently selected columns to INCLUDE clause
- **Keep indexes narrow** — Fewer columns = smaller index = more efficient
- **Limit index count** — Too many indexes slow writes
- **Consider composite index order** — Most selective column first for equality predicates
- **Avoid duplicate indexes** — Redundant indexes waste space and slow writes
- **Use filtered indexes for selective data** — Reduces index size for common query patterns

### 2. Index Order for Composite Indexes

For a composite index on (A, B, C):

```sql
-- Optimal for queries:
-- SELECT * FROM T WHERE A = 1 AND B > 100 ORDER BY C
-- SELECT * FROM T WHERE A = 1 AND B BETWEEN 100 AND 200
-- SELECT * FROM T WHERE A = 1

CREATE NONCLUSTERED INDEX IX_T_A_B_C ON T (A, B, C);
```

**Index usage patterns:**
- Index on (A, B, C) supports: A alone, A+B, A+B+C
- Does NOT support: B alone, C alone, B+C
- Order of columns matters for seek efficiency

### 3. Index Maintenance Schedule

| Fragmentation Level | Action | Frequency |
|---------------------|--------|-----------|
| < 5% | No action | N/A |
| 5-30% | REORGANIZE | Weekly |
| > 30% | REBUILD | Weekly |
| > 50% | REBUILD + review design | Immediately |

## Version-Specific Features

### SQL Server 2017
- **Intelligent Query Processing** — Improved cardinality estimation
- **Batch mode execution** — Columnstore batch processing
- **Memory-optimized tables** — In-Memory OLTP with hash and range indexes
- **Graph indexes** — Automatic indexing for graph edges

### SQL Server 2019
- **Batch mode on rowstore** — Columnar-like processing for rowstore tables
- **Smart memory grant** — Reduced memory grant overestimation from statistics
- **Columnstore improvements** — Online rebuild, batch mode adaptive join
- **Automatic tuning** — Automatic index creation recommendations

### SQL Server 2022
- **Vectorized batch mode** — Enhanced columnar processing
- **Intelligent query processing improvements** — Better cardinality estimation
- **Batch mode memory grant feedback** — Runtime memory adjustment
- **Columnstore improvements** — Better delta rowgroup management

## Troubleshooting Index Issues

### Common Problems

1. **Missing index causing table scan**
   - Check execution plan for table scan operators
   - Review missing index DMV recommendations
   - Evaluate if index would benefit the query
   - Test index with actual query workload

2. **Fragmentation causing slow scans**
   - Check avg_fragmentation_in_percent
   - Reorganize if 5-30%
   - Rebuild if > 30%

3. **Stale statistics causing bad plans**
   - Check stats last_updated date
   - Update statistics with FULLSCAN
   - Consider AUTO_UPDATE_STATISTICS_ASYNC

4. **Index bloat from too many indexes**
   - Identify unused indexes via sys.dm_db_index_usage_stats
   - Drop indexes with only sys_lookup count
   - Consolidate overlapping indexes

5. **Nonclustered index key lookup bottleneck**
   - Add missing columns to INCLUDE clause
   - Consider covering index for the query
   - Evaluate if query can be rewritten

### Diagnostic Queries

```sql
-- Find unused indexes
SELECT OBJECT_NAME(i.object_id) AS table_name,
       i.name AS index_name,
       ius.user_seeks,
       ius.user_scans,
       ius.user_lookups,
       ius.user_updates
FROM sys.dm_db_index_usage_stats ius
JOIN sys.indexes i ON ius.object_id = i.object_id AND ius.index_id = i.index_id
WHERE ius.database_id = DB_ID()
  AND ius.user_seeks = 0
  AND ius.user_scans = 0
  AND ius.user_lookups = 0
  AND ius.object_id IN (SELECT object_id FROM sys.tables)
ORDER BY ius.user_updates DESC;

-- Find index overhead vs benefit
SELECT OBJECT_NAME(s.object_id) AS table_name,
       i.name AS index_name,
       s.user_seeks + s.user_scans + s.user_lookups AS reads,
       s.user_updates AS writes,
       CASE WHEN (s.user_seeks + s.user_scans + s.user_lookups) = 0
            THEN 'Unused'
            ELSE 'Used'
       END AS usage_status
FROM sys.dm_db_index_usage_stats s
JOIN sys.indexes i ON s.object_id = i.object_id AND s.index_id = i.index_id
WHERE s.database_id = DB_ID()
ORDER BY s.user_updates DESC, reads ASC;
```

## Conclusion

Indexes and statistics are the most impactful performance tuning lever in SQL Server. Effective index design considers query patterns, write overhead, and storage costs. Statistics must be kept current for the optimizer to choose optimal plans. Regular index maintenance prevents fragmentation and ensures efficient data access. For enterprise deployments, automate index and statistics maintenance using tools like Ola Hallengren's solution, and continuously monitor index usage through DMVs to identify optimization opportunities.