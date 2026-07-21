# WAL (Write-Ahead Log) Configuration and Management

## Overview

The Write-Ahead Log (WAL) is the core mechanism that ensures PostgreSQL data integrity. It logs all changes to the database before they are written to the data files. This guarantees that in the event of a crash, PostgreSQL can replay the WAL to restore consistency. WAL is also essential for streaming replication, logical replication, point-in-time recovery (PITR), and backup consistency.

## WAL Fundamentals

### WAL Purpose

1. **Crash Recovery**: After an unclean shutdown, PostgreSQL replays WAL records to bring data to a consistent state
2. **Replication**: WAL records are streamed to standby servers for physical replication
3. **Logical Replication**: Logical decoding extracts change events from WAL for subscriber replication
4. **Point-in-Time Recovery (PITR)**: WAL archiving enables recovery to any point in time within the archive window
5. **ACID Compliance**: WAL ensures the durability aspect of ACID transactions

### WAL Structure

WAL files are stored in the `pg_wal` directory (formerly `pg_xlog` in pre-12 versions).

**File Organization**:
- WAL files are 16MB each by default (`wal_segment_size`)
- Files are named in the format `000000010000000000000001` (8 hex characters per 32-bit segment)
- Format: `<timeline_id><log_file_num><segment_num>`
- Timeline ID: 8 hex digits (for failover scenarios)
- Log File Number: 8 hex digits
- Segment Number: 8 hex digits

**WAL Record Format**:
- Each record contains a record header, data, and previous record pointer
- Records are tagged with LSN (Log Sequence Number) for ordering
- LSN format: 64-bit value split into 32-bit log segment offset and 32-bit record offset within segment

### WAL Positions

PostgreSQL uses several WAL position concepts:

1. **Insert Position**: The current WAL insertion point (where new records are being written)
2. **Flush Position**: The WAL position that has been flushed to disk
3. **Current Position**: The position being actively processed
4. **RecPtr**: The replay position on a standby server

These positions are tracked using LSN (Log Sequence Number) values, which are 64-bit monotonically increasing values representing the position in the WAL.

## WAL Configuration Parameters

### Essential Parameters

| Parameter | Default | Description | Tuning Impact |
|-----------|---------|-------------|---------------|
| `wal_level` | `replica` | Minimum WAL information to write | Determines replication capability |
| `max_wal_size` | `1GB` | Dynamic WAL size before checkpoint | Controls checkpoint frequency |
| `min_wal_size` | `80MB` | Minimum size to retain after checkpoint | Affects WAL file retention |
| `wal_buffers` | `-1` (auto) | WAL write buffer size | Impacts write performance |
| `wal_compression` | `off` | Enable WAL compression | Reduces WAL volume, increases CPU |
| `wal_keep_size` | `0` | Minimum WAL to retain for standbys | Prevents WAL removal for lagging standbys |
| `wal_sender_timeout` | `60s` | Timeout for WAL sender | Controls standby connection tolerance |
| `max_wal_senders` | `10` | Maximum standby connections | Limits concurrent replicas |
| `wal_log_hints` | `off` | Write page headers with hint bits | Required for pg_rewind |
| `full_page_writes` | `on` | Write full page images at checkpoints | Critical for crash recovery |
| `wal_init_file` | `on` | Write WAL init file during init | Safety check for new WAL files |
| `wal_recycle` | `on` | Reuse WAL files instead of removing | Efficiency improvement |

### WAL Level Details

```
none < replica < logical
```

| Level | Purpose | Replication Support |
|-------|---------|-------------------|
| `none` | No WAL needed | None |
| `replica` | Streaming replication, PITR | Physical replication, PITR |
| `logical` | Logical replication, decoding | All of replica + logical |

**Important**: `wal_level` cannot be downgraded after cluster initialization. Setting it to `logical` is recommended for future flexibility.

### WAL Buffers

WAL buffers are shared memory buffers for staging WAL records before writing to disk.

**Configuration**:
- PostgreSQL 13+: Auto-tuned to 1/32 of shared_buffers (minimum 64MB, maximum 64MB)
- PostgreSQL 12 and earlier: Manual tuning required
- Typical values: 64MB for most workloads

