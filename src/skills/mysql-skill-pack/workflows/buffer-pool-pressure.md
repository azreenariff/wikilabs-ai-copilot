# WF-006: Buffer Pool Pressure Troubleshooting

## Scenario

MySQL server is experiencing high disk I/O, slow queries, and low buffer pool hit ratio, indicating buffer pool pressure.

## Observation

| Metric | Check With | Concerning Value |
|--------|-----------|-----------------|
| Buffer pool hit ratio | `Innodb_buffer_pool_reads / Innodb_buffer_pool_read_requests` | < 99% |
| Buffer pool writes | `SHOW GLOBAL STATUS LIKE 'Innodb_buffer_pool_pages_flushed'` | High rate |
| Free buffer pages | `SHOW ENGINE INNODB STATUS` | Very few free pages |
| Dirty pages ratio | `SHOW ENGINE INNODB STATUS` | High dirty page count |

## Interpretation

Buffer pool pressure indicates:
- Working set exceeds buffer pool size
- Random access patterns causing excessive disk reads
- Dirty pages not flushed efficiently
- Buffer pool too small for data volume
- Inefficient query patterns accessing too much data

## Possible Causes

| Rank | Cause | Confidence |
|------|-------|------------|
| 1 | Buffer pool undersized for working set | High |
| 2 | Full table scans scanning more data than available | High |
| 3 | Dirty page flush rate too slow | Medium |
| 4 | Checkpoint age too large | Medium |
| 5 | Buffer pool split across too many instances | Low |

## Evidence Required

1. **Buffer pool status**:
   ```sql
   -- ADVISORY: Run on source
   SHOW ENGINE INNODB STATUS\G
   ```
   Look for "BUFFER POOL AND MEMORY" section showing free lists, dirty pages, hit ratio.

2. **Buffer pool metrics**:
   ```sql
   -- ADVISORY: Run on source
   SELECT variable_name, variable_value
   FROM performance_schema.global_status
   WHERE variable_name IN (
     'Innodb_buffer_pool_read_requests',
     'Innodb_buffer_pool_reads',
     'Innodb_buffer_pool_bytes_data',
     'Innodb_buffer_pool_pages_total',
     'Innodb_buffer_pool_pages_free',
     'Innodb_buffer_pool_pages_dirty'
   );
   ```

3. **Buffer pool size configuration**:
   ```sql
   -- ADVISORY: Run on source
   SHOW VARIABLES LIKE 'innodb_buffer_pool_size';
   SHOW VARIABLES LIKE 'innodb_buffer_pool_instances';
   ```

4. **Server memory capacity**:
   ```bash
   free -h
   ```
   Ensure buffer pool doesn't exceed 70-80% of total system memory.

## Investigation Order

1. Calculate current buffer pool hit ratio
2. Check buffer pool size vs server memory
3. Identify queries doing full table scans
4. Check dirty page ratio
5. Check checkpoint age
6. Review working set size estimation
7. Check buffer pool fragmentation

## Recommended Actions

1. **Immediate**: If buffer pool is very undersized, increase it
   ```sql
   -- ADVISORY: This requires MySQL restart in MySQL 8.0 for some sizes
   SET PERSIST innodb_buffer_pool_size = 24G;  -- Adjust based on capacity
   ```

2. **Short-term**: Optimize queries to reduce buffer pool pressure
   - Eliminate full table scans
   - Add indexes to reduce scanned rows
   - Use covering indexes where possible
   - Implement pagination (LIMIT) for large result sets

3. **Medium-term**: Tune related settings
   ```sql
   -- ADVISORY: Test changes before applying
   SET PERSIST innodb_buffer_pool_instances = 8;
   SET PERSIST innodb_max_dirty_pages_pct = 70;
   SET PERSIST innodb_flush_method = 'O_DIRECT';
   ```

4. **Long-term**: Scale infrastructure
   - Add more memory to server
   - Use faster storage (SSD/NVMe)
   - Scale out with read replicas

## Expected Findings

- Hit ratio below 95% indicates serious pressure
- Most reads coming from a few query patterns
- Buffer pool size is small relative to data volume
- High dirty page count indicates write-heavy workload

## Possible Conclusions

- If hit ratio < 95% and buffer pool small: Increase buffer pool
- If full table scans dominate: Add indexes, optimize queries
- If high dirty pages: Improve flush settings or add I/O capacity

## Recommended Next Step

After tuning, monitor hit ratio and I/O metrics for improvement.

## Expected Outcome

- Buffer pool hit ratio exceeds 99%
- Disk I/O decreases
- Query latency improves

## Risk Warnings

- Setting buffer pool > 80% of system memory can cause OOM
- Buffer pool size changes may require restart for some sizes
- Increasing buffer pool is the most impactful tuning but also most resource-intensive
- Monitor system memory after changes
- Test with realistic data volumes

## Documentation References

- [MySQL InnoDB Buffer Pool](https://dev.mysql.com/doc/refman/8.0/en/innodb-buffer-pool.html)
- [MySQL InnoDB Parameters](https://dev.mysql.com/doc/refman/8.0/en/innodb-parameters.html)
- [MySQL Buffer Pool Statistics](https://dev.mysql.com/doc/refman/8.0/en/innodb-buffer-pool-stats.html)
- [MySQL innodb_flush_method](https://dev.mysql.com/doc/refman/8.0/en/innodb-parameters.html#sysvar_innodb_flush_method)