# MySQL Command Reference

## Overview

This reference documents all MySQL commands, utilities, and SQL statements organized by category. Each entry includes purpose, risk assessment, parameters, usage examples, and documentation references.

## Command Categories

### 1. Connection (`connection`)

| ID | Command | Risk | Description |
|----|---------|------|-------------|
| mysql-connect | mysql | Low | Connect to MySQL server |
| mysql-version | mysql --version | Low | Display client version |
| mysql-database | mysql -D | Low | Connect to specific database |
| mysql-tee | mysql --tee | Low | Enable query logging to file |
| mysql-source | source / \. | Medium | Execute SQL script file |

### 2. Schema Management (`schema-management`)

| ID | Command | Risk | Description |
|----|---------|------|-------------|
| mysql-show-databases | SHOW DATABASES | Low | List accessible databases |
| mysql-use-db | USE | Low | Set default database |
| mysql-show-tables | SHOW TABLES | Low | List tables in database |
| mysql-describe | DESC | Low | Show table structure |
| mysql-show-create-table | SHOW CREATE TABLE | Low | Show table creation SQL |
| mysql-show-indices | SHOW INDEX | Low | Show index information |
| mysql-show-charset | SHOW CHARACTER SET | Low | Show available charsets |
| mysql-show-collations | SHOW COLLATION | Low | Show available collations |
| mysql-show-create-db | SHOW CREATE DATABASE | Low | Show database creation SQL |
| mysql-show-create-view | SHOW CREATE VIEW | Low | Show view definition |
| mysql-show-create-procedure | SHOW CREATE PROCEDURE | Low | Show procedure definition |
| mysql-show-create-trigger | SHOW CREATE TRIGGER | Low | Show trigger definition |
| mysql-show-duplicate-keys | SELECT GROUP BY HAVING | Low | Find duplicate values |
| mysql-show-tables-used | INFORMATION_SCHEMA.TABLES | Low | Query table metadata |
| mysql-show-indexes-info-schema | INFORMATION_SCHEMA.STATISTICS | Low | Query index details |
| mysql-create-database | CREATE DATABASE | Low | Create new database |
| mysql-drop-database | DROP DATABASE | High | Delete database |
| mysql-create-table | CREATE TABLE | Medium | Create new table |
| mysql-drop-table | DROP TABLE | High | Delete table |
| mysql-create-table-as | CREATE TABLE AS SELECT | Medium | Create table from query |
| mysql-create-view | CREATE VIEW | Medium | Create virtual table |
| mysql-drop-view | DROP VIEW | Medium | Remove view definition |
| mysql-create-index | CREATE INDEX | Medium | Add index to table |
| mysql-schema-drop-index | DROP INDEX | Medium | Remove index |
| mysql-schema-fulltext-index | CREATE FULLTEXT INDEX | Medium | Add fulltext index |
| mysql-schema-create-trigger | CREATE TRIGGER | Medium | Create table trigger |
| mysql-schema-drop-trigger | DROP TRIGGER | Medium | Remove trigger |
| mysql-schema-create-event | CREATE EVENT | Medium | Create scheduled event |
| mysql-schema-drop-event | DROP EVENT | Low | Remove scheduled event |
| mysql-schema-create-procedure | CREATE PROCEDURE | Medium | Create stored procedure |
| mysql-schema-create-function | CREATE FUNCTION | Medium | Create stored function |
| mysql-schema-partition | CREATE TABLE PARTITIONED | Medium | Create partitioned table |
| mysql-alter-table-add | ALTER TABLE ADD COLUMN | High | Add column to table |
| mysql-alter-table-drop | ALTER TABLE DROP COLUMN | High | Remove column from table |
| mysql-alter-table-modify | ALTER TABLE MODIFY | High | Modify column type |
| mysql-alter-table-add-index | ALTER TABLE ADD INDEX | Medium | Add index via ALTER |
| mysql-alter-table-drop-index | ALTER TABLE DROP INDEX | Medium | Remove index via ALTER |
| mysql-alter-table-add-constraint | ALTER TABLE ADD CONSTRAINT | High | Add foreign key |
| mysql-rename-table | RENAME TABLE | Medium | Rename tables |

### 3. Status and Monitoring (`status`)

