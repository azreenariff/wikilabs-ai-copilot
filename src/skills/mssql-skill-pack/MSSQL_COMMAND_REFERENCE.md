# SQL Server Command Reference

## Overview

This reference documents all SQL Server commands organized by category. Each command includes syntax, purpose, risk level, and usage guidance.

## Connection and Authentication

| Command | Purpose | Risk |
|---------|---------|------|
| CREATE LOGIN | Create server-level login | Low |
| ALTER LOGIN | Modify login properties | Medium |
| DROP LOGIN | Remove login | High |
| CREATE USER | Create database user | Low |
| DROP USER | Remove database user | Medium |
| GRANT | Grant permissions | Medium |
| REVOKE | Revoke permissions | Medium |
| DENY | Explicitly deny permissions | Medium |
| sp_who2 | List active sessions | Low |
| DAC (admin:) | Emergency admin connection | Low |

## Database Management

| Command | Purpose | Risk |
|---------|---------|------|
| CREATE DATABASE | Create new database | Medium |
| ALTER DATABASE | Modify database properties | Medium |
| DROP DATABASE | Delete database | Critical |
| SET COMPATIBILITY_LEVEL | Set optimizer version | Medium |
| SERVERPROPERTY() | Check server configuration | Low |
| @@VERSION | Check SQL Server version | Low |
| DBCC CHECKDB | Verify database integrity | Medium |
| sp_configure | Manage server configuration | Medium |

## Schema Management

| Command | Purpose | Risk |
|---------|---------|------|
| CREATE TABLE | Create table | Medium |
| ALTER TABLE | Modify table structure | High |
| DROP TABLE | Remove table | High |
| CREATE VIEW | Create virtual table | Low |
| CREATE PROCEDURE | Create stored procedure | Low |
| CREATE FUNCTION | Create user function | Low |
| CREATE TRIGGER | Create trigger | Medium |

## Query Execution

| Command | Purpose | Risk |
|---------|---------|------|
| SELECT | Query data | Low |
| INSERT | Add data | Medium |
| UPDATE | Modify data | High |
| DELETE | Remove data | High |
| WITH (CTE) | Common table expressions | Low |
| Window Functions | Analytic computations | Low |
| sp_executesql | Dynamic SQL execution | Medium |
| TRY/CATCH | Error handling | Low |
| OPENJSON | JSON processing | Low |
| Temporary Tables | Intermediate storage | Low |

## Performance Monitoring

| Command | Purpose | Risk |
|---------|---------|------|
| sys.dm_os_wait_stats | Wait statistics | Low |
| sys.dm_exec_query_stats | Query performance | Low |
| sys.dm_db_index_usage_stats | Index usage | Low |
| sys.dm_db_index_physical_stats | Fragmentation | Low |
| sys.dm_os_memory_clerks | Memory usage | Low |
| sys.dm_tran_locks | Lock state | Low |
| sys.dm_exec_query_memory_grants | Memory grants | Low |
| SET STATISTICS IO | I/O statistics | Low |
| SET STATISTICS TIME | Time statistics | Low |
| SHOWPLAN | Estimated plans | Low |
| sys.dm_exec_input_buffer | Session commands | Low |

## Index Management

| Command | Purpose | Risk |
|---------|---------|------|
| CREATE INDEX | Create index | Medium |
| DROP INDEX | Remove index | Medium |
| ALTER INDEX REBUILD | Rebuild index | Medium |
| ALTER INDEX REORGANIZE | Reorganize index | Low |
| CREATE STATISTICS | Create statistics | Low |
| UPDATE STATISTICS | Update statistics | Low |
| CREATE COLUMNSTORE INDEX | Columnstore index | Medium |
| CREATE UNIQUE INDEX | Enforce uniqueness | Medium |
| CREATE FILTERED INDEX | Selective indexing | Low |
| INCLUDE columns | Covering indexes | Low |
| ON = ON | Online index ops | Medium |
| sys.dm_db_missing_index_groups | Missing index info | Low |
| DBCC SHOW_STATISTICS | Statistics histogram | Low |

## Transaction Processing

