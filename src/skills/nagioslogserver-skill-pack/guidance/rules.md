# Nagios Log Server Guidance and Safety Rules

## Engineering Principles

### Evidence-Based Reasoning

1. **Direct Error Messages**: Highest confidence source
2. **Performance Metrics**: Log processing data
3. **Configuration Values**: Supporting evidence
4. **Historical Patterns**: Contextual information

### Safety First

1. **Read-Only First**: Always start with diagnostic commands
2. **Low Risk**: Prefer non-disruptive actions
3. **Testing Required**: Changes should be tested first
4. **Rollback Plan**: Always have a recovery path
5. **Change Window**: Make changes during maintenance windows

## Command Risk Assessment

| Risk Level | Description | Examples |
|-----------|-------------|----------|
| **LOW** | No impact | SELECT, SHOW, status checks |
| **MEDIUM** | Read-only but may impact | Large searches, config validation |
| **HIGH** | Data modification | DDL, bulk operations, index deletion |
| **DISRUPTIVE** | Service impact | Restart, major config change |

## Safety Rules

1. Never execute commands — only recommend and explain
2. Always warn about risks before recommending actions
3. Always provide rollback strategies
4. Always recommend evidence collection before diagnosis
5. Always consider cascade effects of recommended actions
6. Never modify configuration without explicit engineer approval
7. Always recommend testing changes in non-production first
8. Always validate configuration changes before applying
9. Always recommend backup before destructive operations
10. Always monitor after making changes

## Nagios Log Server-Specific Safety Rules

### Elasticsearch Operations

1. Never delete indices without backup
2. Always check cluster health before operations
3. Monitor disk space during index operations
4. Validate snapshot before restore
5. Test configuration changes in non-production first

### Logstash Pipeline Changes

1. Always validate logstash configuration before reload
2. Test log patterns against sample logs
3. Monitor pipeline latency after changes
4. Keep backup of working configuration
5. Apply changes during maintenance windows

### Log Retention Changes

1. Document retention policy changes
2. Test cleanup operations in non-production
3. Verify index health after cleanup
4. Monitor disk space after cleanup
5. Update documentation with new policies

## Operational Guidelines

### Before Making Changes

1. Document current state
2. Take snapshot/backup
3. Test in non-production first
4. Plan rollback procedure
5. Schedule maintenance window

### After Making Changes

1. Verify cluster health
2. Monitor log processing
3. Check for errors in logs
4. Confirm search is operational
5. Update documentation

## References

- Nagios Log Server Safety: https://assets.nagios.com/downloads/nagiosxi/docs/
- Elasticsearch Safety: https://www.elastic.co/guide/en/elasticsearch/reference/
- Monitoring Safety: https://www.monitoringportal.org/