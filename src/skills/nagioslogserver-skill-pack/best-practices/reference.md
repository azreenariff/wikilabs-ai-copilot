# Nagios Log Server Best Practices

## Configuration Best Practices

### 1. Use Daily Index Rotation
- Consistent naming: nagioslog-YYYY.MM.dd
- Manage retention through index deletion
- Plan for storage growth

### 2. Implement Proper Retention Policy
- Standard: 30 days for most environments
- Compliance: 90+ days for regulated environments
- Aggressive: 7 days for non-critical data

### 3. Configure Appropriate Log Parsing
- Use grok patterns for common log formats
- Create custom parsers for application logs
- Test parsers against sample logs

### 4. Optimize Logstash Pipeline
- Use conditional filters efficiently
- Minimize field extraction for high-volume logs
- Monitor pipeline latency

### 5. Implement Proper Alert Thresholds
- Start with conservative thresholds
- Adjust based on false positive rate
- Implement dampening for frequent alerts

## Performance Best Practices

### Elasticsearch Optimization

| Parameter | Recommended | Description |
|-----------|-------------|-------------|
| `heap_size` | 50% of RAM (max 31GB) | JVM heap size |
| `number_of_shards` | 1 per index | Shards per index |
| `number_of_replicas` | 1 | Replica count |
| `refresh_interval` | 30s | Index refresh interval |

### Logstash Optimization

| Parameter | Recommended | Description |
|-----------|-------------|-------------|
| `pipeline.workers` | CPU cores | Worker threads |
| `pipeline.batch.size` | 125-500 | Batch size |
| `pipeline.batch.delay` | 50ms | Batch delay |

### Monitoring Scale Best Practices

| Scale | Log Sources | Elasticsearch | Storage |
|-------|------------|--------------|---------|
| Small (<50) | 1 node | Single | 100GB |
| Medium (50-200) | 1-3 nodes | Cluster | 500GB |
| Large (200+) | 3+ nodes | Multi-node | 2TB+ |

## Operational Best Practices

### Backup Strategy

1. Daily Elasticsearch snapshot
2. Daily configuration backup
3. Weekly full system backup
4. Offsite backup storage

### Monitoring Best Practices

1. Monitor Elasticsearch cluster health
2. Monitor Logstash pipeline latency
3. Monitor disk space usage
4. Monitor alert thresholds
5. Track log growth trends

### Alert Management

1. Review alerts daily
2. Adjust thresholds as needed
3. Document alert response procedures
4. Test notification routes quarterly
5. Implement alert dampening

## Security Best Practices

1. Use HTTPS for web interface access
2. Implement strong password policies
3. Regular user access reviews
4. Restrict Logstash access by IP
5. Enable audit logging
6. Regular security patches
7. Backup configuration securely

## References

- Nagios Log Server Best Practices: https://assets.nagios.com/downloads/nagiosxi/docs/
- Elasticsearch Best Practices: https://www.elastic.co/guide/en/elasticsearch/reference/
- Logstash Best Practices: https://www.elastic.co/guide/en/logstash/