# EDB PostgreSQL Architecture Reference

## Overview

This document provides architectural details for EDB PostgreSQL server deployments.

## Core Architecture

### Components

1. **PostgreSQL Server**: Main database engine
2. **Shared Memory**: Shared buffers, WAL buffers, sync file maps
3. **Background Writers**: Checkpoint and background writing
4. **WAL Writer**: Write-ahead log management
5. **Autovacuum Daemon**: Table maintenance and cleanup
6. **Stats Collector**: Statistics gathering
7. **Archiver**: WAL archiving
8. **Replication Sender**: Primary WAL streaming
9. **Replication Receiver**: Standby WAL application

### Memory Architecture

```
┌─────────────────────────────────────┐
│           Client Processes           │
├─────────────────────────────────────┤
│  Shared Memory (Shared Buffers)     │
│  ├─ Data Cache                      │
│  ├─ WAL Buffers                     │
│  ├─ Sync File Maps                  │
│  └─ Statistics                      │
├─────────────────────────────────────┤
│  Backend Process Memory             │
│  ├─ work_mem (sort/hash)            │
│  ├─ maintenance_work_mem            │
│  └─ temp_buffers                    │
└─────────────────────────────────────┘
```

### WAL Architecture

```
PostgreSQL → WAL Writer → WAL Buffers → Checkpointer → WAL Files
                                                    ↓
                                              Archiver → Archive
                                                    ↓
                                              Replication Sender
```

### Replication Architecture

| Type | Description | Use Case |
|------|-------------|----------|
| **Physical Replication** | Byte-level replication | High availability |
| **Logical Replication** | Row-level replication | Schema changes |
| **Streaming Replication** | WAL streaming | Standard replication |
| **Bi-directional** | Two-way replication | Active-active |

## Scale Architecture

| Scale | Connections | Memory | Storage |
|-------|------------|--------|---------|
| Small (<100) | 100-200 | 4-8GB | 100GB |
| Medium (100-500) | 200-500 | 16-32GB | 500GB |
| Large (500+) | 500+ | 64GB+ | 2TB+ |

## References

- EDB PostgreSQL Architecture: https://www.enterprisedb.com/docs/
- PostgreSQL Architecture: https://www.postgresql.org/docs/current/architecture.html
- PostgreSQL Internals: https://www.postgresql.org/docs/current/internals.html