# EDB PostgreSQL Performance Optimization

## Overview

EDB PostgreSQL performance optimization covers query optimization, indexing, configuration tuning, and monitoring.

## Query Optimization

### EXPLAIN ANALYZE

```sql
-- Analyze query execution plan
EXPLAIN ANALYZE SELECT * FROM orders WHERE customer_id = 12345 AND status = 'pending';

-- Check for sequential scans
EXPLAIN (ANALYZE, BUFFERS) SELECT * FROM large_table WHERE id = 1;
```

### Query Patterns

| Pattern | Optimization | Expected Impact |
|---------|-------------|-----------------|
| Sequential scan on large table | Add index | High |
| Nested loop joins | Use hash joins | Medium |
| Subqueries in SELECT | Convert to JOIN | Medium |
| Missing WHERE clause | Add filter | High |

## Index Management

### Index Types

| Type | Use Case | Example |
|------|----------|---------|
| **B-tree** | Default, equality/range | CREATE INDEX idx ON tbl (col); |
| **Hash** | Equality only | CREATE INDEX idx ON tbl USING hash (col); |
| **GiST** | Geometric, full-text | CREATE INDEX idx ON tbl USING gist (geom); |
| **GIN** | Arrays, full-text | CREATE INDEX idx ON tbl USING gin (tags); |
| **BRIN** | Large sequential data | CREATE INDEX idx ON tbl USING brin (ts); |

### Index Best Practices

1. Index frequently queried columns
2. Cover WHERE, JOIN, ORDER BY clauses
3. Monitor index usage (pg_stat_user_indexes)
4. Remove unused indexes
5. Consider partial indexes for filtered queries

## Configuration Tuning

### Memory Settings

| Parameter | Recommended | Description |
|-----------|-------------|-------------|
| `shared_buffers` | 25% of RAM | Shared memory buffer |
| `work_mem` | 4-16MB | Sort/hash operation memory |
| `effective_cache_size` | 75% of RAM | OS cache estimate |
| `maintenance_work_mem` | 128-512MB | VACUUM/index creation memory |

### WAL Settings

| Parameter | Recommended | Description |
|-----------|-------------|-------------|
| `wal_buffers` | 16MB | WAL write buffer |
| `commit_delay` | 0-1000 | Delay commits |
| `checkpoint_completion_target` | 0.9 | Checkpoint spread |
| `max_wal_size` | 1-2GB | Maximum WAL size |

## Monitoring

### Performance Views

```sql
-- Slow queries
SELECT query, calls, mean_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC LIMIT 10;

-- Table statistics
SELECT relname, seq_scan, seq_tup_read, idx_scan, idx_tup_fetch
FROM pg_stat_user_tables ORDER BY seq_scan DESC;

-- Index statistics
SELECT schemaname, tablename, indexname, idx_scan, idx_tup_read
FROM pg_stat_user_indexes ORDER BY idx_scan ASC;

-- Connection usage
SELECT count(*) FROM pg_stat_activity;
```

## References

- EDB PostgreSQL Performance: https://www.enterprisedb.com/docs/
- PostgreSQL Performance: https://www.postgresql.org/docs/current/performance.html
- PostgreSQL Indexes: https://www.postgresql.org/docs/current/indexes.html