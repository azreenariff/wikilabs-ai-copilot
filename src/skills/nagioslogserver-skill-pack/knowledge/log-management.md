# Nagios Log Server Log Management

## Overview

Nagios Log Server log management covers log collection, parsing, storage, retention, and cleanup procedures.

## Log Collection

### Collection Methods

| Method | Protocol | Use Case |
|--------|----------|----------|
| **Nagios Log Server Agent** | TCP/UDP | Remote Linux hosts |
| **rsyslog** | TCP/UDP | Native syslog forwarding |
| **syslog-ng** | TCP/UDP | Advanced log forwarding |
| **File input** | Local file | Local log collection |
| **Beats** | TCP/UDP | Lightweight shipping |

### Nagios Log Server Agent Configuration

```
# Agent configuration
/etc/nagios/nagios-log-server.cfg

# Configuration parameters
log_destination = tcp://logserver:5544
log_source = /var/log/syslog
log_source = /var/log/messages
log_source = /var/log/auth.log
```

### Rsyslog Configuration

```
# /etc/rsyslog.d/99-nagios.conf
*.* @@logserver:5544    # TCP forwarding
*.* @logserver:5544     # UDP forwarding
```

### Syslog-ng Configuration

```
# /etc/syslog-ng/syslog-ng.conf
destination d_logserver {
    tcp("logserver" port(5544));
};

log {
    source(sources);
    destination(d_logserver);
};
```

## Log Parsing

### Grok Patterns

Grok patterns parse unstructured log data into structured fields.

```
# Apache access log
%{IPORHOST:clientip} %{USER:ident} %{USER:auth} \[%{HTTPDATE:timestamp}\] "%{WORD:method} %{URIPATH:path} HTTP/%{NUMBER:httpversion}" %{NUMBER:response} %{NUMBER:bytes}

# Syslog
%{SYSLOGTIMESTAMP:timestamp} %{HOSTNAME:host} %{PROG:program}[\[%{POSINT:pid}\]]?: %{GREEDYDATA:log_message}

# Application error
%{TIMESTAMP_ISO8601:timestamp} %{LOGLEVEL:level} %{JAVACLASS:class} - %{GREEDYDATA:log_message}
```

### Logstash Filter Configuration

```ruby
# logstash/pipeline.conf
input {
  tcp {
    port => 5544
    type => syslog
  }
}

filter {
  if [type] == "syslog" {
    grok {
      match => { "message" => "%{SYSLOGTIMESTAMP:syslog_timestamp} %{HOSTNAME:syslog_host} %{PROG:syslog_program}[\[%{POSINT:syslog_pid}\]]?: %{GREEDYDATA:syslog_message}" }
    }
    date {
      match => [ "syslog_timestamp", "MMM  d HH:mm:ss", "MMM dd HH:mm:ss" ]
    }
  }
  
  if [message] =~ /error/i {
    mutate {
      add_field => { "log_level" => "ERROR" }
    }
  }
}

output {
  elasticsearch {
    hosts => ["localhost:9200"]
    index => "nagioslog-%{+YYYY.MM.dd}"
  }
}
```

## Log Storage

### Index Management

**Index Naming Convention**:
```
nagioslog-YYYY.MM.dd    # Daily indices
nagioslog-YYYY.MM       # Monthly indices
```

**Index Template**:
```json
{
  "template": "nagioslog-*",
  "settings": {
    "number_of_shards": 1,
    "number_of_replicas": 1,
    "index.lifecycle.name": "nagioslog-policy",
    "index.lifecycle.rollover_alias": "nagioslog"
  },
  "mappings": {
    "properties": {
      "@timestamp": { "type": "date" },
      "host": { "type": "keyword" },
      "source": { "type": "keyword" },
      "log_level": { "type": "keyword" },
      "message": { "type": "text" }
    }
  }
}
```

### Index Lifecycle Management

| Phase | Action | Duration |
|-------|--------|----------|
| **Hot** | Active index, writes and reads | 7 days |
| **Warm** | Read-only, reduced replicas | 23 days |
| **Cold** | Frozen or searchable | 30 days |
| **Delete** | Index removed | N/A |

## Log Retention

### Retention Policies

| Policy | Duration | Storage Impact |
|--------|----------|---------------|
| **Aggressive** | 7 days | Minimal |
| **Standard** | 30 days | Moderate |
| **Extended** | 90 days | Significant |
| **Compliance** | 365+ days | Large |

### Retention Configuration

```bash
# Delete indices older than 30 days
curl -X DELETE "http://localhost:9200/nagioslog-*" \
  -H 'Content-Type: application/json' \
  -d '{"query": {"range": {"@timestamp": {"lt": "now-30d"}}}}'
```

## Log Cleanup

### Automated Cleanup

```bash
#!/bin/bash
# Nagios Log Server Log Cleanup Script

# Delete old Elasticsearch indices
curl -X DELETE "http://localhost:9200/nagioslog-$(date -d '-30 days' +%Y.%m.%d)"

# Clean up Logstash logs
find /var/log/logstash -name "*.log" -mtime +7 -delete

# Clean up Nagios Log Server agent logs
find /var/log/nagios -name "nagios-log-server.log" -mtime +7 -delete
```

### Manual Cleanup

```bash
# Clean Elasticsearch cluster health
curl -X GET "http://localhost:9200/_cluster/health"

# Clean unused indices
curl -X DELETE "http://localhost:9200/nagioslog-old-2024.01.01"

# Verify cleanup
curl -X GET "http://localhost:9200/_cat/indices?v"
```

## References

- Nagios Log Server Documentation: https://assets.nagios.com/downloads/nagiosxi/docs/
- Logstash Filter Reference: https://www.elastic.co/guide/en/logstash/current/
- Elasticsearch Index Management: https://www.elastic.co/guide/en/elasticsearch/reference/