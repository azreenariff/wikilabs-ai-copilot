# MySQL Terminology Glossary

## Overview

This glossary provides definitions for key MySQL terminology used throughout the skill pack. Terms are organized by domain and include cross-references to related concepts.

---

## A

### Aborted_connects
**Category**: Monitoring

Count of connections that have been aborted during the connect phase. A high value may indicate authentication issues, network problems, or clients attempting to connect with incorrect credentials.

**Related**: ERROR 1045, max_connections, Threads_connected

### ACL (Access Control List)
**Category**: Security

Rules that control which users can access which resources. MySQL implements ACLs through the grant tables in the mysql system database, specifying user, host, privilege, and database-level access.

**Related**: privileges, roles, GRANT, REVOKE

### Archive Engine
**Category**: Storage Engine

A storage engine designed for high-compression, append-only storage. Archive tables store data using zlib compression and support only INSERT and SELECT operations (no UPDATE or DELETE).

**Use case**: Logging, data warehousing, archival storage

**Related**: MyISAM, InnoDB, compression

### Auto_increment
**Category**: Data Types

A column attribute that generates a unique sequential integer value. Commonly used for primary key columns. Multiple tables can use auto_increment independently.

**Parameters**: auto_increment_increment (step), auto_increment_offset (starting point)

**Related**: primary key, surrogate key, distributed systems

---

## B

### Binary Log (Binlog)
**Category**: Replication, Backup

A log file that records all data-modifying statements (and potentially DDL) executed on the MySQL server. Used for replication to replicas and point-in-time recovery (PITR).

**Formats**: STATEMENT, ROW, MIXED

**Key parameters**: log_bin, binlog_format, binlog_expire_logs_seconds, gtid_mode

**Related**: replication, PITR, mysqlbinlog, GTID

### Blackhole Engine
**Category**: Storage Engine

A storage engine that accepts INSERT and UPDATE statements but discards the data (stores nothing). DELETE statements also succeed but affect nothing. Used for replication forwarding and as a test data sink.

**Related**: Replication, Federated engine

### Buffer Pool
**Category**: InnoDB, Performance

The InnoDB memory area that caches data and index pages in memory. This is the single most important performance parameter for MySQL. Size is set via innodb_buffer_pool_size.

**Key metrics**:
- Hit ratio: 1 - (reads/read_requests) — target > 99%
- Pages free: Innodb_buffer_pool_pages_free — should not be near zero
- Dirty pages: Innodb_buffer_pool_pages_dirty — pages modified but not flushed

**Related**: InnoDB, redo log, page eviction, LRU

---

## C

### caching_sha2_password
**Category**: Authentication

The default authentication plugin in MySQL 8.0 and 8.4. Provides SHA-256 hashing with caching for improved performance over mysql_native_password.

**Compatibility**: Most modern connectors support it. Older clients may need to fall back to mysql_native_password.

**Related**: mysql_native_password, authentication, SSL/TLS, ALTER USER

### Change Buffer
**Category**: InnoDB

An optimization that caches changes to secondary index pages when the leaf page is not in the buffer pool. Merges changes when the page is later read. Reduces random I/O for secondary index updates.

**Parameter**: innodb_change_buffer_max_size (default 25% of buffer pool)

**Related**: InnoDB, buffer pool, secondary indexes, write performance

### Checksum
**Category**: Integrity

A hash value computed from data for verification purposes. MySQL supports table checksums (CHECKSUM TABLE) for integrity checking and replication verification.

**Related**: pt-table-checksum, replication, data integrity, pt-table-sync

### Clustered Index
**Category**: InnoDB, Indexing

The primary key index of a table, which determines the physical storage order of data rows. In InnoDB, every table has exactly one clustered index (created implicitly on the primary key).

**Related**: Secondary index, primary key, InnoDB, covering index

### Column
**Category**: Schema

A named attribute of a table, defined by data type and constraints. Each column occupies a slot in each row of the table.

**Attributes**: Data type, NULL/NOT NULL, DEFAULT, UNIQUE, AUTO_INCREMENT, COMMENT

