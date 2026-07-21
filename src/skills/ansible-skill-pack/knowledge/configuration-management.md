# Ansible Configuration Management

## Overview

Ansible configuration management covers inventory, settings, playbooks, and roles.

## Configuration Files

### ansible.cfg

```ini
[defaults]
inventory = ./inventory
remote_user = ansible
host_key_checking = False
retry_files_enabled = False
roles_path = ./roles
library = ./library

[privilege_escalation]
become = True
become_method = sudo
become_user = root
become_ask_pass = False
```

### Inventory Configuration

```yaml
# inventory.yaml
all:
  children:
    webservers:
      hosts:
        web01:
          ansible_host: 192.168.1.10
        web02:
          ansible_host: 192.168.1.11
    databases:
      hosts:
        db01:
          ansible_host: 192.168.1.20
```

## Playbook Configuration

### Variables

```yaml
# group_vars/webservers/all.yaml
http_port: 80
https_port: 443
app_user: www-data
app_group: www-data
```

### Playbook Structure

```yaml
---
- name: Deploy Web Server
  hosts: webservers
  become: true
  vars_files:
    - vars/app.yaml
  roles:
    - common
    - apache
    - application
  handlers:
    - name: Restart Apache
      ansible.builtin.service:
        name: httpd
        state: restarted
```

## Role Structure

```
role_name/
├── tasks/
│   └── main.yaml
├── handlers/
│   └── main.yaml
├── templates/
│   └── httpd.conf.j2
├── files/
│   └── index.html
├── vars/
│   └── main.yaml
├── defaults/
│   └── main.yaml
└── meta/
    └── main.yaml
```

## References

- Ansible Configuration: https://docs.ansible.com/ansible/latest/user_guide/
- Ansible Configuration Management: https://docs.ansible.com/ansible/latest/
- Ansible Configuration Best Practices: https://docs.ansible.com/ansible/latest/