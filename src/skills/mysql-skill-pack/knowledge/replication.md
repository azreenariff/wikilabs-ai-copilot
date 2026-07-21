# MySQL Replication

## Overview

MySQL replication is a core capability for data redundancy, read scaling, and disaster recovery. It involves copying data modifications from a source (primary) server to one or more replica (secondary) servers. This document covers master-slave replication, Group Replication, MySQL Shell administration, MySQL Router, semi-synchronous replication, GTID-based replication, and related operational considerations.

## Master-Slave Replication (Asynchronous)

The most common and simplest replication topology in MySQL. One source server (master) writes changes to its binary log, and one or more replica servers read these changes and replay them locally.

### Basic Architecture

```
┌──────────────┐     Binary Log      ┌─────────────────┐
│   Source     │ ──────────────────► │  Replica 1       │
│   (Master)   │  ┌──────────────┐   │                  │
│              │  │  binlog      │   │  I/O Thread     │
│  ──────────  │  │  (writes)    │   │  (reads binlog) │
│  Application │  └──────────────┘   │  ──────────     │
│  (writes)    │                      │  Relay Log      │
│              │                      │  ──────────     │
│              │                      │  SQL Thread     │
│              │                      │  (applies SQL)  │
│              │                      │                  │
│              │                      │  Replica 2       │
│              │                      │                  │
└──────────────┘                      └─────────────────┘
```

### Replication Components

**Binary Log (Source)**: The source server records all data-modifying statements to its binary log. Each statement is logged in the format specified by `binlog_format`.

**I/O Thread (Replica)**: Runs on each replica. Connects to the source, requests binary log events, and writes them to the local relay log.

**Relay Log (Replica)**: Local copy of the binary log events. The SQL thread reads from the relay log and executes the statements locally.

**SQL Thread (Replica)**: Reads events from the relay log and applies them to the local database.

### Replication Formats

| Format | Description | Pros | Cons |
|--------|-------------|------|------|
| STATEMENT | Logs SQL statements | Small binlog, deterministic on same schema | Non-deterministic functions cause inconsistency |
| ROW | Logs row changes | Most reliable, handles all cases | Large binlog, complex to read |
| MIXED | Defaults to STATEMENT, switches to ROW for safety | Balance of both | May still have edge cases |

**Recommendation**: Use ROW format for all production replication. It provides the most reliable replication and is the default in MySQL 8.0+.

### GTID-Based Replication

Global Transaction Identifiers (GTIDs) provide a transaction-level tracking mechanism that simplifies replication management.

**Setup**:

```
# On source
gtid_mode=ON
enforce_gtid_consistency=ON
log_bin=ON
server_id=<unique_number>

# On replica
gtid_mode=ON
enforce_gtid_consistency=ON
relay_log_recovery=ON
```

**Benefits**:
- Automatic position tracking (no file:position pairs)
- Simplified failover and failback
- Easier replication diagnostics
- Consistent recovery after restarts
- Eliminates binlog file:position coordination issues

**GTID State Variables**:

| Variable | Description |
|----------|-------------|
| gtid_mode | Current GTID mode status |
| gtid_executed | GTIDs executed on this server |
| gtid_purged | GTIDs purged from binary log |
| gtid_owned | Currently executing GTIDs |
| gtid_current_pos | Current GTID position in replication stream |

**GTID-Based Replication Setup**:

```sql
-- Source configuration
CHANGE REPLICATION SOURCE TO
  SOURCE_HOST='source_host',
  SOURCE_PORT=3306,
  SOURCE_USER='repl_user',
  SOURCE_PASSWORD='secret',
  SOURCE_AUTO_POSITION=1;

-- Start replica
START REPLICA;

-- Check status
SHOW REPLICA STATUS\G
```

### Semi-Synchronous Replication

Standard asynchronous replication has a risk of data loss if the source crashes before the replica has received the changes. Semi-synchronous replication mitigates this:

**How it works**: The source waits for acknowledgment from at least one replica before committing a transaction.

**Parameters**:

```
# On source
plugin_load_add="semisync_master.so"
rpl_semi_sync_master_enabled=ON
rpl_semi_sync_master_timeout=1000  # ms to wait before falling back to async

# On replica
plugin_load_add="semisync_replica.so"
rpl_semi_sync_replica_enabled=ON
```

**Trade-off**: Slightly higher commit latency but reduced data loss risk. Falls back to asynchronous if the replica becomes unresponsive.

