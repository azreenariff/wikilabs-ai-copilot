# PostgreSQL Architecture and Key Components

## Overview

PostgreSQL is a powerful, open-source object-relational database system with over 35 years of active development. It has earned a strong reputation for reliability, feature robustness, and performance. EDB Postgres Advanced Server is the enterprise distribution that adds Oracle compatibility and additional enterprise features.

## Core Architecture

### Process Model

PostgreSQL uses a process-based architecture rather than a thread-based one. Each client connection receives its own backend process.

**Key Processes**:

1. **Postmaster (PostgreSQL Server)**
   - The main server process that manages child processes
   - Handles client connections, manages shared memory
   - Spawns backend processes for each connection
   - Manages background worker processes

2. **Backend Process (PostgreSQL Backend)**
   - One per client connection
   - Handles individual client queries
   - Manages per-connection state and memory
   - Communicates with shared memory for coordination

3. **Background Writer (bgwriter)**
   - Periodically writes dirty shared buffers to disk
   - Helps reduce checkpoint I/O load
   - Configurable via `bgwriter_lru_maxpages` and `bgwriter_lru_multiplier`

4. **Walwriter (WAL Writer)**
   - Writes WAL records to the WAL files
   - Runs continuously when the system is active
   - Configurable via `wal_writer_delay`

5. **Autovacuum Launcher**
   - Spawns autovacuum worker processes as needed
   - Manages autovacuum scheduling
   - Monitors table statistics for vacuum decisions

6. **Autovacuum Workers**
   - Perform VACUUM, ANALYZE, and VACUUM FULL operations
   - Spawned by the autovacuum launcher
   - Number controlled by `autovacuum_max_workers`

7. **Startup Process**
   - Runs during server startup for crash recovery
   - Replays WAL records to restore database state
   - Terminates after recovery is complete

8. **Checkpointer**
   - Creates checkpoint records in the WAL
   - Flushes dirty buffers to disk at checkpoints
   - Can be configured to trigger periodic checkpoints

9. **Archiver**
   - Archives completed WAL segments
   - Uses `archive_command` to copy WAL to archive storage
   - Critical for WAL archiving and PITR

10. **Logical Replication Launcher**
    - Manages logical decoding workers
    - Spawns workers for each subscription
    - PostgreSQL 10+

11. **Parallel Query Workers**
    - Spawned by individual backends for parallel queries
    - Range from 0 to `max_parallel_workers_per_gather` workers
    - PostgreSQL 9.6+

### Memory Architecture

PostgreSQL uses shared memory for inter-process coordination and per-process memory for individual operations.

**Shared Memory**:

1. **Shared Buffers**
   - Main cache for database pages
   - Configured via `shared_buffers` parameter
   - Recommended: 25% of system RAM
   - Uses buffer pool with LRU-like eviction

2. **WAL Buffers**
   - Temporary storage for WAL records before flushing
   - Configured via `wal_buffers` parameter
   - PostgreSQL 13+ auto-tunes to 1/32 of shared_buffers (minimum 64MB, maximum 64MB)

3. **CLog (Commit Log)**
   - Tracks transaction commit status
   - Shared memory for commit/abort status of transactions
   - Configured via `commit_siblings` (min before CLog is used)

4. **ProcArray**
   - Tracks running transactions and their snapshot information
   - Used by MVCC to determine visibility of tuples
   - Contains process ID, transaction ID, snapshot, and lock information

5. **GlobalBufferIndex**
   - Maps buffer IDs to shared buffer slots
   - Used for quick buffer location

6. **RelFileNode**
   - Maps relation OIDs to file paths on disk
   - Used by buffer manager for file access

7. **LockManager**
   - Manages all lock information
   - Tracks granted and waiting locks
   - Used by concurrency control

**Per-Process Memory**:

1. **Work Mem**
   - Memory for sort operations, hash joins, etc.
   - Configured via `work_mem` parameter
   - Allocated per operation, not per connection
   - Can consume significant memory with many concurrent operations

2. **Maintenance Work Mem**
   - Memory for VACUUM, CREATE INDEX, ALTER TABLE
   - Configured via `maintenance_work_mem` parameter
   - Should be higher than work_mem for efficient maintenance

3. **Temp Buffers**
   - Memory for temporary file operations
   - Used when sort/hash exceeds work_mem
   - Limited by `temp_buffers` parameter

4. **Process Memory**
   - Per-connection state, query execution context
   - Includes query parser, planner, executor state
   - Allocated from OS memory, not shared memory

