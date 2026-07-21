# MySQL Best Practices

## Overview

This document provides best practices organized by topic for MySQL 8.0 and 8.4. These practices are based on enterprise-grade deployment patterns and are continuously refined based on production experience.

## Database Design Best Practices

### Character Set and Collation

- **Use utf8mb4** for all databases and tables — it supports the full Unicode repertoire including emojis and surrogate pairs
- **Use utf8mb4_unicode_ci** as the default collation for most applications (balance of performance and correctness)
- **Use utf8mb4_0900_ai_ci** for MySQL 8.0+ AI-powered collation (improved Unicode 9.0 support)
- Avoid the legacy `utf8` character set — it is only a 3-byte subset of UTF-8 and cannot represent surrogate pairs

```sql
-- Recommended database creation
CREATE DATABASE mydb CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
```

### Table Design

- **Always use InnoDB** as the storage engine (default in MySQL 5.5+)
- **Specify ENGINE=InnoDB** explicitly in CREATE TABLE for clarity and forward compatibility
- **Use AUTO_INCREMENT** for primary keys — it provides efficient sequential inserts
- **Prefer BIGINT** for surrogate keys — the performance difference from INT is negligible with modern hardware
- **Use TIMESTAMP** for columns that need timezone awareness (stored as UTC); use DATETIME for wall-clock time
- **Use ENUM sparingly** — they are stored as integers but can cause migration issues if values change
- **Avoid ENUM for large value sets** — consider a separate lookup table instead

### Index Design

- **Create indexes strategically** — every index adds write overhead and consumes disk space
- **Use composite indexes** to cover multi-column queries; order columns by selectivity (most selective first)
- **Monitor index usage** with sys.schema_unused_indexes — unused indexes should be dropped
- **Use covering indexes** when possible — an index that includes all columns needed by a query avoids table lookups
- **Use DESC indexes** (MySQL 8.0.1+) for reverse-order queries instead of filesort
- **Avoid over-indexing** — each index must justify its existence with query coverage or foreign key enforcement

### Data Types

- **Use appropriate data types** — VARCHAR for variable strings, INT for integers, DECIMAL for monetary values
- **Use DECIMAL for currency** — never use FLOAT or DOUBLE for monetary calculations
- **Use TINYINT for booleans** — MySQL does not have a native BOOLEAN type; TINYINT(1) is the convention
- **Use JSON for flexible schemas** when the schema cannot be predicted in advance
- **Use BLOB sparingly** — consider external storage (S3, filesystem) for large binary data

## Storage Engine Best Practices

### InnoDB Configuration

- **innodb_buffer_pool_size**: Set to 50–70% of available RAM on dedicated MySQL servers
- **innodb_buffer_pool_instances**: Set to number of CPUs (default 8 is usually fine)
- **innodb_log_file_size**: Larger values improve write performance; 1–4 GB for write-heavy workloads
- **innodb_flush_log_at_trx_commit**: Set to 1 for full ACID compliance; 2 for better performance (1 second potential data loss)
- **innodb_flush_method**: Use O_DIRECT or O_SYNC to avoid double buffering
- **innodb_io_capacity**: Tune based on disk type (SSD: 2000–5000; HDD: 100–200)
- **innodb_io_capacity_max**: Set to 2× innodb_io_capacity for burst flushes

### InnoDB Best Practices

- **Never force-stop MySQL** during heavy write workloads — this may require crash recovery
- **Monitor buffer pool hit ratio** — it should be > 99% for healthy workloads
- **Monitor redo log utilization** via Performance Schema — high utilization indicates log file size is too small
- **Monitor purge thread performance** — if purge lags, consider increasing innodb_purge_threads
- **Use innodb_autoinc_lock_mode** = 2 (interleaved) for best insert concurrency (MySQL 5.1+)

### MyISAM

- **Avoid MyISAM** in new deployments — it lacks transactions, row-level locking, and crash recovery
- **Convert MyISAM tables to InnoDB** as a priority for any existing deployments
- **MyISAM only** for read-heavy, non-critical tables where transactions are not needed

### Memory Engine

