# Nagios XI Guidance and Safety Rules

## Engineering Principles

### Evidence-Based Reasoning

1. **Direct Error Messages**: Highest confidence source
2. **Performance Metrics**: Monitoring data
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
| **MEDIUM** | Read-only but may impact | Large queries, config validation |
| **HIGH** | Data modification | DDL, bulk operations |
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

## Nagios XI-Specific Safety Rules

### Configuration Changes

1. Always validate before applying (`nagios -v`)
2. Backup configuration before changes
3. Test in non-production first
4. Monitor service status after apply
5. Have rollback configuration ready

### Database Operations

1. Never run DDL during business hours
2. Always backup database before operations
3. Monitor NDOUtil sync after changes
4. Verify web interface after DB changes
5. Check performance after schema changes

### Notification Changes

1. Test notification configuration before applying
2. Consider notification dampening
3. Verify contact routing after changes
4. Monitor for notification storms
5. Document notification changes

## Operational Guidelines

### Before Making Changes

1. Document current state
2. Take snapshot/backup
3. Test in non-production first
4. Plan rollback procedure
5. Schedule maintenance window

### After Making Changes

1. Verify configuration validity
2. Monitor service status
3. Check for errors in logs
4. Confirm monitoring is operational
5. Update documentation

## References

- Nagios XI Best Practices: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios XI Safety Guidelines: https://assets.nagios.com/downloads/nagiosxi/docs/
- Monitoring Safety: https://www.monitoringportal.org/