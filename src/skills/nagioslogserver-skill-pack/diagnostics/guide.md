# Nagios Log Server Diagnostic Procedures

## Overview

Systematic diagnostic procedures for common Nagios Log Server issues.

## Elasticsearch Issues

### Cluster Health Issues

**Symptoms**:
- Cluster status red or yellow
- Search failures
- Slow queries

**Diagnostic Steps**:
```bash
# Check cluster health
curl -X GET "http://localhost:9200/_cluster/health?pretty"

# Check node status
curl -X GET "http://localhost:9200/_cat/nodes?v"

# Check disk usage
curl -X GET "http://localhost:9200/_cat/allocation?v"
```

**Evidence Required**:
- Cluster health status
- Node status
- Disk usage metrics

**Remediation**:
1. Check disk space
2. Verify node connectivity
3. Review shard allocation
4. Restart nodes if needed
5. Monitor cluster recovery

### Index Performance Issues

**Symptoms**:
- Slow search response
- High query latency
- Thread pool rejections

**Diagnostic Steps**:
```bash
# Check index stats
curl -X GET "http://localhost:9200/_cat/indices?v&s=store.size:desc"

# Check thread pool
curl -X GET "http://localhost:9200/_cat/thread_pool?v"

# Check slow queries
curl -X GET "http://localhost:9200/_search?slowlog.threshold=1s"
```

**Evidence Required**:
- Index size and shard count
- Thread pool status
- Query performance metrics

**Remediation**:
1. Optimize query patterns
2. Clean old indices
3. Adjust shard count
4. Increase heap size
5. Consider index warm-up

## Logstash Issues

### Pipeline Performance

**Symptoms**:
- Log processing delays
- High memory usage
- Pipeline backpressure

**Diagnostic Steps**:
```bash
# Check Logstash metrics
curl -X GET "http://localhost:9600/_node/stats?pretty"

# Check pipeline status
curl -X GET "http://localhost:9600/_node/pipelines?pretty"

# Check Logstash log
tail -n 100 /var/log/logstash/logstash.log
```

**Evidence Required**:
- Pipeline throughput metrics
- Memory usage trends
- Queue depth

**Remediation**:
1. Optimize filter configuration
2. Increase pipeline workers
3. Adjust batch size
4. Check output plugin performance
5. Monitor queue depth

### Input Collection Issues

**Symptoms**:
- Missing logs from sources
- Log collection gaps
- High error rate

**Diagnostic Steps**:
```bash
# Check input plugin status
curl -X GET "http://localhost:9600/_node/pipelines/main?pretty"

# Check network connectivity
netstat -tlnp | grep 5544

# Check input log
tail -n 100 /var/log/logstash/input.log
```

**Evidence Required**:
- Input plugin metrics
- Network connectivity status
- Log collection patterns

**Remediation**:
1. Verify input configuration
2. Check network connectivity
3. Restart input plugin
4. Review source log configuration
5. Monitor collection recovery

## Web Interface Issues

### Search Interface Issues

**Symptoms**:
- Search not returning results
- Interface not loading
- Filter failures

**Diagnostic Steps**:
```bash
# Check web server status
systemctl status httpd

# Check web server log
tail -n 100 /var/log/httpd/error_log

# Check Elasticsearch connectivity
curl -X GET "http://localhost:9200/_cluster/health?pretty"
```

**Evidence Required**:
- Web server status
- Error log entries
- Elasticsearch connectivity

**Remediation**:
1. Check web server status
2. Verify Elasticsearch connectivity
3. Clear browser cache
4. Check user permissions
5. Restart web server if needed

## References

- Nagios Log Server Diagnostics: https://assets.nagios.com/downloads/nagiosxi/docs/
- Elasticsearch Diagnostics: https://www.elastic.co/guide/en/elasticsearch/reference/
- Logstash Diagnostics: https://www.elastic.co/guide/en/logstash/