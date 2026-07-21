# Backup and Recovery Strategies

## Overview

Microsoft SQL Server provides comprehensive backup and recovery capabilities essential for enterprise data protection. Effective backup strategies balance Recovery Point Objective (RPO) and Recovery Time Objective (RTO) against storage costs and operational overhead. This document covers all backup types, recovery models, strategies, and procedures for SQL Server 2017, 2019, and 2022.

## Recovery Models

SQL Server supports three recovery models that determine how transactions are logged and how recovery operations are performed:

### Full Recovery Model

In the full recovery model, all transactions are fully logged, and log records are retained until the transaction log is backed up. This model provides the most granular recovery options.

**Characteristics:**
- All transactions are fully logged
- Transaction log grows continuously until backed up
- Point-in-time recovery is possible
- Supports log shipping, replication, and CDC
- Requires regular transaction log backups

**Best for:** Production databases where data loss tolerance is zero.

```sql
ALTER DATABASE [MyDatabase] SET RECOVERY FULL;
```

### Bulk-Logged Recovery Model

The bulk-logged recovery model minimally logs bulk operations while still fully logging other transactions. This reduces log space usage for bulk operations.

**Characteristics:**
- Bulk operations (BULK INSERT, SELECT INTO, ALTER INDEX REBUILD) are minimally logged
- Full transactions are still fully logged
- Point-in-time recovery is NOT possible if bulk operations occurred
- Can revert to full recovery model without backup
- Reduces transaction log growth for bulk operations

**Best for:** Databases with periodic large bulk loads where some data loss tolerance exists.

**Warning:** Do not switch to bulk-logged during backup windows if point-in-time recovery is required.

```sql
ALTER DATABASE [MyDatabase] SET RECOVERY BULK_LOGGED;
-- Revert to full after bulk operations
ALTER DATABASE [MyDatabase] SET RECOVERY FULL;
-- Full backup required after switching back to full
BACKUP DATABASE [MyDatabase] TO DISK = 'C:\Backups\MyDatabase_Full.blb';
```

### Simple Recovery Model

In the simple recovery model, the transaction log is automatically truncated after each checkpoint. Only full and differential backups are supported.

**Characteristics:**
- Transaction log is automatically truncated
- No transaction log backups possible
- Only full and differential backups supported
- Cannot perform point-in-time recovery
- Cannot restore from transaction log backups
- Lowest operational overhead

**Best for:** Development databases, data warehouses, databases with infrequent changes.

```sql
ALTER DATABASE [MyDatabase] SET RECOVERY SIMPLE;
```

## Backup Types

### Full Backup

A full backup copies all data pages and required parts of the transaction log. It is the foundation of all backup strategies.

**Characteristics:**
- Copies all data pages and log records needed for recovery
- Can be taken for any database in any recovery model
- Self-contained — can restore without other backups
- Slowest to create (reads all pages)
- Largest backup size

```sql
-- Standard full backup
BACKUP DATABASE [MyDatabase]
TO DISK = 'C:\Backups\MyDatabase_Full.bak'
WITH COMPRESSION, INIT, CHECKSUM;

-- Verify backup integrity
RESTORE VERIFYONLY FROM DISK = 'C:\Backups\MyDatabase_Full.bak';

-- Read header information
RESTORE HEADERONLY FROM DISK = 'C:\Backups\MyDatabase_Full.bak';
```

### Differential Backup

A differential backup copies all data pages that have changed since the last full backup. It significantly reduces backup time and storage compared to full backups.

**Characteristics:**
- Backs up all pages changed since last full backup
- Size grows until next full backup
- Faster to create than full backup
- Faster to restore than applying log backups
- Must be applied on top of a full backup

```sql
-- Differential backup
BACKUP DATABASE [MyDatabase]
TO DISK = 'C:\Backups\MyDatabase_Differential.bak'
WITH DIFFERENTIAL, COMPRESSION, CHECKSUM;
```

### Transaction Log Backup

A transaction log backup backs up the portion of the transaction log that has not yet been backed up. This is only possible in Full and Bulk-Logged recovery models.

**Characteristics:**
- Backs up only log records since last log backup
- Very small backup size
- Very fast to create
- Required for point-in-time recovery
- Forms a chain — each backup depends on the previous one
- Truncates the log (makes VLFs reusable)

**Log backup chains must be contiguous** — any gap in the chain breaks point-in-time recovery.

```sql
-- Transaction log backup
BACKUP LOG [MyDatabase]
TO DISK = 'C:\Backups\MyDatabase_Log.trn'
WITH COMPRESSION, CHECKSUM;

-- Tail-log backup (before restore)
BACKUP LOG [MyDatabase]
TO DISK = 'C:\Backups\MyDatabase_TailLog.trn'
WITH NORECOVERY, NO_TRUNCATE;
```

