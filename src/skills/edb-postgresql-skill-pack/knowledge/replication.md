# PostgreSQL Replication

## Overview

PostgreSQL supports two primary replication models: **Streaming (Physical) Replication** and **Logical Replication**. Both are designed for high availability and read scaling, but serve different use cases. EDB Postgres Advanced Server also provides enterprise-grade replication tools including EDB Replicator.

## Streaming (Physical) Replication

### Architecture

Streaming replication works by sending WAL records from the primary server to one or more standby servers in near real-time. The standby servers replay the WAL records to maintain an identical copy of the primary database.

```
┌─────────────────────┐         ┌─────────────────────┐
│    Primary Server   │         │    Standby Server    │
│                     │ WAL     │                     │
│  ┌───────────────┐  │Records│  ┌───────────────┐   │
│  │  WAL Writer   │──┼───────→│  WAL Receiver │   │
│  └───────────────┘  │         │  (wal_receiver)│   │
│                     │         └───────────────┘   │
│  ┌───────────────┐  │              │               │
│  │  Backend Proc │  │              ▼               │
│  └───────────────┘  │  ┌─────────────────────┐   │
│                     │  │  WAL Replayer       │   │
│  ┌───────────────┐  │  │  (XLogReplayer)   │   │
│  │  Checkpointer │  │  └─────────────────────┘   │
│  └───────────────┘  │              │               │
│                     │              ▼               │
└─────────────────────┘         ┌───────────────┐   │
                                │  Standby      │   │
                                │  Backend      │   │
                                └───────────────┘   │
└───────────────────────────────────────────────────┘
```

### Configuration

**Primary Server Settings**:
```ini
# postgresql.conf on primary
wal_level = replica          # Required for streaming replication
max_wal_senders = 10         # Maximum concurrent standby connections
wal_keep_size = 1GB          # Retain at least this much WAL for standbys
archive_mode = on            # Enable WAL archiving
archive_command = 'cp %p /archive/wal/%f'
```

**Standby Server Configuration**:
- PostgreSQL 12+: Create `standby.signal` file in data directory
- Pre-12: Use `recovery.conf` (deprecated)
- Set `primary_conninfo` and `primary_slot_name` in `postgresql.auto.conf`

```ini
# postgresql.auto.conf on standby
primary_conninfo = 'host=primary_host port=5432 user=replicator password=secret'
primary_slot_name = 'standby1_slot'
recovery_target_timeline = 'latest'
hot_standby = on               # Allow queries on standby
```

### Replication Slot Management

Replication slots prevent WAL removal on the primary while needed by standby servers.

**Types**:

1. **Physical Replication Slots**
   - Track the minimum WAL position needed by each standby
   - Prevent WAL removal even if standby is disconnected
   - Created with `pg_create_physical_replication_slot()`
   - Each slot is independent of others

2. **Logical Replication Slots**
   - Track logical decoding progress
   - Used for logical replication or logical decoding
   - Created with `pg_create_logical_replication_slot()`
   - Require `wal_level = logical`

**Slot Monitoring**:
```sql
SELECT slot_name, slot_type, active, restart_lsn,
       pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn) as retained_wal_bytes
FROM pg_replication_slots;
```

**Important**: Inactive slots consume WAL indefinitely, potentially filling disk. Monitor and clean up unused slots.

### Standby Modes

1. **Hot Standby**
   - Read-only queries allowed
   - Most common standby mode
   - `hot_standby = on`
   - Available since PostgreSQL 9.0

2. **Streaming Standby**
   - Receives WAL via streaming
   - Replays WAL continuously
   - Standard mode for most deployments

3. **Logical Standby**
   - Uses logical replication
   - Can transform data during replication
   - More flexible than physical

4. **Physical Standby**
   - Bit-for-bit copy of primary
   - Same filesystem structure
   - Cannot modify data directly

### Synchronous Replication

Ensures data durability by confirming standby has received/flushed WAL before confirming transaction.

```ini
# Primary configuration
synchronous_commit = on
synchronous_standary = 'standby1, standby2'  # Or 'FIRST n (standby1, standby2)'
```

