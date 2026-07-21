# EDB PostgreSQL Command Reference

## Overview

This reference documents all PostgreSQL commands covered by this skill pack, organized by operational category. Each entry includes syntax, risk level, usage context, and examples.

## Connection Commands

### psql
PostgreSQL interactive terminal — primary CLI for SQL execution, database inspection, and meta-commands.

```
psql [OPTIONS] [DBNAME [USERNAME]]
```

- **Risk**: Low
- **Example**: `psql -h localhost -p 5432 -U postgres -d mydb`
- **Meta-commands**: `\q` quit, `\l` list databases, `\d` list tables, `\d tablename` table details
- **Note**: Use `-f file.sql` to execute SQL from a file; use `-c "SQL"` for one-off queries

### pg_isready
Check if PostgreSQL server is accepting connections.

```
pg_isready [-h host] [-p port] [-d database]
```

- **Risk**: Low
- **Example**: `pg_isready -h localhost -p 5432`
- **Returns**: Exit code 0 if accepting connections, 1 if not
- **Use**: Health checks, startup verification, monitoring scripts

## Status and Lifecycle Commands

### pg_ctl start
Start the PostgreSQL server.

```
pg_ctl start -D <data_directory> [-l <logfile>] [-o "<options>"]
```

- **Risk**: Low
- **Example**: `pg_ctl start -D /var/lib/pgsql/data -l /var/lib/pgsql/logs/pg.log`
- **Note**: Use `-w` to wait for startup completion

### pg_ctl stop
Stop PostgreSQL with configurable shutdown mode.

```
pg_ctl stop -D <data_directory> [-m <mode>] [-W]
```

- **Shutdown modes**:
  - `smart` (default): wait for clients to disconnect
  - `fast`: immediate — abort all transactions
  - `immediate`: abort all transactions and shut down immediately
- **Risk**: Medium (immediate mode)
- **Example**: `pg_ctl stop -D /var/lib/pgsql/data -m fast`

### pg_ctl restart
Restart PostgreSQL (stop then start).

```
pg_ctl restart -D <data_directory> [-m <mode>] [-W]
```

- **Risk**: Medium — brief service interruption
- **Note**: Applies configuration changes after restart

### pg_ctl reload
Reload configuration without restarting.

```
pg_ctl reload -D <data_directory>
```

- **Risk**: Low
- **Reloads**: postgresql.conf, pg_hba.conf, pg_ident.conf
- **Note**: Most postgresql.conf changes require reload or restart

### pg_ctl status
Check server status.

```
pg_ctl status -D <data_directory>
```

- **Risk**: Low
- **Example**: `pg_ctl status -D /var/lib/pgsql/data`
- **Note**: Exit code 0 = running, 1 = not running

### pg_ctl promote
Promote standby to primary.

```
pg_ctl promote -D <data_directory>
```

- **Risk**: High — changes replication topology
- **Use**: Failover procedures, converting standby to primary

### pg_upgrade
Upgrade between major PostgreSQL versions.

```
pg_upgrade -b <old_bin_dir> -B <new_bin_dir> -d <old_data_dir> -D <new_data_dir>
```

- **Risk**: High — replaces data directory content
- **Example**: `pg_upgrade -b /opt/pgsql/15/bin -B /opt/pgsql/17/bin -d /data/15 -D /data/17`
- **Note**: Requires downtime; consider pg_dumpall for zero-downtime upgrade

### pg_ctlcluster
Debian/Ubuntu cluster management.

```
pg_ctlcluster <version> <cluster> start|stop|restart|reload
```

- **Risk**: Low
- **Example**: `pg_ctlcluster 15 main start`
- **Note**: Debian-specific wrapper around pg_ctl

## Schema Management Commands

### CREATE DATABASE
Create a new database.

```
CREATE DATABASE dbname OWNER owner TEMPLATE template0;
```

- **Risk**: Low
- **Note**: Requires superuser or CREATEDB privilege

### DROP DATABASE
Delete a database and all its objects.

```
DROP DATABASE [IF EXISTS] dbname [FORCE];
```

