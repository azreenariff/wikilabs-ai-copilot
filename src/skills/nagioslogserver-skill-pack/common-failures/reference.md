# Nagios Log Server Common Failure Patterns

## Overview

This reference documents common Nagios Log Server failure patterns, their symptoms, diagnosis methods, and recommended actions.

## Common Failures

### Elasticsearch Cluster Yellow/Red

**Symptoms**:
- Search queries slow or failing
- Indices showing yellow or red status
- Missing shard warnings

**Diagnosis**:
```bash
# Check cluster health
curl -X GET "http://localhost:9200/_cluster/health?pretty"

# Check shard allocation
curl -X GET "http://localhost:9200/_cat/shards?v"

# Check unassigned shards
curl -X GET "http://localhost:9200/_cat/allocation?v"
```

**Remediation**:
1. Check disk space on all nodes
2. Verify node connectivity
3. Review shard allocation settings
4. Consider adding more nodes or shards
5. Reset allocation filtering if needed

### Logstash Pipeline Failure

**Symptoms**:
- Logs not appearing in search
- Logstash process not running
- Pipeline errors in logs

**Diagnosis**:
```bash
# Check Logstash process
ps aux | grep logstash

# Check Logstash log
tail -n 100 /var/log/logstash/logstash.log

# Test pipeline configuration
/usr/share/logstash/bin/logstash --path.settings /etc/logstash -t
```

**Remediation**:
1. Check Elasticsearch connectivity
2. Validate Logstash configuration
3. Check input/output plugins
4. Restart Logstash service
5. Monitor pipeline output

### Search Performance Degradation

**Symptoms**:
- Slow search response times
- High Elasticsearch CPU usage
- Query timeouts

**Diagnosis**:
```bash
# Check slow queries
curl -X GET "http://localhost:9200/_search?slowlog.threshold=1s"

# Check index performance
curl -X GET "http://localhost:9200/_cat/indices?v&v=true&s=store.size:desc"

# Check thread pool
curl -X GET "http://localhost:9200/_cat/thread_pool?v"
```

**Remediation**:
1. Clean up old indices
2. Optimize query patterns
3. Increase heap size if needed
4. Consider index warm-up
5. Review shard count

### Log Collection Failure

**Symptoms**:
- No logs from specific source
- Agent showing disconnected
- Collection gaps

**Diagnosis**:
```bash
# Check agent status
tail -n 100 /var/log/nagios/nagios-log-server.log

# Check network connectivity
telnet logserver 5544

# Check firewall rules
iptables -L -n | grep 5544
```

**Remediation**:
1. Verify agent configuration
2. Check network connectivity
3. Verify firewall rules
4. Restart agent service
5. Check disk space on agent

### Index Deletion Issues

**Symptoms**:
- Data not searchable
- Index deletion errors
- Retention policy failures

**Diagnosis**:
```bash
# List all indices
curl -X GET "http://localhost:9200/_cat/indices?v"

# Check deletion policy
curl -X GET "http://localhost:9200/_ilm/policy?pretty"

# Test index deletion
curl -X DELETE "http://localhost:9200/nagioslog-2024.01.01"
```

**Remediation**:
1. Verify index name pattern
2. Check index lifecycle policy
3. Manually delete if needed
4. Verify disk space freed
5. Monitor cluster health

### Alert Not Triggering

**Symptoms**:
- Alert pattern not matching
- Alert threshold not reached
- Alert not sending notifications

**Diagnosis**:
```bash
# Test pattern match
curl -X GET "http://localhost:9200/nagioslog-*/_search" \
  -H 'Content-Type: application/json' \
  -d '{"query": {"match": {"message": "ERROR"}}}'

# Check alert configuration
grep -r "alert" /etc/nagios/

# Check alert logs
tail -n 100 /var/log/nagios/nagios.log
```

**Remediation**:
1. Verify pattern syntax
2. Check threshold settings
3. Verify Nagios XI integration
4. Test notification configuration
5. Review alert dampening settings

## References

- Nagios Log Server Troubleshooting: https://assets.nagios.com/downloads/nagiosxi/docs/
- Elasticsearch Troubleshooting: https://www.elastic.co/guide/en/elasticsearch/reference/
- Nagios Log Server Support: https://support.nagios.com/