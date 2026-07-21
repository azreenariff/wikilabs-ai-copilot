# MySQL Worked Examples

## Example 1: Production Deadlock Investigation

### Scenario

A production e-commerce application reports intermittent `ERROR 1213 (40001): Deadlock found when trying to get lock` errors. The errors occur during peak traffic hours and affect the order processing module.

### Initial Observations

- Deadlock errors appearing in application logs during 9-11 AM
- 2-3 deadlocks per minute during peak
- Orders stuck in "processing" state (deadlocked transaction rolled back)
- No user-reported data corruption
- Database is MySQL 8.0 with single-primary InnoDB Cluster

### Evidence Collection

**Step 1: Check recent deadlock information**
```
# Advised command (do not execute in production without approval)
SHOW ENGINE INNODB STATUS\G
```
Look for the `LATEST DETECTED DEADLOCK` section. Identify the two conflicting transactions.

**Step 2: Check lock wait metrics**
```
# Advised command (do not execute in production without approval)
SELECT * FROM performance_schema.data_lock_waits;
SELECT * FROM performance_schema.data_locks;
```
Check if deadlocks are currently occurring or if this is historical.

**Step 3: Check the affected tables**
```
# Advised command (do not execute in production without approval)
SHOW CREATE TABLE orders\G
SHOW CREATE TABLE order_items\G
SHOW INDEX FROM orders\G
SHOW INDEX FROM order_items\G
```
Verify index structure on both tables, especially foreign keys.

### Diagnosis

**Finding 1**: The deadlock occurs between two application threads:
- Transaction A: Updates `orders.status` and `orders.total` (single row)
- Transaction B: Deletes an `order_items` row for the same order

**Finding 2**: The `orders` table has a foreign key on `order_items.order_id`, causing Transaction B to acquire an X lock on `orders` (parent) while Transaction A already holds locks on `order_items`.

**Finding 3**: Both transactions modify the same order, creating a circular lock dependency.

**Root Cause**: Application processes order payments and item updates in different lock order depending on timing. When payment processing and item deletion overlap on the same order, deadlock occurs.

### Resolution

**Recommended Actions** (in order of implementation):

1. **Immediate (low risk)**: Ensure application always accesses tables in the same lock order — `orders` before `order_items` for any transaction touching both.

2. **Short-term (medium risk)**: Add composite index `(order_id, status)` on `orders` table to reduce lock scope:
   ```sql
   -- ADVISORY: Test this on staging first
   ALTER TABLE orders ADD INDEX idx_order_status (order_id, status);
   ```

3. **Long-term (medium risk)**: Refactor the payment processing to use a single transaction that covers both order status update and item processing. This reduces the window for concurrent access conflicts.

4. **Application-level**: Implement retry logic for deadlock errors (deadlocks are recoverable by retrying the failed transaction).

### Verification

- Monitor `SHOW ENGINE INNODB STATUS` for `LATEST DETECTED DEADLOCK` — should stop appearing
- Monitor application error logs for deadlock errors — should return to zero
- Run load test simulating concurrent payment and item operations

### Confidence Assessment

| Factor | Confidence |
|--------|-----------|
| Deadlock evidence (SHOW ENGINE INNODB STATUS) | 0.95 |
| Lock order analysis | 0.90 |
| Root cause identification | 0.85 |
| Resolution effectiveness | 0.80 |

### Risk Warnings

- Modifying indexes on production tables requires careful planning
- Application code changes require staging environment validation
- Always test resolution in non-production before applying to production
- Monitor replication lag after DDL changes on primary

### Lessons Learned

- Consistent lock ordering is the simplest deadlock prevention strategy
- Application-level retry for deadlock errors is essential for robust systems
- Foreign key relationships affect lock scope — understand them when designing transactions
- `SHOW ENGINE INNODB STATUS` provides the most detailed deadlock information

---

## Example 2: Replication Failure Recovery

### Scenario

A MySQL 8.0 master-slave replication setup shows replication errors on one replica. The replica was performing well but started showing errors after a maintenance window where the source server was restarted.

### Initial Observations

- `Slave_SQL_Running: No` on replica
- `Last_Error: Error 'Table 'mydb.temp_import' doesn't exist' on query`
- `Seconds_Behind_Source: NULL`
- Source server had a planned restart during maintenance window
- No errors reported on the source

