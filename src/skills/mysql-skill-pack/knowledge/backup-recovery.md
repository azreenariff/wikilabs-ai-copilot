# MySQL Backup and Recovery

## Overview

MySQL backup and recovery strategies ensure data availability and disaster recovery capability. Effective backup planning balances recovery time objectives (RTO) with recovery point objectives (RPO). This document covers backup methods, recovery strategies, incremental backups, binary log management, encryption, and disaster recovery procedures.

## Backup Methods

### mysqldump

Logical backup tool for small databases and migration.

**Characteristics**:
- Exports data as SQL statements
- Supports selective database/table backup
- Point-in-time recovery using binary logs
- Not ideal for large databases (>100GB)
- Can be slow for large datasets due to sequential I/O

**Usage**:

```sql
-- Full backup with consistent snapshot
mysqldump --single-transaction --routines --triggers --all-databases > full_backup.sql

-- Specific database with GTID info
mysqldump --single-transaction --master-data=2 --set-gtid-purged=OFF mydb > mydb_backup.sql

-- Specific table only
mysqldump --single-transaction mydb users > users_backup.sql
```

**Best Practices**:
- Use `--single-transaction` for InnoDB (consistent without locking)
- Include `--routines` and `--triggers` for complete backup
- Use `--set-gtid-purged=OFF` when replicating to avoid GTID conflicts
- Schedule during low-traffic periods
- Use `--compress` for network transfers to reduce bandwidth
- Use `--databases` flag for explicit database list
- Consider `--hex-blob` for binary data safety

**mysqldump Limitations**:
- Single-threaded (slow for large databases)
- No incremental backup support
- Restoring involves replaying SQL (slow)
- Cannot guarantee consistency for MyISAM tables without --lock-all-tables
- Does not include stored procedures for all storage engines by default

### mysqlpump

Enhanced version of mysqldump with parallel operations.

**Advantages**:
- Parallel dump of tables and databases (up to 16 parallel threads)
- More flexible output formatting
- Better performance for large databases (up to 4x faster)
- Compatible with most mysqldump options
- Can exclude specific databases

**Usage**:

```sql
-- Parallel dump with custom parallelism
mysqlpump --default-parallelism=4 --skip-lock-tables mydb > backup.sql

-- Exclude specific databases
mysqlpump --exclude-database=test --default-parallelism=4 > backup.sql

-- Compress output
mysqlpump mydb | gzip > backup.sql.gz
```

**Note**: mysqlpump is available from MySQL 5.7 through 8.0.30. It was deprecated in MySQL 8.0.30 and removed in later versions. Use mysqldump or Percona XtraBackup for MySQL 8.4.

### Percona XtraBackup

Physical backup tool for InnoDB and XtraDB.

**Characteristics**:
- Online backup without locking (zero downtime)
- Faster for large databases (physical copy, not SQL replay)
- Compressed backup options
- Incremental backup support
- Supports parallel I/O operations

**Types**:

| Type | Description | Use Case |
|------|-------------|----------|
| Full | Complete backup | Initial backup, weekly |
| Incremental | Changes since last backup | Daily backups |
| Differential | Changes since last full | Weekly + daily |

**Usage**:

```sql
-- Full backup
xtrabackup --backup --target-dir=/backup/full --user=backup_user --password=secret --parallel=4

-- Incremental backup
xtrabackup --backup --target-dir=/backup/incr --incremental-basedir=/backup/full

-- Prepare backup (apply redo log, roll back uncommitted transactions)
xtrabackup --prepare --target-dir=/backup/full

-- Restore (copy-back mode)
xtrabackup --copy-back --target-dir=/backup/full
chown -R mysql:mysql /var/lib/mysql
```

**XtraBackup Considerations**:

| Factor | Full Backup | Incremental Backup | Differential Backup |
|--------|------------|-------------------|---------------------|
| Backup Speed | Slowest | Fastest | Medium |
| Storage | Largest | Smallest | Medium |
| Restore Time | Fastest | Slowest (chain) | Medium |
| Complexity | Simple | Complex chain | Moderate |
| Recovery Granularity | Coarse | Fine-grained PITR | Moderate |

**Incremental Backup Workflow**:

