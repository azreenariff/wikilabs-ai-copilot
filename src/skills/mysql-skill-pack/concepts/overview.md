# MySQL Architecture Overview

## Overview

This document provides a comprehensive overview of MySQL server architecture, covering the core components, data flow, storage layers, and the relationship between server subsystems. This knowledge is foundational for all MySQL troubleshooting and optimization activities.

## Core Architecture Components

MySQL follows a modular, multi-threaded architecture designed for high performance, reliability, and extensibility. The server is organized into distinct layers that process SQL statements from client input through to data persistence.

### Client-Server Model

MySQL operates as a client-server database system. Clients connect via:

1. **TCP/IP sockets** — Standard network connection (default port 3306)
2. **Unix domain sockets** — Local connections on Linux/Unix (`/var/run/mysqld/mysqld.sock`)
3. **Named pipes** — Windows local connections
4. **Shared memory** — Windows local connections

Each connection is handled by a dedicated thread (or thread pool member), maintaining its own session state, buffers, and temporary objects.

### The MySQL Server Stack

```
┌─────────────────────────────────────────────────────┐
│                    MySQL Server                      │
│                                                      │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │   Connectors│  │   SQL Layer  │  │  Storage   │ │
│  │             │  │              │  │   Engine   │ │
│  │  · JDBC     │  │  · Parser    │  │  · InnoDB  │ │
│  │  · ODBC     │  │  · Optimizer │  │  · MyISAM  │ │
│  │  · C API    │  │  · Executor  │  │  · Archive │ │
│  │  · Python   │  │  · Caching   │  │  · Memory  │ │
│  │  · PHP      │  │  · Pluggable │  │  · Blackhole│ │
│  │  · Node.js  │  │  · Auth      │  │  · Federated│ │
│  └─────────────┘  └──────────────┘  └────────────┘ │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │              Management Layer                 │   │
│  │  · Performance Schema  · Sys Schema           │   │
│  │  · Error Log            · Binary Log           │   │
│  │  · Slow Query Log       · General Log          │   │
│  │  · Audit Log            · Data Dictionary      │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │              I/O Layer                        │   │
│  │  · File System    · Redo Log Buffer          │   │
│  │  · Undo Log       · Change Buffer            │   │
│  │  · Thread Pool    · Buffer Pool              │   │
│  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

## SQL Layer Architecture

The SQL layer handles all SQL processing, from parsing to execution. It consists of several subsystems that work together.

### Connector Layer

The connector layer manages client connections:

- **Connection handling**: Accepts incoming connections, authenticates users
- **Thread management**: Each connection gets a thread (or thread pool slot)
- **Protocol management**: Implements the MySQL client-server protocol
- **SSL/TLS encryption**: Manages encrypted connections
- **Keepalive handling**: Manages idle connection timeouts

**MySQL 8.0 Connection Threading**:
- Each connection gets its own OS thread by default
- Thread pool plugin can multiplex connections across fewer threads
- `thread_cache_size` caches idle threads for reuse
- `thread_pool_size` controls the number of worker threads in thread pool

### SQL Parser

The parser converts SQL text into an abstract syntax tree (AST):

1. **Lexical analysis**: Tokenizes the SQL input
2. **Syntactic analysis**: Builds the AST based on grammar rules
3. **Semantic analysis**: Validates object references, types, and privileges

**Parser capabilities**:
- Supports ANSI SQL:2011 standard
- MySQL-specific extensions (INSERT ... ON DUPLICATE KEY UPDATE, REPLACE, etc.)
- JSON column operations (MySQL 5.7+)
- Common Table Expressions (CTEs) (MySQL 8.0+)
- Window functions (MySQL 8.0+)
- Recursive CTEs (MySQL 8.0.1+)

### Optimizer

The optimizer is the brain of MySQL — it determines the most efficient execution plan for each query:

**Optimization stages**:
1. **Logical optimization**: Query transformations (predicate pushdown, subquery flattening)
2. **Access path selection**: Choose between index scans, full table scans, index range scans
3. **Join ordering**: Determine the optimal table join order
4. **Join method selection**: Choose between nested loop, hash join (MySQL 8.0.18+), merge join
5. **Cost estimation**: Uses table statistics to estimate row counts and costs

**Key optimizer parameters**:
- `optimizer_search_depth`: How deep to search for optimal join order (default 62)
- `optimizer_switch`: Enable/disable specific optimization features
- `join_buffer_size`, `sort_buffer_size`: Memory allocation for join/sort operations

**MySQL 8.0 Optimizer Improvements**:
- Hash join implementation (8.0.18+)
- Improved cost model for range access
- Better subquery optimization
- Invisible indexes for safe testing of index removal
- Partition pruning enhancements

### Execution Engine

The executor translates the optimized plan into actual operations:

- **Query execution**: Iterates through tables, applies filters, performs joins
- **Function execution**: Evaluates built-in and stored functions
- **Temporary table management**: Creates and manages temp tables for GROUP BY, DISTINCT, ORDER BY
- **Sorting**: Uses in-memory or disk-based sorting
- **Result delivery**: Sends results back to the client

**Execution model**:
- Server executes one statement per connection thread
- Parallel query execution for scans is available in MySQL 8.0.14+ (enterprise and community in 8.0.30+)
- Result streaming reduces memory usage for large result sets

## Storage Engine Architecture

MySQL's pluggable storage engine architecture allows different engines to handle data storage with different characteristics.

### InnoDB (Default Engine)

InnoDB is the default and most widely used storage engine, providing:

- **ACID compliance**: Full transaction support with commit, rollback, and crash recovery
- **Row-level locking**: Fine-grained locking for high concurrency
- **Foreign key constraints**: Referential integrity enforcement
- **Clustered indexes**: Data stored in primary key order
- **Buffer pool**: In-memory cache for data and indexes
- **Crash recovery**: Redo log and undo log for durability
- **Change buffer**: Optimizes secondary index updates for non-unique indexes
- **Adaptive hash index**: Automatically creates hash indexes on frequently accessed data

**InnoDB internal components**:

| Component | Purpose | Key Parameters |
|-----------|---------|----------------|
| Buffer Pool | Cache data and indexes in memory | innodb_buffer_pool_size, innodb_buffer_pool_instances |
| Redo Log | Ensure crash recovery durability | innodb_log_file_size, innodb_log_files_in_group |
| Undo Log | Support rollback and MVCC | innodb_undo_tablespaces, innodb_undo_logs |
| Change Buffer | Optimize secondary index inserts/updates | innodb_change_buffer_max_size |
| Doublewrite Buffer | Prevent partial page writes | innodb_flush_method=O_DIRECT |
| Read View | Support MVCC snapshots | Internal — controlled by isolation level |

### Other Storage Engines

| Engine | Transaction | Locking | Use Case |
|--------|------------|---------|----------|
| MyISAM | No | Table-level | Legacy, read-heavy, no transaction needs |
| Archive | No | Table-level | Data archival, append-only logging |
| Memory | Yes | Table-level | Temporary tables, cache tables |
| Blackhole | No | Table-level | Replication forwarding, test data sink |
| Federated | No | Remote | Access remote MySQL servers as local tables |

**Engine Selection Guide**:
- **Default to InnoDB** for all production workloads
- Use **MyISAM** only for read-only, non-critical lookup tables (rare)
- Use **Archive** for time-series logging and data warehousing
- Use **Memory** for ephemeral cache tables
- Avoid **Federated** in production due to network dependency

## Data Dictionary Architecture

The data dictionary stores all database metadata (schemas, tables, columns, indexes, users, privileges).

### MySQL 8.0 Data Dictionary

MySQL 8.0 introduced a transactional data dictionary, replacing the file-based metadata of MySQL 5.7:

**Components**:
- **System tables**: `mysql` database tables (mysql.user, mysql.db, mysql.tables_priv, etc.)
- **Data dictionary tables**: Hidden tables in the InnoDB system tablespace
- **File-based storage**: .frm files for table definitions (retained for backward compatibility)

**Improvements over MySQL 5.7**:
- Atomic DDL operations — metadata changes are transactional
- Reduced metadata locking contention
- Single source of truth for all metadata
- Improved crash recovery for metadata
- Hidden columns for generated columns, invisible indexes

**Key system schemas**:
- `information_schema`: Standard SQL metadata views
- `performance_schema`: Performance instrumentation
- `sys`: Human-friendly views over performance_schema
- `mysql`: System tables (users, privileges, plugins)

## I/O and Buffer Architecture

### Buffer Pool

The InnoDB buffer pool is the most critical performance component:

**Structure**:
- Divided into pages (default 16KB)
- Contains data pages, index pages, insert buffer, undo logs
- Uses LRU (Least Recently Used) list for page replacement
- Multiple buffer pool instances reduce locking contention

**Memory allocation**:
- Set via `innodb_buffer_pool_size` (60-80% of RAM on dedicated servers)
- Multiple instances via `innodb_buffer_pool_instances` (start with 8 for high concurrency)
- Monitored via `Innodb_buffer_pool_read_requests` vs `Innodb_buffer_pool_reads`

**Hit ratio calculation**:
```
Hit Ratio = 1 - (Innodb_buffer_pool_reads / Innodb_buffer_pool_read_requests)
Target: > 99%
```

### Redo Log

The redo log ensures crash recovery and durability:

**Purpose**:
- Record physical changes to data pages
- Enable crash recovery by replaying changes
- Support write-ahead logging (WAL)

**Structure**:
- Circular buffer of log files
- Default: 2 files of 48MB each (MySQL 8.0)
- Tuned via `innodb_log_file_size` (1-4 GB recommended)
- Written by checkpoint process

**Key parameters**:
- `innodb_log_file_size`: Size of each log file
- `innodb_log_buffer_size`: Memory buffer for redo log (default 16MB)
- `innodb_flush_log_at_trx_commit`: Flush strategy (1=ACID, 2=fast with risk, 0=fastest with risk)

### Undo Log

Undo log supports transaction rollback and MVCC:

**Purpose**:
- Store before-images for transaction rollback
- Provide read views for consistent snapshots (MVCC)
- Support crash recovery

**Structure**:
- Stored in undo tablespaces
- Truncated automatically when no longer needed
- Tuned via `innodb_undo_tablespaces` and `innodb_undo_logs`

## Thread Architecture

### Connection Threads

Each client connection gets a thread:

**Thread lifecycle**:
1. Accept connection
2. Authenticate user
3. Allocate thread context (session variables, buffers, locks)
4. Execute queries
5. Return thread to cache (if `thread_cache_size` > 0) or destroy

**Thread monitoring**:
- `Threads_connected`: Currently connected threads
- `Threads_running`: Currently executing threads
- `Threads_created`: Total threads created since startup
- `Threads_cached`: Threads in cache (from `SHOW STATUS LIKE 'Threads_cached'`)

### Thread Pool (Optional)

MySQL 8.0 includes a thread pool plugin for managing high connection counts:

**Benefits**:
- Multiplexes connections onto fewer OS threads
- Reduces memory overhead per connection
- Provides connection queuing for overload protection
- Reduces context switching on systems with many connections

**Configuration**:
- `thread_pool_size`: Number of worker threads
- `thread_pool_max_threads`: Maximum threads per user
- Requires `thread_pool` plugin to be loaded

## Replication Architecture

MySQL replication provides data redundancy, read scaling, and disaster recovery.

### Binary Log (Binlog)

The binary log records all data-modifying statements for replication and PITR:

**Formats**:
- **STATEMENT**: Records SQL statements (original format)
- **ROW**: Records row changes (recommended, more reliable)
- **MIXED**: Combines statement and row formats

**Key parameters**:
- `log_bin`: Enable binary logging
- `binlog_format`: ROW (recommended), STATEMENT, or MIXED
- `binlog_expire_logs_seconds`: Auto-expire old logs (MySQL 8.0.1+)
- `gtid_mode`: Enable GTID-based replication

### Replication Topologies

| Topology | Description | Use Case |
|----------|-------------|----------|
| Master-Slave | Single master, one or more replicas | Read scaling, backup |
| Master-Master | Two masters, bidirectional replication | Multi-site HA (rarely used) |
| Group Replication | Multi-primary cluster with consensus | High availability, automatic failover |
| Multi-Master | Multiple masters, specific replica target | Geographic distribution |
| Semi-Sync | Master waits for at least one ACK | Reduced data loss risk |

## Configuration Architecture

### Configuration File Processing Order

MySQL reads configuration files in this order (first found wins):

1. `/etc/my.cnf`
2. `/etc/mysql/my.cnf`
3. `SYSCONFDIR/my.cnf`
4. `--defaults-extra-file` (command line)
5. `~/.my.cnf`

**Configuration sections**:
- `[mysqld]`: Server options
- `[client]`: Client options
- `[mysql]`: mysql client options
- `[mysqld_safe]`: Safe server startup options
- `[mysqldump]`: mysqldump options

### Variable Scope

| Scope | Description | Persistence |
|-------|-------------|-------------|
| GLOBAL | Server-wide setting | Lost on restart (unless persisted) |
| SESSION | Per-connection setting | Lost when connection closes |
| PERSIST | Survives restart via mysqld-auto.cnf | Written automatically |

**MySQL 8.0+ Persistent Variables**:
- `SET PERSIST variable_name = value` — writes to mysqld-auto.cnf
- `RESET PERSIST variable_name` — removes from mysqld-auto.cnf
- `SHOW PERSISTED VARIABLES` — display persisted variables

## Monitoring Architecture

### Performance Schema

The Performance Schema instruments server internals for monitoring:

**Major instrument categories**:
- `events_waits_*`: Wait events (I/O, lock waits, synchronization)
- `events_statements_*`: SQL statement execution events
- `events_stages_*`: Statement execution stage events
- `events_transactions_*`: Transaction events
- `setup_actors`: Filter definitions for monitoring
- `setup_objects`: Filter definitions for table/monitoring

**Performance Schema overhead**:
- Can add 10-30% overhead when fully enabled
- Enable only needed instruments
- Use `sys` schema for simplified views
- Disable for production with known-good configuration

### Sys Schema

The sys schema provides human-friendly views over Performance Schema:

**Key views**:
- `schema_table_statistics` — Table I/O and lock statistics
- `host_by_*` — Host-level statistics
- `user_by_*` — User-level statistics
- `statement_analysis` — SQL statement analysis
- `memory_summary` — Memory usage by thread/operation
- `waits_*_lost_instrumented` — Dropped wait events

## MySQL 8.0 vs 8.4 Architecture Differences

| Area | MySQL 8.0 | MySQL 8.4 |
|------|-----------|-----------|
| Data Dictionary | File-based (transactional tables) | JSON-based data dictionary |
| Buffer Pool | LRU-based | Enhanced page life cycle |
| Optimizer | Hash joins (8.0.18+) | Improved cost model |
| Replication | Semi-sync support | Improved group replication |
| Security | caching_sha2_password default | Enhanced password policy |
| Upgrades | In-place upgrade from 5.7 | In-place upgrade from 8.0 |

## References

- [MySQL 8.0 Reference Manual: Architecture](https://dev.mysql.com/doc/refman/8.0/en/mysql-architecture.html)
- [MySQL 8.0 Reference Manual: InnoDB Architecture](https://dev.mysql.com/doc/refman/8.0/en/innodb-architecture.html)
- [MySQL 8.0 Reference Manual: Server Architecture](https://dev.mysql.com/doc/refman/8.0/en/server-architecture.html)
- [MySQL 8.0 Reference Manual: Data Dictionary](https://dev.mysql.com/doc/refman/8.0/en/data-dictionary.html)
- [MySQL 8.4 Reference Manual: Architecture](https://dev.mysql.com/doc/refman/8.4/en/mysql-architecture.html)