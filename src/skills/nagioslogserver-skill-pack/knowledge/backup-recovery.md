# Nagios Log Server Backup and Recovery

## Overview

Backup and recovery procedures for Nagios Log Server cover Elasticsearch data, configuration, and web interface settings.

## Backup Components

### Components to Backup

| Component | Location | Frequency |
|-----------|----------|-----------|
| **Elasticsearch Indices** | /var/lib/elasticsearch/ | Daily |
| **Logstash Configuration** | /etc/logstash/ | Daily |
| **Nagios Log Server Config** | /etc/nagios/ | Daily |
| **MySQL Database** | MySQL (nagiosxi) | Daily |
| **Elasticsearch Snapshots** | Snapshot repository | Weekly |

## Backup Procedures

### Elasticsearch Backup

```bash
# Create snapshot repository
curl -X PUT "http://localhost:9200/_snapshot/nagioslog_backup" \
  -H 'Content-Type: application/json' \
  -d '{
    "type": "fs",
    "settings": {
      "location": "/backup/elasticsearch",
      "compress": true
    }
  }'

# Create snapshot
curl -X PUT "http://localhost:9200/_snapshot/nagioslog_backup/snapshot_$(date +%Y%m%d)" \
  -H 'Content-Type: application/json' \
  -d '{
    "indices": "nagioslog-*",
    "ignore_unavailable": true,
    "include_global_state": false
  }'

# Verify snapshot
curl -X GET "http://localhost:9200/_snapshot/nagioslog_backup/snapshot_$(date +%Y%m%d)?pretty"
```

### Configuration Backup

```bash
# Backup all configuration files
tar czf /backup/nagioslog-config-$(date +%Y%m%d).tar.gz \
  /etc/logstash/ \
  /etc/nagios/ \
  /etc/elasticsearch/

# Backup MySQL database
mysqldump -u root -p nagiosxi > /backup/nagioslog-db-$(date +%Y%m%d).sql
```

### Automated Backup Script

```bash
#!/bin/bash
# Nagios Log Server Backup Script

BACKUP_DIR="/backup/nagioslog"
DATE=$(date +%Y%m%d)
RETENTION_DAYS=30

# Create backup directory
mkdir -p $BACKUP_DIR/elasticsearch
mkdir -p $BACKUP_DIR/config

# Create Elasticsearch snapshot
curl -X PUT "http://localhost:9200/_snapshot/nagioslog_backup/snapshot_$DATE" \
  -H 'Content-Type: application/json' \
  -d '{"indices": "nagioslog-*", "ignore_unavailable": true}'

# Wait for snapshot to complete
sleep 10

# Backup configuration files
tar czf $BACKUP_DIR/config/nagioslog-config-$DATE.tar.gz \
  /etc/logstash/ /etc/nagios/ /etc/elasticsearch/

# Backup database
mysqldump -u root -p nagiosxi | gzip > $BACKUP_DIR/config/nagioslog-db-$DATE.sql.gz

# Clean old backups
find $BACKUP_DIR -name "*.tar.gz" -mtime +$RETENTION_DAYS -delete
find $BACKUP_DIR -name "*.sql.gz" -mtime +$RETENTION_DAYS -delete
```

## Recovery Procedures

### Elasticsearch Recovery

```bash
# Verify snapshot
curl -X GET "http://localhost:9200/_snapshot/nagioslog_backup/snapshot_20240115?pretty"

# Restore snapshot
curl -X POST "http://localhost:9200/_snapshot/nagioslog_backup/snapshot_20240115/_restore" \
  -H 'Content-Type: application/json' \
  -d '{
    "indices": "nagioslog-*",
    "ignore_unavailable": true,
    "include_global_state": false
  }'

# Verify restoration
curl -X GET "http://localhost:9200/_cat/indices?v"
```

### Configuration Recovery

```bash
# Restore configuration files
tar xzf /backup/nagioslog-config-latest.tar.gz -C /

# Restore database
gunzip -c /backup/nagioslog-db-latest.sql.gz | mysql -u root -p nagiosxi

# Verify restoration
curl -X GET "http://localhost:9200/_cluster/health?pretty"
```

## Recovery Time Objectives

| Scenario | RTO | RPO |
|----------|-----|-----|
| Index corruption | 1 hour | Last snapshot |
| Configuration error | 30 min | Last config backup |
| Full system failure | 2 hours | Last full backup |
| Data loss | 2 hours | Last snapshot |

## References

- Nagios Log Server Backup Guide: https://assets.nagios.com/downloads/nagiosxi/docs/
- Elasticsearch Snapshot/Restore: https://www.elastic.co/guide/en/elasticsearch/reference/
- Nagios Log Server Recovery: https://assets.nagios.com/downloads/nagiosxi/docs/