**Commit Levels**:
- `local`: WAL written locally only (fastest)
- `remote_write`: WAL sent to standby, standby receives it
- `on`: WAL sent and flushed on standby (default for sync)
- `remote_apply`: WAL applied and replayed on standby (slowest)
- `off`: No synchronous commit

**Note**: With `synchronous_standary` set, if any standby is unreachable, the primary will wait indefinitely. Use `synchronous_standary = 'FIRST n (...)'` with n < total standbys for fault tolerance.

### Failover Procedures

**Manual Failover**:
1. Verify old primary is truly unreachable
2. Promote standby: `pg_ctl promote -D <data_dir>`
3. Update application connection strings
4. Optionally configure old primary as new standby (using pg_rewind)

**Automated Failover** (EDB):
- EDB Postgres Operator
- Patroni
- EDB Replicator
- Custom monitoring + promotion scripts

**Post-Failover**:
1. Run `pg_rewind` on old primary if it had accepted writes
2. Configure old primary as new standby
3. Verify replication is flowing correctly

## Logical Replication

### Architecture

Logical replication works at the SQL statement level, replicating individual changes (INSERT, UPDATE, DELETE) rather than WAL records. This provides more flexibility than physical replication.

```
┌───────────────────────────────────────────────────────────────┐
│                       Source (Publisher)                       │
│                                                               │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐    │
│  │  Users Table  │  │ Orders Table  │  │ Products Table│    │
│  └───────┬───────┘  └───────┬───────┘  └───────┬───────┘    │
│          │                  │                   │              │
│          └──────────────────┼───────────────────┘              │
│                             ▼                                  │
│                   ┌─────────────────┐                         │
│                   │  Publication    │                         │
│                   │  (my_pub)       │                         │
│                   └────────┬────────┘                         │
└────────────────────────────┼──────────────────────────────────┘
                             │  pgoutput
                             │  (Logical Decoding)
┌────────────────────────────┼──────────────────────────────────┐
│                       Target (Subscriber)                       │
│                                                               │
│                   ┌─────────────────┐                         │
│                   │  Subscription   │                         │
│                   │  (my_sub)       │                         │
│                   └────────┬────────┘                         │
│                            │                                  │
│          ┌─────────────────┼─────────────────┐                │
│          ▼                 ▼                 ▼                 │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐    │
│  │  Users Table  │  │ Orders Table  │  │ Products Table│    │
│  └───────────────┘  └───────────────┘  └───────────────┘    │
└───────────────────────────────────────────────────────────────┘
```

### Publication Configuration (Publisher Side)

**Create Publication**:
```sql
-- Publish all tables
CREATE PUBLICATION my_pub FOR ALL TABLES;

-- Publish specific tables
CREATE PUBLICATION my_pub FOR TABLE users, orders, products;

-- Publish with filters
CREATE PUBLICATION my_pub FOR TABLE users
    WHERE (id > 1000);

-- Control which operations are published
CREATE PUBLICATION my_pub FOR TABLE users
    WITH (publish = 'insert, update, delete');
-- Options: insert, update, delete, truncate (default: all enabled)
```

**Publication Settings**:
- `publish_via_partition_root`: Whether to replicate via partition root (default: false)
- `insert`, `update`, `delete`, `truncate`: Control which DML operations are published

**Alter Publication**:
```sql
ALTER PUBLICATION my_pub ADD TABLE new_table;
ALTER PUBLICATION my_pub DROP TABLE old_table;
ALTER PUBLICATION my_pub SET (publish_via_partition_root = true);
```

### Subscription Configuration (Subscriber Side)

**Create Subscription**:
```sql
CREATE SUBSCRIPTION my_sub
    CONNECTION 'host=pub-host dbname=mydb user=replicator password=secret'
    PUBLICATION my_pub
    WITH (
        copy_data = true,        -- Copy existing data on creation
        create_slot = true,      -- Create replication slot
        slot_name = 'my_sub_slot',
        plugin = 'pgoutput',     -- Output plugin
        synchronous_commit = 'on',
        link_copy_worker_owner = true
    );
```

**Subscription Settings**:
- `copy_data`: Copy existing data when subscription created (default: true)
- `create_slot`: Create replication slot automatically (default: true)
- `slot_name`: Name of replication slot (default: derived from subscription name)
- `plugin`: Output plugin for logical decoding (default: pgoutput)
- `synchronous_commit`: Commit level for replication (default: on)
- `connected`: Whether subscriber is connected
- `link_copy_worker_owner`: Allow copy worker to use same owner as subscriber

