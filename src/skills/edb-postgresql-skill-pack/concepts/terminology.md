# PostgreSQL Terminology

## Glossary of PostgreSQL and EDB Terms

### Core Concepts

| Term | Definition |
|------|------------|
| **Postmaster** | The main PostgreSQL server process that manages child processes and shared memory |
| **Backend** | A process handling an individual client connection |
| **Relation** | A database table, view, or sequence |
| **Tuple** | A row in a table |
| **Page** | An 8KB block of data on disk |
| **LSN** | Log Sequence Number — a position identifier in the WAL |
| **XID** | Transaction ID — a unique identifier for each transaction |
| **Snapshot** | A consistent view of the database at a point in time |

### WAL Terms

| Term | Definition |
|------|------------|
| **WAL** | Write-Ahead Log — the log of all changes before they are applied to data files |
| **WAL Segment** | A 16MB WAL file |
| **WAL Record** | An entry in the WAL describing a change |
| **Checkpoint** | A point where all dirty buffers are flushed and a record is written to WAL |
| **WAL Archiving** | Copying completed WAL segments to archive storage |
| **WAL Level** | Minimum amount of information written to WAL (none/replica/logical) |
| **WAL Receiver** | Process on standby that receives WAL from primary |
| **WAL Sender** | Process on primary that sends WAL to standbys |

### Replication Terms

| Term | Definition |
|------|------------|
| **Primary** | The server accepting writes |
| **Standby** | A replica server that receives WAL from primary |
| **Streaming Replication** | Physical replication via WAL streaming |
| **Logical Replication** | Replication at the SQL change level |
| **Replication Slot** | A mechanism to prevent WAL removal for a standby |
| **Publication** | A set of tables published for logical replication |
| **Subscription** | A configuration for receiving replicated data |
| **Logical Decoding** | Extracting change events from the WAL |
| **pg_rewind** | Tool to sync a diverged primary after failover |
| **pg_basebackup** | Tool to create a physical backup of the cluster |

### MVCC Terms

| Term | Definition |
|------|------------|
| **MVCC** | Multi-Version Concurrency Control — allows concurrent reads and writes |
| **xmin** | Transaction ID that created a tuple version |
| **xmax** | Transaction ID that deleted or updated a tuple version |
| **Visibility** | Whether a tuple version is visible to a specific transaction |
| **Dead Tuple** | A tuple version that is no longer visible to any transaction |
| **Vacuum** | Process of reclaiming space from dead tuples |
| **Freeze** | Special update that replaces xmin/xmax with special values |
| **Transaction ID Wraparound** | Condition when XID space is exhausted |

### Index Terms

| Term | Definition |
|------|------------|
| **B-Tree** | Balanced tree index (default index type) |
| **GiST** | Generalized Search Tree — supports range and spatial queries |
| **GIN** | Generalized Inverted Index — optimized for arrays, JSONB, full-text |
| **BRIN** | Block Range Index — compact index for large ordered tables |
| **Index Scan** | Retrieving data using an index |
| **Bitmap Scan** | Using bitmap to combine multiple index results |
| **Index Bloat** | Excess space in an index due to frequent updates/deletes |
| **Partial Index** | Index on a subset of rows |
| **Expression Index** | Index on a computed expression |

### Lock Terms

| Term | Definition |
|------|------------|
| **Lock** | Mechanism to control concurrent access to resources |
| **Row Exclusive Lock** | Lock acquired by INSERT/UPDATE/DELETE |
| **Share Lock** | Lock for SELECT FOR SHARE |
| **Access Exclusive Lock** | Strongest lock — required by ALTER TABLE, DROP |
| **Deadlock** | Two or more transactions waiting for each other's locks |
| **Lock Timeout** | Maximum time to wait for a lock |
| **Lock Manager** | Component managing all lock operations |

### Configuration Terms

| Term | Definition |
|------|------------|
| **postgresql.conf** | Main configuration file |
| **postgresql.auto.conf** | Auto-generated config from ALTER SYSTEM |
| **pg_hba.conf** | Host-based authentication configuration |
| **ALTER SYSTEM** | Command to set configuration parameters |
| **pg_reload_conf()** | Function to reload configuration without restart |
| **shared_preload_libraries** | Libraries loaded at server start |

### Maintenance Terms

| Term | Definition |
|------|------------|
| **VACUUM** | Reclaim space from dead tuples |
| **VACUUM FULL** | Rebuild entire table (exclusive lock) |
| **ANALYZE** | Update table statistics for query planner |
| **REINDEX** | Rebuild an index |
| **pg_repack** | Third-party tool for online table/index optimization |
| **AutoVacuum** | Automatic vacuum process |

### Performance Terms

| Term | Definition |
|------|------------|
| **shared_buffers** | Main data cache in shared memory |
| **work_mem** | Per-operation memory for sorts and joins |
| **effective_cache_size** | Estimated memory available for caching |
| **Cache Hit Ratio** | Proportion of buffer reads that hit in cache |
| **Seq Scan** | Sequential table scan |
| **Index Scan** | Table access via index |
| **Parallel Query** | Query execution using multiple workers |
| **Hash Join** | Join method using hash table |
| **Merge Join** | Join method using sorted inputs |
| **Nested Loop Join** | Join method using nested iteration |

### EDB-Specific Terms

| Term | Definition |
|------|------------|
| **EDB Postgres Advanced Server** | Enterprise distribution of PostgreSQL by EDB |
| **EDB Replicator** | EDB's enterprise replication solution |
| **EDB SpeedDB** | EDB's embedded key-value store |
| **Oracle Compatibility** | EDB's Oracle-compatible SQL support |
| **Resource Manager** | EDB's query resource management |
| **Patroni** | Open-source HA solution for PostgreSQL clusters |
| **EDB Postgres Operator** | Kubernetes operator for PostgreSQL |

### Backup Terms

| Term | Definition |
|------|------------|
| **pg_dump** | Logical backup tool |
| **pg_restore** | Logical restore tool |
| **pg_basebackup** | Physical backup tool |
| **pg_dumpall** | Backup all databases in a cluster |
| **PITR** | Point-in-Time Recovery |
| **Archive Command** | Command for copying WAL to archive |
| **pg_archivecleanup** | Tool for removing old WAL files |

### Monitoring Terms

| Term | Definition |
|------|------------|
| **pg_stat_activity** | View of current active sessions |
| **pg_stat_database** | Database-level statistics |
| **pg_stat_user_tables** | Table-level statistics |
| **pg_stat_replication** | Replication status |
| **pg_stat_bgwriter** | Background writer statistics |
| **pg_stat_statements** | Query performance extension |
| **pg_stat_archiver** | WAL archiving statistics |
| **pg_stat_user_indexes** | Index usage statistics |

## Version-Specific Terminology

### PostgreSQL 15+
- **REINDEX CONCURRENTLY**: Online index rebuild without exclusive lock
- **Parallel pg_basebackup**: Multiple workers for physical backup
- **Parallel pg_restore**: Multiple workers for logical restore

### PostgreSQL 16+
- **Resource Groups**: Query resource management feature
- **Work Groups**: Groups of queries with shared resource limits
- **Publication Filtering**: Table-level filtering in logical replication

### PostgreSQL 17+
- **Parallel Planning**: Parallel query planning optimization
- **Enhanced Logical Replication**: Improved logical replication performance
- **Improved Cost Estimation**: Better query plan cost estimation

## References

- [PostgreSQL Glossary](https://www.postgresql.org/docs/current/glossary.html)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [EDB Documentation](https://www.enterprisedb.com/docs/)