### Evidence Collection

**Step 1: Check replica status**
```
# Advised command (do not execute in production without approval)
SHOW REPLICA STATUS\G
```
Note the `Last_Error` message and the position gap.

**Step 2: Check source binary logs**
```
# Advised command (do not execute in production without approval)
SHOW BINARY LOGS;
SHOW MASTER STATUS;
```
Verify the source's current binary log position.

**Step 3: Check replica relay log**
```
# Advised command (do not execute in production without approval)
SHOW RELAYLOG EVENTS;
```
Identify where the replica stopped.

**Step 4: Check GTID state**
```
# Advised command (do not execute in production without approval)
SHOW GLOBAL VARIABLES LIKE 'gtid_executed';
SHOW GLOBAL VARIABLES LIKE 'gtid_purged';
```
Verify GTID consistency between source and replica.

### Diagnosis

**Finding 1**: The replica stopped when executing a query that created a temporary table `temp_import` on the source. This table existed only during the DDL operation and was dropped after.

**Finding 2**: The source was restarted during maintenance, causing the binary log position to be lost from the replica's perspective. The replica is trying to replay a binlog event that references a table creation that no longer exists on the replica.

**Finding 3**: Since the source was restarted, the replica's I/O thread may have lost its connection to the correct binlog position.

**Root Cause**: The DDL that created and dropped `temp_import` was logged in the binary log. The replica failed to apply it because the table was created and dropped within a short window, or the source restart caused the replica to lose its place in the binlog stream.

### Resolution

**Recommended Actions** (in order of implementation):

1. **Skip the error (if safe)**: If the error is about a non-critical temporary table:
   ```sql
   -- ADVISORY: Only skip if the operation is idempotent or non-critical
   STOP REPLICA;
   SET GLOBAL sql_slave_skip_counter = 1;
   START REPLICA;
   ```

2. **Reset replication with GTID (if skipping is not safe)**:
   ```sql
   -- ADVISORY: Requires data consistency check first
   STOP REPLICA;
   RESET SLAVE ALL;
   CHANGE REPLICATION SOURCE TO
     SOURCE_HOST='source_host',
     SOURCE_USER='repl_user',
     SOURCE_AUTO_POSITION = 1;
   START REPLICA;
   ```

3. **Rebuild replica (most thorough)**: If data inconsistency is suspected:
   - Take a consistent backup from the source
   - Restore on the replica
   - Restart replication with GTID auto-positioning

4. **Preventive measure**: Set `binlog_format=ROW` to avoid statement-based replication issues with temporary tables.

### Verification

- `SHOW REPLICA STATUS\G` should show `Slave_IO_Running: Yes` and `Slave_SQL_Running: Yes`
- `Seconds_Behind_Source` should decrease to 0
- No new errors in `Last_Error` field
- Run `pt-table-checksum` to verify data consistency

### Confidence Assessment

| Factor | Confidence |
|--------|-----------|
| Error diagnosis | 0.90 |
| Root cause identification | 0.85 |
| Resolution effectiveness | 0.80 |
| Data consistency after recovery | 0.75 |

### Risk Warnings

- Skipping replication errors can cause data inconsistency
- Always verify data consistency after any replication recovery
- Consider using `pt-table-checksum` to validate replica data
- Rebuilding a replica is disruptive — plan during maintenance window
- GTID-based recovery is safer than position-based

### Lessons Learned

- Temporary table operations in replication require careful handling
- ROW format is more reliable than STATEMENT for replication
- Always maintain binary logs for at least 2x the maximum replication lag period
- Test failover procedures regularly to verify recovery works

---

## Example 3: Slow Query Optimization

### Scenario

A customer portal reports that the "my orders" page is extremely slow, taking 15-30 seconds to load during business hours. The page queries order history for a specific customer with join to products table.

### Initial Observations

- Page load time: 15-30 seconds (expected: < 2 seconds)
- Specific to the customer portal "my orders" page
- Slow query detected in slow query log
- No errors in MySQL error log
- Database has adequate buffer pool (16GB on 32GB server)

### Evidence Collection