**Related**: Table, schema, ALTER TABLE, data type

### Commit
**Category**: Transaction

The operation that makes a transaction's changes permanent. A commit marks the point of no return — changes cannot be undone after commit.

**Related**: Rollback, transaction, ACID, InnoDB, redo log

### Connection Pool
**Category**: Performance, Architecture

A cache of database connections maintained so that connections can be reused when future requests to the database are required. Reduces the overhead of establishing new connections.

**Implementations**: ProxySQL, PgBouncer, application-level pools (HikariCP, SQLAlchemy, etc.)

**Related**: max_connections, wait_timeout, Threads_connected, ProxySQL

---

## D

### Data Dictionary
**Category**: Architecture

The system of tables and files that store all metadata about the MySQL database — schemas, tables, columns, indexes, users, privileges, and more. MySQL 8.0 introduced a transactional data dictionary replacing the file-based system of MySQL 5.7.

**MySQL 8.0**: File-based with transactional tables in InnoDB system tablespace

**MySQL 8.4**: JSON-based data dictionary with enhanced metadata management

**Related**: information_schema, mysql system tables, DDL, atomic transactions

### DATETIME
**Category**: Data Types

A date and time data type that stores values from '1000-01-01 00:00:00' to '9999-12-31 23:59:59'. Stored as 8 bytes, unaffected by timezone settings.

**Comparison**: TIMESTAMP stores as UTC and converts to current timezone on retrieval; limited to 2038-01-19 03:14:07.

**Related**: TIMESTAMP, DATE, TIME, JSON

### Deadlock
**Category**: InnoDB, Locking

A situation where two or more transactions are waiting for each other to release locks, creating a circular dependency. Neither transaction can proceed. InnoDB automatically detects deadlocks and rolls back one transaction.

**Error**: ERROR 1213 (40001) — "Deadlock found when trying to get lock"

**Related**: innodb_lock_wait_timeout, locking, isolation level, SHOW ENGINE INNODB STATUS

### Default Value
**Category**: Schema

A value automatically assigned to a column when a row is inserted without specifying a value for that column. Can be a literal value, NOW(), or a generated column expression.

**Related**: INSERT, generated columns, NOT NULL

### Doublewrite Buffer
**Category**: InnoDB

A special buffer that InnoDB writes data pages to before writing them to their final location. Protects against partial page writes caused by crashes, ensuring page integrity during recovery.

**Related**: InnoDB, crash recovery, innodb_flush_method, durability

---

## E

### EXPLAIN
**Category**: Query Optimization

A MySQL statement that shows the execution plan for a SELECT, INSERT, UPDATE, or DELETE query. Reveals how MySQL accesses data (indexes used, join order, filters, sorting).

**Extended**: EXPLAIN ANALYZE (MySQL 8.0.18+) shows actual execution metrics, not just estimates.

**Key output**: type (access method), key (index used), rows (estimated rows), Extra (additional info)

**Related**: Index, query optimization, slow query, EXPLAIN ANALYZE

### Extended State Info (ESIS)
**Category**: Diagnostics

Additional diagnostic information about a query or operation. Available through Performance Schema and optimizer trace. Provides detailed execution steps and cost breakdowns.

**Related**: optimizer_trace, Performance Schema, query plan, optimization

---

## F

### Federated Engine
**Category**: Storage Engine

A storage engine that allows access to tables on remote MySQL servers as if they were local. The local server sends SQL to the remote server and processes the results.

**Limitations**: Network dependency, no local caching, limited feature support

**Related**: Remote access, replication, performance, ProxySQL

### Filesort
**Category**: Query Optimization

An operation where MySQL sorts query results because no suitable index exists for the ORDER BY clause. Appears as "Using filesort" in EXPLAIN Extra column. Indicates a performance optimization opportunity.

**Related**: Index, ORDER BY, EXPLAIN, sort_buffer_size

### Foreign Key
**Category**: Schema, Integrity

A column (or set of columns) that references the primary key of another table, enforcing referential integrity. Supported by InnoDB but not by MyISAM.