- **Risk**: High — irreversible data destruction
- **Note**: Cannot execute inside transaction block

### CREATE TABLE
Create a table with columns and constraints.

```
CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT NOT NULL);
```

- **Risk**: Low
- **Note**: Use IF NOT EXISTS for idempotent DDL

### ALTER TABLE
Modify table structure.

```
ALTER TABLE users ADD COLUMN email TEXT UNIQUE;
```

- **Risk**: Medium — may require AccessExclusiveLock
- **Use**: Adding columns, modifying constraints, changing types

### CREATE INDEX / DROP INDEX
Create or remove indexes.

```
CREATE [UNIQUE] INDEX CONCURRENTLY idx_name ON table (column);
DROP INDEX CONCURRENTLY idx_name;
```

- **Risk**: Low (CONCURRENTLY mode does not block writes)
- **Note**: CONCURRENTLY cannot run inside transaction blocks

### CREATE TABLESPACE
Create a tablespace at a filesystem location.

```
CREATE TABLESPACE name OWNER owner LOCATION 'path';
```

- **Risk**: Low
- **Note**: Directory must be owned by PostgreSQL OS user

### CREATE SCHEMA / DROP SCHEMA
Create or remove schemas.

```
CREATE SCHEMA app_data AUTHORIZATION app_user;
DROP SCHEMA IF EXISTS old_app CASCADE;
```

- **Risk**: Low to Medium (CASCADE removes all objects)

### CREATE TRIGGER / DROP TRIGGER
Manage triggers for automated actions.

```
CREATE TRIGGER audit_trigger AFTER INSERT ON users EXECUTE PROCEDURE audit_log();
```

- **Risk**: Low

### CREATE VIEW / DROP VIEW
Create or remove views.

```
CREATE VIEW active_users AS SELECT * FROM users WHERE active = true;
```

- **Risk**: Low

### CREATE SEQUENCE / NEXTVAL
Create and use sequence generators.

```
CREATE SEQUENCE users_id_seq START WITH 1000 INCREMENT BY 1;
SELECT NEXTVAL('users_id_seq');
```

- **Risk**: Low

### CREATE TYPE
Create composite data types.

```
CREATE TYPE address AS (street TEXT, city TEXT, zip TEXT);
```

- **Risk**: Low

### CLUSTER
Physically reorder a table based on an index.

```
CLUSTER users USING idx_users_email;
```

- **Risk**: Medium — requires AccessExclusiveLock
- **Note**: Consider pg_repack for production

### TRUNCATE
Remove all rows from a table.

```
TRUNCATE TABLE temp_data CASCADE;
```

- **Risk**: High — irreversible
- **Note**: Cannot be rolled back in a transaction

### REINDEX DATABASE / TABLE
Rebuild indexes.

```
REINDEX DATABASE mydb;
REINDEX CONCURRENTLY TABLE users;  -- PostgreSQL 15+
```

- **Risk**: Medium
- **Use**: After bulk operations, fixing corrupted indexes

## Security Commands

### CREATE ROLE / ALTER ROLE
Manage database roles.

```
CREATE ROLE app_user WITH LOGIN PASSWORD 'secret';
ALTER ROLE app_user WITH SUPERUSER;
```

- **Risk**: Low to Medium (privilege escalation)

### GRANT / REVOKE
Manage object privileges.

```
GRANT SELECT, INSERT, UPDATE ON users TO app_user;
REVOKE DELETE ON users FROM app_user;
```

- **Risk**: Low to Medium

### pg_checksums
Enable or disable data checksums.

```
pg_checksums --enable -D /var/lib/pgsql/data
```

- **Risk**: Medium — enables integrity detection
- **Note**: Cannot be changed after cluster initialization

## Backup and Recovery Commands

### pg_dump
Logical backup of a single database.

```
pg_dump -h localhost -U postgres -Fc mydb > mydb.backup
```

- **Formats**: `-F p` (plain SQL), `-F c` (custom), `-F d` (directory), `-F t` (tar)
- **Risk**: Low — read-only
- **Use**: Logical backup, schema migration