| Command | Purpose | Risk |
|---------|---------|------|
| BEGIN TRAN | Start transaction | Medium |
| COMMIT | Commit transaction | Medium |
| ROLLBACK | Rollback transaction | Medium |
| SAVE TRAN | Create savepoint | Low |
| SET TRANSACTION ISOLATION LEVEL | Set isolation level | Low |
| DELAYED_DURABILITY | Configure log durability | Medium |
| @@TRANCOUNT | Check transaction depth | Low |

## Always On Availability Groups

| Command | Purpose | Risk |
|---------|---------|------|
| CREATE AVAILABILITY GROUP | Create AG | High |
| ALTER AVAILABILITY GROUP | Modify AG | High |
| ALTER AVAILABILITY GROUP FAILOVER | Manual failover | Critical |
| ALTER AVAILABILITY GROUP SEEDING_MODE | Configure seeding | Medium |
| CREATE AVAILABILITY GROUP LISTENER | Create listener | Medium |
| sys.dm_hadr_availability_replica_states | AG health | Low |
| sys.dm_hadr_database_replica_states | DB sync state | Low |
| BACKUP_PREFERENCE | Configure backup replica | Low |

## Backup and Recovery

| Command | Purpose | Risk |
|---------|---------|------|
| BACKUP DATABASE (Full) | Full backup | Low |
| BACKUP DATABASE (Diff) | Differential backup | Low |
| BACKUP LOG | Transaction log backup | Low |
| BACKUP DATABASE TO URL | Azure backup | Low |
| RESTORE DATABASE | Restore database | Critical |
| RESTORE VERIFYONLY | Verify backup | Low |
| BACKUP CERTIFICATE | Export certificate | Medium |
| DBCC SQLPERF(LOGSPACE) | Log space usage | Low |
| DBCC LOGINFO | VLF information | Low |

## TempDB and Memory

| Command | Purpose | Risk |
|---------|---------|------|
| sys.dm_db_file_space_usage | File space usage | Low |
| sys.dm_db_session_space_usage | Session space | Low |
| sys.dm_db_task_space_usage | Task space | Low |
| sys.dm_tran_version_snapshot_space_usage | Version store | Low |
| sp_configure 'max server memory' | Memory limit | Medium |
| sys.dm_os_performance_counters | Performance counters | Low |

## Maintenance

| Command | Purpose | Risk |
|---------|---------|------|
| ALTER INDEX REBUILD | Heavy fragmentation | Medium |
| ALTER INDEX REORGANIZE | Moderate fragmentation | Low |
| UPDATE STATISTICS | Statistics refresh | Low |
| DBCC CHECKDB | Integrity verification | Medium |
| sp_updatestats | Quick stats update | Low |
| DBCC SHRINKFILE | File shrinkage | Medium |
| DBCC CHECKALLOC | Allocation check | Low |
| DBCC CONFIGURATION | Config review | Low |

## Security

| Command | Purpose | Risk |
|---------|---------|------|
| CREATE LOGIN | Server login | Low |
| CREATE USER | Database user | Low |
| GRANT / REVOKE / DENY | Permissions | Medium |
| CREATE MASTER KEY | Encryption key | Medium |
| CREATE CERTIFICATE | Certificate | Medium |
| CREATE DATABASE ENCRYPTION KEY | TDE key | Medium |
| ALTER DATABASE SET ENCRYPTION | Enable TDE | Medium |
| CREATE SERVER AUDIT | Audit creation | Medium |
| ALTER SERVER AUDIT | Enable/disable audit | Medium |
| CREATE SECURITY POLICY | RLS policy | Medium |
| CREATE EVENT SESSION | Extended events | Low |

## Automation

| Command | Purpose | Risk |
|---------|---------|------|
| sp_add_job | Create Agent job | Medium |
| sp_add_alert | Create Agent alert | Low |

## Version-Specific Commands

| SQL Server | New Features |
|------------|-------------|
| 2017 | Intelligent Query Processing, Python/R, Graph |
| 2019 | Batch mode rowstore, Smart memory grant, ADR |
| 2022 | Vectorized batch mode, Intelligent performance |

## Documentation References

- [T-SQL Language Reference](https://learn.microsoft.com/sql/t-sql/language-reference)
- [Database Engine Transact-SQL](https://learn.microsoft.com/sql/t-sql/statements/)
- [Database Console Commands](https://learn.microsoft.com/sql/database-console-commands/)
- [System Stored Procedures](https://learn.microsoft.com/sql/relational-databases/system-stored-procedures/)