**Monitoring Semi-Sync**:

```sql
-- Check semi-sync status on source
SHOW VARIABLES LIKE 'rpl_semi_sync_master_enabled';
SHOW STATUS LIKE 'Rpl_semi_sync_master_status';
SHOW STATUS LIKE 'Rpl_semi_sync_master_clients';

-- Check semi-sync status on replica
SHOW VARIABLES LIKE 'rpl_semi_sync_replica_enabled';
SHOW STATUS LIKE 'Rpl_semi_sync_replica_status';
```

### Monitoring Replication

**Key status fields**:

| Field | Meaning | Healthy Range |
|-------|---------|---------------|
| Seconds_Behind_Source | Replication lag in seconds | < 30 (production) |
| Last_Error | Last replication error | Empty |
| Last_SQL_Errno | Last SQL error code | 0 |
| Slave_IO_Running | IO thread status | Yes |
| Slave_SQL_Running | SQL thread status | Yes |

**Monitoring Commands**:

```sql
SHOW REPLICA STATUS\G
SHOW BINARY LOGS
SHOW BINARY LOG STATUS
```

### Multi-Source Replication

MySQL supports multi-source replication — a single replica receiving streams from multiple sources.

**Configuration**:

```sql
-- Each source needs a unique connection name
CHANGE REPLICATION SOURCE TO SOURCE_CONNECTION_NAME='source1'
  FOR CHANNEL 'source1'
  SOURCE_HOST='host1' SOURCE_USER='repl' SOURCE_AUTO_POSITION=1;

CHANGE REPLICATION SOURCE TO SOURCE_CONNECTION_NAME='source2'
  FOR CHANNEL 'source2'
  SOURCE_HOST='host2' SOURCE_USER='repl' SOURCE_AUTO_POSITION=1;

START REPLICA FOR CHANNEL 'source1';
START REPLICA FOR CHANNEL 'source2';
```

**Use Cases**:
- Aggregating data from multiple application databases
- Geo-distributed replicas
- Consolidating MySQL shards

### Common Replication Issues

**Replication Lag**:

| Cause | Detection | Mitigation |
|-------|-----------|-----------|
| Large transactions on source | Large GTID sets in binary log | Split large transactions |
| Insufficient replica resources | High CPU, disk I/O on replica | Scale up replica |
| Network latency | High network latency between source/replica | Optimize network path |
| Long-running queries on replica | Slow queries visible on replica | Optimize replica queries |
| Lock contention on replica | Lock waits on replica | Reduce write contention |

**Mitigation via Parallel Replication**:

```ini
# Enable parallel replication (MySQL 8.0)
slave_parallel_workers = 8
slave_parallel_type = LOGICAL_CLOCK
```

**Parallel Replication Types**:

| Type | Parallelism Method | Requirements |
|------|-------------------|-------------|
| DATABASE (deprecated) | By database | Not recommended |
| LOGICAL_CLOCK | By transaction dependency | Default in 8.0, requires GTID |
| COMMIT_ORDER | By commit order | MySQL 8.0.22+ |

### Delayed Replication

Create replicas with intentional lag for protection against accidental data loss:

```sql
CHANGE REPLICATION SOURCE TO SOURCE_DELAY=3600
  FOR CHANNEL 'delayed_replica'
  SOURCE_HOST='source' SOURCE_USER='repl' SOURCE_AUTO_POSITION=1;

START REPLICA FOR CHANNEL 'delayed_replica';
```

**Use Cases**:
- Protection against accidental DROP TABLE (can restore from delayed replica)
- Testing time-dependent queries
- Point-in-time recovery scenarios

## Group Replication

MySQL Group Replication (GRR) provides a multi-primary, consensus-based replication framework that ensures high availability and automatic failover.

### Architecture

Group Replication uses the Paxos consensus algorithm to coordinate writes across all group members. Every write must be agreed upon by a quorum of members before being committed.

```
┌──────────┐    ┌──────────┐    ┌──────────┐
│ Member 1 │    │ Member 2 │    │ Member 3 │
│  Primary │    │  Primary │    │  Primary │
│          │◄──►│          │◄──►│          │
│ MySQL    │    │ MySQL    │    │ MySQL    │
└──────────┘    └──────────┘    └──────────┘
     ▲
     │
  MySQL Router
     │
     ▼
  Applications
```

### Single-Primary vs Multi-Primary Mode

