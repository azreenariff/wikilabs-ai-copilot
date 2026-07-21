# Nagios XI Backup and Recovery

## Overview

Backup and recovery procedures ensure data availability and disaster recovery capability for Nagios XI monitoring infrastructure.

## Backup Components

### Nagios XI Backup Components

| Component | Location | Frequency |
|-----------|----------|-----------|
| **Configuration Files** | /etc/nagios/ | Daily |
| **Database** | MySQL (nagiosxi) | Daily |
| **Plugins** | /usr/lib/nagios/plugins/ | Weekly |
| **Logs** | /var/log/nagios/ | As needed |
| **Custom Scripts** | /usr/local/nagios/libexec/ | Weekly |

## Backup Procedures

### Configuration Backup

```bash
# Backup all Nagios XI configuration files
tar czf /backup/nagiosxi-config-$(date +%Y%m%d).tar.gz \
  /etc/nagios/ \
  /etc/nagiosxi/ \
  /usr/local/nagios/etc/

# Backup Nagios XI database
mysqldump -u root -p$(cat /etc/nagiosxi/db.php | grep dbpass | cut -d"'" -f4) \
  nagiosxi > /backup/nagiosxi-db-$(date +%Y%m%d).sql

# Backup custom plugins
tar czf /backup/nagiosxi-plugins-$(date +%Y%m%d).tar.gz \
  /usr/local/nagios/libexec/
```

### Database Backup

```bash
# Full database backup
mysqldump -u root -p nagiosxi > /backup/nagiosxi-full.sql

# Compressed backup
mysqldump -u root -p nagiosxi | gzip > /backup/nagiosxi-full.sql.gz

# Backup specific tables
mysqldump -u root -p nagiosxi nagios_hosts nagios_services > /backup/nagiosxi-tables.sql
```

### Automated Backup Script

```bash
#!/bin/bash
# Nagios XI Backup Script

BACKUP_DIR="/backup/nagiosxi"
DATE=$(date +%Y%m%d)
RETENTION_DAYS=30

# Create backup directory
mkdir -p $BACKUP_DIR

# Backup configuration
tar czf $BACKUP_DIR/nagiosxi-config-$DATE.tar.gz /etc/nagios /etc/nagiosxi

# Backup database
mysqldump -u root -p nagiosxi | gzip > $BACKUP_DIR/nagiosxi-db-$DATE.sql.gz

# Backup plugins
tar czf $BACKUP_DIR/nagiosxi-plugins-$DATE.tar.gz /usr/local/nagios/libexec/

# Clean old backups
find $BACKUP_DIR -name "*.tar.gz" -mtime +$RETENTION_DAYS -delete
find $BACKUP_DIR -name "*.sql.gz" -mtime +$RETENTION_DAYS -delete

# Log completion
echo "$(date): Backup completed successfully" >> $BACKUP_DIR/backup.log
```

## Recovery Procedures

### Full Recovery from Backup

```bash
# 1. Stop Nagios services
/etc/init.d/nagios stop
/etc/init.d/ndo2db stop
/etc/init.d/httpd stop

# 2. Restore configuration files
tar xzf /backup/nagiosxi-config-latest.tar.gz -C /

# 3. Restore database
gunzip -c /backup/nagiosxi-db-latest.sql.gz | mysql -u root -p nagiosxi

# 4. Restore plugins
tar xzf /backup/nagiosxi-plugins-latest.tar.gz -C /

# 5. Start services
/etc/init.d/nagios start
/etc/init.d/ndo2db start
/etc/init.d/httpd start
```

### Database Recovery Only

```bash
# Stop NDOUtil
/etc/init.d/ndo2db stop

# Restore database
gunzip -c /backup/nagiosxi-db-latest.sql.gz | mysql -u root -p nagiosxi

# Start NDOUtil
/etc/init.d/ndo2db start

# Verify database
mysql -u root -p nagiosxi -e "SELECT COUNT(*) FROM nagios_hosts;"
```

### Configuration Recovery Only

```bash
# Restore configuration files
tar xzf /backup/nagiosxi-config-latest.tar.gz -C /

# Validate configuration
/usr/local/nagios/bin/nagios -v /usr/local/nagios/etc/nagios.cfg

# Restart Nagios
/etc/init.d/nagios restart
```

## Recovery Time Objectives

| Scenario | RTO | RPO |
|----------|-----|-----|
| Database corruption | 30 min | Last backup |
| Configuration error | 15 min | Last config backup |
| Plugin failure | 1 hour | Last plugin backup |
| Full system failure | 2 hours | Last full backup |
| Single host monitoring loss | 30 min | N/A |

## Verification Procedures

### Post-Backup Verification

```bash
# Verify backup file integrity
tar tzf /backup/nagiosxi-config-latest.tar.gz

# Verify database backup
gunzip -c /backup/nagiosxi-db-latest.sql.gz | head -n 10

# Verify plugin backup
tar tzf /backup/nagiosxi-plugins-latest.tar.gz
```

### Post-Recovery Verification

```bash
# Verify Nagios is running
/etc/init.d/nagios status

# Verify NDOUtil is running
/etc/init.d/ndo2db status

# Verify database connection
mysql -u root -p nagiosxi -e "SELECT COUNT(*) FROM nagios_hosts;"

# Verify web interface is accessible
curl -I http://localhost/nagiosxi/

# Verify configuration is valid
/usr/local/nagios/bin/nagios -v /usr/local/nagios/etc/nagios.cfg
```

## Disaster Recovery Planning

### DR Site Requirements

1. **Nagios XI Server**: Identical or better specs than primary
2. **MySQL Server**: For database storage
3. **Network Connectivity**: Access to monitored infrastructure
4. **Backup Storage**: Offsite backup copy
5. **Documentation**: DR procedures and contact information

### DR Testing Schedule

| Frequency | Test Type |
|-----------|-----------|
| Monthly | Configuration restore test |
| Quarterly | Full system restore test |
| Annually | Complete DR failover test |

## References

- Nagios XI Backup Guide: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios XI Recovery Procedures: https://assets.nagios.com/downloads/nagiosxi/docs/
- MySQL Backup Best Practices: https://dev.mysql.com/doc/refman/8.0/en/backup-restore.html