### File System Layout

**Critical Files and Directories**:

1. **Data Directory** (`$PGDATA` or `data_directory`)
   - Root directory for all PostgreSQL data
   - Contains all database objects and configuration
   - Managed by the postmaster process

2. **base Directory**
   - Contains individual database directories
   - Each database has a subdirectory with its files
   - File format: OID-based naming

3. **global Directory**
   - Contains cluster-wide catalog files
   - pg_global: shared system catalogs
   - pg_control: cluster state control file

4. **pg_wal Directory** (formerly pg_xlog)
   - Write-Ahead Log files
   - Segmented into 16MB files
   - Configurable via `wal_segment_size` (must be power of 2)

5. **pg_wal/archive_status Directory**
   - Archive status files for WAL segments
   - .ready: segment ready for archiving
   - .done: segment archived successfully

6. **pg_stat_tmp Directory**
   - Temporary statistics files
   - Written by various processes for monitoring

7. **pg_notify Directory**
   - NOTIFY/LISTEN communication
   - Used by advisory locking mechanisms

8. **pg_multixact Directory**
   - Multi-transaction information
   - Used for FOR SHARE queries and shared row locks

9. **pg_tblspc Directory**
   - Symbolic links to tablespace locations
   - Maps tablespace OIDs to actual paths

10. **postgresql.conf**
    - Main configuration file
    - Contains server configuration parameters

11. **pg_hba.conf**
    - Host-based authentication configuration
    - Controls which clients can connect and how

12. **postgresql.auto.conf**
    - Auto-generated configuration
    - Written by ALTER SYSTEM commands
    - Overrides postgresql.conf

### Data Storage Model

**File Organization**:

1. **Relations (Tables, Indexes, Sequences)**
   - Each relation is stored as one or more files
   - Default format: one file per relation
   - Extension files: .main, .fsm, vm, init
   - TOAST tables: separate storage for oversized columns

2. **Main Relation File**
   - Contains the bulk of the relation data
   - `.main` suffix
   - Organized in 8KB pages

3. **Free Space Map (FSM)**
   - Tracks free space in each page
   - `.fsm` suffix
   - Used by VACUUM to find reusable space

4. **Visibility Map (VM)**
   - Tracks which pages have no dead tuples
   - `.vm` suffix
   - Used by VACUUM to skip pages

5. **Init File**
   - `.init` suffix
   - Marks pages as zeroed (no WAL needed)

6. **TOAST Tables**
   - TOAST = The Oversized-Attribute Storage Technique
   - Stores values that don't fit in the main table row
   - Each table can have one TOAST table
   - Supports compression and out-of-line storage

7. **Storage Manager (SMgr)**
   - Abstracts file operations
   - Opens, reads, writes, and unlinks relation files
   - Provides consistent interface regardless of storage type

8. **Buffer Manager**
   - Manages shared buffer pool
   - Handles buffer allocation, pinning, flushing
   - Uses LRU-like eviction policy
   - Interfaces with SMgr for I/O

9. **Page Format**
   - 8KB per page (8192 bytes)
   - 8 bytes header (LP_OFFSET, LP_ID, PD_LSN, PD_FLAGS, PD_SPECIAL)
   - Offset array for line pointers
   - Item data area
   - Special space at end (for index access methods)

### Transaction Model

**Transaction Processing**:

1. **Transaction IDs (XIDs)**
   - Each transaction gets a unique 32-bit XID
   - Monotonically increasing
   - Wraparound prevention critical for long-running systems
   - `autovacuum_freeze_max_age` defaults to 200 million

2. **Snapshot Isolation**
   - Each transaction sees a consistent snapshot of data
   - Snapshots created at transaction start (READ COMMITTED) or first query (REPEATABLE READ)
   - Based on running transaction list in ProcArray

3. **Multi-Version Concurrency Control (MVCC)**
   - Each tuple has xmin and xmax tracking
   - xmin: transaction that created the tuple
   - xmax: transaction that deleted or updated the tuple
   - Multiple versions of the same tuple can coexist

4. **Write-Ahead Logging (WAL)**
   - All changes are logged before being applied to data pages
   - Ensures crash recovery consistency
   - Supports point-in-time recovery
   - Required for replication

## EDB-Specific Architectural Features

### EDB Postgres Advanced Server Additions

1. **Oracle Compatibility Layer**
   - PL/SQL compatibility
   - Oracle data types (NUMBER, DATE, TIMESTAMP)
   - Oracle functions and packages
   - Oracle-compatible query syntax