| Mode | Writes | Failover | Use Case |
|------|--------|----------|----------|
| Single-primary | One member handles all writes | Automatic | Traditional master-slave HA |
| Multi-primary | All members accept writes | Member failover, not primary swap | Multi-site deployment |

**Recommendation**: Use single-primary mode for most deployments. Multi-primary mode has higher complexity and conflict detection overhead.

### Group Replication Components

1. **Consensus Layer**: Paxos-based consensus protocol for ordering and committing transactions
2. **Certification Layer**: Optimistic concurrency control — checks for conflicts before commit
3. **Recovery Service**: New members catch up by receiving state transfers from existing members
4. **Membership Service**: Manages group membership changes (add/remove members)

### Group Replication Setup

**Prerequisites**:
- MySQL 5.7+ or 8.0+
- GTID-based replication
- Unique server_id for each member
- binlog_row_image=FULL (for change tracking)

**Key configuration**:

```
# Required on all members
gtid_mode=ON
enforce_gtid_consistency=ON
binlog_row_image=FULL
server_uuid=<unique_uuid>
binlog_checksum=NONE  # or CRC32

# Group Replication plugin
plugin_load_add='group_replication.so'
group_replication_group_name="<UUID>"
group_replication_local_address="host:port"
group_replication_group_seeds="host1:port,host2:port,host3:port"
group_replication_bootstrap_group=OFF
```

**Bootstrap the first member**:

```sql
-- Set bootstrap group on first member only
SET GLOBAL group_replication_bootstrap_group=ON;
START GROUP_REPLICATION;
SET GLOBAL group_replication_bootstrap_group=OFF;

-- Add additional members
START GROUP_REPLICATION;
```

### Monitoring Group Replication

```sql
# Check group status
SELECT * FROM performance_schema.replication_group_members;

# Check member membership
SELECT MEMBER_ID, MEMBER_HOST, MEMBER_PORT, MEMBER_ROLE, MEMBER_STATE
FROM performance_schema.replication_group_members;
```

**Member states**:
- **ONLINE**: Fully participating member
- **RECOVERING**: Catching up via state transfer
- **ERROR**: Member has problems (network, disk, etc.)
- **OFFLINE**: Member is offline

**Group Replication Limitations**:

| Limitation | Description | Workaround |
|-----------|-------------|-----------|
| Conflict resolution | Multi-primary conflicts cause rollback | Single-primary mode or conflict-aware application |
| Quorum requirement | Majority needed for writes | Odd number of members (3, 5) |
| Performance | Higher write latency due to consensus | Single-primary reduces conflict overhead |
| Complexity | More complex to set up and manage | Use MySQL Shell/InnoDB Cluster |
| Not for read scaling | All members are write-capable | Use ProxySQL for read distribution |

### Group Replication Failure Modes

| Failure | Symptoms | Recovery |
|---------|----------|----------|
| Network partition | Members go to ERROR or OFFLINE | Fix network, wait for rejoin |
| Disk full | Members can't write | Free disk, restart group replication |
| Too many errors | Member self-removed from group | Investigate root cause, rejoin |
| Quorum loss | Group stops accepting writes | Restore majority of members |
| Slow member recovery | Member stuck in RECOVERING | Check I/O, increase state transfer limits |

## MySQL Shell

MySQL Shell is a programmable client for MySQL that supports SQL, JavaScript, and Python. It is the primary tool for administering InnoDB Cluster and Group Replication.

### Capabilities

1. **InnoDB Cluster management**: Create, configure, and manage database clusters
2. **Group Replication administration**: Configure and monitor Group Replication
3. **Metadata Management**: Manage InnoDB Cluster metadata
4. **Schema management**: Schema-level operations
5. **Data loading**: Bulk data import with various formats

### InnoDB Cluster Architecture

An InnoDB Cluster consists of:
- **Database Servers**: MySQL servers with Group Replication configured
- **MySQL Shell**: Administration tool
- **MySQL Router**: Transparent proxy for application connections

```
┌─────────────────────────────────────────┐
│            InnoDB Cluster                │
│                                          │
│  ┌────────┐  ┌────────┐  ┌────────┐    │
│  │  MGR 1 │  │  MGR 2 │  │  MGR 3 │    │
│  └────────┘  └────────┘  └────────┘    │
│       ▲          ▲          ▲            │
│       └──────────┼──────────┘            │
│                  │                        │
│           MySQL Router                     │
│         (read/write routing)              │
└─────────────────────────────────────────┘
         ▲
         │
   Applications
```

