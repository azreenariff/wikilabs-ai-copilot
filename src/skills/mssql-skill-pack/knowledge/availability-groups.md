# Always On Availability Groups

## Overview

Always On Availability Groups (AG) is Microsoft SQL Server's high availability and disaster recovery solution introduced in SQL Server 2012 and significantly enhanced in SQL Server 2014, 2016, 2017, 2019, and 2022. It provides automatic failover for user databases and supports up to eight secondary replicas for each availability group.

The primary benefit of Always On Availability Groups over traditional database mirroring is the ability to:
- Group multiple databases into a single availability group
- Support up to eight secondary replicas per group
- Offer both high availability (synchronous-commit) and disaster recovery (asynchronous-commit)
- Provide readable secondaries for offloading read workloads
- Support failover clustering across data centers

## Components of Always On Availability Groups

### Availability Group

An availability group is a container that holds one or more availability databases. It defines the failure domain and failover behavior.

**Key properties:**
- **Failure domain** — All replicas in the same group fail together
- **Minimum replicas** — Two replicas required (one primary, one secondary)
- **Maximum replicas** — Eight replicas total (one primary + seven secondaries)
- **Availability databases** — User databases within the group

### Availability Databases

Availability databases are user databases that participate in the availability group. Each database must:
- Be on a different server instance (cannot span multiple databases on same instance)
- Have a full backup taken before adding to AG
- Be in restoring state on secondary replicas initially
- Synchronize via log shipping mechanism

### Availability Replicas

Availability replicas define the servers that participate in the availability group. Each replica has:

- **Role** — Primary or Secondary
- **Availability mode** — Synchronous-commit or Asynchronous-commit
- **Failover mode** — Automatic, Manual, or External
- **Connection settings** — Allow connections to readable secondaries
- **Endpoint URL** — Network endpoint for replica communication

### Availability Group Listener

The availability group listener provides a single connection point for clients to connect to the primary replica. It is configured with:
- **DNS name** — Static DNS name for the AG listener
- **Port** — Default port 1433
- **Network** — Subnet-specific DNS records for multi-subnet scenarios
- **Static IP** — For multi-subnet failover, static IP addresses per subnet

## Replica Types and Modes

### Synchronous-Commit Mode

In synchronous-commit mode, the primary replica waits for each secondary replica to harden the transaction log before confirming the commit to the client. This ensures zero data loss but can introduce latency.

**Characteristics:**
- **Zero data loss** — All committed transactions are hardened on both replicas
- **Commit latency** — Primary waits for secondary acknowledgment
- **Automatic failover** — Supported when failover mode is Automatic
- **Network requirements** — Low-latency network (< 5ms RTT recommended)
- **Performance impact** — Can impact write performance on primary

**Use cases:**
- High availability requiring zero data loss
- Same data center deployments
- Latency-tolerant write workloads

### Asynchronous-Commit Mode

In asynchronous-commit mode, the primary replica does not wait for secondary replicas to acknowledge log records. This provides better write performance but introduces the possibility of data loss during failover.

**Characteristics:**
- **Potential data loss** — Transactions not yet hardened on secondary may be lost
- **No commit latency** — Primary does not wait for secondary
- **Manual failover** — Only manual failover supported (Automatic not supported)
- **Network flexibility** — Can span data centers or regions
- **Performance** — Better write performance on primary

**Use cases:**
- Disaster recovery across data centers
- High-latency network connections
- Read-heavy workloads where secondary can serve stale data

### Read-Only Routing

Read-only routing allows read-only queries to be directed to secondary replicas, offloading read workloads from the primary.

**Configuration:**
```sql
ALTER AVAILABILITY GROUP [MyAG]
MODIFY REPLICA ON 'Server1'
WITH (SECONDARY_ROLE (
    READ_ONLY_ROUTING_URL = 'TCP://Server1.domain.com:1433'
));
```

**Routing list:**
- Define a prioritized list of secondaries for read-only routing
- Client must use `ApplicationIntent=ReadOnly` in connection string
- Router resolves to the next available secondary in the list

## Failover Behavior

### Automatic Failover

Automatic failover occurs when the primary replica becomes unavailable and a secondary replica with all data synchronized takes over as primary. This requires:

- **Synchronous-commit mode** — Only synchronous secondaries qualify
- **Automatic failover mode** — Failover mode set to Automatic
- **Majority of replicas** — At least one other secondary is available
- **Data synchronization** — Secondary is caught up with primary

**Failure detection:**
- **Heartbeat mechanism** — Replicas monitor each other via TCP heartbeats
- **Quorum** — Majority of replicas must agree on failure
- **Timeout** — Configurable failure detection timeout

