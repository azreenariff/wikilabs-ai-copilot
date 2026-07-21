# MSSQL Backup and Recovery

## Overview

Microsoft SQL Server backup and recovery procedures cover full, differential, transaction log backups, and disaster recovery.

## Backup Components

### Components to Backup

| Component | Method | Frequency |
|-----------|--------|-----------|
| **Database Data** | Full backup | Daily |
| **Transaction Log** | Log backup | Every 15 minutes |
| **Differential** | Diff backup | Hourly |
| **Configuration** | Script out objects | Weekly |
| **SSIS Packages** | Backup project | Weekly |

## Backup Procedures

### Full Backup

```sql
-- Full database backup
BACKUP DATABASE mydb
TO DISK = 'C:\Backup\mydb_full.bak'
WITH COMPRESSION, STATS = 10;
GO

-- Verify backup
RESTORE VERIFYONLY
FROM DISK = 'C:\Backup\mydb_full.bak';
GO
```

### Differential Backup

```sql
-- Differential database backup
BACKUP DATABASE mydb
TO DISK = 'C:\Backup\mydb_diff.bak'
WITH DIFFERENTIAL, COMPRESSION, STATS = 10;
GO
```

### Transaction Log Backup

```sql
-- Transaction log backup
BACKUP LOG mydb
TO DISK = 'C:\Backup\mydb_log.trn'
WITH COMPRESSION, STATS = 10;
GO
```

## Recovery Procedures

### Point-in-Time Recovery

```sql
-- Restore full backup
RESTORE DATABASE mydb
FROM DISK = 'C:\Backup\mydb_full.bak'
WITH NORECOVERY, REPLACE;
GO

-- Restore differential backup
RESTORE DATABASE mydb
FROM DISK = 'C:\Backup\mydb_diff.bak'
WITH NORECOVERY;
GO

-- Restore transaction logs
RESTORE LOG mydb
FROM DISK = 'C:\Backup\mydb_log1.trn'
WITH NORECOVERY;
GO

RESTORE LOG mydb
FROM DISK = 'C:\Backup\mydb_log2.trn'
WITH RECOVERY, STOPAT = '2024-01-15 10:30:00';
GO
```

### Database Restore

```sql
-- Restore database
RESTORE DATABASE mydb
FROM DISK = 'C:\Backup\mydb_full.bak'
WITH RECOVERY;
GO
```

## References

- SQL Server Backup: https://learn.microsoft.com/en-us/sql/
- SQL Server Recovery: https://learn.microsoft.com/en-us/sql/
- SQL Server Disaster Recovery: https://learn.microsoft.com/en-us/sql/