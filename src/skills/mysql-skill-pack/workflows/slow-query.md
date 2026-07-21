# WF-003: Slow Query Performance Troubleshooting

## Scenario

A specific query or set of queries is performing poorly, causing application slowness and user complaints.

## Observation

| Metric | Check With | Concerning Value |
|--------|-----------|-----------------|
| Query execution time | Slow query log | > threshold (e.g. 2s) |
| Rows examined | `EXPLAIN` | Much higher than rows returned |
| Using filesort | `EXPLAIN` output | Present for large result sets |
| Using temporary | `EXPLAIN` output | Present for large result sets |
| Full table scans | `EXPLAIN` | `type=ALL` |

## Interpretation

Slow queries are typically caused by:
- Missing or suboptimal indexes
- Inefficient JOIN patterns
- Unoptimized query structure
- Large result sets without pagination
- Outdated table statistics
- Lock contention on tables

## Possible Causes

| Rank | Cause | Confidence |
|------|-------|------------|
| 1 | Missing or suboptimal indexes | High |
| 2 | Inefficient JOIN patterns | High |
| 3 | Outdated table statistics | Medium |
| 4 | Query structure issues | Medium |
| 5 | Data volume growth | Low-Medium |
| 6 | Lock contention | Low |

## Evidence Required

1. **Identify the slow query**:
   ```sql
   -- ADVISORY: Run on source
   -- Check slow query log configuration
   SHOW VARIABLES LIKE 'slow_query_log%';
   SHOW VARIABLES LIKE 'long_query_time';
   ```

2. **Run EXPLAIN on the query**:
   ```sql
   -- ADVISORY: Run on source
   EXPLAIN FORMAT=JSON
   <slow_query>;
   ```
   Look for:
   - `type` column (const > eq_ref > ref > range > index > ALL)
   - `rows` examined vs rows returned
   - `Extra` column for "Using filesort", "Using temporary", "Using where"
   - `key` column showing which indexes are used

3. **Check table statistics**:
   ```sql
   -- ADVISORY: Run on source
   SHOW TABLE STATUS LIKE 'table_name'\G
   ```

4. **Check existing indexes**:
   ```sql
   -- ADVISORY: Run on source
   SHOW INDEX FROM table_name;
   ```

5. **Check buffer pool efficiency**:
   ```sql
   -- ADVISORY: Run on source
   SELECT variable_name, variable_value 
   FROM performance_schema.global_status 
   WHERE variable_name IN (
     'Innodb_buffer_pool_read_requests',
     'Innodb_buffer_pool_reads',
     'Innodb_data_read',
     'Innodb_data_read_requests'
   );
   ```

## Investigation Order

1. Identify the slowest queries from slow query log
2. Run EXPLAIN on identified queries
3. Analyze index usage from EXPLAIN output
4. Check for full table scans (type=ALL)
5. Check for missing indexes on WHERE, JOIN, ORDER BY, GROUP BY columns
6. Analyze query structure for optimization opportunities
7. Check table statistics freshness
8. Verify buffer pool configuration

## Recommended Actions

1. **Immediate (query-level)**:
   - Add `LIMIT` clause to limit result set size
   - Optimize the query structure (remove unnecessary JOINs, subqueries)
   ```sql
   -- ADVISORY: Test query optimization on staging
   ```

2. **Short-term (index-level)**:
   - Add composite indexes matching WHERE + ORDER BY patterns
   - Remove duplicate or unused indexes
   - Add covering indexes for frequently accessed queries
   ```sql
   -- ADVISORY: Add index on staging first
   ALTER TABLE table_name ADD INDEX idx_new (col1, col2, col3);
   ```

3. **Medium-term (statistics)**:
   - Run `ANALYZE TABLE` to update statistics
   - Monitor for statistics staleness
   ```sql
   -- ADVISORY: Run during low-traffic period
   ANALYZE TABLE table_name;
   ```

4. **Long-term (architecture)**:
   - Implement query caching at application level
   - Add read replicas for heavy read workloads
   - Consider partitioning for large tables
   - Implement materialized views for complex aggregations

## Expected Findings

- EXPLAIN shows `type=ALL` (full table scan) on large tables
- Missing index on WHERE or JOIN columns
- Filesort present when ORDER BY columns not indexed
- Outdated table statistics causing suboptimal plan
- Large result sets without LIMIT clause

## Possible Conclusions

- If full table scans: Add appropriate indexes for WHERE/JOIN columns
- If filesort: Add indexes covering ORDER BY columns
- If poor join performance: Check index on JOIN columns
- If statistics outdated: Run ANALYZE TABLE, schedule regular analysis

## Recommended Next Step

After optimization, verify improvements with EXPLAIN and performance testing.

## Expected Outcome

- Query execution time drops to acceptable levels
- EXPLAIN shows efficient index usage
- No full table scans on identified tables
- Buffer pool hit ratio improves

## Risk Warnings

- Adding indexes has write performance overhead
- Index additions on large tables may require maintenance window
- Query rewrites may change application behavior
- ANALYZE TABLE can be expensive on large tables
- Always test changes on staging before production
- Monitor slow query log after changes to confirm improvement

## Documentation References

- [MySQL EXPLAIN](https://dev.mysql.com/doc/refman/8.0/en/explain.html)
- [MySQL Index Optimization](https://dev.mysql.com/doc/refman/8.0/en/optimizing-indexes.html)
- [MySQL Query Optimization](https://dev.mysql.com/doc/refman/8.0/en/optimization.html)
- [MySQL Slow Query Log](https://dev.mysql.com/doc/refman/8.0/en/slow-query-log.html)
- [MySQL Table Statistics](https://dev.mysql.com/doc/refman/8.0/en/table-statistics.html)