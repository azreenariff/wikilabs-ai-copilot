# Nagios Log Server Alerting and Monitoring

## Overview

Nagios Log Server alerting monitors log patterns and triggers alerts based on configurable thresholds and conditions.

## Alert Configuration

### Alert Components

1. **Pattern**: Log pattern to match (regular expression)
2. **Threshold**: Number of matches before alerting
3. **Window**: Time window for threshold evaluation
4. **Severity**: Alert severity level (INFO, WARNING, CRITICAL)
5. **Actions**: Notification method and target

### Alert Types

| Type | Description | Use Case |
|------|-------------|----------|
| **Count** | Number of matches in time window | Error rate monitoring |
| **Percentage** | Percentage of total matches | Error ratio monitoring |
| **Rate** | Matches per unit time | Real-time alerting |
| **Presence** | Pattern present in time window | Security events |

### Alert Configuration

```
# Alert configuration example
Pattern: ERROR|FATAL|CRITICAL
Threshold: 10
Window: 5 minutes
Severity: CRITICAL
Actions: nagiosxi (Nagios XI integration), email (admin@example.com)
```

## Alert Integration with Nagios XI

### Nagios XI Integration

Nagios Log Server integrates with Nagios XI for:
- Alert notifications via existing contact infrastructure
- Service check integration
- Centralized monitoring dashboard

**Configuration**:
- Nagios XI server URL
- API credentials
- Contact groups for alert routing
- Escalation policies

### Alert Escalation

| Level | Contact | Condition |
|-------|---------|-----------|
| **Level 1** | On-call engineer | Initial alert |
| **Level 2** | Team lead | Alert persists >30 min |
| **Level 3** | Management | Alert persists >1 hour |

## Log-Based Service Checks

### Service Check Configuration

```
# Nagios Log Server service check
Command: check_nagios_log
Parameters: --index nagioslog --pattern "ERROR" --threshold 5 --window 300
```

### Service Check Types

| Type | Command | Description |
|------|---------|-------------|
| **Pattern Match** | check_nagios_log --pattern | Pattern-based alerting |
| **Count** | check_nagios_log --count | Count-based alerting |
| **Rate** | check_nagios_log --rate | Rate-based alerting |
| **Custom** | check_nagios_log --custom | Custom query |

## Log Pattern Management

### Pattern Development

1. **Identify**: Find problematic log patterns
2. **Test**: Validate pattern against sample logs
3. **Deploy**: Apply pattern as alert configuration
4. **Monitor**: Watch for false positives
5. **Refine**: Adjust pattern as needed

### Pattern Testing

```
# Test pattern against logs
curl -X GET "http://localhost:9200/nagioslog-*/_search" \
  -H 'Content-Type: application/json' \
  -d '{"query": {"match": {"message": "ERROR"}}}'

# Validate pattern regex
echo "ERROR: connection refused" | grep -E "ERROR"
```

## Alert Performance

### Performance Considerations

| Factor | Impact | Optimization |
|--------|--------|-------------|
| **Pattern Complexity** | Regex performance | Use simple patterns |
| **Threshold Sensitivity** | Alert frequency | Adjust thresholds |
| **Window Size** | Evaluation frequency | Balance speed and accuracy |
| **Index Size** | Search performance | Regular cleanup |

### Alert Threshold Tuning

| Condition | Adjustment |
|-----------|-----------|
| **Too many false positives** | Increase threshold or refine pattern |
| **Too many missed alerts** | Decrease threshold or widen window |
| **Alert fatigue** | Implement dampening or grouping |
| **Delayed alerts** | Reduce evaluation window |

## References

- Nagios Log Server Alerting: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios Log Server Integration: https://assets.nagios.com/downloads/nagiosxi/docs/
- Elasticsearch Alerting: https://www.elastic.co/guide/en/elasticsearch/reference/