# Ansible Diagnostic Reasoning Reference

## Reasoning Patterns

### Pattern 1: Connection Failure

```
Observation: Host unreachable
  ├── SSH working?
  │   ├── No: Fix SSH configuration
  │   └── Yes: Check Ansible settings
  ├── Inventory correct?
  │   ├── No: Fix inventory
  │   └── Yes: Check credentials
  └── Firewall blocking?
      ├── No: Check network
      └── Yes: Open firewall
```

### Pattern 2: Task Failure

```
Observation: Task failed
  ├── Module correct?
  │   ├── No: Fix module name/params
  │   └── Yes: Check task logic
  ├── Variables defined?
  │   ├── No: Define variables
  │   └── Yes: Check variable values
  └── Target state reachable?
      ├── No: Check target system
      └── Yes: Check prerequisites
```

### Pattern 3: Performance Issues

```
Observation: Slow execution
  ├── Forks low?
  │   ├── Yes: Increase forks
  │   └── No: Check other factors
  ├── Facts gathering slow?
  │   ├── Yes: Disable or optimize
  │   └── No: Check module performance
  └── Network slow?
      ├── Yes: Optimize transport
      └── No: Check host resources
```

## Confidence Assessment

### Evidence Quality

| Evidence Type | Quality | Example |
|--------------|---------|---------|
| Direct error | High | Ansible error message |
| Module output | High | Task output |
| Inventory data | Medium | ansible-inventory output |
| User reports | Low | "Playbook is slow" |

### Confidence Scoring

1. **High (85-100%)**: Multiple high-quality evidence points
2. **Medium (60-84%)**: Some evidence with supporting data
3. **Low (30-59%)**: Limited evidence, needs more investigation
4. **Very Low (0-29%)**: Insufficient evidence

## References

- Ansible Diagnostics: https://docs.ansible.com/ansible/latest/user_guide/
- Ansible Troubleshooting: https://docs.ansible.com/ansible/latest/
- Ansible Debugging: https://docs.ansible.com/ansible/latest/