# SQL Server Best Practices Reference

## Overview

This document provides comprehensive best practices for SQL Server 2017, 2019, and 2022 enterprise deployments. These practices are organized by topic and reflect industry standards and Microsoft recommendations.

## General Best Practices

### 1. Edition Selection

| Workload | Recommended Edition |
|----------|-------------------|
| OLTP production | Enterprise or Standard |
| Data warehouse | Enterprise |
| Development/test | Developer (free) |
| Web/small business | Standard |
| Reporting | Standard or Enterprise |

**Developer Edition** provides all Enterprise features for development and testing.

### 2. Patch Management

- Apply latest Cumulative Update (CU) or Service Pack (SP)
- Test patches in non-production before deployment
- Monitor Microsoft security advisories
- Maintain patch level across HA/DR pairs

### 3. Documentation

- Document server configurations
- Document database schemas and dependencies
- Document maintenance procedures
- Document disaster recovery procedures

### 4. Naming Conventions

- Database names: `db_<application>_<environment>`
- Table names: `tbl_<entity>` or `<entity>`
- Column names: `camelCase` or `snake_case` (consistent)
- Index names: `IX_<table>_<columns>` or `CX_<table>_<columns>`
- Constraint names: `CK_<table>_<column>` or `PK_<table>`

## Performance Best Practices

### 1. Memory Configuration

- Set `max server memory` to leave RAM for OS
- On dedicated servers, leave 4 GB for OS
- Monitor page life expectancy (target: > 300 seconds)
- Monitor buffer cache hit ratio (target: > 95%)

### 2. File Placement

- Separate data, log, and tempdb files onto different disks
- Use SSD storage for data and tempdb files
- Use fast storage for log files (low latency required)
- Distribute TempDB files across different controllers

### 3. MAXDOP Configuration

| Core Count | Recommended MAXDOP |
|------------|-------------------|
| 1-4 | 4 |
| 4-8 | 8 |
| 8-16 | 8 |
| > 16 | 8 (or 4 for OLTP) |

**Query Store can help validate MAXDOP settings.**

### 4. Cost Threshold for Parallelism

- Default value: 50 (typically too high)
- Recommended: 5-15 for OLTP workloads
- Recommended: 15-50 for analytical workloads

### 5. Index Design

- Clustered index on monotonically increasing column
- Nonclustered indexes for frequently queried columns
- Covering indexes for high-volume queries
- Regular index maintenance using Ola Hallengren solution
- Monitor unused indexes and remove them

### 6. Statistics

- Enable AUTO_CREATE_STATISTICS
- Enable AUTO_UPDATE_STATISTICS
- Consider AUTO_UPDATE_STATISTICS_ASYNC for high-concurrency systems
- Schedule regular FULLSCAN updates for critical tables

## Backup and Recovery Best Practices

### 1. Recovery Model

- Production OLTP: Full recovery model
- Development: Simple recovery model acceptable
- Data warehouse: Full or Bulk-Logged based on requirements
- Always test recovery procedures

### 2. Backup Strategy

| Backup Type | Frequency |
|-------------|-----------|
| Full backup | Weekly (minimum) |
| Differential backup | Daily or every 6 hours |
| Transaction log backup | Every 5-15 minutes |
| Copy-only backup | Before major maintenance |

### 3. Backup Verification

- Test restores quarterly on a non-production server
- Verify backup integrity using RESTORE VERIFYONLY
- Monitor backup job success/failure
- Track backup sizes for capacity planning

### 4. Offsite Backup

- Store backups on separate physical storage
- Consider cloud backup (Azure Blob Storage)
- Maintain at least one offsite copy
- Test offsite restore procedures

## High Availability Best Practices

### 1. Always On Availability Groups

- Use synchronous-commit mode for zero data loss
- Use asynchronous-commit mode for disaster recovery
- Configure AG listener for application transparency
- Regularly test failover procedures
- Monitor synchronization health