| ID | Command | Risk | Description |
|----|---------|------|-------------|
| mysql-show-status | SHOW STATUS | Low | Show server status vars |
| mysql-show-variables | SHOW VARIABLES | Low | Show server variables |
| mysql-show-processlist | SHOW FULL PROCESSLIST | Low | List active threads |
| mysql-show-engines | SHOW ENGINES | Low | Show storage engines |
| mysql-show-grants | SHOW GRANTS | Low | Show user privileges |
| mysql-show-slave-status | SHOW REPLICA STATUS | Low | Show replication status |
| mysql-show-open-tables | SHOW OPEN TABLES | Low | Show open table cache |
| mysql-show-table-status | SHOW TABLE STATUS | Low | Show table status |
| mysql-show-warnings | SHOW WARNINGS | Low | Show statement warnings |
| mysql-show-errors | SHOW ERRORS | Low | Show statement errors |
| mysql-show-global-status | SHOW GLOBAL STATUS | Low | Show global counters |
| mysql-show-global-variables | SHOW GLOBAL VARIABLES | Low | Show global variables |
| mysql-kill-process | KILL | High | Terminate a process |
| mysql-ping | mysqladmin ping | Low | Check server liveness |
| mysqladmin-status | mysqladmin status | Low | Show brief status |
| mysqladmin-variables | mysqladmin variables | Low | Show server variables |
| mysqladmin-processlist | mysqladmin processlist | Low | Show active processes |
| mysqladmin-ext-status | mysqladmin extended-status | Low | Show extended stats |
| mysqladmin-kill | mysqladmin kill | High | Kill a connection |
| mysqladmin-version | mysqladmin version | Low | Show version info |
| mysqladmin-debug | mysqladmin debug | Low | Print debug info |
| mysql-admin-show-slave-hosts | SHOW SLAVE HOSTS | Low | Show registered replicas |
| mysql-admin-show-binary-log-pos | SHOW MASTER STATUS | Low | Show current binlog pos |
| mysql-admin-show-triggers | SHOW TRIGGERS | Low | Show all triggers |
| mysql-admin-show-events | SHOW EVENTS | Low | Show scheduled events |
| mysql-admin-show-procedures | SHOW PROCEDURE STATUS | Low | Show stored procedures |

### 4. Query Execution (`query-execution`)

| ID | Command | Risk | Description |
|----|---------|------|-------------|
| mysql-select | SELECT | Medium | Query data |
| mysql-insert | INSERT | Medium | Insert rows |
| mysql-update | UPDATE | High | Modify rows |
| mysql-delete | DELETE | High | Remove rows |
| mysql-truncate | TRUNCATE TABLE | High | Wipe all rows |

### 5. Performance (`performance`)

| ID | Command | Risk | Description |
|----|---------|------|-------------|
| mysql-explain | EXPLAIN | Low | Show query execution plan |
| mysql-explain-analyze | EXPLAIN ANALYZE | Low | Actual execution plan with metrics |
| mysql-analyze-table | ANALYZE TABLE | Low | Update table statistics |
| mysql-perf-show-profile | SHOW PROFILE | Low | Show query profile (deprecated) |
| mysql-perf-optimizer-trace | OPTIMIZER_TRACE | Low | Trace optimizer decisions |
| mysql-perf-slow-log | Slow Query Log | Low | Log slow queries |
| mysql-perf-analyze | ANALYZE TABLE | Low | Update optimizer stats |
| mysql-admin-show-engine | SHOW ENGINE INNODB STATUS | Low | InnoDB internal status |
| mysql-sys-schema | SYS Schema Queries | Low | Performance insights |
| mysql-sys-schema-waits | schema_table_lock_waits | Low | Lock wait analysis |
| mysql-sys-schema-io | schema_io_global_by_file | Low | I/O by file |
| mysql-sys-schema-top-statements | statement_analysis | Low | Top statements |
| mysql-sys-schema-unused-indexes | schema_unused_indexes | Low | Unused index detection |
| mysql-sys-schema-memory | memory_global_total | Low | Memory usage |
| mysql-sys-schema-hosts | host_summary | Low | Host statistics |
| mysql-sys-schema-stages | stages | Low | Statement stages |
| mysql-sys-schema-host-latency | host_summary_by_stages | Low | Host latency |
| mysql-sys-schema-user-summary | user_summary | Low | User statistics |
| mysql-sys-schema-processlist | processlist | Low | Enhanced processlist |
| mysql-admin-optimizer-switch | optimizer_switch | Medium | Toggle optimizer features |

### 6. Replication (`replication`)

| ID | Command | Risk | Description |
|----|---------|------|-------------|
| mysql-show-slave-status | SHOW REPLICA STATUS | Low | Replica status |
| mysql-show-binary-logs | SHOW BINARY LOGS | Low | List binlog files |
| mysql-show-binlog-events | SHOW BINLOG EVENTS | Low | Show binlog events |
| mysql-admin-start-replica | START REPLICA | Medium | Start replica threads |
| mysql-admin-stop-replica | STOP REPLICA | Medium | Stop replica threads |
| mysql-admin-reset-slave | RESET REPLICA | High | Reset replication config |
| mysql-admin-reset-master | RESET BINARY LOGS | High | Clear binary logs |
| mysql-admin-purge-binary-logs | PURGE BINARY LOGS | High | Delete old binlogs |
| mysql-admin-change-master | CHANGE REPLICATION SOURCE | Medium | Configure replication |
| mysql-replication-gtid-variables | GTID Variables | Low | Check GTID config |
| mysql-replication-source-set | SET REPLICATION SOURCE | Medium | Configure replica source |
| mysql-replication-filter | CHANGE REPLICATION FILTER | Medium | Set replication filters |
| mysql-replication-group | Group Replication | High | Manage MGR cluster |

### 7. Backup (`backup`)

