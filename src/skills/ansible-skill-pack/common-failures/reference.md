# Ansible Common Failure Patterns

## Overview

This reference documents common Ansible failure patterns, their symptoms, diagnosis methods, and recommended actions.

## Common Failures

### SSH Connection Failure

**Symptoms**:
- Host unreachable
- Authentication failed
- Connection timeout

**Diagnosis**:
```bash
# Test SSH connectivity
ssh -i ~/.ssh/id_ed25519 ansible@managed-node

# Test Ansible connectivity
ansible all -m ping

# Check SSH configuration
ansible-config dump | grep host_key_checking
```

**Remediation**:
1. Verify SSH key permissions
2. Check SSH service on managed node
3. Verify inventory host addresses
4. Check firewall rules
5. Test SSH manually first

### Module Failures

**Symptoms**:
- Task failed with error
- Module not found
- Invalid parameter

**Diagnosis**:
```bash
# Check module documentation
ansible-doc module_name

# List available modules
ansible-doc -l | grep module_name

# Test module ad-hoc
ansible all -m module_name -a "param=value"
```

**Remediation**:
1. Verify module name is correct
2. Check module parameters
3. Ensure required dependencies installed
4. Check Ansible version compatibility
5. Review module documentation

### Variable Errors

**Symptoms**:
- Undefined variable error
- Variable override not working
- Variable interpolation failure

**Diagnosis**:
```bash
# List all variables
ansible-playbook playbook.yaml --list-tasks -v

# Debug variables
ansible-playbook playbook.yaml -v --start-at-task="Debug"

# Check variable hierarchy
ansible all -m debug -a "var=vars"
```

**Remediation**:
1. Check variable definition location
2. Verify variable scope
3. Use group_vars and host_vars
4. Check variable naming
5. Test with debug module

### Idempotency Issues

**Symptoms**:
- Tasks report changed on every run
- Configuration drift
- Unexpected state changes

**Diagnosis**:
```bash
# Test idempotency
ansible-playbook playbook.yaml
ansible-playbook playbook.yaml

# Check diff
ansible-playbook playbook.yaml --diff
```

**Remediation**:
1. Review task logic
2. Use state: present/absent appropriately
3. Check for race conditions
4. Verify module idempotency
5. Use changed_when when needed

### Vault Errors

**Symptoms**:
- Vault decryption failed
- Password not found
- Encrypted file corrupted

**Diagnosis**:
```bash
# Test vault decryption
ansible-vault decrypt secrets.yaml

# View encrypted file
ansible-vault view secrets.yaml

# Check vault password
ansible-vault view --ask-vault-pass secrets.yaml
```

**Remediation**:
1. Verify vault password
2. Check vault file integrity
3. Use vault-id for multiple files
4. Test vault operations manually
5. Check vault file permissions

### Performance Issues

**Symptoms**:
- Slow playbook execution
- High resource usage
- Timeout errors

**Diagnosis**:
```bash
# Check execution times
ansible-playbook playbook.yaml -v

# Check fact gathering time
ansible-playbook playbook.yaml -c local --connection=local -e "gather_facts=false"

# Monitor resource usage
top -bn1 | head -20
```

**Remediation**:
1. Optimize forks setting
2. Disable unnecessary fact gathering
3. Use serial deployment for large groups
4. Optimize task ordering
5. Consider distributed execution

## References

- Ansible Troubleshooting: https://docs.ansible.com/ansible/latest/user_guide/
- Ansible Common Issues: https://docs.ansible.com/ansible/latest/
- Ansible Support: https://docs.ansible.com/ansible/latest/