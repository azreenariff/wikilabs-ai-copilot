# Nagios XI Context Interpretation

## Overview

This document explains how to interpret Nagios XI outputs, logs, metrics, and configuration. Understanding context is essential for accurate diagnosis and resolution.

## Interpreting Nagios Core Log

### Location

```bash
/var/log/nagios/nagios.log
```

### Key Log Patterns

```
[timestamp] [pid] [level] Message
```

### Critical Log Patterns

| Pattern | Meaning | Action |
|---------|---------|--------|
| `PASSIVE SERVICE CHECK` | External check result received | Verify source |
| `SERVICE NOTIFICATION` | Alert notification sent | Check contact config |
| `HOST NOTIFICATION` | Host alert notification | Check contact config |
| `SERVICE ALERT` | Service state changed | Review state change |
| `HOST ALERT` | Host state changed | Review state change |
| `START HOST CHECK` | Check initiated | Monitor timing |
| `END HOST CHECK` | Check completed | Review result |
| `ERROR` | Configuration or runtime error | Investigate immediately |
| `WARNING` | Non-critical issue | Monitor trend |
| `NOTIFICATION` | Alert sent | Verify contact routing |

### Example Log Entries

```
[1625097600] SERVICE ALERT: webserver;HTTP;CRITICAL;SOFT;1;Connection refused
[1625097660] SERVICE ALERT: webserver;HTTP;CRITICAL;HARD;3;Connection refused
[1625097660] SERVICE NOTIFICATION: admin;webserver;HTTP;CRITICAL;HARD;Connection refused
```

## Interpreting NDOUtil Output

### Location

```bash
/var/log/nagiosxi/ndomod.log
```

### Key Patterns

| Pattern | Meaning | Action |
|---------|---------|--------|
| `Connecting to database` | Starting DB connection | Verify connectivity |
| `Connected to database` | DB connection established | OK |
| `Disconnecting from database` | Ending DB connection | Verify shutdown |
| `ERROR: Failed to connect` | Database connection failed | Check DB status |
| `ERROR: Query failed` | SQL query error | Check table integrity |
| `ERROR: Duplicate entry` | Configuration conflict | Review config |

## Interpreting MySQL Performance

### Key Tables

```sql
-- Check NDO database size
SELECT table_name, table_rows, data_length, index_length
FROM information_schema.tables
WHERE table_schema = 'nagios';

-- Check for large tables
SELECT table_name, ROUND(data_length/1024/1024,2) AS 'Size MB'
FROM information_schema.tables
WHERE table_schema = 'nagios'
ORDER BY data_length DESC;
```

### Performance Indicators

| Metric | Good | Warning | Critical |
|--------|------|---------|----------|
| Table size growth | <10%/month | 10-25%/month | >25%/month |
| Query response time | <100ms | 100-500ms | >500ms |
| NDOUtil sync time | <1s | 1-5s | >5s |
| Database connections | <80% max | 80-95% max | >95% max |

## Interpreting Web Interface Metrics

### Status Map Interpretation

- **Green Circle**: Service OK
- **Yellow Circle**: Service WARNING
- **Red Circle**: Service CRITICAL
- **Grey Circle**: Service UNKNOWN
- **Green Square**: Host UP
- **Red Square**: Host DOWN
- **Grey Square**: Host UNREACHABLE

### Performance Graphs

- **CPU Usage**: Monitor for sustained high usage (>80%)
- **Memory Usage**: Watch for growing trends
- **Disk Usage**: Alert before 90% capacity
- **Network Traffic**: Identify unusual patterns

## Interpreting Notification Configuration

### Contact Options

| Option | Meaning |
|--------|---------|
| **d** | Host DOWN notification |
| **u** | Host UP notification |
| **r** | Host recovery notification |
| **f** | Host flapping notification |
| **s** | Host scheduled downtime notification |
| **D** | Service DOWN notification |
| **U** | Service UNKNOWN notification |
| **R** | Service recovery notification |
| **F** | Service flapping notification |
| **S** | Service scheduled downtime notification |

### Escalation Configuration

```
Host Escalation:
  - first_notification_delay: 0
  - notification_interval: 30
  - escalation_period: workhours

Service Escalation:
  - first_notification_delay: 0
  - notification_interval: 15
  - escalation_period: workhours
```

## Interpreting Flapping Detection

### Detection Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| **host_flap_detection_enabled** | 1 | Enable host flapping detection |
| **service_flap_detection_enabled** | 1 | Enable service flapping detection |
| **host_flap_detection_threshold** | 30% | Threshold for host flapping |
| **service_flap_detection_threshold** | 30% | Threshold for service flapping |
| **flap_detection_on_up** | 1 | Detect flapping on UP state |
| **flap_detection_on_down** | 1 | Detect flapping on DOWN state |

### Flapping Behavior

When flapping is detected:
1. Flap detection triggers (state changes too frequently)
2. Notifications are suppressed for that host/service
3. Host/service is marked as "Flapping"
4. Manual intervention may be required to clear

## References

- Nagios XI Log Analysis: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios XI Configuration: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios XI Troubleshooting: https://assets.nagios.com/downloads/nagiosxi/docs/