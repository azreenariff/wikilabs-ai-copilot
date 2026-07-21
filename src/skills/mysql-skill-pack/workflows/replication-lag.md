# WF-002: Replication Lag Troubleshooting

## Scenario

A MySQL replica is falling behind the source (primary) server, as indicated by increasing `Seconds_Behind_Source` on the replica.

## Observation

| Metric | Check With | Concerning Value |
|--------|-----------|-----------------|
| Seconds_Behind_Source | `SHOW REPLICA STATUS` | > 60s for short-term, > 300s for long-term |
| Slave_IO_Running | `SHOW REPLICA STATUS` | Must be `Yes` |
| Slave_SQL_Running | `SHOW REPLICA STATUS` | Must be `Yes` |
| Relay_Log_Space | `SHOW REPLICA STATUS` | Growing indicates I/O catching up |

## Interpretation

Replication lag can be caused by:
- Source server generating too many writes for the replica to keep up
- Replica hardware insufficient for the workload
- Large transactions on the source
- Network latency between source and replica
- Replication configuration issues (row vs statement format)
- Replica running read queries on top of replication

## Possible Causes

| Rank | Cause | Confidence |
|------|-------|------------|
| 1 | Replica hardware underpowered for write workload | High |
| 2 | Large/complex transactions on source | High |
| 3 | Network latency between source and replica | Medium |
| 4 | Replica serving read queries slowing SQL thread | Medium |
| 5 | STATEMENT-based replication format overhead | Low-Medium |
| 6 | InnoDB flush settings on replica | Low |

## Evidence Required

1. **Replica status**:
   ```sql
   -- ADVISORY: Run on replica
   SHOW REPLICA STATUS\G
   ```
   Check both IO and SQL thread status, last error, and seconds behind.

2. **Source write volume**:
   ```sql
   -- ADVISORY: Run on source
   SHOW GLOBAL STATUS LIKE 'Innodb_rows%';
   SHOW GLOBAL STATUS LIKE 'Com_insert%';
   SHOW GLOBAL STATUS LIKE 'Com_update%';
   SHOW GLOBAL STATUS LIKE 'Com_delete%';
   ```

3. **Replica query analysis**:
   ```sql
   -- ADVISORY: Run on replica
   SELECT * FROM performance_schema.events_statements_history
   WHERE TIMER_WAIT > 1000000000000
   ORDER BY TIMER_WAIT DESC
   LIMIT 10;
   ```

4. **Binlog size and format**:
   ```sql
   -- ADVISORY: Run on source
   SHOW VARIABLES LIKE 'binlog_format';
   SHOW VARIABLES LIKE 'binlog_row_image';
   SHOW BINARY LOGS;
   ```

5. **Replica hardware resources**:
   - CPU utilization
   - Disk I/O wait
   - Memory usage
   - Network throughput

## Investigation Order

1. Verify both IO and SQL threads are running
2. Check if lag is increasing or stable
3. Measure source write throughput
4. Identify replica read workload impact
5. Check binlog format and row image settings
6. Review replica hardware resources
7. Check for large transactions in relay logs

## Recommended Actions

1. **Immediate**: Stop read queries from replica if they're interfering
   ```sql
   -- ADVISORY: Pause read workload temporarily
   -- Stop application queries to replica while catching up
   ```

2. **Short-term**: Optimize replica hardware
   - Increase CPU allocation
   - Use faster disks (SSD/NVMe) for replica
   - Increase memory for buffer pool
   - Use `innodb_flush_log_at_trx_commit=2` on replica

3. **Medium-term**: Tune replication settings
   ```sql
   -- ADVISORY: On replica
   SET GLOBAL innodb_flush_log_at_trx_commit = 2;
   SET GLOBAL sync_binlog = 0;
   SET PERSIST innodb_flush_log_at_trx_commit = 2;
   SET PERSIST sync_binlog = 0;
   ```

4. **Long-term**: Scale replication architecture
   - Add more replicas for read distribution
   - Use GTID-based multi-source replication
   - Consider MySQL Shell InnoDB Cluster for automatic management
   - Implement read/write splitting at application level

## Expected Findings

- SQL thread behind but IO thread caught up — replica is applying slowly
- Source has high write throughput during lag period
- Replica serving significant read workload
- `binlog_format=STATEMENT` causing extra replication overhead
- Large transaction in relay log causing SQL thread backlog

## Possible Conclusions

- If IO thread caught up but SQL thread behind: Replica I/O is fine, SQL apply is bottleneck
- If both threads lagging: Network or source generating more than replica can handle
- If lag increases during specific time windows: Application workload pattern is the cause
- If STATEMENT format: Switch to ROW for better performance

## Recommended Next Step

After catching up, tune replica configuration and implement monitoring for replication lag.

## Expected Outcome

- Replication lag stabilizes below acceptable threshold
- `Seconds_Behind_Source` stays under 60s (or defined SLO)
- No more replication lag alerts

## Risk Warnings

- Stopping read queries on replica impacts read availability
- `innodb_flush_log_at_trx_commit=2` on replica reduces durability slightly
- Increasing replica buffer pool may cause OOM if memory limited
- Always verify data consistency with `pt-table-checksum` after recovery
- GTID auto-positioning is safer than manual position-based recovery

## Documentation References

- [MySQL Replication](https://dev.mysql.com/doc/refman/8.0/en/replication.html)
- [MySQL Replication Configuration](https://dev.mysql.com/doc/refman/8.0/en/replication-configuration.html)
- [GTID Replication](https://dev.mysql.com/doc/refman/8.0/en/replication-gtids.html)
- [pt-table-checksum](https://www.percona.com/doc/percona-toolkit/LATEST/pt-table-checksum.html)