### Common MySQL Shell Operations

**Create a cluster**:

```javascript
// Connect to a server
mysqlsh root@host:3306

// Create cluster
dba.createCluster("myCluster")

// Add instances
cluster.addInstance("root@host2:3306")
cluster.addInstance("root@host3:3306")

// Check cluster status
cluster.status()
```

**Manage cluster**:

```javascript
// Check cluster health
cluster.status()

// Resume cluster after downtime
cluster.resume()

// Remove instance
cluster.removeInstance("root@host:3306")

// Resize cluster
cluster.rejoinCluster()
```

**Cluster configuration options**:

```javascript
// Create cluster with specific options
var cluster = dba.createCluster("myCluster", {
  replicaSet: 'myReplicaSet',
  multiPrimary: false,
  encryption: 'required'
});

// Set cluster instance options
cluster.reconfigureInstance("root@host:3306", {
  autoPortFw: true,
  waitForRestore: true
});
```

### MySQL Shell Modes

| Mode | Language | Use Case |
|------|----------|----------|
| SQL | SQL | Standard SQL operations |
| JavaScript | JavaScript | InnoDB Cluster management, automation |
| Python | Python | InnoDB Cluster management, automation |

**Recommendation**: Use JavaScript mode for cluster management (primary interface) and SQL mode for day-to-day database operations.

## MySQL Router

MySQL Router is a lightweight middleware that transparently routes application connections to MySQL servers. It works with InnoDB Cluster to provide automatic failover and read/write splitting.

### Connection Routing

- **Read/Write routing**: Routes write connections to the primary and read connections to replicas
- **Failover transparency**: Applications reconnect transparently when the primary changes
- **Metadata caching**: Maintains metadata about cluster topology without constant queries

### Deployment

```ini
[router]
bind_address=0.0.0.0
bind_port=6446
user=mysqlrouter

# Default cluster routing
[default]
plugin_dir=/usr/lib/mysqlrouter
metadata_cache_ssl=DISABLED
logging_folder=/var/log/mysqlrouter
connect_timeout=15
```

### Routing Modes

| Mode | Behavior |
|------|----------|
| read-write | Route writes to primary, reads to replicas |
| read-only | Route all reads to replicas |
| read-only-legacy | Same as read-only but without cluster awareness |
| read-write-legacy | Same as read-write but without cluster awareness |
| route | Manual routing configuration |

### Router Configuration

```bash
# Auto-configure router from cluster
mysqlrouter --configure --conf-user=mysqlrouter \
  --conf-source=cluster_config.cnf \
  --conf-use-hostname \
  --name=mysqlrouter

# Start router
mysqlrouter -c /etc/mysqlrouter/mysqlrouter.cnf &

# Check router status
mysqlrouter --status
```

## Replication Topology Comparison

| Topology | Write Capacity | Read Scaling | HA | Data Loss Risk | Complexity |
|----------|---------------|-------------|-----|---------------|------------|
| Master-Slave | Single master | Yes (scale replicas) | Manual failover | High (async) | Low |
| Master-Slave + Semi-Sync | Single master | Yes | Manual failover | Low | Medium |
| Group Replication (Single-Primary) | Single primary | Yes (all members) | Automatic failover | None (with quorum) | High |
| Group Replication (Multi-Primary) | All members | Yes (all members) | Member failover | None (with quorum) | Very High |
| InnoDB Cluster | Single primary | Yes | Automatic | None | Medium (with Router) |

## Replication Best Practices

1. **Use GTIDs** for simplified management and recovery
2. **Enable binary logging** on source for replication and PITR
3. **Monitor lag continuously** — set alerts for lag exceeding thresholds
4. **Use ROW format** for reliable replication
5. **Consider semi-sync** for production HA requirements
6. **Scale replicas horizontally** — add more replicas for read-heavy workloads
7. **Regularly test failover** — verify HA procedures work
8. **Use MySQL Router** for transparent application routing with InnoDB Cluster
9. **Configure relay_log_recovery** — automatically re-execute relay logs lost during crash
10. **Monitor replication threads** — both IO and SQL threads must be running
11. **Use parallel replication** — enable `slave_parallel_workers` for write-heavy sources
12. **Deploy delayed replicas** — for accidental data loss protection
13. **Secure replication** — use SSL/TLS for replication connections
14. **Document topology** — maintain replication diagram and configuration records