**Cascades**: ON DELETE CASCADE, ON UPDATE CASCADE, ON DELETE SET NULL, etc.

**Related**: InnoDB, referential integrity, table relationship, index

### Full Table Scan
**Category**: Query Optimization

Reading every row of a table to find matching data. Appears as type=ALL in EXPLAIN output. Should be minimized — indicates a missing or unusable index.

**Related**: Index, EXPLAIN, query optimization, performance

---

## G

### General Log
**Category**: Logging

A log that records every client connection and every statement received by the server. Useful for debugging but can be very verbose and impact performance. Generally disabled in production.

**Related**: slow query log, error log, binary log, Performance Schema

### Generated Column
**Category**: Schema

A column whose value is computed from an expression involving other columns. Can be VIRTUAL (computed on read) or STORED (computed on write and persisted). Stored generated columns can be indexed.

**Example**: `total_price DECIMAL(10,2) GENERATED ALWAYS AS (quantity * unit_price) STORED`

**Related**: Virtual columns, indexes, JSON columns, performance

### GTID (Global Transaction Identifier)
**Category**: Replication

A unique identifier assigned to every transaction committed on the source server. GTIDs make replication management simpler — replicas track executed GTIDs rather than log file/position pairs.

**Parameter**: gtid_mode=ON, gtid_executed, gtid_purged

**Related**: Replication, PITR, CHANGE REPLICATION SOURCE TO, MySQL Shell

---

## I

### Index
**Category**: Performance, Schema

A data structure that provides fast lookup of rows based on column values. MySQL supports B-tree indexes (default), hash indexes (Memory engine), full-text indexes, and spatial indexes.

**Types**: UNIQUE, FULLTEXT, SPATIAL, composite (multi-column), invisible (MySQL 8.0.13+)

**Related**: Primary key, secondary index, covering index, EXPLAIN, slow query

### InnoDB
**Category**: Storage Engine

The default storage engine for MySQL. Provides ACID-compliant transactions, row-level locking, crash recovery, foreign keys, and clustered indexing. Recommended for all production workloads.

**Key features**: MVCC, undo log, redo log, buffer pool, change buffer, adaptive hash index

**Related**: MyISAM, Archive, Memory engine, clustering, replication

### InnoDB Cluster
**Category**: High Availability

Oracle's recommended high-availability solution for MySQL. Combines MySQL Group Replication, MySQL Shell, and MySQL Router to provide automated failover, read/write splitting, and configuration management.

**Components**:
- Group Replication (consensus-based replication)
- MySQL Shell (configuration and management)
- MySQL Router (transparent connection routing)

**Related**: Group Replication, MySQL Router, ProxySQL, failover

### Invisible Index
**Category**: Indexing (MySQL 8.0.13+)

An index that the optimizer ignores but is still maintained on writes. Allows safe testing of index removal without actually dropping the index.

**Syntax**: `ALTER TABLE t ALTER INDEX idx_name INVISIBLE;`

**Related**: Index, testing, performance, optimizer

### ISOLATION LEVEL
**Category**: Transaction

A setting that controls the degree to which transactions are isolated from each other. MySQL supports four isolation levels:

| Level | Dirty Read | Non-repeatable Read | Phantom Read |
|-------|-----------|--------------------|-------------|
| READ UNCOMMITTED | Possible | Possible | Possible |
| READ COMMITTED | No | Possible | Possible |
| REPEATABLE READ (default) | No | No | Possible |
| SERIALIZABLE | No | No | No |

**Related**: MVCC, LOCKS, transaction, InnoDB

---

## L

### Lock Wait Timeout
**Category**: InnoDB, Locking

The time (in seconds) that a statement waits for a lock before failing. Default is 50 seconds. Controlled by innodb_lock_wait_timeout (GLOBAL or SESSION).

**Error**: ERROR 1205 (HY000) — "Lock wait timeout exceeded"

**Related**: Deadlock, data_lock_waits, blocking query, innodb_locks_unsafe_for_binlog

### Log File
**Category**: Storage

InnoDB files that record redo log entries for crash recovery. There are typically 2 log files in a circular buffer, configured via innodb_log_file_size.

