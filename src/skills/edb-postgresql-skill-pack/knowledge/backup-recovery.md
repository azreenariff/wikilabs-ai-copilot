# EDB PostgreSQL Backup and Recovery

## Overview

EDB PostgreSQL backup and recovery procedures cover pg_dump, pg_basebackup, WAL archiving, and disaster recovery.

## Backup Components

### Components to Backup

| Component | Location | Frequency |
|-----------|----------|-----------|
| **Database Data** | pg_dump, pg_basebackup | Daily |
| **WAL Files** | pg_wal/ | Continuous |
| **Configuration** | postgresql.conf, pg_hba.conf | Daily |
| **Custom Types** | Schema definitions | Weekly |

## Backup Procedures

### Logical Backup (pg_dump)

```bash
# Full database backup
pg_dump -U postgres -F c -f backup.dump mydb

# Schema-only backup
pg_dump -U postgres -s -f schema.sql mydb

# Parallel backup
pg_dump -U postgres -j 4 -F c -f backup.dump mydb
```

### Physical Backup (pg_basebackup)

```bash
# Full cluster backup
pg_basebackup -U postgres -D /backup/pgdata -Ft -z -P

# Streaming replication base backup
pg_basebackup -U postgres -h primary -D /backup/replica -Ft -z -P -R
```

### WAL Archiving

```ini
# postgresql.conf
wal_level = replica
archive_mode = on
archive_command = 'cp %p /backup/wal/%f'
```

## Recovery Procedures

### Point-in-Time Recovery

```bash
# Stop PostgreSQL
pg_ctl stop

# Restore base backup
rm -rf /var/lib/postgresql/data/*
tar xzf /backup/pgdata.tar.gz -C /var/lib/postgresql/data/

# Configure recovery
cat > recovery.signal << EOF
restore_command = 'cp /backup/wal/%f %p'
recovery_target_time = '2024-01-15 10:30:00'
EOF

# Start PostgreSQL
pg_ctl start
```

### Logical Recovery

```bash
# Restore from pg_dump
pg_restore -U postgres -d mydb backup.dump

# Restore schema only
psql -U postgres -d mydb -f schema.sql
```

## References

- EDB PostgreSQL Backup: https://www.enterprisedb.com/docs/
- PostgreSQL Backup: https://www.postgresql.org/docs/current/backup.html
- PostgreSQL PITR: https://www.postgresql.org/docs/current/warm-standby.html