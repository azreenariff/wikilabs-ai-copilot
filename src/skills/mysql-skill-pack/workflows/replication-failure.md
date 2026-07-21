# WF-005: Replication Failure Troubleshooting

## Scenario

MySQL replication has stopped on a replica. The SQL thread has encountered an error and is no longer applying events from the binary log.

## Observation

| Metric | Check With | Concerning Value |
|--------|-----------|-----------------|
| Slave_IO_Running | `SHOW REPLICA STATUS` | `No` |
| Slave_SQL_Running | `SHOW REPLICA STATUS` | `No` |
| Last_Error | `SHOW REPLICA STATUS` | Non-empty error message |
| Seconds_Behind_Source | `SHOW REPLICA STATUS` | `NULL` (when SQL thread stopped) |

## Interpretation

Replication failures are caused by:
- Data inconsistency between source and replica
- DDL operations that failed on replica but succeeded on source
- Row format mismatches
- Missing tables or databases on replica
- Large transactions causing timeouts
- GTID position mismatches

## Possible Causes

| Rank | Cause | Confidence |
|------|-------|------------|
| 1 | DDL/DML inconsistency | High |
| 2 | GTID position mismatch | High |
| 3 | Large transaction timeout | Medium |
| 4 | Schema mismatch | Medium |
| 5 | Network issues between source/replica | Low |

## Evidence Required

1. **Replica status**:
   ```sql
   -- ADVISORY: Run on replica
   SHOW REPLICA STATUS\G
   ```
   Note `Last_Error`, `Last_Error_State`, and positions.

2. **Source binlog position**:
   ```sql
   -- ADVISORY: Run on source
   SHOW BINARY LOGS;
   SHOW MASTER STATUS;
   ```

3. **GTID state**:
   ```sql
   -- ADVISORY: Run on both source and replica
   SHOW GLOBAL VARIABLES LIKE 'gtid_executed';
   SHOW GLOBAL VARIABLES LIKE 'gtid_purged';
   ```

4. **Replica relay log**:
   ```sql
   -- ADVISORY: Run on replica
   SHOW RELAYLOG EVENTS;
   ```

## Investigation Order

1. Check `SHOW REPLICA STATUS` for error message and position
2. Verify both IO and SQL thread status
3. Compare GTID sets between source and replica
4. Identify the failing event from relay log
5. Determine if skipping is safe or if rebuild is needed

## Recommended Actions

1. **Skip error (if safe and non-critical)**:
   ```sql
   -- ADVISORY: Only skip if operation is idempotent or non-critical
   STOP REPLICA;
   SET GLOBAL sql_slave_skip_counter = 1;
   START REPLICA;
   ```

2. **Fix GTID mismatch (preferred for GTID-enabled replication)**:
   ```sql
   -- ADVISORY: Ensure data consistency before reset
   STOP REPLICA;
   RESET SLAVE ALL;
   CHANGE REPLICATION SOURCE TO
     SOURCE_HOST='source_host',
     SOURCE_USER='repl_user',
     SOURCE_AUTO_POSITION = 1;
   START REPLICA;
   ```

3. **Rebuild replica (if data inconsistency suspected)**:
   - Take consistent backup from source (using `mysqldump --single-transaction` or XtraBackup)
   - Restore on replica
   - Re-enable replication

4. **Preventive**: Set `binlog_format=ROW` to minimize statement-based replication issues.

## Expected Findings

- `Last_Error` shows specific table or statement that failed
- GTID set on replica is missing the GTID of the failed event
- Relay log shows the event causing the failure
- Source and replica diverged after a specific DDL operation

## Possible Conclusions

- If single DDL failed: Skip the error or rebuild replica
- If multiple failures: Rebuild replica from backup
- If GTID mismatch: Reset with `SOURCE_AUTO_POSITION = 1`
- If network-related: Fix network and restart IO thread

## Recommended Next Step

After recovery, verify data consistency with `pt-table-checksum`.

## Expected Outcome

- `Slave_IO_Running: Yes` and `Slave_SQL_Running: Yes`
- `Seconds_Behind_Source` decreases to 0
- `Last_Error` is empty

## Risk Warnings

- Skipping replication errors can cause data inconsistency
- Always verify data consistency after recovery
- Rebuilding a replica is disruptive — plan during maintenance window
- GTID-based recovery is safer than position-based
- Test recovery procedures regularly

## Documentation References

- [MySQL Replication Errors](https://dev.mysql.com/doc/refman/8.0/en/replication-solutions-replication-errors.html)
- [GTID Replication](https://dev.mysql.com/doc/refman/8.0/en/replication-gtids.html)
- [pt-table-checksum](https://www.percona.com/doc/percona-toolkit/LATEST/pt-table-checksum.html)
- [MySQL CHANGE REPLICATION SOURCE](https://dev.mysql.com/doc/refman/8.0/en/change-replication-source.html)