### pg_dumpall
Backup all databases and global objects.

```
pg_dumpall -U postgres -f all_databases.sql
```

- **Risk**: Low — read-only
- **Note**: Includes roles, tablespaces, global objects

### pg_restore
Restore from pg_dump custom/directory format.

```
pg_restore -h localhost -U postgres -d mydb mydb.backup
```

- **Risk**: Medium — modifies database content
- **Use**: Restoring logical backups

### pg_basebackup
Physical base backup.

```
pg_basebackup -h localhost -U postgres -D /backup/pg_base -Fp -z -P -X fetch
```

- **Risk**: Low — read-only, consumes I/O
- **Use**: Replication setup, disaster recovery

### pg_resetwal
Reset WAL on corrupted cluster (emergency).

```
pg_resetwal -D /var/lib/pgsql/data -x 123456789
```

- **Risk**: Critical — can cause data loss
- **Use**: Emergency recovery only

### pg_rewind
Synchronize a diverged primary after failover.

```
pg_rewind -D /var/lib/pgsql/data --source-server=host=primary
```

- **Risk**: High — replaces data directory
- **Note**: Requires wal_log_hints=on or data_checksums=on

### initdb
Initialize a new cluster.

```
initdb -D /var/lib/pgsql/data -U postgres -A scram-sha-256
```

- **Risk**: Critical — creates new cluster
- **Use**: Initial cluster setup only

## Performance Commands

### EXPLAIN ANALYZE
Show query plan with actual execution statistics.

```
EXPLAIN (ANALYZE, BUFFERS, VERBOSE) SELECT * FROM users WHERE email = 'test';
```

- **Risk**: Low (ANALYZE executes the query)
- **Use**: Query optimization, verifying index usage

### SHOW / ALTER SYSTEM SET
Inspect and modify runtime parameters.

```
SHOW shared_buffers;
ALTER SYSTEM SET work_mem = '128MB';
SELECT pg_reload_conf();
```

- **Risk**: Medium — global server impact

### VACUUM / ANALYZE
Reclaim storage and update statistics.

```
VACUUM FULL ANALYZE;  -- Full vacuum with lock
VACUUM (VERBOSE, ANALYZE) users;  -- Regular vacuum
```

- **Risk**: Medium for FULL mode
- **Use**: Table maintenance, xid wraparound prevention

## Replication Commands

### pg_is_in_recovery()
Check if server is in standby mode.

```
SELECT pg_is_in_recovery();
```

- **Returns**: true on standby, false on primary

### Replication Slots

```
SELECT pg_create_physical_replication_slot('standby1_slot');
SELECT pg_create_logical_replication_slot('logical_slot1', 'pgoutput');
SELECT pg_drop_replication_slot('standby1_slot');
```

- **Risk**: Low
- **Note**: Inactive slots consume WAL and may fill disk

### Logical Replication

```
CREATE PUBLICATION my_pub FOR TABLE users, orders;
CREATE SUBSCRIPTION my_sub CONNECTION 'host=pub dbname=mydb' PUBLICATION my_pub;
DROP PUBLICATION IF EXISTS my_pub;
DROP SUBSCRIPTION IF EXISTS my_sub;
```

- **Use**: Setting up logical replication

### pg_wal_* Functions
WAL position monitoring.

```
SELECT pg_current_wal_insert_lsn();
SELECT pg_wal_lsn_diff(pg_current_wal_lsn(), pg_last_wal_replay_lsn());
```

- **Risk**: Low
- **Use**: Replication lag calculation

## Monitoring and Statistics

### pg_stat_activity
Monitor active sessions.

```
SELECT pid, usename, datname, state, query FROM pg_stat_activity WHERE state = 'active';
```

- **Use**: Identifying blocking queries, finding idle sessions

### pg_stat_database
Database-level statistics.

```
SELECT datname, xact_commit, xact_rollback, blks_hit FROM pg_stat_database;
```

- **Use**: Hit ratios, transaction counts, database health