- **Use Memory engine** only for tables that fit entirely in RAM (temp tables, session data)
- **Data is lost on server restart** — do not use for persistent data
- **Table structure is preserved** across restarts even if data is not

### Archive Engine

- **Use Archive engine** for historical data with append-only access patterns
- **Good for audit logs, event records, and time-series data**
- **Compressed storage** — significantly smaller than InnoDB for large datasets

## Performance Best Practices

### Query Optimization

1. **Use EXPLAIN or EXPLAIN ANALYZE** for every non-trivial query to verify index usage
2. **Avoid SELECT *** — specify only the columns you need
3. **Use LIMIT** in ad-hoc queries to prevent returning massive result sets
4. **Use prepared statements** to reduce parsing overhead and prevent SQL injection
5. **Batch INSERT operations** — multi-row inserts are more efficient than individual inserts
6. **Use batch transactions** — commit fewer, larger transactions instead of many small ones
7. **Avoid functions on indexed columns in WHERE clauses** — this prevents index usage

```sql
-- Bad: function on column prevents index usage
SELECT * FROM users WHERE YEAR(created_at) = 2024;

-- Good: use range query to preserve index usage
SELECT * FROM users WHERE created_at >= '2024-01-01' AND created_at < '2025-01-01';
```

### Connection Management

- **Use connection pooling** (e.g., PgBouncer for proxies, or application-level pool)
- **Set max_connections** based on application concurrency requirements
- **Monitor Aborted_connects** — high values indicate connection issues
- **Use persistent connections** where supported to reduce connection overhead
- **Set wait_timeout** to 600–900 seconds for idle connection cleanup

### Table Cache

- **Set table_open_cache** to at least 2–4× the number of tables in the database
- **Monitor Table_open_cache_hits** vs Table_open_cache_misses — misses indicate cache is too small
- **Monitor Opened_tables** — if it grows rapidly, increase table_open_cache

### Partitioning

- **Use RANGE partitioning** for time-series data (by month or year)
- **Use HASH partitioning** for even data distribution across disks
- **Test partitioning thoroughly** — it adds complexity to DDL operations
- **Monitor partition pruning** — ensure queries are actually pruning unused partitions

## Security Best Practices

### User Management

- **Follow the principle of least privilege** — grant only the minimum privileges needed
- **Use specific host patterns** instead of '%' in production — e.g., '10.0.0.%' instead of '%'
- **Use roles** (MySQL 8.0+) to manage privilege groups efficiently
- **Use password expiration** for service accounts (PASSWORD EXPIRE INTERVAL)
- **Regularly audit user privileges** with SELECT user, host, plugin FROM mysql.user

### SSL/TLS

- **Enable SSL/TLS** for all remote connections
- **Use TLS 1.2 or higher** — disable SSLv3 and TLS 1.0/1.1
- **Use strong cipher suites** — prefer ECDHE-RSA-AES256-GCM-SHA384
- **Use REQUIRE SSL or REQUIRE X509** at the user level for mandatory encryption
- **Rotate certificates regularly** — automate renewal where possible

### Password Policy

- **Enable password validation** with validate_password component
- **Set minimum password length** to at least 12 characters
- **Enable password history** (password_history) to prevent password reuse
- **Set password reuse interval** (password_reuse_interval) to enforce rotation

### Firewall and Network

- **Bind to specific interfaces** — do not bind to 0.0.0.0 in production
- **Use firewall rules** to restrict MySQL port access (default 3306)
- **Use SSH tunneling** for remote administration instead of exposing MySQL directly

## Backup and Recovery Best Practices

### Backup Strategy

- **Follow the 3-2-1 rule**: 3 copies of data, 2 different storage media, 1 offsite
- **Use mysqldump with --single-transaction** for consistent InnoDB backups without locking
- **Use Percona XtraBackup** for large databases (> 50GB) — physical backup without downtime
- **Test restores regularly** — a backup that cannot be restored is not a backup
- **Automate backups** with cron or orchestration tools
- **Monitor backup completion** — set up alerts for failed or delayed backups

### Binary Log Retention