**Step 1: Identify the slow query**
```
# Advised command (do not execute in production without approval)
-- Check slow query log
-- Look for the customer portal query pattern
```

**Step 2: Run EXPLAIN on the query**
```
# Advised command (do not execute in production without approval)
EXPLAIN FORMAT=JSON
SELECT c.name, c.email, o.id AS order_id, o.status, o.total, o.created_at,
       p.name AS product_name, p.sku
FROM customers c
JOIN orders o ON c.id = o.customer_id
JOIN order_items oi ON o.id = oi.order_id
JOIN products p ON oi.product_id = p.id
WHERE c.id = 12345
ORDER BY o.created_at DESC;
```

**Step 3: Check table sizes and statistics**
```
# Advised command (do not execute in production without approval)
SELECT table_name, row_format, table_rows, avg_row_length, data_length, index_length
FROM information_schema.tables
WHERE table_schema = 'mydb'
  AND table_name IN ('customers', 'orders', 'order_items', 'products')
ORDER BY table_name;
```

**Step 4: Check existing indexes**
```
# Advised command (do not execute in production without approval)
SHOW INDEX FROM orders\G
SHOW INDEX FROM order_items\G
```

### Diagnosis

**EXPLAIN Output Analysis**:
- `customers` table: `type=const`, `key=PRIMARY` — OK, single row lookup
- `orders` table: `type=ref`, `key=idx_customer`, `rows=2500` — Uses index but scans 2500 rows
- `order_items` table: `type=ref`, `key=idx_order_id`, `rows=15` — OK
- `products` table: `type=ref`, `key=PRIMARY`, `rows=1` — OK
- `Extra`: "Using filesort" — ORDER BY requires filesort
- Total estimated rows examined: ~37,500 for a result set of 200

**Finding 1**: The `orders.customer_id` index exists but covers only 2500 rows per customer. This is acceptable.

**Finding 2**: The `ORDER BY o.created_at DESC` causes a filesort because there is no composite index `(customer_id, created_at DESC)`.

**Finding 3**: The JOIN with `order_items` creates a fan-out effect — each order has 15 items on average, resulting in ~37,500 rows examined for 200 result rows.

**Finding 4**: The query returns product names for each item, but the page only shows order summaries with a link to view items. The product JOIN is unnecessary for this page.

**Root Cause**: Two issues combine to create poor performance:
1. Missing composite index on `(customer_id, created_at)` causing filesort
2. Unnecessary JOIN with `order_items` and `products` tables when the page only needs order summaries

### Resolution

**Recommended Actions**:

1. **Immediate (low risk)**: Remove the unnecessary JOINs from the query:
   ```sql
   -- ADVISORY: Change application query
   SELECT c.name, c.email, o.id AS order_id, o.status, o.total, o.created_at
   FROM customers c
   JOIN orders o ON c.id = o.customer_id
   WHERE c.id = 12345
   ORDER BY o.created_at DESC
   LIMIT 50;
   ```

2. **Short-term (low risk)**: Add composite index for the ORDER BY:
   ```sql
   -- ADVISORY: Add index for order retrieval pattern
   ALTER TABLE orders ADD INDEX idx_customer_created (customer_id, created_at DESC);
   ```

3. **Verify improvement**:
   - Page load time should drop from 15-30s to < 1s
   - EXPLAIN should show `type=ref` with `key=idx_customer_created` and no filesort

### Verification

- `EXPLAIN` should show `Using index` (covering index) instead of filesort
- `Rows` examined should drop from 37,500 to ~50
- Page load time should be under 1 second
- Monitor slow query log for absence of this query pattern

### Confidence Assessment

| Factor | Confidence |
|--------|-----------|
| Query analysis (EXPLAIN) | 0.95 |
| Index design | 0.90 |
| Root cause identification | 0.90 |
| Resolution effectiveness | 0.85 |

### Risk Warnings

- Adding indexes has a minor write performance cost
- Index additions on large tables may take significant time
- Test index addition on staging with similar data volume
- Monitor query performance after changes with Performance Schema

### Lessons Learned

- Always EXPLAIN before modifying queries or indexes
- Application queries should match data access patterns for the specific page
- Removing unnecessary JOINs is often more effective than adding indexes
- Composite indexes should match both WHERE and ORDER BY clauses
- Consider pagination (LIMIT) for large result sets