## Replication Failure Recovery

### Common Failure Scenarios

| Scenario | Cause | Recovery |
|----------|-------|----------|
| Missing binlog on source | Binlog expired before replica reads | Use GTID to find new position, CHANGE REPLICATION SOURCE TO |
| Data inconsistency | Manual modification on replica | pt-table-checksum, pt-table-sync, or rebuild replica |
| Replication lag spike | Large transaction or replica overload | Identify cause, optimize, scale replica |
| Deadlock on replica | Different transaction ordering | Skip error or fix underlying query |
| Network partition | Source unreachable | Wait for recovery, resume replication |
| Replica restart during relay gap | Server restart lost relay log state | relay_log_recovery=ON, restart replication |
| Schema mismatch | DDL on source not applied to replica | Apply missing DDL to replica |

### GTID-Based Failover

```sql
-- Check current primary
SELECT * FROM performance_schema.replication_group_members 
WHERE MEMBER_ROLE='PRIMARY';

-- Promote new primary (single-primary mode)
mysqlsh --uri root@new_primary:3306
\cluster switchToInstance new_primary:3306

-- Update Router configuration
mysqlrouter --configure --conf-user=mysqlrouter

-- Update application connection string
# Point to new primary through MySQL Router
```

## Replication Monitoring Best Practices

**Key Metrics to Monitor**:

| Metric | Alert Threshold | Impact |
|--------|----------------|--------|
| Seconds_Behind_Source | > 30 seconds | Data staleness on reads |
| Slave_IO_Running | = No | No replication occurring |
| Slave_SQL_Running | = No | Replication halted |
| Last_Error | Non-empty | Replication error to investigate |
| Relay_Log_Space | Rapidly increasing | IO thread falling behind |
| Binlog_Files | > 20 | May indicate purge not configured |

**Monitoring Implementation**:

```sql
-- Quick replication health check
SELECT 
  slave_io_running, 
  slave_sql_running, 
  seconds_behind_source, 
  last_errno, 
  last_error
FROM information_schema.processlist 
WHERE command = 'Connect' AND user LIKE '%repl%';

-- Check all replicas status
SELECT 
  r.channel_name,
  r.source_host,
  r.state,
  r.seconds_behind_source,
  r.last_error
FROM performance_schema.replication_connection_status r;
```

## Version-Specific Considerations

### MySQL 8.0 Replication

- GTID-based replication is the recommended approach
- SHOW REPLICA STATUS replaces SHOW SLAVE STATUS
- LOGICAL_CLOCK parallel replication available
- InnoDB Cluster and Group Replication matured
- MySQL Router handles failover transparency

### MySQL 8.4 Replication

- Enhanced performance schema for replication monitoring
- Improved group replication with better performance
- Better replication lag detection and reporting
- In-place upgrade from 8.0 supported
- Enhanced metadata locking reduces replication delays

## References

- [MySQL 8.0 Reference Manual: Replication](https://dev.mysql.com/doc/refman/8.0/en/replication.html)
- [MySQL 8.0 Reference Manual: Group Replication](https://dev.mysql.com/doc/refman/8.0/en/group-replication.html)
- [MySQL 8.0 Reference Manual: InnoDB Cluster](https://dev.mysql.com/doc/refman/8.0/en/inno-db-cluster.html)
- [MySQL 8.0 Reference Manual: MySQL Shell](https://dev.mysql.com/doc/refman/8.0/en/mysql-shell.html)
- [MySQL 8.0 Reference Manual: MySQL Router](https://dev.mysql.com/doc/refman/8.0/en/mysql-router.html)
- [MySQL 8.0 Reference Manual: Semi-Synchronous Replication](https://dev.mysql.com/doc/refman/8.0/en/replication-semisync.html)
- [MySQL 8.0 Reference Manual: Replication Formats](https://dev.mysql.com/doc/refman/8.0/en/replication-formats.html)
- [MySQL 8.0 Reference Manual: Replication Solution Guide](https://dev.mysql.com/doc/refman/8.0/en/replication-solutions.html)
- [MySQL 8.0 Reference Manual: Multi-Source Replication](https://dev.mysql.com/doc/refman/8.0/en/replication-multi-source.html)
- [MySQL 8.0 Reference Manual: Delayed Replication](https://dev.mysql.com/doc/refman/8.0/en/replication-delayed.html)