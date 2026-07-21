# Checkmk Backup and Recovery

## Overview

Checkmk backup and recovery procedures cover site configuration, monitoring data, and disaster recovery.

## Backup Components

### Components to Backup

| Component | Location | Frequency |
|-----------|----------|-----------|
| **Site Configuration** | /opt/omd/sites/SITENAME/etc/ | Daily |
| **Site Data** | /opt/omd/sites/SITENAME/var/ | Daily |
| **Custom Checks** | /opt/omd/sites/SITENAME/local/ | Daily |
| **Agent Configuration** | /etc/check_mk/ | Weekly |
| **Database** | PostgreSQL/MySQL | Daily |
| **RRD Files** | /opt/omd/sites/SITENAME/var/push/ | Weekly |

## Backup Procedures

### Full Site Backup

```bash
# Stop site
omd sitename stop

# Create backup
tar czf /backup/checkmk-sitename-$(date +%Y%m%d).tar.gz \
  /opt/omd/sites/sitename/

# Restart site
omd sitename start
```

### Live Backup (with site running)

```bash
# Backup configuration only
tar czf /backup/checkmk-config-$(date +%Y%m%d).tar.gz \
  /opt/omd/sites/sitename/etc/ \
  /opt/omd/sites/sitename/local/

# Backup database
pg_dump -U sitename sitename | gzip > /backup/db-$(date +%Y%m%d).sql.gz
```

### Automated Backup Script

```bash
#!/bin/bash
# Checkmk Backup Script
SITENAME=sitename
DATE=$(date +%Y%m%d)
BACKUP_DIR="/backup/checkmk"
RETENTION_DAYS=30

mkdir -p $BACKUP_DIR

# Stop site
omd $SITENAME stop

# Create backup
tar czf $BACKUP_DIR/sitename-$DATE.tar.gz /opt/omd/sites/$SITENAME/

# Restart site
omd $SITENAME start

# Clean old backups
find $BACKUP_DIR -name "*.tar.gz" -mtime +$RETENTION_DAYS -delete
```

## Recovery Procedures

### Full Site Recovery

```bash
# Stop site
omd sitename stop

# Remove current site
omd remove sitename --yes

# Restore from backup
tar xzf /backup/checkmk-sitename-latest.tar.gz -C /

# Start site
omd sitename start

# Verify
omd sitename status
```

### Configuration Recovery

```bash
# Restore configuration
tar xzf /backup/checkmk-config-latest.tar.gz -C /

# Reload configuration
omd sitename reload

# Verify
omd sitename status
```

## References

- Checkmk Backup: https://docs.checkmk.com/master/en/
- Checkmk Recovery: https://docs.checkmk.com/master/en/
- Checkmk Disaster Recovery: https://docs.checkmk.com/master/en/