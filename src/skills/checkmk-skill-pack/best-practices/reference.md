# Checkmk Best Practices

## Configuration Best Practices

### 1. Use Rulesets for Centralized Config
- Centralize check parameters in rulesets
- Use hierarchical ruleset organization
- Document ruleset purpose

### 2. Implement Proper Time Periods
- Define 24x7 and business hours time periods
- Match check periods to notification periods
- Use consistent naming conventions

### 3. Organize Hosts and Services Logically
- Use host groups and service groups
- Group by environment (prod, staging, dev)
- Group by location (datacenter, branch office)

### 4. Configure Proper Check Intervals
- Critical services: Check every 30 seconds
- Standard services: Check every 60 seconds
- Non-critical services: Check every 180 seconds

### 5. Implement Notification Dampening
- Set notification intervals (30-60 minutes)
- Configure flapping detection
- Use contact groups for escalation

## Performance Best Practices

### Checkmk Optimization

| Parameter | Recommended | Description |
|-----------|-------------|-------------|
| `check_interval` | 60 seconds | Standard check interval |
| `parallel_checks` | CPU cores * 2 | Parallel check workers |
| `max_check_attempts` | 3-5 | Check attempts before hard state |
| `retry_interval` | 30 seconds | Retry check interval |

### Scale Optimization

| Scale | Check Interval | Max Hosts | Max Services |
|-------|---------------|-----------|-------------|
| Small (<500) | 30-60 seconds | 2000 | 5000 |
| Medium (500-5000) | 60 seconds | 10000 | 50000 |
| Large (5000+) | 60+ seconds | 20000 | 100000 |

## Operational Best Practices

### Backup Strategy

1. Daily configuration backup
2. Weekly full system backup
3. Monthly database backup
4. Offsite backup storage

### Monitoring Best Practices

1. Monitor the monitor (Checkmk itself)
2. Implement health checks for Checkmk services
3. Monitor Checkmk database growth
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
4. Restrict agent access by IP
5. Enable audit logging
6. Regular security patches
7. Backup configuration securely

## References

- Checkmk Best Practices: https://docs.checkmk.com/master/en/
- Checkmk Performance: https://docs.checkmk.com/master/en/
- Checkmk Administration: https://docs.checkmk.com/master/en/