---

## Example 4: Server Won't Start After Disk Full

### Scenario

A MySQL 8.0 server fails to start after the data directory disk filled up due to unrotated binary logs. The server has a single InnoDB instance with no replication configured.

### Initial Observations

- `systemctl status mysqld` shows service failed to start
- Error log shows: `InnoDB: Unable to lock ./ibdata1 error: 11`
- Disk usage on data directory: 100%
- No other services affected

### Evidence Collection

**Step 1: Check disk usage**
```
# Advised command (do not execute in production without approval)
df -h /var/lib/mysql
```

**Step 2: Check error log**
```
# Advised command (do not execute in production without approval)
-- Read /var/log/mysql/error.log (or configured error_log location)
```
Look for InnoDB-specific errors and the exact failure reason.

**Step 3: Check for orphaned processes**
```
# Advised command (do not execute in production without approval)
ps aux | grep mysqld
```
Verify no other mysqld processes are holding the ibdata1 lock.

### Diagnosis

**Finding 1**: Disk is full on the data partition — binary logs have not been rotated or purged.

**Finding 2**: InnoDB cannot create or lock the ibdata1 file because there is no space.

**Finding 3**: No other mysqld process is running, so the lock error is due to insufficient space, not a competing process.

**Root Cause**: Binary logs grew uncontrolled over several weeks. The `binlog_expire_logs_seconds` was not configured, and automated purging was not in place.

### Resolution

**Recommended Actions** (in order of implementation):

1. **Free disk space (immediate)**:
   ```sql
   -- ADVISORY: Only purge old binary logs
   -- First, check which logs can be safely purged
   SHOW BINARY LOGS;
   
   -- Then purge safely
   PURGE BINARY LOGS BEFORE DATE_SUB(NOW(), INTERVAL 7 DAY);
   ```

2. **Start MySQL**:
   ```
   -- ADVISORY: Start the server
   systemctl start mysqld
   ```

3. **Configure binlog expiration**:
   ```sql
   -- ADVISORY: Set binlog auto-expiration
   SET GLOBAL binlog_expire_logs_seconds = 604800; -- 7 days
   SET PERSIST binlog_expire_logs_seconds = 604800;
   ```

4. **Monitor disk usage**:
   ```
   -- ADVISORY: Set up disk space monitoring
   -- Alert when disk usage exceeds 80%
   ```

5. **Preventive measures**:
   - Configure `binlog_expire_logs_seconds` permanently
   - Set up disk space monitoring with alerts
   - Implement automated binlog purging
   - Consider reducing `max_binlog_size` if binlog files grow too large

### Verification

- `systemctl status mysqld` shows active and running
- `SHOW BINARY LOGS` shows only recent logs
- `SHOW VARIABLES LIKE 'binlog_expire_logs_seconds'` shows configured value
- Disk usage shows healthy free space

### Confidence Assessment

| Factor | Confidence |
|--------|-----------|
| Disk full diagnosis | 0.95 |
| Root cause identification | 0.95 |
| Resolution effectiveness | 0.90 |
| Prevention effectiveness | 0.85 |

### Risk Warnings

- Purging binary logs removes data needed for PITR to points within the purged period
- Verify backup exists before purging logs
- Always ensure free disk space before starting MySQL
- Monitor disk usage after fix to confirm logs are being properly managed

### Lessons Learned

- Binary log expiration must be configured for all production servers
- Disk space monitoring is essential — alert before disk fills
- Regular binlog purge schedule prevents uncontrolled growth
- Always verify backups exist before purging binary logs
- Consider reducing `max_binlog_size` to prevent individual files from growing too large

---

## References

- [MySQL 8.0 Reference Manual: Troubleshooting](https://dev.mysql.com/doc/refman/8.0/en/troubleshooting.html)
- [MySQL 8.0 Reference Manual: Replication](https://dev.mysql.com/doc/refman/8.0/en/replication.html)
- [MySQL 8.0 Reference Manual: Query Optimization](https://dev.mysql.com/doc/refman/8.0/en/optimizing-queries.html)
- [MySQL 8.0 Reference Manual: Performance Schema](https://dev.mysql.com/doc/refman/8.0/en/performance-schema.html)