```sql
-- Step 1: Full backup
xtrabackup --backup --target-dir=/backup/full

-- Step 2: First incremental (since full)
xtrabackup --backup --target-dir=/backup/inc1 --incremental-basedir=/backup/full

-- Step 3: Prepare incrementals for restore
xtrabackup --prepare --target-dir=/backup/full --incremental-dir=/backup/inc1

-- Step 4: Second incremental (since inc1)
xtrabackup --backup --target-dir=/backup/inc2 --incremental-basedir=/backup/inc1

-- Step 5: Prepare all incrementals with full
xtrabackup --prepare --target-dir=/backup/full --incremental-dir=/backup/inc1 --incremental-dir=/backup/inc2

-- Step 6: Copy back
xtrabackup --copy-back --target-dir=/backup/full
```

**Encryption Support**:
```sql
-- Encrypted backup with AES256
xtrabackup --backup --target-dir=/backup/enc \
  --encrypt=AES256 --encrypt-key="your-encryption-key"
```

### Binary Log Backup

Binary logs enable point-in-time recovery after base backup.

**Configuration**:

```sql
-- Enable binary logging
SET GLOBAL log_bin = 'ON';
SET GLOBAL binlog_format = 'ROW';
SET GLOBAL binlog_expire_logs_seconds = 604800; -- 7 days
SET GLOBAL max_binlog_size = 1073741824; -- 1GB
SET GLOBAL sync_binlog = 1; -- durability guarantee
```

**Binary Log Format Selection**:

| Format | Description | Replication Safety | Storage |
|--------|-------------|-------------------|---------|
| STATEMENT | Logs SQL statements | Lower (non-deterministic funcs) | Smallest |
| ROW | Logs row changes | Highest | Largest |
| MIXED | Auto-switches | Medium | Medium |

**Recommendation**: Use ROW format for all production replication. It provides the most reliable replication and is the default in MySQL 8.0+.

**Management**:

```sql
-- View binary logs
SHOW BINARY LOGS;

-- View current binlog and position
SHOW MASTER STATUS;
SHOW BINARY LOG STATUS;

-- Purge old logs safely
PURGE BINARY LOGS BEFORE DATE_SUB(NOW(), INTERVAL 7 DAY);
PURGE BINARY LOGS TO 'mysql-bin.000020';

-- Rotate logs manually
FLUSH LOGS;
```

**Binary Log Analysis**:

```sql
-- Analyze binlog content (row format, verbose)
mysqlbinlog --base64-output=decode-rows -v mysql-bin.000010

-- Filter by time range
mysqlbinlog --start-datetime="2024-01-15 10:00:00" --stop-datetime="2024-01-15 11:00:00" mysql-bin.000010

-- Filter by database
mysqlbinlog --database=mydb mysql-bin.000010
```

## Recovery Strategies

### Point-in-Time Recovery (PITR)

PITR combines a base backup with binary log replay to recover to a specific moment.

**PITR Procedure**:

```sql
-- Step 1: Restore base backup
cp -r /backup/full/* /var/lib/mysql/
chown -R mysql:mysql /var/lib/mysql/

-- Step 2: Start MySQL for prepare (if using XtraBackup)
mysqladmin shutdown
mysqld_safe &

-- Step 3: Apply binary logs to specific recovery point
mysqlbinlog --stop-datetime="2024-01-15 14:30:00" mysql-bin.000001 mysql-bin.000002 | mysql

-- Alternative: Using GTID-based recovery
mysqlbinlog --skip-gtids --include-gtids="uuid:start:end" mysql-bin.000010 | mysql
```

**PITR Best Practices**:
- Always have a recent full backup before attempting PITR
- Verify binary logs are available and intact
- Test PITR procedures regularly in non-production
- Use GTID mode for simpler position tracking
- Document exact recovery points for each PITR

### Disaster Recovery

**DR Plan Steps**:

1. **Assess Damage**: Determine extent of data loss and outage scope
2. **Notify**: Alert stakeholders and incident management
3. **Prepare Environment**: Spin up replacement infrastructure
4. **Restore**: Apply latest full backup + incremental backups + binary logs
5. **Verify**: Validate data integrity and application connectivity
6. **Failover**: Redirect application traffic to recovered instance
7. **Monitor**: Watch for issues during transition period
8. **Document**: Record timeline and root cause

**RTO/RPO Targets**:

