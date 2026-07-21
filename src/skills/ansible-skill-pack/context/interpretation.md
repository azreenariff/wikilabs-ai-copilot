# Ansible Context Interpretation

## Overview

This document explains how to interpret Ansible outputs, logs, and configuration.

## Interpreting Ansible Output

### Ansible Output Levels

| Level | Description | Example |
|-------|-------------|---------|
| **INFO** | Normal operation | PLAY [Deploy Web Server] |
| **DEBUG** | Debug information | TASK [debug] |
| **WARNING** | Non-critical issues | [WARNING] Deprecated |
| **ERROR** | Error condition | FAILED - No such file |
| **CHANGED** | State changed | changed=1 |
| **OK** | No change needed | ok=1 |

### Ansible Output Example

```
PLAY [Deploy Web Server] **************************************************

TASK [Gathering Facts] ****************************************************
ok: [web01]

TASK [Install Apache] *****************************************************
changed: [web01]

TASK [Start Apache] *******************************************************
ok: [web01]

PLAY RECAP ****************************************************************
web01 : ok=3  changed=1  unreachable=0  failed=0
```

## Interpreting Inventory Output

### Static Inventory

```ini
# inventory.ini
[webservers]
web01 ansible_host=192.168.1.10
web02 ansible_host=192.168.1.11

[databases]
db01 ansible_host=192.168.1.20
```

### Dynamic Inventory

```bash
# List inventory
ansible-inventory --list

# Graph inventory
ansible-inventory --graph

# Specific inventory file
ansible-inventory -i inventory.yaml --list
```

## Interpreting Playbook Output

### Playbook Structure

```yaml
---
- name: Deploy Web Server
  hosts: webservers
  become: true
  tasks:
    - name: Install Apache
      ansible.builtin.package:
        name: httpd
        state: present
    - name: Start Apache
      ansible.builtin.service:
        name: httpd
        state: started
```

### Output Interpretation

| Output | Meaning | Action |
|--------|---------|--------|
| **ok** | Task ran successfully, no change | None |
| **changed** | Task ran and modified state | Verify change |
| **failed** | Task failed | Investigate error |
| **skipped** | Task was skipped | Check conditions |
| **unreachable** | Host not reachable | Check connectivity |

## References

- Ansible Interpreting Output: https://docs.ansible.com/ansible/latest/user_guide/
- Ansible Output Interpretation: https://docs.ansible.com/ansible/latest/
- Ansible Interpreting Configuration: https://docs.ansible.com/ansible/latest/