| ID | Command | Risk | Description |
|----|---------|------|-------------|
| mysql-backup-mysqldump | mysqldump | Low | Logical backup |
| mysql-backup-mysqldump-all | mysqldump --all | Medium | Dump all databases |
| mysql-backup-mysqldump-tables | mysqldump --tables | Low | Dump specific tables |
| mysql-backup-mysqldump-master-data | mysqldump --master-data | Low | Backup with binlog position |
| mysql-backup-mysqlpump | mysqlpump | Low | Parallel logical backup |
| mysql-backup-restore | mysql (restore) | Medium | Restore from dump |
| mysql-backup-mysqlbinlog | mysqlbinlog | Low | Decode binlog events |
| mysql-backup-flush-logs | mysqladmin flush-logs | Low | Rotate binary logs |
| mysql-backup-gtid | GTID Execution Tracking | Low | Track GTID sets |
| mysql-backup-xtrabackup-backup | xtrabackup --backup | Low | Physical backup |
| mysql-backup-xtrabackup-prepare | xtrabackup --prepare | Medium | Prepare physical backup |
| mysql-backup-xtrabackup-copy | xtrabackup --copy-back | High | Copy backup to data dir |
| mysql-backup-xtrabackup-incremental | xtrabackup incremental | Low | Incremental backup |

### 8. Security (`security`)

| ID | Command | Risk | Description |
|----|---------|------|-------------|
| mysql-create-user | CREATE USER | Low | Create user account |
| mysql-drop-user | DROP USER | Medium | Remove user account |
| mysql-alter-user-rename | ALTER USER RENAME | Medium | Rename user |
| mysql-alter-user-password | ALTER USER PASSWORD | Medium | Change password |
| mysql-create-role | CREATE ROLE | Low | Create role |
| mysql-drop-role | DROP ROLE | Medium | Remove role |
| mysql-grant | GRANT | Medium | Grant privileges |
| mysql-revoke | REVOKE | Medium | Revoke privileges |
| mysql-flush-privileges | FLUSH PRIVILEGES | Low | Reload grant tables |
| mysql-rename-user | RENAME USER | Medium | Rename account |
| mysql-create-user-ssl | CREATE USER REQUIRE SSL | Low | SSL-required user |

### 9. Administration (`administration`)

| ID | Command | Risk | Description |
|----|---------|------|-------------|
| mysql-admin-set-global | SET GLOBAL | Medium | Set global variable |
| mysql-admin-set-session | SET SESSION | Low | Set session variable |
| mysqladmin-shutdown | mysqladmin shutdown | High | Stop MySQL server |
| mysqladmin-flush-logs | mysqladmin flush-logs | Low | Rotate logs |
| mysqladmin-flush-hosts | mysqladmin flush-hosts | Low | Flush DNS cache |
| mysqladmin-refresh | mysqladmin refresh | Low | Flush tables and logs |
| mysql-admin-persist | SET PERSIST | Medium | Set persistent variable |
| mysql-admin-reset-persist | RESET PERSIST | Low | Remove persisted vars |
| mysql-admin-secure-file | --secure-file-priv | Low | Check file privilege |
| mysql-admin-log-config | log variables | Low | Check log configuration |
| mysql-admin-defaults-file | --defaults-file | Low | Custom config file |

### 10. Maintenance (`maintenance`)

| ID | Command | Risk | Description |
|----|---------|------|-------------|
| mysql-check-table | CHECK TABLE | Low | Check table corruption |
| mysql-optimize-table | OPTIMIZE TABLE | Medium | Defragment table |
| mysql-repair-table | REPAIR TABLE | Medium | Repair corrupted table |
| mysql-maintenance-binlog-expire | binlog_expire_logs_seconds | Low | Binlog expiration |
| mysql-maintenance-table-cache | table_open_cache | Low | Table cache tuning |
| mysql-maintenance-log-size | innodb_log_file_size | Low | Redo log tuning |
| mysql-maintenance-dd | INFORMATION_SCHEMA | Low | Data dictionary queries |

## Version-Aware Notes

### MySQL 8.0 vs 8.4 Differences

- `SHOW REPLICA STATUS` replaces `SHOW SLAVE STATUS` (both still work in 8.0)
- `CHANGE REPLICATION SOURCE TO` replaces `CHANGE MASTER TO`
- `RESET REPLICA` replaces `RESET SLAVE`
- `binlog_expire_logs_seconds` replaces `expire_logs_days` (MySQL 8.0.22+)
- SET PERSIST and SET PERSIST_ONLY are new persistence mechanisms (MySQL 8.0.2+)
- InnoDB data dictionary is internal (no longer filesystem-based .frm files)
- MySQL 8.4 removes several deprecated features: LOAD DATA INFILE local option, PASSWORD() function, certain cipher suites

### Percona Server Compatibility

- Most MySQL commands work identically on Percona Server
- Percona adds: pt-archiver, pt-osc, pt-deadlock-logger, pt-summary
- Percona XtraBackup replaces native mysqldump for large datasets