2. **EDB Replicator**
   - Enterprise-grade replication solution
   - Supports bidirectional replication
   - Real-time data synchronization
   - Multi-master replication

3. **EDB SpeedDB**
   - Embedded key-value store
   - Integration with PostgreSQL
   - High-performance embedded database

4. **Resource Manager**
   - Query resource management
   - Work group management
   - CPU and memory limits per query group

5. **EDB Accelerator**
   - In-memory acceleration for PostgreSQL
   - Improves read performance for frequently accessed data
   - Cache management and eviction policies

6. **Oracle Call Interface (OCI) Support**
   - OCI-compatible interface for application connectivity
   - Enables Oracle applications to use PostgreSQL

### PostgreSQL 15+ Architectural Changes

1. **Improved Logical Replication**
   - Publication filtering
   - Schema replication
   - Better conflict resolution

2. **Parallel Query Improvements**
   - Parallel index builds
   - Parallel maintenance operations
   - Improved parallel worker management

3. **Enhanced Partitioning**
   - Declarative partitioning (built-in since 10)
   - Automatic partitioning in 15+
   - Better partition pruning

4. **Performance Improvements**
   - Improved parallel seq scan
   - Better join ordering
   - Enhanced sort performance

### PostgreSQL 16+ Architectural Changes

1. **Logical Replication Enhancements**
   - Per-table publication options
   - Better conflict detection
   - Publication-level subscriptions

2. **Resource Management**
   - Work group management
   - Resource consumer groups
   - Query resource limits

3. **Performance Improvements**
   - Improved bitmap scan
   - Better parallel query
   - Enhanced vacuum performance

### PostgreSQL 17+ Architectural Changes

1. **Query Planning Improvements**
   - Better join strategy selection
   - Improved cost estimation
   - Enhanced parallel planning

2. **Replication Improvements**
   - Better logical replication performance
   - Enhanced conflict resolution
   - Improved replication slot management

3. **Operational Improvements**
   - Better monitoring
   - Enhanced statistics
   - Improved maintenance operations

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    PostgreSQL Server                         │
│                                                              │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐              │
│  │ Backend  │    │ Backend  │    │ Backend  │  ...          │
│  │ Process  │    │ Process  │    │ Process  │               │
│  │  #1      │    │  #2      │    │  #N      │               │
│  └────┬─────┘    └────┬─────┘    └────┬─────┘              │
│       │               │               │                     │
│       └───────────────┼───────────────┘                     │
│                       │                                     │
│              ┌────────▼────────┐                            │
│              │   Postmaster    │                            │
│              │   (Main Process)│                            │
│              └────────┬────────┘                            │
│                       │                                     │
│         ┌─────────────┼─────────────┐                       │
│         │             │             │                       │
│    ┌────▼───┐  ┌─────▼────┐  ┌────▼────┐                  │
│    │BgWriter│  │WalWriter │  │Checkpnt │                  │
│    └────────┘  └──────────┘  └─────────┘                  │
│         │             │             │                       │
│    ┌────▼───┐  ┌─────▼────┐  ┌────▼────┐                  │
│    │AutoVac │  │  Archiver │  │ Startup │                  │
│    └────────┘  └──────────┘  └─────────┘                  │
│                                                              │
│  ┌─────────────────────────────────────────────────┐        │
│  │              Shared Memory                      │        │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────┐  │        │
│  │  │Shared Buf│  │  WAL Buf │  │  ProcArray   │  │        │
│  │  └──────────┘  └──────────┘  └──────────────┘  │        │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────┐  │        │
│  │  │ LockMgr  │  │  CLog    │  │ Buffer Index │  │        │
│  │  └──────────┘  └──────────┘  └──────────────┘  │        │
│  └─────────────────────────────────────────────────┘        │
│                                                              │
│  ┌─────────────────────────────────────────────────┐        │
│  │              File System                         │        │
│  │  base/  pg_wal/  pg_tblspc/  global/  ...       │        │
│  └─────────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

## References

- [PostgreSQL Architecture Documentation](https://www.postgresql.org/docs/current/architecture.html)
- [PostgreSQL Internals - Shared Memory](https://www.postgresql.org/docs/current/runtime-config-connection.html)
- [PostgreSQL Internals - WAL](https://www.postgresql.org/docs/current/wal-internals.html)
- [PostgreSQL Internals - MVCC](https://www.postgresql.org/docs/current/mvcc.html)
- [EnterpriseDB Documentation](https://www.enterprisedb.com/docs/)