| Scenario | RTO Target | RPO Target | Strategy |
|----------|-----------|-----------|----------|
| Minor failure | < 15 min | < 5 min | Automatic failover, replica promotion |
| Major failure | < 1 hour | < 15 min | Restore from backup + binlog replay |
| Catastrophic | < 4 hours | < 1 hour | Cross-region restore from offsite backup |

**Disaster Recovery Topologies**:

| Topology | RTO | RPO | Complexity | Cost |
|----------|-----|-----|-----------|------|
| Single server with backup | 1-4 hours | 15-60 min | Low | Low |
| Master-slave replication | 5-30 min | 0-5 min | Medium | Medium |
| Group Replication | < 1 min | 0 min | High | High |
| Cross-region replication | 15-60 min | 0-1 min | High | Very High |

### Failback Procedures

When recovering a failed primary to a new environment:

1. **Build Replacement**: Set up new server with latest backup
2. **Sync Data**: Replicate from primary to new server
3. **Cutover**: Switch traffic to recovered server
4. **Monitor**: Watch for data consistency issues
5. **Decommission**: Remove failed server after confirmation

## Backup Scheduling

### Recommended Schedule

| Frequency | Type | Duration |
|-----------|------|----------|
| Daily | Incremental | 5-15 min |
| Weekly | Full | 30-60 min |
| Monthly | Full (retained) | 60-120 min |
| Continuous | Binary logs | Ongoing |

### Retention Policy

| Type | Retention | Storage Location |
|------|-----------|-----------------|
| Daily incremental | 7 days | Primary storage |
| Weekly full | 4 weeks | Primary storage |
| Monthly full | 12 months | Offsite/archive |
| Binary logs | 7-14 days | Primary storage |

### Automated Backup Scripts

```bash
#!/bin/bash
# Daily incremental backup script
BACKUP_DIR="/backup/daily"
DATE=$(date +%Y%m%d)
FULL_BACKUP="/backup/weekly"

# Create incremental directory
mkdir -p ${BACKUP_DIR}/${DATE}

# Check if full backup exists, otherwise do full
if [ -d "${FULL_BACKUP}/latest" ]; then
  xtrabackup --backup --target-dir=${BACKUP_DIR}/${DATE} \
    --incremental-basedir=${FULL_BACKUP}/latest --compress
else
  xtrabackup --backup --target-dir=${BACKUP_DIR}/${DATE} --compress
fi

# Update full backup pointer for next incremental
cp -a ${BACKUP_DIR}/${DATE} ${FULL_BACKUP}/latest

# Rotate old backups (keep 7 days)
find ${BACKUP_DIR} -maxdepth 1 -mtime +7 -exec rm -rf {} +
```

## Monitoring Backups

### Success Indicators

1. **Backup Completeness**: All databases backed up
2. **Backup Integrity**: Files not corrupted
3. **Backup Timing**: Completed within expected window
4. **Space Availability**: Sufficient disk space
5. **Backup Size**: Consistent with expectations (sudden drops indicate issues)

### Health Checks

```sql
-- Check binary log availability
SHOW BINARY LOGS;
SHOW MASTER STATUS;

-- Check backup space usage
SELECT table_schema, ROUND(SUM(data_length + index_length) / 1024 / 1024, 2) AS size_mb
FROM information_schema.tables GROUP BY table_schema;

-- Check backup status (if using XtraBackup)
ls -la /backup/latest/
```

### Backup Monitoring Dashboard Metrics

| Metric | Check | Alert Threshold |
|--------|-------|----------------|
| Last backup timestamp | File modification time | > 24 hours |
| Backup size | File size comparison | > 20% change |
| Binary log count | SHOW BINARY LOGS | > 20 files growing |
| Disk usage | df -h /backup | > 80% full |
| Backup duration | Start/end timestamps | > 2x normal |

## High Availability Backup Integration

### Replication Backup

Back up from replica to avoid primary load:

1. **Promote Replica**: Temporarily make replica primary
2. **Perform Backup**: Run backup without impacting production
3. **Resume Replication**: Connect back to primary
4. **Monitor**: Verify replication continues normally

**Replica Backup Procedure**:

```sql
-- On replica: ensure we're in read-only mode
SET GLOBAL read_only = ON;

-- Take backup from replica
mysqldump --single-transaction --all-databases > replica_backup.sql

-- Or use XtraBackup from replica (lower impact)
xtrabackup --backup --target-dir=/backup/replica

-- Resume normal replication
SET GLOBAL read_only = OFF;
```

