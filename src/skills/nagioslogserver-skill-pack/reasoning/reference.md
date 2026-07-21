# Nagios Log Server Diagnostic Reasoning Reference

## Reasoning Patterns

### Pattern 1: Data Not Appearing

```
Observation: Logs not appearing in search
  ├── Logstash running?
  │   ├── No: Start/fix Logstash
  │   └── Yes: Check pipeline
  ├── Elasticsearch reachable?
  │   ├── No: Fix connectivity
  │   └── Yes: Check output plugin
  └── Input collection working?
      ├── No: Fix input configuration
      └── Yes: Check network/firewall
```

### Pattern 2: Search Failures

```
Observation: Search returning no results
  ├── Index exists?
  │   ├── No: Create index or fix pipeline
  │   └── Yes: Check query syntax
  ├── Data in index?
  │   ├── No: Fix data collection
  │   └── Yes: Check filter mapping
  └── Query syntax correct?
      ├── No: Fix query
      └── Yes: Check field mappings
```

### Pattern 3: Performance Issues

```
Observation: Slow search or processing
  ├── Elasticsearch cluster healthy?
  │   ├── No: Fix cluster issues
  │   └── Yes: Check performance metrics
  ├── Logstash pipeline slow?
  │   ├── Yes: Optimize pipeline
  │   └── No: Check Elasticsearch
  └── Index size large?
      ├── Yes: Clean old indices
      └── No: Check query optimization
```

## Confidence Assessment

### Evidence Quality

| Evidence Type | Quality | Example |
|--------------|---------|---------|
| Direct error | High | Log entry with error message |
| Service status | High | ps/kill/status output |
| Elasticsearch metrics | Medium | Cluster health, shard status |
| Logstash metrics | Medium | Pipeline throughput |
| User reports | Low | "Search is slow" |

### Confidence Scoring

1. **High (85-100%)**: Multiple high-quality evidence points
2. **Medium (60-84%)**: Some evidence with supporting data
3. **Low (30-59%)**: Limited evidence, needs more investigation
4. **Very Low (0-29%)**: Insufficient evidence

## References

- Nagios Log Server Diagnostics: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios Log Server Troubleshooting: https://assets.nagios.com/downloads/nagiosxi/docs/
- Elasticsearch Diagnostics: https://www.elastic.co/guide/en/elasticsearch/reference/