# Ansible Diagnostic Procedures

## Overview

Systematic diagnostic procedures for common Ansible issues.

## Connectivity Issues

### SSH Connectivity

**Symptoms**:
- Host unreachable
- Authentication failed

**Diagnostic Steps**:
```bash
# Check SSH manually
ssh -i ~/.ssh/id_ed25519 ansible@managed-node

# Test with Ansible
ansible all -m ping

# Check SSH logs
tail -n 100 /var/log/ssh/auth.log
```

**Evidence Required**:
- SSH manual test result
- Ansible ping result
- SSH log entries

**Remediation**:
1. Fix SSH configuration
2. Verify credentials
3. Check firewall rules
4. Restart SSH service
5. Test again

### Inventory Issues

**Symptoms**:
- Hosts not found
- Group membership incorrect

**Diagnostic Steps**:
```bash
# List inventory
ansible-inventory --list

# Graph inventory
ansible-inventory --graph

# Test specific host
ansible all --list-hosts
```

**Evidence Required**:
- Inventory listing
- Graph output
- Host availability

**Remediation**:
1. Fix inventory file
2. Verify YAML syntax
3. Check group definitions
4. Update inventory
5. Test connectivity

## Playbook Issues

### Syntax Errors

**Symptoms**:
- Playbook fails to execute
- YAML syntax error

**Diagnostic Steps**:
```bash
# Check syntax
ansible-playbook playbook.yaml --syntax-check

# List tasks
ansible-playbook playbook.yaml --list-tasks

# List plays
ansible-playbook playbook.yaml --list-play
```

**Evidence Required**:
- Syntax check result
- Error message
- Line number

**Remediation**:
1. Fix YAML syntax
2. Validate structure
3. Test with --syntax-check
4. Apply changes
5. Verify fix

### Variable Issues

**Symptoms**:
- Undefined variable
- Wrong variable value

**Diagnostic Steps**:
```bash
# Debug variables
ansible-playbook playbook.yaml -v --start-at-task="Debug"

# Check variable scope
ansible all -m debug -a "var=vars"
```

**Evidence Required**:
- Variable value
- Scope information
- Error message

**Remediation**:
1. Define variable
2. Check variable scope
3. Use group_vars
4. Update playbook
5. Test fix

## References

- Ansible Diagnostics: https://docs.ansible.com/ansible/latest/user_guide/
- Ansible Troubleshooting: https://docs.ansible.com/ansible/latest/
- Ansible Debugging: https://docs.ansible.com/ansible/latest/