**Related**: Redo log, crash recovery, innodb_flush_log_at_trx_commit, checkpoint

---

## M

### MariaDB
**Category**: Platform

A community-developed fork of MySQL created by the original MySQL developers. MariaDB is generally compatible with MySQL but adds features and changes default configurations. The MariaDB client (mariadb) is a drop-in replacement for the MySQL client.

**Compatibility**: Compatible with MySQL 5.x; differences grow with version gap

**Related**: MySQL, Galera Cluster, Percona Server

### Max_connections
**Category**: Configuration

The maximum number of simultaneous client connections allowed. Default is 151. Setting too low causes ERROR 1040; setting too high wastes memory (each connection uses ~256KB per thread).

**Related**: Threads_connected, wait_timeout, thread_cache_size, connection pool

### Memory Engine
**Category**: Storage Engine

A storage engine that stores data in memory (RAM). Provides very fast access but data is lost on server restart. Tables are defined in .frm and .MYD files. Supports only HEAPT storage format.

**Parameters**: max_heap_table_size, tmp_table_size

**Related**: Temporary tables, cache, volatile data

### mysqld
**Category**: Server

The MySQL server daemon — the main process that handles client connections, executes queries, manages storage, and maintains the data directory.

**Related**: mysql client, mysqld_safe, systemctl, systemd

### mysqldump
**Category**: Backup

The logical backup utility that exports database content as SQL statements. Supports single-transaction mode for InnoDB (non-blocking), consistent backups of all tables.

**Key options**: --single-transaction, --master-data, --routines, --triggers, --events

**Related**: mysqlpump, xtrabackup, PITR, binary log

### mysqlpump
**Category**: Backup

An enhanced backup utility that supports parallel table dumping for faster backups. Available in MySQL 5.7+ but deprecated in MySQL 8.0.30+.

**Related**: mysqldump, parallelism, backup performance

---

## N

### Non-Unique Index
**Category**: Indexing

An index that allows duplicate values in the indexed column(s). Also called a secondary index or non-clustered index. All indexes except the primary key are non-unique by default.

**Related**: Unique index, primary key, secondary index, composite index

---

## O

### O_DIRECT
**Category**: I/O

A flush method that bypasses the OS file system cache for InnoDB data and index files. Reduces double buffering (OS cache + InnoDB buffer pool), improving I/O performance.

**Parameter**: innodb_flush_method=O_DIRECT

**Related**: Doublewrite buffer, I/O performance, file system cache, fsync

### Optimizer
**Category**: Query Processing

The MySQL component that determines the most efficient execution plan for each query. Analyzes table statistics, available indexes, join possibilities, and cost estimates to select the best plan.

**Parameters**: optimizer_search_depth, optimizer_switch, optimizer_prune_level

**Related**: EXPLAIN, query plan, table statistics, cost model

### OOM (Out of Memory)
**Category**: System

When the operating system kills a process due to memory exhaustion. On Linux, the kernel's OOM killer may terminate mysqld if available memory is insufficient.

**Prevention**: Set innodb_buffer_pool_size appropriately, monitor memory usage, use swap cautiously

**Related**: innodb_buffer_pool_size, memory, kernel, swap

---

## P

### Partitioning
**Category**: Schema, Performance

Dividing a large table into smaller, more manageable pieces based on a partitioning key. Supported partition types: RANGE, LIST, HASH, KEY. Improves query performance through partition pruning.

**Related**: Partition pruning, RANGE, HASH, maintenance, ALTER TABLE

### Percona XtraBackup
**Category**: Backup

Percona's tool for hot physical backup of MySQL/Percona Server databases. Backs up InnoDB data without locking tables, supports incremental backups, and is significantly faster than logical backups.

**Commands**: xtrabackup, innobackupex, xbstream

**Related**: mysqldump, physical backup, PITR, incremental backup

### Primary Key
**Category**: Schema, Indexing

A column (or set of columns) that uniquely identifies each row in a table. Every table should have a primary key. In InnoDB, the primary key determines the physical storage order (clustered index).

**Requirements**: UNIQUE, NOT NULL, preferably immutable