### Manual Failover

Manual failover is initiated by the DBA through T-SQL or SSMS. This can be used for:
- Planned maintenance
- Testing failover behavior
- Asynchronous replicas (where automatic failover is not supported)

**Preconditions:**
- All databases must be synchronized on the target secondary
- No pending transactions on the primary
- Database must be in RESTORING state with NORECOVERY on secondary

### Forced Failover

Forced failover is used in emergency situations where the primary is irrecoverable:
- **Forced Failover with Data Loss** — Secondary becomes primary without catching up
- **Forced No Data Loss** — Requires a recent snapshot of the primary transaction log

```sql
-- Force failover with data loss (last known good state)
ALTER AVAILABILITY GROUP [MyAG] FORCE_FAILOVER_ALLOW_DATA_LOSS;
```

## Backup Preferences

The backup preference setting determines which replica performs backups for the availability group:

- **Primary** — Backup performed on primary replica (default)
- **Prefer Secondary** — Backup performed on secondary replica (offloads primary)
- **Secondary Only** — Backup only on secondaries (primary cannot perform backups)
- **Any Secondary** — Backup on any secondary that is available

**Configuration:**
```sql
ALTER AVAILABILITY GROUP [MyAG]
MODIFY REPLICA ON 'Server1'
WITH (PRIMARY_ROLE (
    BACKUP_PRIORITY = 50
));

-- Set backup preference for availability group
ALTER AVAILABILITY GROUP [MyAG] SET (
    BACKUP_PREFERENCE = 'PreferSecondary'
);
```

## Monitoring Always On

### Health Monitoring DMVs

```sql
-- Check availability group status
SELECT ag.name AS availability_group,
       ar.replica_server_name,
       ar.role_desc,
       ar.operational_state_desc,
       ar.connected_state_desc,
       ar.synchronization_health_desc,
       ar.failover_mode_desc,
       ar.availability_mode_desc
FROM sys.dm_hadr_availability_replica_states ar
JOIN sys.availability_groups ag ON ar.group_id = ag.group_id;

-- Check database replica states
SELECT ar.replica_server_name,
       drd.database_name,
       drs.synchronization_state_desc,
       drs.synchronization_health_desc,
       drs.last_hardened_lsn,
       drs.last_sent_lsn,
       drs.last_received_lsn
FROM sys.dm_hadr_database_replica_states drs
JOIN sys.dm_hadr_availability_replica_states ar ON drs.replica_id = ar.replica_id;

-- Check AG listener status
SELECT name,
       dns_record,
       port,
       type_desc
FROM sys.availability_group_listeners;
```

### Extended Events for AG

Extended Events sessions can monitor Always On events:
- **hadr_db_commit** — Database commit events
- **hadr_local_cast** — Local commit and rollback events
- **hadr_physical_seelog_flush** — Physical log flush events
- **hadr_streaming_data_send/recv** — Streaming data send/receive events

### Performance Counters

Key performance counters for Always On:
- **SQLServer:Replica (Log Bytes Sent/sec)** — Volume of log sent to secondaries
- **SQLServer:Replica (Flow Control Time/ms)** — Time spent in flow control
- **SQLServer:Database Replica (Log Send Queue KB)** — Queue size at send side
- **SQLServer:Database Replica (Redo Queue KB)** — Queue size at redo side
- **SQLServer:Database Replica (Redo Bytes Remaining)** — Bytes remaining to redo

## Cross-Platform Availability Groups

Starting with SQL Server 2017, Always On availability groups support Linux (Ubuntu, RHEL, SUSE, Debian) replicas alongside Windows replicas.

**Requirements:**
- Same SQL Server version across Windows and Linux replicas
- Linux replicas must be enterprise or standard edition
- Network connectivity between Windows and Linux hosts
- Same version of the OS within the Linux group

**Limitations:**
- All replicas in an AG must be the same SQL Server version
- Mixed Windows/Linux replica support for read-only routing
- Cross-platform AG does not support failover clustering on Linux

```sql
-- Cross-platform AG example
-- On Linux replica
ALTER AVAILABILITY GROUP [MyAG] ADD REPLICA ON 'linux-server1.domain.com'
WITH (
    ENDPOINT_URL = 'TCP://linux-server1.domain.com:5022',
    AVAILABILITY_MODE = ASYNCHRONOUS_COMMIT,
    FAILOVER_MODE = MANUAL
);
```

## Disaster Recovery Scenarios

### Multi-Site Disaster Recovery

For disaster recovery across sites:
- Use asynchronous-commit mode for the distant site
- Configure manual failover
- Test failover procedures regularly
- Monitor log send queue for lag