**Alter Subscription**:
```sql
ALTER SUBSCRIPTION my_sub SET (copy_data = false);
ALTER SUBSCRIPTION my_sub REFRESH PUBLICATION;
ALTER SUBSCRIPTION my_sub DISABLE;
ALTER SUBSCRIPTION my_sub ENABLE;
```

**Drop Subscription**:
```sql
DROP SUBSCRIPTION my_sub;
-- Also drops the associated replication slot
```

### Conflict Resolution

Logical replication must handle conflicts when the subscriber receives changes that conflict with local modifications.

**Conflict Resolution Options**:

1. **Ignore Conflict** (default)
   - The conflicting change is discarded
   - Publisher's version is applied

2. **Overwrite Conflict**
   - Subscriber's version is kept
   - Publisher's change is discarded
   - Not recommended for production use

**Row-Conflict Resolution Functions**:
```sql
CREATE OR REPLACE FUNCTION my_conflict_resolver()
RETURNS conflict_resolver AS $$
BEGIN
    RETURN 'overwrite';
END;
$$ LANGUAGE plpgsql;

ALTER SUBSCRIPTION my_sub
    SET (conflict_resolver = 'my_conflict_resolver');
```

### Logical Replication Limitations

1. **No support for TRUNCATE** (by default, must enable in publication)
2. **No schema replication** (DDL not replicated)
3. **No support for partitioned tables replication** (limited support in PG 14+)
4. **No support for foreign key constraints replication**
5. **No support for serial columns replication** (identity columns supported in PG 10+)
6. **No support for concurrent modifications** on replicated tables
7. **Single subscriber per publication** per table
8. **Cannot replicate system catalogs**

### Logical Replication Monitoring

```sql
-- Check subscription status
SELECT subname, subenabled, subconninfo, subpublications,
       subslot_name, subfailover
FROM pg_subscription;

-- Check replication lag
SELECT pubname, relname, n_live_tup, n_dead_tup
FROM pg_publication_tables;

-- Check publisher-side slot status
SELECT slot_name, slot_type, active, restart_lsn,
       pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn) as retained_bytes
FROM pg_replication_slots
WHERE slot_name LIKE 'pg_%';
```

## Physical Replication with pg_basebackup

### Creating a Standby via pg_basebackup

**Basic Standby Setup**:
```bash
# 1. Create replication user on primary
psql -c "CREATE ROLE replicator WITH REPLICATION LOGIN PASSWORD 'secret';"
psql -c "GRANT pg_read_all_data TO replicator;"  # PostgreSQL 16+

# 2. Create replication slot
psql -c "SELECT pg_create_physical_replication_slot('standby1_slot');"

# 3. Ensure pg_hba.conf allows replication connections
# host replication replicator <standby-ip>/32 scram-sha-256

# 4. Take base backup
pg_basebackup -h primary_host -U replicator \
    -D /var/lib/pgsql/data_standby \
    -P -R -X fetch -C -S standby1_slot

# -R: Creates standby.signal and postgresql.auto.conf
# -X fetch: Include WAL files in backup
# -P: Show progress
# -C: Create replication slot (redundant if slot already created)
# -S: Name of the replication slot

# 5. Start standby
pg_ctl start -D /var/lib/pgsql/data_standby
```

### pg_basebackup Options

| Option | Description |
|--------|-------------|
| `-D` | Target directory for backup |
| `-Fp` | Plain format (default) |
| `-Fd` | Directory format (for pg_rewind) |
| `-Ft` | Tar format |
| `-z` | Compress output (gzip) |
| `-Z` | Compress level (0-9) |
| `-X` | WAL method: fetch, stream, or none |
| `-P` | Show progress |
| `-R` | Create standby.signal and postgresql.auto.conf |
| `-S` | Replication slot name |
| `-c` | Compression mode: fast, on, off |
| `-l` | Label for the backup |
| `-n` | No overwrite (fail if directory exists) |
| `-N` | Do not create replication slot |

## pg_rewind for Post-Failover Recovery