- **Retain binary logs** for at least 7 days for replication rebuilds
- **Set binlog_expire_logs_seconds** = 604800 (7 days) as a starting point
- **Monitor binary log disk usage** — expired logs should be purged automatically
- **Consider GTID-based replication** to simplify recovery without binlog position tracking

### Recovery Procedures

- **Document recovery procedures** and test them regularly
- **Maintain a runbook** for common recovery scenarios (data loss, corruption, failover)
- **Use point-in-time recovery** with binary logs for precise recovery
- **Test failover procedures** for replication and cluster configurations

## Replication Best Practices

### Master-Slave Replication

- **Use GTID-based replication** (auto_position=1) — it simplifies failover and recovery
- **Monitor replication lag** — alert when Seconds_Behind_Master exceeds your SLA
- **Use semi-sync replication** for critical databases to ensure at-least-one replica has the data
- **Plan for replica rebuilds** — maintain a process for adding new replicas quickly

### Multi-Master Replication

- **Avoid direct multi-master** — use Group Replication or Orchestrator-based solutions
- **Group Replication** provides automatic conflict detection but adds coordination overhead
- **Consider application-level sharding** instead of multi-master for write-heavy workloads

### Monitoring

- **Monitor replication status** with SHOW REPLICA STATUS or Performance Schema
- **Monitor relay log usage** — large relay logs indicate replication lag
- **Monitor binlog size** on the master — growth rate indicates write volume
- **Set up automated alerting** for replication failures

## High Availability Best Practices

### MySQL Router

- **Use MySQL Router** for automatic routing between application and cluster nodes
- **Monitor router health** — configure health checks and automatic failover
- **Run multiple router instances** for router high availability

### ProxySQL

- **Use ProxySQL** for advanced routing, read/write splitting, and query caching
- **Configure read/write splitting** rules to route SELECT queries to replicas
- **Monitor backend health** and automatically remove unhealthy backends from rotation

### InnoDB Cluster

- **Use MySQL Shell** to set up and manage InnoDB Cluster
- **Configure a sufficient number of replicas** for failover capacity (minimum 3 nodes)
- **Test failover regularly** — use mysqlsh cluster.failover() to practice

## Monitoring Best Practices

### Key Metrics

Monitor these key metrics continuously:

1. **Connections**: Total, active, idle, Aborted_connects
2. **Queries**: QPS (queries per second), slow queries
3. **InnoDB buffer pool**: Hit ratio, pages free, pages dirty
4. **Replication**: Seconds_Behind_Master, thread status
5. **Lock waits**: Blocking queries, deadlock frequency
6. **Disk space**: Data directory, binary logs, temporary files
7. **Performance**: Query latency percentiles, thread CPU usage

### Slow Query Log

- **Enable slow query log** with an appropriate long_query_time threshold (start with 1–2 seconds)
- **Use log_queries_unused_indexes** to identify queries not using indexes
- **Log to table** (slow_log table) for easier querying and analysis
- **Rotate logs** regularly to prevent disk space exhaustion

### Performance Schema

- **Enable relevant instrumentation** — Performance Schema has negligible overhead when properly configured
- **Use sys schema** for pre-built queries that are easier to understand
- **Monitor table locks** with sys.schema_table_lock_waits
- **Monitor statement latency** with sys.statements_with_runtimes_in_95th_percentile

## Change Management Best Practices

### Schema Changes

- **Use online DDL** (MySQL 5.6+) for ALTER TABLE operations to avoid extended locking
- **Test schema changes** in staging with production-like data volumes
- **Use pt-osc or gh-ost** for large table alterations that require more control
- **Document all schema changes** in a migration tracking system
- **Run ANALYZE TABLE** after major schema changes to update optimizer statistics

### Configuration Changes

- **Use SET PERSIST** for runtime changes that should survive restarts
- **Document all configuration changes** with the rationale and impact
- **Test configuration changes** in staging before production
- **Monitor the impact** after configuration changes — watch for regressions

### Deployment

- **Use versioned migrations** (e.g., Flyway, Liquibase) for schema management
- **Deploy during maintenance windows** for high-risk changes
- **Have a rollback plan** ready before any deployment
- **Monitor after deployment** — watch for performance regressions and error rate increases