**Tuning Guidelines**:
- Most workloads don't need manual tuning (auto-tune works well)
- Increase only if WAL buffer contention is observed (check `pg_stat_bgwriter.wal_write`)
- Each WAL buffer is 8KB internally
- Too many WAL buffers wastes memory; too few causes buffer flush overhead

### Max WAL Size

Controls when automatic checkpoints are triggered:

- `min_wal_size`: Minimum WAL size to retain (default 80MB)
- `max_wal_size`: Dynamic WAL size before checkpoint (default 1GB)

**Tuning Guidelines**:
- For write-heavy workloads: Increase to 4-8GB or more
- For stable, moderate workloads: 1-2GB is usually sufficient
- Too small → frequent checkpoints → I/O spikes
- Too large → longer recovery time, more WAL to send to replicas

**Checkpoint Behavior**:
- Checkpoints are triggered when WAL size exceeds max_wal_size
- The background writer gradually flushes dirty buffers before the checkpoint
- The checkpoint process then writes a checkpoint record to the WAL
- The interval between checkpoints is controlled by `checkpoint_timeout` (default 5 minutes)

## Checkpoint Configuration

### Checkpoint Parameters

| Parameter | Default | Description | Tuning Impact |
|-----------|---------|-------------|---------------|
| `checkpoint_timeout` | `5min` | Maximum time between checkpoints | Lower = more frequent, smoother I/O |
| `checkpoint_completion_target` | `0.9` | Time to spread checkpoint I/O | Higher = smoother, longer checkpoint |
| `checkpoint_flush_after` | `256kB` | Frequency of fsync during checkpoint | Lower = more frequent fsync |
| `checkpoint_warning` | `80%` | Warning if checkpoint occurs within 80% of max_wal_size | Can be disabled |

### Checkpoint Behavior

**Normal Checkpoint Flow**:
1. Background writer gradually flushes dirty buffers
2. When WAL size approaches max_wal_size or timeout reached:
3. Checkpointer writes dirty buffers to disk
4. Checkpoint record is written to WAL
5. All WAL up to checkpoint record is fsynced
6. WAL segment rotation

**Optimal Configuration for Write-Heavy Workloads**:
```ini
checkpoint_timeout = 15min        # Longer interval between checkpoints
max_wal_size = 8GB                # Larger WAL before checkpoint
min_wal_size = 2GB                # Retain more WAL
checkpoint_completion_target = 0.9  # Spread I/O over longer period
```

**For High-Throughput OLTP**:
```ini
checkpoint_timeout = 15min
max_wal_size = 16GB
min_wal_size = 4GB
checkpoint_completion_target = 0.9
```

## WAL Archiving

### Archive Configuration

WAL archiving enables point-in-time recovery by copying completed WAL segments to archive storage.

**Essential Configuration**:
```ini
# postgresql.conf
archive_mode = on
archive_command = 'test ! -f /archive/wal/%f && cp %p /archive/wal/%f'
archive_timeout = 0              # Force WAL switch every N seconds (0 = off)
```

**Archive Command Best Practices**:
- Use `test ! -f` to avoid overwriting existing files
- Return exit code 0 on success, non-zero on failure
- Handle errors gracefully — PostgreSQL will retry failed archives
- Consider compression for remote storage (e.g., `pigz` instead of `cp`)

**Archive Timeout**:
- `archive_timeout = 0` (default): WAL segments archived only when full (16MB)
- `archive_timeout = 60`: Force WAL switch every 60 seconds
- Useful for reducing data loss window in case of catastrophic failure
- Trade-off: more frequent WAL segment switches

### WAL Archive Management

**Archive Cleanup**:
Use `pg_archivecleanup` to remove old WAL files from the archive:

```bash
# Remove all WAL files older than a specific WAL segment
pg_archivecleanup /archive/wal/ 000000010000000000000050
```

**Automated Cleanup via `archive_cleanup_command`**:
```ini
# In postgresql.conf or recovery configuration
archive_cleanup_command = 'pg_archivecleanup /archive/wal %r'
```