**Example configuration:**
```sql
-- Primary replica (synchronous)
ALTER AVAILABILITY GROUP [MyAG]
MODIFY REPLICA ON 'PrimaryServer'
WITH (
    AVAILABILITY_MODE = SYNCHRONOUS_COMMIT,
    FAILOVER_MODE = AUTOMATIC,
    SEEDING_MODE = AUTOMATIC
);

-- Secondary replica (synchronous)
ALTER AVAILABILITY GROUP [MyAG]
MODIFY REPLICA ON 'SecondaryServer'
WITH (
    AVAILABILITY_MODE = SYNCHRONOUS_COMMIT,
    FAILOVER_MODE = AUTOMATIC,
    SEEDING_MODE = AUTOMATIC
);

-- Disaster recovery replica (asynchronous)
ALTER AVAILABILITY GROUP [MyAG]
MODIFY REPLICA ON 'DRAgainServer'
WITH (
    AVAILABILITY_MODE = ASYNCHRONOUS_COMMIT,
    FAILOVER_MODE = MANUAL,
    SEEDING_MODE = AUTOMATIC
);
```

### Multi-Subnet Failover

For deployments across multiple subnets:
- Configure static IP addresses for the AG listener on each subnet
- Register the AG listener DNS name in all subnets
- Client connection strings should include `MultiSubnetFailover=True`
- SQL Server 2012+ handles multi-subnet failover transparently

### Azure SQL Managed Instance Integration

SQL Server 2022+ supports linking on-premises availability groups to Azure SQL Managed Instance:
- Synchronous-commit mode with Managed Instance
- Asynchronous-commit mode for DR
- Backup to Azure Blob Storage
- Stretch Database support for warm storage

## Always On Failover Cluster Instance (FCI)

While Always On Availability Groups provide database-level HA, Failover Cluster Instances (FCI) provide instance-level HA:
- **FCI** — Windows Server failover cluster for the entire SQL Server instance
- **AG** — SQL Server native HA for individual databases
- **Combined** — Can deploy AG with FCI on both primary and secondary for maximum HA

**FCI considerations:**
- Requires shared storage (SAN, FC, iSCSI)
- Limited to two nodes (no multi-subnet clustering)
- No read-only secondary benefit
- Manual failover only for planned operations
- Automatic failover for unplanned failures

## Version-Specific Enhancements

### SQL Server 2012 (Initial Release)
- Basic availability groups with 1 secondary
- Synchronous and asynchronous modes
- Backup preference (primary only)

### SQL Server 2014
- Up to 4 secondaries
- Read-only routing
- Backup on secondary replicas
- Cross-forest authentication support

### SQL Server 2016
- Up to 8 secondaries
- Read-only routing intent
- Seeding mode improvements (automatic seeding)
- Partially readable secondaries

### SQL Server 2017
- Cross-platform availability groups (Linux)
- Instant file initialization for seeding
- Automatic seeding improvements
- Delayed durability

### SQL Server 2019
- Always On auto-failover support for Linux
- Enhanced health monitoring
- Improved automatic seeding reliability

### SQL Server 2022
- Always On auto-failover for Linux replicas
- Enhanced security for AG communication
- Improved performance for large databases
- Integration with Azure SQL Managed Instance

## Troubleshooting Always On

### Common Issues

1. **Database not synchronizing**
   - Check transaction log shipping
   - Verify sufficient disk space on secondary
   - Review error logs for errors
   - Check network connectivity between replicas

2. **High lag on asynchronous replica**
   - Monitor log send queue
   - Check disk I/O on primary
   - Review network bandwidth
   - Consider adjusting backup preferences

3. **Automatic failover failure**
   - Verify majority of replicas are online
   - Check secondary is caught up
   - Review automatic failover mode settings
   - Check heartbeat connectivity

4. **AG listener DNS issues**
   - Verify DNS records are correct
   - Check static IP configuration
   - Test DNS resolution from client
   - Verify multi-subnet failover setting

5. **Memory pressure affecting AG**
   - Monitor buffer pool usage
   - Check memory grants
   - Review version store usage
   - Consider increasing max server memory

## Conclusion

Always On Availability Groups is the premier high availability solution for SQL Server. The combination of synchronous-commit replicas for zero data loss and asynchronous-commit replicas for disaster recovery provides flexible HA/DR options. Key success factors include proper network configuration, regular failover testing, monitoring synchronization health, and understanding the trade-offs between synchronous and asynchronous modes. With cross-platform support and Azure integration, Always On continues to be the most comprehensive HA solution in the SQL Server portfolio.