### Copy-Only Backup

A copy-only backup is a special backup that does not affect the normal backup chain. It is used for ad-hoc or one-off backups without disrupting scheduled backup operations.

**Characteristics:**
- Does not affect backup chain
- Does not truncate transaction log
- Useful for one-off backups before maintenance
- Cannot be used as base for differential backups

```sql
BACKUP DATABASE [MyDatabase]
TO DISK = 'C:\Backups\MyDatabase_CopyOnly.bak'
WITH COPY_ONLY, COMPRESSION, CHECKSUM;
```

### Page Backup

A page backup backs up individual pages within a database. This is useful for restoring a single corrupted page without restoring the entire database.

**Characteristics:**
- Backs up specific pages identified by page number
- Requires CHECKSUM to detect corruption
- Only available in Enterprise Edition
- Restored using RESTORE WITH PAGE clause
- Used for piecemeal restore scenarios

```sql
-- Page backup (Enterprise Edition only)
BACKUP DATABASE [MyDatabase]
PAGE = '1:100, 1:200, 2:50'
TO DISK = 'C:\Backups\MyDatabase_Pages.bak'
WITH CHECKSUM;
```

### File or Filegroup Backup

A file or filegroup backup backs up specific files or filegroups within a database. This is useful for very large databases where backing up entire database is impractical.

**Characteristics:**
- Backs up specific files or filegroups
- Requires full backup of the primary filegroup
- Supports partial backups (read-only filegroups excluded)
- Useful for large databases with multiple filegroups

```sql
-- File backup
BACKUP DATABASE [MyDatabase]
FILE = 'MyDatabase_Data1',
FILE = 'MyDatabase_Data2'
TO DISK = 'C:\Backups\MyDatabase_Files.bak'
WITH COMPRESSION, CHECKSUM;

-- Filegroup backup
BACKUP DATABASE [MyDatabase]
FILEGROUP = 'FG_ReadOnly'
TO DISK = 'C:\Backups\MyDatabase_FG.bak'
WITH COMPRESSION, CHECKSUM;
```

## Backup Strategies

### Full Recovery Model Strategy (Recommended for Production)

This is the standard strategy for production databases requiring zero data loss tolerance:

1. **Initial full backup** — Establish baseline
2. **Differential backups** — Every 6-24 hours (reduces log backup restore time)
3. **Transaction log backups** — Every 15 minutes to 1 hour
4. **Full backups** — Weekly (or after major schema changes)
5. **Backup verification** — Regularly restore backups to verify integrity

**Example schedule:**
- Sunday: Full backup
- Monday-Saturday: Differential backup
- Every 15 minutes: Transaction log backup

This approach minimizes both RPO (recovery point) and RTO (recovery time):
- **RPO:** Determined by log backup frequency (15 minutes in this example)
- **RTO:** Determined by differential backup frequency + log restore time

### Simple Recovery Model Strategy

For databases in simple recovery model:

1. **Full backup** — Daily or weekly depending on change rate
2. **Differential backup** — Between full backups for faster restore

**Example schedule:**
- Sunday: Full backup
- Monday-Saturday: Differential backup

### Large Database Strategy

For very large databases (terabytes):

1. **Split files across filegroups** — Separate data by age or type
2. **File/filegroup backups** — Back up active filegroups more frequently
3. **Page backups** — For critical pages
4. **Backup compression** — Essential for large databases
5. **Striping** — Distribute backup across multiple devices

## Backup Compression

Backup compression reduces backup file size and I/O at the cost of CPU:

**Compression options:**
- **Backup compression** — `WITH COMPRESSION` clause
- **Default compression** — Set via `sp_configure 'backup compression default'`

**SQL Server edition differences:**
- **Express Edition** — Backup compression available
- **Standard Edition** — Backup compression available
- **Enterprise Edition** — Backup compression available (with additional advanced options)

```sql
-- Enable default backup compression
EXEC sp_configure 'backup compression default', 1;
RECONFIGURE;

-- Per-backup compression
BACKUP DATABASE [MyDatabase]
TO DISK = 'C:\Backups\MyDatabase_Full.bak'
WITH COMPRESSION, CHECKSUM;

-- Check compression ratio
SELECT backup_set_id,
       database_name,
       compressed_backup_size / (1024.0 * 1024) AS compressed_mb,
       backup_size / (1024.0 * 1024) AS uncompressed_mb,
       ROUND(100.0 * compressed_backup_size / backup_size, 2) AS compression_pct
FROM msdb.dbo.backupset
WHERE database_name = 'MyDatabase'
ORDER BY backup_set_id DESC;
```

## Backup Encryption