### Cloud Integration

For cloud deployments:

1. **Automated Snapshots**: Use cloud provider backup tools (AWS RDS snapshots, Azure MySQL backups)
2. **Cross-Region Replication**: Backup to different region for disaster recovery
3. **Lifecycle Policies**: Automated retention management with cloud storage tiers
4. **Monitoring Integration**: Alert on backup failures via CloudWatch, Stackdriver, etc.

**Cloud Backup Best Practices**:

| Platform | Tool | Key Features |
|----------|------|-------------|
| AWS RDS | Automated Snapshots | Cross-region copies, encryption, fast restore |
| Azure MySQL | Automated Backups | Geo-redundant, PITR up to 35 days |
| GCP Cloud SQL | Automated Backups | Cross-region, PITR, point-in-time restore |
| OCI MySQL | Automated Backups | Lifecycle policies, encryption, cross-region |

## Operational Best Practices

1. **Test Restores Regularly**: Verify backup viability by performing restore tests monthly
2. **Automate Backups**: Reduce human error with scheduled backup jobs
3. **Monitor Backup Health**: Alert on failures with PagerDuty, Slack, or email
4. **Document Recovery Procedures**: Clear runbooks with step-by-step recovery instructions
5. **Use Encrypted Backups**: Protect sensitive data at rest and in transit
6. **Store Offsite**: Separate location for disaster recovery (cloud storage, physical media)
7. **Review Retention Policies**: Align with compliance requirements (PCI-DSS, HIPAA, GDPR)
8. **Capacity Planning**: Ensure storage for backup growth
9. **Version Testing**: Test backups with new MySQL versions before upgrading production
10. **Security Review**: Protect backup credentials and data access controls

## Security for Backups

**Backup Encryption Options**:

1. **Encryption at Rest**: Use OS-level disk encryption (LUKS) for backup storage
2. **TLS for Replication**: Use SSL/TLS for replication connections
3. **Backup File Encryption**: Pipe through gpg or openssl
4. **XtraBackup Encryption**: Percona XtraBackup supports encrypted backups natively

**GPG-Encrypted Backup**:
```sql
-- Create encrypted backup
mysqldump --single-transaction mydb | openssl enc -aes-256-cbc -salt -pbkdf2 -out backup.sql.enc -pass pass:strong_password
```

**Secure Backup Access**:

- Restrict backup file permissions (chmod 600)
- Use dedicated backup user with minimal privileges
- Store backup credentials in secret management system (HashiCorp Vault, AWS Secrets Manager)
- Audit backup access and restore operations

## Recovery Testing

**Restore Test Checklist**:

1. **Verify Backup Completeness**: Ensure all databases and tables present
2. **Check Data Integrity**: Verify row counts match source
3. **Test Application Connection**: Confirm application can connect and query
4. **Validate Binary Logs**: Verify PITR point is available
5. **Measure Restore Time**: Compare against RTO targets
6. **Test Indexes**: Verify index integrity and performance
7. **Test Stored Objects**: Validate stored procedures, triggers, views

**Recovery Drills Schedule**:

| Frequency | Test | Scope |
|-----------|------|-------|
| Weekly | Backup verification | File integrity check |
| Monthly | Restore test | Full restore to test environment |
| Quarterly | PITR test | Point-in-time recovery drill |
| Annually | Full DR drill | Complete disaster recovery simulation |

## References

- MySQL 8.0 Backup and Restore: https://dev.mysql.com/doc/refman/8.0/en/backup-restore.html
- Percona XtraBackup: https://www.percona.com/doc/percona-xtrabackup/LATEST/index.html
- MySQL Binary Logs: https://dev.mysql.com/doc/refman/8.0/en/binary-log.html
- MySQL mysqldump: https://dev.mysql.com/doc/refman/8.0/en/mysqldump.html
- MySQL mysqlpump: https://dev.mysql.com/doc/refman/8.0/en/mysqlpump.html
- MySQL Point-in-Time Recovery: https://dev.mysql.com/doc/refman/8.0/en/point-in-time-recovery.html
- MySQL Enterprise Backup: https://dev.mysql.com/doc/refman/8.0/en/mysql-enterprise-backup.html