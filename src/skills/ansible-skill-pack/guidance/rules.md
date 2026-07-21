# Ansible Guidance and Safety Rules

## Engineering Principles

### Evidence-Based Reasoning

1. **Direct Error Messages**: Highest confidence source
2. **Output Metrics**: Ansible output data
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
| **LOW** | No impact | ansible --list-tasks, ansible-inventory --list |
| **MEDIUM** | Read-only but may impact | ansible-playbook --check, ansible-doc |
| **HIGH** | Data modification | ansible-playbook with state: present |
| **DISRUPTIVE** | Service impact | ansible-playbook with state: restarted |

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

## Ansible-Specific Safety Rules

### Playbook Changes

1. Always test with --check first
2. Use --diff to see changes before applying
3. Use --step for interactive review
4. Backup configuration before changes
5. Test in non-production first

### Inventory Changes

1. Validate inventory syntax before use
2. Test connectivity with ansible -m ping
3. Verify group membership before running
4. Keep backup of working inventory
5. Document inventory changes

### Role Changes

1. Test role tasks individually first
2. Use --start-at-task for testing
3. Verify role defaults
4. Check role dependencies
5. Test with --check before running

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
4. Confirm automation is operational
5. Update documentation

## References

- Ansible Safety: https://docs.ansible.com/ansible/latest/user_guide/
- Ansible Best Practices: https://docs.ansible.com/ansible/latest/
- Monitoring Safety: https://www.monitoringportal.org/