SQL Server supports backup encryption to protect sensitive data at rest:

**Encryption types:**
- **Backup encryption** — Encrypts backup files (Enterprise Edition only)
- **Transparent Data Encryption (TDE)** — Encrypts database files at rest

```sql
-- Create master key and certificate for backup encryption
CREATE MASTER KEY ENCRYPTION BY PASSWORD = 'StrongPassword123!';
CREATE CERTIFICATE MyBackupCert
    WITH SUBJECT = 'Backup Encryption Certificate';

-- Backup certificate for restore on another server
BACKUP CERTIFICATE MyBackupCert
    TO FILE = 'C:\Backups\MyBackupCert.cer'
    WITH PRIVATE KEY (
        FILE = 'C:\Backups\MyBackupCertKey.pvk',
        ENCRYPTION BY PASSWORD = 'StrongPassword123!'
    );

-- Encrypted backup
BACKUP DATABASE [MyDatabase]
TO DISK = 'C:\Backups\MyDatabase_Full.bak'
WITH COMPRESSION, CHECKSUM,
    ENCRYPTION (
        ALGORITHM = AES_256,
        SERVER CERTIFICATE = MyBackupCert
    );
```

## Restore Procedures

### Full Restore with Transaction Log Recovery

This is the standard restore procedure for point-in-time recovery:

```sql
-- Step 1: Restore full backup with NORECOVERY
RESTORE DATABASE [MyDatabase]
FROM DISK = 'C:\Backups\MyDatabase_Full.bak'
WITH NORECOVERY;

-- Step 2: Restore differential backup (if applicable)
RESTORE DATABASE [MyDatabase]
FROM DISK = 'C:\Backups\MyDatabase_Differential.bak'
WITH NORECOVERY;

-- Step 3: Restore transaction log backups in order
RESTORE LOG [MyDatabase]
FROM DISK = 'C:\Backups\MyDatabase_Log_1.trn'
WITH NORECOVERY;

RESTORE LOG [MyDatabase]
FROM DISK = 'C:\Backups\MyDatabase_Log_2.trn'
WITH NORECOVERY;

-- Step 4: Restore to point-in-time (optional)
RESTORE LOG [MyDatabase]
FROM DISK = 'C:\Backups\MyDatabase_Log_3.trn'
WITH RECOVERY,
    STOPAT = '2024-01-15 14:30:00';

-- Step 5: Bring database online
RESTORE DATABASE [MyDatabase] WITH RECOVERY;
```

### Point-in-Time Recovery

Point-in-time recovery restores the database to a specific moment:

```sql
-- Restore to a specific point in time
RESTORE LOG [MyDatabase]
FROM DISK = 'C:\Backups\MyDatabase_Log.trn'
WITH RECOVERY,
    STOPAT = '2024-01-15 14:30:00';

-- Restore to a transaction name
RESTORE LOG [MyDatabase]
FROM DISK = 'C:\Backups\MyDatabase_Log.trn'
WITH RECOVERY,
    STOPATMARK = 'txn_name';

-- Restore before a transaction
RESTORE LOG [MyDatabase]
FROM DISK = 'C:\Backups\MyDatabase_Log.trn'
WITH RECOVERY,
    STOPBEFOREMARK = 'txn_name';
```

### Page Restore

Restore a single corrupted page from a page backup:

```sql
-- Identify corrupted page
DBCC CHECKDB ('MyDatabase') WITH NO_INFOMSGS;

-- Restore specific page
RESTORE DATABASE [MyDatabase]
PAGE = '1:100'
FROM DISK = 'C:\Backups\MyDatabase_Pages.bak'
WITH NORECOVERY;

-- Restore subsequent log backups to complete recovery
RESTORE LOG [MyDatabase]
FROM DISK = 'C:\Backups\MyDatabase_Log.trn'
WITH RECOVERY;
```

### Piecemeal Restore

Piecemeal restore allows selective restoration of filegroups:

```sql
-- Restore primary filegroup first
RESTORE DATABASE [MyDatabase]
FILEGROUP = 'PRIMARY'
FROM DISK = 'C:\Backups\MyDatabase_Primary.bak'
WITH NORECOVERY, PARTIAL;

-- Restore subsequent filegroups as needed
RESTORE DATABASE [MyDatabase]
FILEGROUP = 'FG_ReadOnly'
FROM DISK = 'C:\Backups\MyDatabase_FG_ReadOnly.bak'
WITH RECOVERY;
```

## Tail-Log Backup

A tail-log backup captures the active portion of the transaction log at the time of failure. This is critical for minimizing data loss:

```sql
-- Tail-log backup (requires database to be accessible)
BACKUP LOG [MyDatabase]
TO DISK = 'C:\Backups\MyDatabase_TailLog.trn'
WITH NORECOVERY, NO_TRUNCATE;

-- If database is unavailable, the WITH NO_TRUNCATE clause
-- still captures the tail of the log
```

## Backup to Azure (URL)

SQL Server supports backup directly to Azure Blob Storage:

```sql
-- Create credential for Azure access
CREATE CREDENTIAL [https://mystorageaccount.blob.core.windows.net/backups]
WITH IDENTITY = 'SHARED ACCESS SIGNATURE',
    SECRET = 'sv=2020-08-04&ss=bf&srt=sco&sp=rl&se=2025-01-01T00:00:00Z&st=2024-01-01T00:00:00Z&spr=https&sig=signature';

-- Backup to Azure URL
BACKUP DATABASE [MyDatabase]
TO URL = 'https://mystorageaccount.blob.core.windows.net/backups/MyDatabase_Full.bak'
WITH COMPRESSION, CHECKSUM, STATS = 10;
```

## Backup Verification

Always verify backups by restoring them to a test server:

```sql
-- Verify backup without restoring
RESTORE VERIFYONLY FROM DISK = 'C:\Backups\MyDatabase_Full.bak';

-- Restore to test server (restore with recovery)
RESTORE DATABASE [MyDatabase_Test]
FROM DISK = 'C:\Backups\MyDatabase_Full.bak'
WITH RECOVERY,
    MOVE 'MyDatabase_Data' TO 'C:\Data\MyDatabase_Test.mdf',
    MOVE 'MyDatabase_Log' TO 'C:\Log\MyDatabase_Test.ldf';

-- Verify database consistency
DBCC CHECKDB ('MyDatabase_Test');
```

## Backup Monitoring

### Backup History

```sql
-- Recent backups
SELECT bs.database_name,
       bs.backup_start_date,
       bs.backup_finish_date,
       bs.type AS backup_type,
       bs.backup_size / (1024*1024) AS size_mb,
       bs.compressed_backup_size / (1024*1024) AS compressed_mb,
       bs.expiration_date,
       bmf.physical_device_name
FROM msdb.dbo.backupset bs
JOIN msdb.dbo.backupmediafamily bmf ON bs.media_set_id = bmf.media_set_id
WHERE bs.database_name = 'MyDatabase'
ORDER BY bs.backup_start_date DESC;

-- Detect missing log backups
-- Check for gaps in log backup sequence
```

### Backup Failure Alerts

Set up alerts for backup failures using SQL Server Agent jobs and operator notifications. Key alert conditions:
- Backup job failure
- Backup duration exceeds threshold
- Backup size exceeds threshold
- Backup verification failure
- Insufficient disk space

## Version-Specific Features

### SQL Server 2017
- **Backup to URL** — Enhanced Azure Blob Storage support
- **Automatic backup verification** — Built-in backup integrity checks
- **Backup compression** — Available in all editions
- **Backup encryption** — Enterprise Edition with AES_128, AES_192, AES_256, DES3

### SQL Server 2019
- **Backup checksum** — Improved corruption detection
- **Backup compression** — Better compression ratios
- **Backup to S3** — Native S3 support via credential abstraction
- **Accelerated database recovery** — Reduces backup impact during recovery

### SQL Server 2022
- **Backup compression** — Improved compression for compressed files
- **Backup to GCS** — Google Cloud Storage support
- **Backup encryption** — Additional encryption algorithms
- **Backup verification** — Enhanced backup verification with CHECKSUM
- **Azure Blob Storage** — Enhanced URL backup with SAS token support

## Troubleshooting Backup Issues

### Common Issues

1. **Insufficient disk space**
   - Monitor backup destination disk space
   - Implement backup retention policies
   - Use backup compression
   - Consider offloading to network storage

2. **Backup chain broken**
   - Never skip transaction log backups in full recovery model
   - Use copy-only backups for ad-hoc backups
   - Maintain backup history records
   - Regularly verify backup chains with restores

3. **Backup performance issues**
   - Use backup compression to reduce I/O
   - Spread backups across time windows
   - Monitor I/O subsystem during backup
   - Consider striping across multiple devices

4. **Restore failures**
   - Always verify backups before relying on them
   - Test restore procedures regularly
   - Maintain restore scripts for quick recovery
   - Keep certificate and key backups current

## Conclusion

Effective backup and recovery is the foundation of database reliability. The full recovery model with regular transaction log backups provides the most granular recovery options. Backup compression and encryption reduce storage costs while maintaining data security. Regular backup verification through test restores is essential — a backup that cannot be restored provides no value. Enterprise SQL Server deployments should implement a comprehensive backup strategy that considers RPO, RTO, storage costs, and operational complexity.