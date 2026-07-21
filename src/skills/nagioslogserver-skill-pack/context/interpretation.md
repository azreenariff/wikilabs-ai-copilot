# Nagios Log Server Context Interpretation

## Overview

This document explains how to interpret Nagios Log Server outputs, logs, metrics, and configuration.

## Interpreting Logstash Logs

### Location

```bash
/var/log/logstash/logstash.log
```

### Key Log Patterns

| Pattern | Meaning | Action |
|---------|---------|--------|
| `INFO` | Normal operation | No action |
| `WARN` | Non-critical issue | Monitor trend |
| `ERROR` | Error condition | Investigate |
| `FATAL` | Critical failure | Immediate action |
| `Pipeline started` | Logstash pipeline active | Verify |
| `Elasticsearch cluster changed` | Cluster topology changed | Review |

### Example Log Entries

```
[2024-01-15T10:30:00,000][INFO ][logstash.runner          ] Starting Logstash
[2024-01-15T10:30:01,500][INFO ][logstash.outputs.elasticsearch] Elasticsearch URL configured
[2024-01-15T10:30:02,000][INFO ][logstash.pipeline        ] Pipeline started successfully
[2024-01-15T10:30:15,000][WARN ][logstash.outputs.elasticsearch] Failed to connect to Elasticsearch
```

## Interpreting Elasticsearch Logs

### Location

```bash
/var/log/elasticsearch/
```

### Key Log Patterns

| Pattern | Meaning | Action |
|---------|---------|--------|
| `Cluster health status changed` | Cluster state change | Review |
| `Failed to connect to cluster` | Cluster connectivity issue | Investigate |
| `Index created` | New index created | Verify |
| `Index deleted` | Index deleted | Review retention |
| `Circuit breaker tripped` | Memory pressure | Increase memory |

## Interpreting Search Results

### Query Results

| Field | Description |
|-------|-------------|
| **@timestamp** | Log entry timestamp |
| **host** | Source host name |
| **source** | Log source identifier |
| **level** | Log level |
| **message** | Log entry content |
| **_index** | Elasticsearch index name |

### Search Syntax

```
# Basic search
error

# Filtered search
error AND host:webserver01

# Date range search
error AND @timestamp:2024-01-15

# Log level search
level:ERROR OR level:FATAL

# Pattern match
exception OR stacktrace OR traceback
```

## Interpreting Alert Patterns

### Alert Configuration

| Parameter | Description |
|-----------|-------------|
| **Pattern** | Log pattern to match |
| **Threshold** | Count before alert triggers |
| **Window** | Time window for threshold |
| **Severity** | Alert severity level |
| **Actions** | Alert notification method |

### Alert Interpretation

| Condition | Meaning | Action |
|-----------|---------|--------|
| **Pattern match > threshold** | Log pattern exceeded threshold | Investigate logs |
| **No matches** | Pattern not found | Verify pattern |
| **Frequent alerts** | Alert firing repeatedly | Adjust threshold |
| **Alert cleared** | Pattern matches below threshold | Review root cause |

## Interpreting Log Parser Configuration

### Parser Configuration

| Parameter | Description |
|-----------|-------------|
| **Parser Name** | Unique parser identifier |
| **Log Pattern** | Grok pattern for parsing |
| **Fields** | Extracted field names |
| **Date Format** | Timestamp format |
| **Fallback** | Default action for unmatched logs |

### Common Grok Patterns

| Pattern | Example | Description |
|---------|---------|-------------|
| `%{IPORHOST:clientip}` | 192.168.1.100 | IP address or hostname |
| `%{USERNAME:username}` | admin | User name |
| `%{NUMBER:bytes}` | 1234 | Number value |
| `%{DATA:log_message}` | Error: connection refused | Free text |
| `%{SYSLOGTIMESTAMP:timestamp}` | Jan 15 10:30:00 | Syslog timestamp |
| `%{LOGLEVEL:level}` | ERROR, INFO | Log level |

## References

- Nagios Log Server Documentation: https://assets.nagios.com/downloads/nagiosxi/docs/
- Logstash Filter Reference: https://www.elastic.co/guide/en/logstash/current/
- Elasticsearch Query DSL: https://www.elastic.co/guide/en/elasticsearch/reference/