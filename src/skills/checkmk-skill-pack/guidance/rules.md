# Checkmk Guidance and Safety Rules

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
| **LOW** | No impact | Livequery, status checks |
| **MEDIUM** | Read-only but may impact | Large queries, config validation |
| **HIGH** | Data modification | DDL, bulk operations |
| **DISRUPTIVE** | Service impact | Site restart, major config |

## Safety Rules

1. Never execute commands — only recommend and explain
2. Always warn about risks before recommending actions
3. Always provide rollback strategies
4. Always recommend evidence collection before diagnosis
5. Always consider cascade effects of recommended actions
6. Never modify thresholds or configuration without explicit engineer approval
7. Always recommend config validation before reload
8. Always recommend testing changes in non-production first
9. Always validate configuration changes before applying
10. Always recommend backup before destructive operations

## Checkmk-Specific Safety Rules

### Ruleset Changes

1. Always test rules in non-production first
2. Use check_mk --dry-run for validation
3. Monitor check execution after changes
4. Keep backup of working configuration
5. Apply changes during maintenance windows

### Host/Service Changes

1. Validate configuration before reload
2. Test connectivity before adding hosts
3. Monitor check execution after changes
4. Verify notifications after contact changes
5. Document all configuration changes

### Threshold Changes

1. Never modify thresholds without explicit engineer approval
2. Always review historical data before changes
3. Consider false positive/negative impact
4. Document threshold change rationale
5. Monitor alert rates after changes

## Operational Guidelines

### Before Making Changes

1. Document current state
2. Take snapshot/backup
3. Test in non-production first
4. Plan rollback procedure
5. Schedule maintenance window

### After Making Changes

1. Verify configuration validity
2. Monitor check execution
3. Check for errors in logs
4. Confirm monitoring is operational
5. Update documentation

## References

- Checkmk Safety: https://docs.checkmk.com/master/en/
- Checkmk Best Practices: https://docs.checkmk.com/master/en/
- Monitoring Safety: https://www.monitoringportal.org/