**Related**: Clustered index, surrogate key, foreign key, index

### Performance Schema
**Category**: Monitoring

A MySQL instrumentation framework that collects runtime data about internal server operations. Provides detailed insights into wait events, statement execution, memory usage, and more.

**Views**: sys schema provides human-friendly views over Performance Schema

**Related**: sys schema, monitoring, optimization, troubleshooting

### ProxySQL
**Category**: Proxy, HA

A high-performance, high-availability SQL proxy for MySQL. Provides connection pooling, query routing (read/write splitting), query caching, and monitoring. Acts as a transparent middle layer between applications and database servers.

**Related**: MySQL Router, connection pool, read/write splitting, HA

### PITR (Point-in-Time Recovery)
**Category**: Backup, Recovery

Recovering a database to a specific point in time using a base backup plus binary log replay. Requires binary logging enabled and a consistent base backup.

**Tools**: mysqldump, mysqlbinlog, Percona XtraBackup

**Related**: Binary log, GTID, backup, recovery

---

## R

### Redo Log
**Category**: InnoDB

The InnoDB log that records physical changes to data pages. Used for crash recovery to replay changes made but not yet flushed to disk. Enables write-ahead logging (WAL).

**Parameter**: innodb_log_file_size (1-4GB per file), innodb_flush_log_at_trx_commit

**Related**: Undo log, crash recovery, WAL, checkpoint, durability

### Replication
**Category**: High Availability

Copying database changes from a source (master) server to one or more replica (slave) servers. Provides data redundancy, read scaling, and disaster recovery.

**Types**: Master-Slave, Group Replication, Multi-Master, Semi-Sync

**Related**: Binary log, GTID, replica, failover, ProxySQL

### Replica (MySQL 8.0) / Slave (MySQL 5.7)
**Category**: Replication

A MySQL server that receives and applies changes from a source (master) server. In MySQL 8.0, the term "replica" is preferred over "slave" for better terminology.

**Commands**: SHOW REPLICA STATUS, START REPLICA, STOP REPLICA

**Related**: Master, replication, lag, GTID, Group Replication

### Rollback
**Category**: Transaction

The operation that undoes changes made by an uncommitted transaction. Rollback restores data to its state before the transaction began. Uses undo log for this purpose.

**Related**: Commit, transaction, undo log, InnoDB, MVCC

### Row-based Replication (RBR)
**Category**: Replication

A binary log format that records the actual row changes rather than the SQL statements. More reliable than statement-based replication, especially for non-deterministic operations.

**Related**: Binary log, STATEMENT-based, MIXED, GTID

---

## S

### Secondary Index
**Category**: Indexing

An index other than the primary key (clustered index). Secondary indexes store a pointer to the primary key value rather than the full row data. Also called non-clustered index.

**Related**: Clustered index, covering index, composite index, EXPLAIN

### Semi-Sync Replication
**Category**: Replication

A replication mode where the source waits for acknowledgment from at least one replica before confirming a transaction commit. Reduces data loss risk compared to async replication.

**Parameter**: rpl_semi_sync_master_enabled, rpl_semi_sync_slave_enabled

**Related**: Group Replication, async replication, data safety, failover

### SHOW
**Category**: Diagnostics

MySQL diagnostic statements for viewing server state. Common forms:

| Statement | Purpose |
|-----------|---------|
| SHOW VARIABLES | Display configuration parameters |
| SHOW STATUS | Display runtime statistics |
| SHOW PROCESSLIST | Display active connections and queries |
| SHOW GRANTS | Display user privileges |
| SHOW CREATE TABLE | Display table structure |
| SHOW ENGINE INNODB STATUS | Display InnoDB diagnostics |
| SHOW REPLICA STATUS | Display replication status |

**Related**: Diagnostics, monitoring, troubleshooting, Performance Schema

### Slow Query Log
**Category**: Logging

A log that records all queries that exceed the long_query_time threshold. Essential for identifying performance problems. Records full SQL text, execution time, lock time, and rows examined.

**Parameters**: slow_query_log, long_query_time, slow_query_log_file

