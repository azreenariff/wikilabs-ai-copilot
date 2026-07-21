# PostgreSQL Architecture Overview

## Purpose

This document provides a comprehensive overview of PostgreSQL architecture for the EDB PostgreSQL skill pack. It covers the core components, process model, memory architecture, and storage model that form the foundation of PostgreSQL operations.

## Architecture Overview

PostgreSQL follows a multiprocess architecture where each client connection gets its own backend process. This design provides:

1. **Stability**: A crashed backend doesn't take down the server
2. **Security**: Process isolation between connections
3. **Simplicity**: No complex threading model to manage
4. **Reliability**: Shared memory coordination is well-understood

## Process Model

The PostgreSQL process hierarchy includes:

1. **Postmaster**: The parent process managing the cluster
2. **Backend Processes**: One per client connection
3. **Background Writer**: Flushes dirty buffers to disk
4. **WAL Writer**: Writes WAL records
5. **Checkpointer**: Creates checkpoint records
6. **Autovacuum Launcher**: Manages autovacuum workers
7. **Wal Sender**: Sends WAL to standbys
8. **WAL Receiver**: Receives WAL on standbys
9. **Logical Replication Workers**: For logical decoding
10. **Parallel Query Workers**: For parallel operations

## Memory Architecture

PostgreSQL uses two memory areas:

1. **Shared Memory**: Between-process communication (shared_buffers, lock manager, proc array)
2. **Private Memory**: Per-connection memory (work_mem, maintenance_work_mem)

The key parameters are:
- `shared_buffers`: Main data cache (25% of RAM)
- `work_mem`: Per-operation memory (sorts, joins)
- `maintenance_work_mem`: Maintenance operation memory
- `effective_cache_size`: Estimated cache size (75% of RAM)

## Storage Model

PostgreSQL stores data in files organized by database, relation, and type:

1. **base/**: Database directories
2. **pg_wal/**: Write-ahead log files
3. **pg_tblspc/**: Tablespaces
4. **global/**: Cluster-wide catalogs

Each relation (table/index) may have:
- Main file (.main)
- Free Space Map (.fsm)
- Visibility Map (.vm)
- TOAST table for oversized columns

## Key Concepts

- **MVCC**: Multi-Version Concurrency Control for consistent reads
- **WAL**: Write-Ahead Log for crash recovery and replication
- **Buffers**: Shared memory page cache
- **LSN**: Log Sequence Number for WAL position tracking
- **XID**: Transaction ID for MVCC version tracking

## Version Notes

- PostgreSQL 15+: Improved parallel query, REINDEX CONCURRENTLY
- PostgreSQL 16: Resource group management, logical replication improvements
- PostgreSQL 17: Further query optimization and replication improvements

## References

- [PostgreSQL Architecture](https://www.postgresql.org/docs/current/architecture.html)
- [PostgreSQL Internals](https://www.postgresql.org/docs/current/protocol-protocol.html)