### 2. Failover Cluster Instances

- Use FCI for instance-level high availability
- Configure shared storage with multipath I/O
- Monitor cluster health
- Test failover procedures

### 3. Monitoring

- Monitor AG synchronization lag
- Monitor replica health status
- Monitor log send queue size
- Monitor redo queue size

## Security Best Practices

### 1. Authentication

- Prefer Windows Authentication
- Use Kerberos for cross-domain authentication
- Disable unused SQL Server logins
- Enforce strong passwords

### 2. Authorization

- Follow principle of least privilege
- Use database roles instead of individual grants
- Separate development and production access
- Regular permission reviews

### 3. Encryption

- Enable TDE for data at rest
- Use TLS/SSL for data in transit
- Use Always Encrypted for sensitive columns
- Protect encryption keys and certificates

### 4. Auditing

- Enable SQL Server Audit for compliance
- Monitor failed login attempts
- Monitor privileged operations
- Review audit logs regularly

## Maintenance Best Practices

### 1. Index Maintenance

- Reorganize indexes with 5-30% fragmentation
- Rebuild indexes with > 30% fragmentation
- Use Ola Hallengren IndexOptimize solution
- Schedule maintenance during off-peak hours

### 2. Statistics Maintenance

- Update statistics when data changes significantly
- Consider FULLSCAN for critical tables
- Schedule updates during maintenance windows
- Monitor plan cache impact

### 3. Database Health Checks

- Run DBCC CHECKDB regularly
- Monitor wait statistics
- Monitor DMV-based metrics
- Review ERRORLOG regularly

### 4. TempDB Optimization

- One data file per CPU core (up to 8+)
- Equal-size files with even growth
- Fast SSD storage
- Monitor latch contention

## Monitoring Best Practices

### 1. Key Metrics

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Page life expectancy | > 300 seconds | < 100 seconds |
| Buffer cache hit ratio | > 95% | < 90% |
| CPU utilization | < 70% | > 85% |
| Disk I/O latency | < 20 ms | > 50 ms |
| Wait stats dominant | CXPACKET acceptable | Other dominant waits |
| Backup success | 100% | Any failure |

### 2. Monitoring Tools

- SQL Server Agent jobs for automated checks
- Extended Events for targeted tracing
- Performance Monitor (perfmon) for system metrics
- Third-party monitoring tools for comprehensive coverage

### 3. Alerting

- Configure alerts for critical conditions
- Alert on backup failures
- Alert on high wait statistics
- Alert on disk space low
- Alert on replication lag

## Documentation Best Practices

### 1. Server Documentation

- Server configuration (OS, SQL Server version)
- Hardware specifications (CPU, RAM, storage)
- Network configuration
- Service accounts and permissions

### 2. Database Documentation

- Database size and growth trend
- File layout (data, log, filegroups)
- Index structure
- Key queries and performance characteristics

### 3. Runbook Documentation

- Common procedures (restore, patch, maintenance)
- Emergency procedures (failover, disaster recovery)
- Escalation procedures
- Contact information

## Monitoring and Tuning Best Practices

### 1. Wait Statistics Analysis

- Collect wait statistics regularly
- Focus on the top 3-5 wait types
- Correlate with workload changes
- Track trends over time

### 2. Query Performance

- Use Query Store for historical analysis
- Identify top resource-consuming queries
- Monitor for plan regressions
- Review execution plans for optimization

### 3. Resource Utilization

- Monitor CPU, memory, disk, and network
- Plan capacity based on growth trends
- Set up alerts for resource thresholds
- Perform regular capacity reviews

## Conclusion

These best practices provide a comprehensive framework for SQL Server enterprise deployments. Apply these practices consistently, adapt them to your specific environment, and regularly review and update them as requirements evolve. The key to effective SQL Server management is consistent application of these practices combined with proactive monitoring and regular maintenance.