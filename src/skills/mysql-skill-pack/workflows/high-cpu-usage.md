# WF-008: High CPU Usage Troubleshooting

## Scenario

MySQL server is consuming excessive CPU, causing slow query response times and overall performance degradation.

## Observation

| Metric | Check With | Concerning Value |
|--------|-----------|-----------------|
| MySQL CPU usage | `top` / `htop` | > 80% consistently |
| Threads running | `SHOW GLOBAL STATUS LIKE 'Threads_running'` | > 20 consistently |
| User vs system mode | `mpstat 1` | High user mode = query processing |
| Context switches | `vmstat 1` | High rate indicates contention |

## Interpretation

High CPU on MySQL is caused by:
- Complex queries doing excessive computation
- Too many concurrent queries
- Missing indexes causing full table scans
- Sorting operations (ORDER BY, GROUP BY, DISTINCT)
- Hash joins or temporary tables
- Row-based replication applying many rows
- High connection count creating thread overhead

## Possible Causes

| Rank | Cause | Confidence |
|------|-------|------------|
| 1 | Complex unoptimized queries | High |
| 2 | Too many concurrent threads | High |
| 3 | Missing indexes causing table scans | High |
| 4 | Large sort operations | Medium |
| 5 | Row-based replication overhead | Medium |
| 6 | Thread creation overhead | Low |

## Evidence Required

1. **Top queries by CPU time**:
   ```sql
   -- ADVISORY: Run on source
   SELECT * FROM sys.statements_with_runtimes_in_95th_percentile
   ORDER BY avg_runtime DESC
   LIMIT 10;
   ```

2. **Active queries**:
   ```sql
   -- ADVISORY: Run on source
   SHOW PROCESSLIST;
   ```

3. **Thread status**:
   ```sql
   -- ADVISORY: Run on source
   SHOW GLOBAL STATUS LIKE 'Threads_running';
   SHOW GLOBAL STATUS LIKE 'Threads_created';
   SHOW VARIABLES LIKE 'thread_cache_size';
   ```

4. **Top consumers**:
   ```sql
   -- ADVISORY: Run on source
   SELECT db, COUNT(*) as thread_count FROM information_schema.processlist
   GROUP BY db ORDER BY thread_count DESC;
   ```

## Investigation Order

1. Check current thread count and running queries
2. Identify top queries by CPU time using Performance Schema
3. Run EXPLAIN on identified queries
4. Check for full table scans
5. Review query patterns (sorting, joins, aggregation)
6. Check connection count and thread cache
7. Review replication settings if applicable

## Recommended Actions

1. **Immediate**: Kill runaway queries
   ```sql
   -- ADVISORY: Review before killing
   -- SELECT CONCAT('KILL ', id, ';') FROM information_schema.processlist
   -- WHERE time > 300 AND command != 'Sleep';
   ```

2. **Short-term**: Optimize identified queries
   - Add appropriate indexes
   - Reduce result set size with LIMIT
   - Simplify complex JOINs
   - Use EXPLAIN to validate improvements

3. **Medium-term**: Tune server settings
   ```sql
   -- ADVISORY: Test on staging
   SET PERSIST thread_cache_size = 50;
   SET PERSIST max_connections = 500;  -- Adjust based on capacity
   SET PERSIST innodb_thread_concurrency = 0;  -- 0 = unlimited, adjust if needed
   ```

4. **Long-term**: Architecture improvements
   - Scale horizontally with read replicas
   - Implement query caching at application level
   - Use connection pooling
   - Consider read/write splitting

## Expected Findings

- A few queries consuming disproportionate CPU
- High `Threads_running` count correlating with peak traffic
- Full table scans in EXPLAIN output for top queries
- Large sort operations or hash joins in query plans

## Possible Conclusions

- If few queries dominate: Optimize those queries first
- If many queries running: Scale read capacity or implement connection pooling
- If high system mode: Check for disk I/O pressure
- If high user mode: Query processing is the bottleneck

## Recommended Next Step

After optimization, monitor CPU usage and query performance.

## Expected Outcome

- CPU usage drops to normal range
- Query response times improve
- `Threads_running` stays below threshold

## Risk Warnings

- Killing queries may disrupt in-flight operations
- Increasing `max_connections` increases memory usage
- `innodb_thread_concurrency` changes may have unexpected effects
- Always test changes on staging first
- Monitor after changes to verify improvement

## Documentation References

- [MySQL Performance Schema](https://dev.mysql.com/doc/refman/8.0/en/performance-schema.html)
- [MySQL Thread Management](https://dev.mysql.com/doc/refman/8.0/en/server-threads.html)
- [MySQL Query Optimization](https://dev.mysql.com/doc/refman/8.0/en/optimization.html)
- [MySQL sys Schema](https://dev.mysql.com/doc/refman/8.0/en/sys-schema.html)