pg_rewind synchronizes a primary server to match another primary after they have diverged. This allows the old primary to become a standby of the new primary after a failover.

**Prerequisites**:
1. `wal_log_hints = on` in postgresql.conf
2. Or `data_checksums = on` during cluster initialization
3. Both servers must be the same major version

**Usage**:
```bash
# 1. Ensure old primary is stopped
pg_ctl stop -D /var/lib/pgsql/data

# 2. Run pg_rewind
pg_rewind -D /var/lib/pgsql/data \
    --source-server='host=new_primary dbname=postgres user=postgres'

# 3. Configure old primary as standby
cat > /var/lib/pgsql/data/postgresql.auto.conf <<EOF
primary_conninfo = 'host=new_primary port=5432 user=replicator password=secret'
primary_slot_name = 'old_primary_slot'
recovery_target_timeline = 'latest'
EOF

# 4. Start old primary as new standby
pg_ctl start -D /var/lib/pgsql/data
```

### pg_rewind vs pg_basebackup for Re-sync

| Scenario | Method | Reason |
|----------|--------|--------|
| Old primary had no writes | pg_rewind | Fast, efficient |
| Old primary had writes | pg_rewind (if wal_log_hints=on) | Only syncs changed blocks |
| Old primary had writes | pg_basebackup (if wal_log_hints=off) | Must resync everything |
| Different major version | pg_basebackup | pg_rewind requires same version |

## EDB-Specific Replication Features

### EDB Replicator

EDB Replicator is EnterpriseDB's enterprise-grade replication solution:

1. **Bidirectional Replication**: Changes flow both ways between databases
2. **Real-Time Synchronization**: Near-zero latency replication
3. **Multi-Master**: Multiple nodes can accept writes
4. **Conflict Resolution**: Built-in conflict detection and resolution
5. **Heterogeneous Replication**: Replicate between PostgreSQL and other databases
6. **Schema Changes**: Automatic DDL replication
7. **Monitoring**: Comprehensive replication monitoring

### EDB Replication Manager

EDB provides tools for managing replication clusters:

1. **Cluster Management**: Automated cluster creation and configuration
2. **Health Monitoring**: Continuous health checks
3. **Failover Automation**: Automated failover procedures
4. **Backup Integration**: Integrated with EDB backup tools

## Replication Best Practices

### Design Principles

1. **Network**: Low-latency, high-bandwidth connection between primary and standby
2. **Hardware**: Standby hardware should match or exceed primary
3. **Storage**: Same I/O characteristics for primary and standby
4. **Monitoring**: Monitor replication lag continuously
5. **Testing**: Test failover procedures regularly
6. **Documentation**: Document topology and procedures

### Monitoring Checklist

- [ ] `pg_stat_replication` shows all standbys streaming
- [ ] Replication lag within SLA thresholds
- [ ] Replication slots are active
- [ ] WAL archiving is functioning
- [ ] Standby is accepting read queries (if hot standby)
- [ ] No split-brain scenarios
- [ ] Sufficient disk space on all nodes

### High Availability Architecture Options

| Architecture | RPO | RTO | Complexity | Cost |
|--------------|-----|-----|------------|------|
| Single Hot Standby | Near-zero | Seconds | Low | Low |
| Multiple Hot Standbys | Near-zero | Seconds | Medium | Medium |
| Synchronous Standby | Zero | Seconds | Medium | Medium |
| Logical Replication | Near-zero | Minutes | Medium | Medium |
| EDB Replicator | Near-zero | Seconds | High | High |
| Patroni + etcd/Consul | Near-zero | Seconds | High | Medium |

## References

- [PostgreSQL Streaming Replication](https://www.postgresql.org/docs/current/runtime-config-wal.html)
- [PostgreSQL Logical Replication](https://www.postgresql.org/docs/current/logical-replication.html)
- [PostgreSQL Replication Architecture](https://www.postgresql.org/docs/current/replication.html)
- [pg_basebackup Documentation](https://www.postgresql.org/docs/current/app-pgbasebackup.html)
- [pg_rewind Documentation](https://www.postgresql.org/docs/current/pgrewind.html)
- [EnterpriseDB Replicator Documentation](https://www.enterprisedb.com/docs/)