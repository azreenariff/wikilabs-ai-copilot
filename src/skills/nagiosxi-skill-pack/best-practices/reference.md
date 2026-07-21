# Nagios XI Best Practices

## Configuration Best Practices

### 1. Use WATO for Primary Configuration
- WATO provides validation and error checking
- Configuration is stored in the database
- Rollback is supported through configuration versions

### 2. Implement Proper Time Periods
- Define 24x7 and business hours time periods
- Match check periods to notification periods
- Use consistent naming conventions

### 3. Organize Hosts and Services Logically
- Use host groups and service groups
- Group by environment (prod, staging, dev)
- Group by location (datacenter, branch office)

### 4. Configure Proper Check Intervals
- Critical services: Check every 1-5 minutes
- Standard services: Check every 5-10 minutes
- Non-critical services: Check every 15-30 minutes

### 5. Implement Notification Dampening
- Set notification intervals (15-30 minutes)
- Configure flapping detection (30% threshold)
- Use contact groups for escalation

## Performance Best Practices

### Database Optimization

| Parameter | Recommended | Description |
|-----------|-------------|-------------|
| `innodb_buffer_pool_size` | 50-80% RAM | Buffer pool for NDO tables |
| `innodb_log_file_size` | 256MB | Redo log size |
| `max_connections` | 200-500 | Max MySQL connections |
| `query_cache_size` | 64MB | Query cache (if MySQL 5.7) |

### Nagios XI Optimization

| Setting | Recommended | Description |
|---------|-------------|-------------|
| `max_check_attempts` | 3-5 | Check attempts before hard state |
| `check_interval` | 5 minutes | Standard check interval |
| `retry_interval` | 1 minute | Retry check interval |
| `event_handler_enabled` | 1 | Enable event handlers |

### Monitoring Scale Best Practices

| Scale | Check Interval | Max Hosts | Max Services |
|-------|---------------|-----------|-------------|
| Small (<200 hosts) | 5 minutes | 500 | 2000 |
| Medium (200-1000) | 5-10 minutes | 2000 | 10000 |
| Large (1000+) | 10-15 minutes | 5000 | 25000 |

## Operational Best Practices

### Backup Strategy

1. Daily configuration backup
2. Daily database backup
3. Weekly plugin backup
4. Monthly full system backup
5. Offsite backup storage

### Monitoring Best Practices

1. Monitor the monitor (Nagios XI itself)
2. Implement health checks for Nagios services
3. Monitor Nagios XI database growth
4. Monitor check execution times
5. Track flapping hosts and services

### Alert Management

1. Review alerts daily
2. Clear acknowledged alerts regularly
3. Update contact information promptly
4. Test notification routes quarterly
5. Document alert response procedures

## Security Best Practices

1. Use HTTPS for web interface access
2. Implement strong password policies
3. Regular user access reviews
4. Restrict NRPE access by IP
5. Enable audit logging
6. Regular security patches
7. Backup before any security changes

## References

- Nagios XI Best Practices: https://assets.nagios.com/downloads/nagiosxi/docs/
- Monitoring Best Practices: https://www.monitoringportal.org/best-practices/
- Nagios XI Performance: https://assets.nagios.com/downloads/nagiosxi/docs/