**Related**: pt-query-digest, EXPLAIN, optimizer, performance tuning

### SSL/TLS
**Category**: Security

Secure communication between MySQL client and server using SSL/TLS encryption. Protects data in transit from eavesdropping and man-in-the-middle attacks.

**Parameters**: require_secure_transport, tls_version, ssl_ca, ssl_cert, ssl_key

**Related**: Authentication, encryption, certificates, security

---

## T

### table_open_cache
**Category**: Configuration

The number of open tables that can be cached. Increasing this reduces file descriptor usage when many tables are accessed. Related to table_open_cache_instances (MySQL 8.0).

**Monitoring**: SHOW OPEN TABLES, table_open_cache_hit ratio

**Related**: open_files_limit, file descriptors, performance

### Table Status
**Category**: Monitoring

Metadata about a table including row count, data length, index length, engine, collation, and more. Available through SHOW TABLE STATUS or information_schema.TABLES.

**Related**: information_schema, ANALYZE TABLE, OPTIMIZE TABLE, statistics

### Temporary Table
**Category**: Query Processing

A transient table created by MySQL for query execution (GROUP BY, DISTINCT, ORDER BY, UNION). Stored in memory (memory engine) or on disk (InnoDB) depending on size.

**Parameters**: max_heap_table_size, tmp_table_size

**Related**: filesort, disk I/O, performance, EXPLAIN

### TIMESTAMP
**Category**: Data Types

A date and time data type stored as UTC internally. Converts to current timezone on retrieval. Limited to 2038-01-19 03:14:07 (32-bit second timestamp). Auto-updates with CURRENT_TIMESTAMP.

**Related**: DATETIME, timezone, generated columns, NOW()

### Transaction
**Category**: ACID, InnoDB

A logical unit of work that is atomic, consistent, isolated, and durable (ACID). A transaction consists of one or more SQL statements that either all succeed (commit) or all fail (rollback).

**Isolation levels**: READ UNCOMMITTED, READ COMMITTED, REPEATABLE READ, SERIALIZABLE

**Related**: Commit, rollback, MVCC, lock, InnoDB, redo log

---

## U

### Undo Log
**Category**: InnoDB

The InnoDB log that stores before-images of data for transaction rollback and MVCC read views. Truncated automatically when no transaction needs them.

**Parameter**: innodb_undo_tablespaces, innodb_undo_logs

**Related**: Redo log, transaction, rollback, MVCC, crash recovery

### Unique Index
**Category**: Indexing

An index that enforces uniqueness of values in the indexed column(s). The primary key is an implicit unique index. Multiple unique indexes can exist on a table.

**Related**: Primary key, secondary index, constraint, duplicate entry

---

## V

### Virtual Column
**Category**: Schema

A computed column whose value is derived from an expression. VIRTUAL columns are computed at read time; STORED columns are computed at write time and persisted.

**Related**: Generated column, JSON columns, index, performance

---

## W

### wait_timeout
**Category**: Configuration

The number of seconds the server waits for activity on a non-interactive connection before closing it. Default is 28800 seconds (8 hours), often too high for web applications.

**Recommended**: 300-600 seconds for web apps, 28800 for batch/ETL

**Related**: max_connections, Threads_connected, connection pool, idle connections

### Window Function
**Category**: SQL Feature

A SQL function that performs calculations across a set of rows related to the current row (e.g., ROW_NUMBER, RANK, SUM OVER, LAG/LEAD). Supported in MySQL 8.0+.

**Syntax**: `SELECT name, salary, RANK() OVER (ORDER BY salary DESC) FROM employees;`

**Related**: CTE, SQL:2011, MySQL 8.0, aggregation

---

## References

- [MySQL 8.0 Reference Manual: Terminology](https://dev.mysql.com/doc/refman/8.0/en/glossary.html)
- [MySQL 8.4 Reference Manual: Terminology](https://dev.mysql.com/doc/refman/8.4/en/glossary.html)
- [MySQL 8.0 InnoDB Glossary](https://dev.mysql.com/doc/refman/8.0/en/innodb-glossary.html)