The `%r` variable is replaced with the oldest WAL file still needed (determined by replication slots).

### Archive Monitoring

```sql
-- Archive statistics
SELECT archived_count, last_archived_wal, last_archived_time,
       failed_count, last_failed_wal, last_failed_time
FROM pg_stat_archiver;

-- Check if archiving is working
SELECT count(*) FROM pg_stat_archiver WHERE failed_count > 0;

-- WAL size
SELECT pg_size_pretty(pg_wal_size());
```

**Key Metrics**:
- `archived_count`: Total successful archives
- `failed_count`: Total failed archives (should be 0 in production)
- `last_archived_wal`: Most recently archived WAL file
- `last_failed_wal`: Most recent failed archive (if any)

## WAL Retention

### Retention Policy

WAL files must be retained for:

1. **Active replication slots**: Physical slots need all WAL from their restart_lsn
2. **Active replication slots**: Logical slots need all WAL from their restart_lsn
3. **In-progress backups**: pg_basebackup may need WAL from backup start
4. **Unsent WAL**: WAL not yet sent to standby servers
5. **Archive**: WAL not yet archived (if archive_mode = on)
6. **Recovery**: WAL needed for crash recovery

### WAL Retention Calculation

```sql
-- Check WAL retention by replication slots
SELECT slot_name, slot_type, active, restart_lsn,
       pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn) as retained_wal_bytes,
       pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn)) as retained_wal_pretty
FROM pg_replication_slots;

-- Total WAL size on disk
SELECT pg_size_pretty(pg_wal_size());

-- Number of WAL files
SELECT count(*) FROM pg_ls_dir('pg_wal') WHERE name ~ '^[0-9A-F]+$';
```

**Disk Space Estimation**:
- Each WAL file is 16MB by default
- Retained files = (wal_position_diff) / 16MB
- Monitor `retained_wal_bytes` to predict disk space needs

## WAL and Replication

### WAL Senders

WAL sender processes on the primary send WAL records to standby servers.

**Configuration**:
- `max_wal_senders`: Maximum concurrent WAL senders (default 10)
- `wal_sender_timeout`: Timeout for WAL sender to respond (default 60s)
- `wal_keep_size`: Minimum WAL to retain for standbys (default 0)

**WAL Sender Behavior**:
- One WAL sender per connected standby
- Sends WAL from the current insert position to the standby
- Waits for standby acknowledgment (depending on synchronous_commit setting)
- Times out if standby doesn't respond within wal_sender_timeout

**Monitoring WAL Senders**:
```sql
SELECT pid, usename, application_name, client_addr, state,
       sent_lsn, write_lsn, flush_lsn, replay_lsn,
       pg_wal_lsn_diff(sent_lsn, replay_lsn) as replay_lag
FROM pg_stat_replication;
```

### Replication Slots and WAL Retention

Replication slots ensure WAL is not removed until consumed by the standby:

**Impact of Inactive Slots**:
- An inactive (disconnected) slot still retains WAL from its restart_lsn
- If many slots accumulate, disk can fill up
- Regularly audit and clean up inactive slots

```sql
-- Find potentially problematic slots
SELECT slot_name, slot_type, active,
       pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn)) as retained_wal
FROM pg_replication_slots
WHERE NOT active
ORDER BY pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn) DESC;
```

## WAL and Recovery

### Crash Recovery

When PostgreSQL starts after an unclean shutdown:

1. **Startup Phase**: Reads the control file to determine last checkpoint
2. **WAL Recovery Phase**: Replays WAL records from last checkpoint
3. **Shutdown Phase**: Shuts down cleanly

**Recovery Process**:
1. Read pg_control to find last checkpoint LSN
2. Find the WAL segment containing the checkpoint
3. Replay all WAL records from checkpoint to the end of the WAL
4. Apply changes to data files
5. Create a new checkpoint record
6. Start accepting connections

**Recovery Log Messages**:
- `LOG: entering emergency recovery mode` — Crash detected
- `LOG: redo starts at <LSN>` — Recovery starting
- `LOG: redo done at <LSN>` — Recovery complete
- `LOG: last completed transaction was at log time <time>` — Recovery summary