### pg_stat_user_tables
Table-level statistics.

```
SELECT relname, seq_scan, idx_scan, n_dead_tup FROM pg_stat_user_tables ORDER BY n_dead_tup DESC;
```

- **Use**: Table health, vacuum readiness

### pg_stat_replication
Replication status from primary.

```
SELECT client_addr, state, sent_lsn, write_lsn, flush_lsn, replay_lsn FROM pg_stat_replication;
```

- **Use**: Monitoring replication, lag calculation

### pg_stat_bgwriter
Background writer statistics.

```
SELECT checkpoints_timed, checkpoints_requested, buffers_checkpoint FROM pg_stat_bgwriter;
```

- **Use**: Checkpoint behavior analysis

### pg_stat_wal (PG 14+)
WAL subsystem statistics.

```
SELECT wal_records, wal_bytes, wal_fpi FROM pg_stat_wal;
```

- **Use**: WAL generation rate monitoring

### pg_stat_archiver
WAL archiving statistics.

```
SELECT archived_count, last_archived_wal, failed_count FROM pg_stat_archiver;
```

- **Use**: Detecting archive failures

### pg_stat_statements
Performance monitoring extension.

```
SELECT query, calls, mean_exec_time, total_exec_time FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;
```

- **Note**: Must be in shared_preload_libraries
- **Use**: Identifying top resource-consuming queries

### pg_size_* Functions
Size reporting functions.

```
SELECT pg_size_pretty(pg_database_size('mydb'));
SELECT pg_size_pretty(pg_table_size('users'));
SELECT pg_size_pretty(pg_indexes_size('users'));
SELECT pg_size_pretty(pg_wal_size());  -- PostgreSQL 13+
```

- **Use**: Capacity planning, reporting

### pg_blocking_pids() (PG 11+)
Find blocking sessions.

```
SELECT pg_blocking_pids(1234);
```

- **Use**: Identifying lock blockers

## Lock Management

### LOCK TABLE
Explicit table locking.

```
LOCK TABLE users IN EXCLUSIVE MODE;
```

- **Lock modes**: AccessShare, RowShare, RowExclusive, Share, ShareRowExclusive, Exclusive, AccessExclusive
- **Risk**: Medium — can block other transactions

### SET lock_timeout / statement_timeout
Timeout settings.

```
SET lock_timeout = '5s';
SET statement_timeout = '30s';
```

- **Risk**: Low
- **Use**: Preventing indefinite waits

### pg_cancel_backend() / pg_terminate_backend()
Session management.

```
SELECT pg_cancel_backend(1234);    -- Cancel current query
SELECT pg_terminate_backend(1234); -- Terminate connection
```

- **Risk**: Medium to High
- **Use**: Cancelling stuck queries, killing unresponsive sessions

## Maintenance Commands

### VACUUM variants

```
VACUUM ANALYZE;                        -- Regular vacuum
VACUUM FULL ANALYZE;                   -- Full vacuum (exclusive lock)
VACUUM (VERBOSE, ANALYZE) users;       -- Targeted vacuum
```

- **Risk**: Medium for FULL mode
- **Note**: Consider pg_repack for zero-downtime vacuum

### vacuumdb
Command-line vacuum wrapper.

```
vacuumdb --all --analyze --jobs=4
```

- **Use**: Automated vacuum scheduling

### pg_test_fsync / pg_test_timing
Storage validation utilities.

```
pg_test_fsync -t 5;
pg_test_timing;
```

- **Use**: Verifying fsync support, timer precision

### pg_archivecleanup
WAL archive cleanup.

```
pg_archivecleanup /archive/wal 000000010000000000000010
```

- **Use**: Managing WAL archive disk space

## Extension Management

### CREATE EXTENSION / DROP EXTENSION
Install or remove extensions.

```
CREATE EXTENSION pg_stat_statements;
ALTER EXTENSION pg_stat_statements UPDATE;
DROP EXTENSION IF EXISTS pg_stat_statements;
```

- **Risk**: Low to Medium
- **Note**: Some extensions require shared_preload_libraries