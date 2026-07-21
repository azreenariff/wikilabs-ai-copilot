# WF-004: Lock Contention / Deadlock Troubleshooting

## Scenario

MySQL is reporting deadlock errors (`ERROR 1213`) or lock wait timeout errors (`ERROR 1205`), causing transaction failures and application errors.

## Observation

| Metric | Check With | Concerning Value |
|--------|-----------|-----------------|
| Deadlocks | `SHOW GLOBAL STATUS LIKE 'Innodb_deadlocks'` | Increasing count |
| Lock wait timeouts | `SHOW GLOBAL STATUS LIKE 'Innodb_row_lock_timeouts'` | Any non-zero increase |
| Current waits | `SELECT * FROM performance_schema.data_lock_waits` | Any rows present |
| Process time | `SHOW PROCESSLIST` | Long-running queries |

## Interpretation

Lock contention and deadlocks are caused by:
- Concurrent transactions accessing tables/resources in different order
- Long-running transactions holding locks while others wait
- Missing indexes causing extended lock scope (table locks instead of row locks)
- Large transactions holding locks for too long
- Uncommitted transactions blocking others

## Possible Causes

| Rank | Cause | Confidence |
|------|-------|------------|
| 1 | Non-consistent lock ordering across transactions | High |
| 2 | Missing indexes causing extended lock scope | High |
| 3 | Long-running unoptimized transactions | Medium |
| 4 | Large bulk operations during peak traffic | Medium |
| 5 | Metadata lock contention from DDL | Low |

## Evidence Required

1. **InnoDB deadlock status**:
   ```sql
   -- ADVISORY: Run on source
   SHOW ENGINE INNODB STATUS\G
   ```
   Look for `LATEST DETECTED DEADLOCK` section showing conflicting transactions.

2. **Current lock waits**:
   ```sql
   -- ADVISORY: Run on source
   SELECT * FROM performance_schema.data_lock_waits;
   SELECT * FROM performance_schema.data_locks;
   ```

3. **Long-running transactions**:
   ```sql
   -- ADVISORY: Run on source
   SELECT * FROM performance_schema.events_transactions_current
   WHERE TIMER_WAIT > 1000000000000  -- > 1 second
   ORDER BY TIMER_WAIT DESC;
   ```

4. **Processlist for long queries**:
   ```sql
   -- ADVISORY: Run on source
   SHOW PROCESSLIST;
   ```

5. **Deadlock history**:
   ```sql
   -- ADVISORY: Run on source
   SELECT variable_name, variable_value 
   FROM performance_schema.global_status 
   WHERE variable_name IN ('Innodb_deadlocks', 'Innodb_row_lock_time', 'Innodb_row_lock_time_avg', 'Innodb_row_lock_time_max', 'Innodb_row_lock_waits');
   ```

## Investigation Order

1. Check `SHOW ENGINE INNODB STATUS` for latest deadlock details
2. Identify conflicting transactions and lock types
3. Find which tables and rows are involved
4. Analyze application transaction patterns
5. Check for missing indexes on JOIN/WHERE columns
6. Review transaction scope and duration
7. Check for long-running uncommitted transactions

## Recommended Actions

1. **Immediate**: Kill blocking transactions if safe
   ```sql
   -- ADVISORY: Review carefully before killing
   -- SELECT CONCAT('KILL ', id, ';') FROM information_schema.processlist WHERE time > 300;
   ```

2. **Short-term (application-level)**:
   - Ensure all transactions access tables in consistent order
   - Reduce transaction scope (do only necessary operations)
   - Add deadlock retry logic to application code
   - Use `SELECT ... FOR UPDATE` consistently

3. **Short-term (index-level)**:
   - Add indexes on JOIN and WHERE columns to reduce lock scope
   - Add composite indexes for compound WHERE conditions
   ```sql
   -- ADVISORY: Add indexes on staging first
   ALTER TABLE orders ADD INDEX idx_status_created (status, created_at);
   ```

4. **Medium-term**:
   - Split large transactions into smaller ones
   - Use optimistic locking where possible
   - Consider application-level queue for conflicting operations
   - Set `innodb_lock_wait_timeout` appropriately (50-120s)

## Expected Findings

- Deadlock section shows two transactions locking the same rows in different order
- Missing index on foreign key column causing gap/next-key locks
- Long-running transaction holding shared or exclusive locks
- Multiple transactions accessing parent/child tables in inconsistent order

## Possible Conclusions

- If same two tables always deadlocking: Application lock ordering issue
- If different tables each time: Index optimization needed
- If deadlocks during bulk operations: Schedule bulk operations off-peak
- If metadata lock contention: DDL operations blocking transactions

## Recommended Next Step

Fix application lock ordering and add retry logic. Verify deadlock count returns to zero.

## Expected Outcome

- Deadlock count stops increasing
- Transaction timeout errors eliminated
- Application throughput improves

## Risk Warnings

- Killing transactions may cause partial data state — ensure application has rollback
- Adding indexes has write performance cost
- `innodb_lock_wait_timeout` too short causes spurious errors; too long causes hung transactions
- Always monitor after changes with Performance Schema
- Deadlocks are recoverable — implement retry logic rather than trying to eliminate entirely

## Documentation References

- [MySQL Deadlocks](https://dev.mysql.com/doc/refman/8.0/en/innodb-deadlocks.html)
- [MySQL InnoDB Locking](https://dev.mysql.com/doc/refman/8.0/en/innodb-locking.html)
- [MySQL Lock Wait Timeout](https://dev.mysql.com/doc/refman/8.0/en/server-system-variables.html#sysvar_innodb_lock_wait_timeout)
- [MySQL Data Locks Performance Schema](https://dev.mysql.com/doc/refman/8.0/en/performance-schema-data-locks-tables.html)