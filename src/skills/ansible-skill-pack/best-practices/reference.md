# Ansible Best Practices

## Configuration Best Practices

### 1. Use Inventory Groups
- Organize hosts by environment and role
- Use hierarchical groups
- Define group variables

### 2. Use Roles for Reusability
- Structure roles with standard layout
- Use defaults and vars appropriately
- Document role dependencies

### 3. Implement Proper Variable Hierarchy
- Use group_vars and host_vars
- Override defaults appropriately
- Use vault for sensitive data

### 4. Use Check Mode
- Always test with --check first
- Use --diff to see changes
- Verify idempotency

### 5. Implement Proper Error Handling
- Use failed_when for custom failure conditions
- Use ignore_errors when appropriate
- Use block/rescue/always for error handling

## Performance Best Practices

### Execution Optimization

| Parameter | Recommended | Description |
|-----------|-------------|-------------|
| `forks` | 5-10 | Parallel host count |
| `gather_facts` | false | Skip facts collection if not needed |
| `serial` | 1 or percentage | Control deployment order |
| `throttle` | 1-5 | Rate limit task execution |

### Module Selection

| Goal | Recommended Module | Reason |
|------|-------------------|--------|
| Package management | ansible.builtin.package | OS-agnostic |
| Service management | ansible.builtin.service | OS-agnostic |
| File management | ansible.builtin.file | OS-agnostic |
| Template | ansible.builtin.template | Jinja2 support |
| Copy | ansible.builtin.copy | Simple file copy |

### Inventory Optimization

| Scale | Forks | Serial | Note |
|-------|-------|--------|------|
| Small (<50) | 10 | all | Fast deployment |
| Medium (50-200) | 5 | 20% | Controlled deployment |
| Large (200+) | 5 | 1 | Slowest, safest |

## Operational Best Practices

### Backup Strategy

1. Backup inventory before changes
2. Backup playbooks before changes
3. Backup roles before changes
4. Keep version control history
5. Test changes in non-production first

### Monitoring Best Practices

1. Monitor playbook execution times
2. Track failed tasks
3. Monitor resource usage during runs
4. Review change impact
5. Document successful deployments

### Version Control

1. Store playbooks in version control
2. Use branches for features
3. Document changes in commit messages
4. Use tags for releases
5. Review changes before merge

## Security Best Practices

1. Use Ansible Vault for secrets
2. Restrict SSH access to managed nodes
3. Implement RBAC in AWX/AAP
4. Regular security audits
5. Encrypt sensitive data at rest
6. Follow least privilege principle

## References

- Ansible Best Practices: https://docs.ansible.com/ansible/latest/user_guide/
- Ansible Performance: https://docs.ansible.com/ansible/latest/
- Ansible Security: https://docs.ansible.com/ansible/latest/