### Point-in-Time Recovery (PITR)

PITR enables recovery to any specific point in time within the archive window.

**PITR Configuration**:
```ini
# postgresql.auto.conf or recovery settings
recovery_target_time = '2024-01-15 14:30:00 UTC'
recovery_target_action = 'promote'    # promote to primary
recovery_target_inclusive = true       # include changes at target time
recovery_target_timeline = 'latest'    # follow latest timeline
```

**PITR Target Types**:
- `recovery_target_time`: Specific timestamp
- `recovery_target_xid`: Specific transaction ID
- `recovery_target_lsn`: Specific WAL position
- `recovery_target_name`: Named savepoint
- `recovery_target`: Named restore point (created with `pg_create_restore_point()`)

**PITR Process**:
1. Stop PostgreSQL
2. Back up current data directory
3. Restore from pg_basebackup
4. Configure recovery_target settings
5. Start PostgreSQL (enters recovery mode)
6. WAL is replayed until target is reached
7. Server promotes to primary (if recovery_target_action = 'promote')
8. Server becomes available for connections

## WAL Best Practices

### Performance Tuning

1. **Increase max_wal_size** for write-heavy workloads (4-16GB)
2. **Increase checkpoint_completion_target** to 0.9 for smoother I/O
3. **Increase checkpoint_timeout** to 15min+ for steady-state write loads
4. **Enable WAL compression** if disk space is a concern (wal_compression = on)
5. **Enable full_page_writes** (default on) — critical for crash recovery
6. **Ensure fast storage** for WAL files (SSD/NVMe recommended)

### Monitoring

1. Monitor `pg_stat_archiver.failed_count` — should be 0
2. Monitor `pg_wal_size()` — should be stable or slowly growing
3. Monitor replication slot retention — watch for growing retained_wal_bytes
4. Monitor checkpoint frequency via `pg_stat_bgwriter`
5. Monitor WAL generation rate via `pg_stat_wal` (PostgreSQL 14+)

### Safety

1. **Never delete WAL files manually** — only use pg_archivecleanup
2. **Never remove pg_wal directory contents** without understanding implications
3. **Always verify archive_command** returns correct exit codes
4. **Test PITR procedures regularly** — assume backup is useless until tested
5. **Monitor archive disk space** — WAL archives can accumulate indefinitely
6. **Enable wal_log_hints** if planning to use pg_rewind
7. **Enable data_checksums** during initdb for data integrity detection

### Common WAL Issues

| Issue | Symptom | Resolution |
|-------|---------|------------|
| WAL archive filling disk | `no space left on device` | Use pg_archivecleanup to remove old WAL |
| WAL growth from inactive slots | Growing pg_wal_size | Drop inactive replication slots |
| Frequent checkpoints | I/O spikes, slow queries | Increase max_wal_size |
| Archive failures | failed_count > 0 | Fix archive_command, check archive disk space |
| WAL corruption | Recovery fails | Use pg_resetwal (emergency), restore from backup |
| High WAL generation | Fast disk fill | Check for large writes, reduce max_wal_size temporarily |

## Version-Specific WAL Features

### PostgreSQL 15
- Improved parallel WAL writing
- Enhanced logical replication WAL handling
- Better WAL archiving error handling

### PostgreSQL 16
- Enhanced WAL statistics (`pg_stat_wal`)
- Improved WAL archiver monitoring
- Better checkpoint performance

### PostgreSQL 17
- Further WAL writing improvements
- Enhanced logical decoding performance
- Better WAL retention management

## References

- [PostgreSQL WAL Internals](https://www.postgresql.org/docs/current/wal-internals.html)
- [PostgreSQL WAL Configuration](https://www.postgresql.org/docs/current/runtime-config-wal.html)
- [PostgreSQL Recovery Configuration](https://www.postgresql.org/docs/current/recovery-config.html)
- [PostgreSQL Point-in-Time Recovery](https://www.postgresql.org/docs/current/backup-wal.html)
- [EnterpriseDB Documentation](https://www.enterprisedb.com/docs/)