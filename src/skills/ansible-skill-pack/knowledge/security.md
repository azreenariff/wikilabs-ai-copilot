# Ansible Security Configuration

## Overview

Ansible security configuration covers Vault, SSH, RBAC, and compliance.

## Ansible Vault

### Encrypting Files

```bash
# Encrypt file
ansible-vault encrypt secrets.yaml

# Decrypt file
ansible-vault decrypt secrets.yaml

# Edit encrypted file
ansible-vault edit secrets.yaml

# Create new encrypted file
ansible-vault create new_secrets.yaml
```

### Using Vault in Playbooks

```yaml
---
- name: Deploy Application
  hosts: webservers
  become: true
  vars_files:
    - vault/secrets.yaml
  tasks:
    - name: Configure application
      ansible.builtin.template:
        src: app.conf.j2
        dest: /etc/app/app.conf
        owner: app
        group: app
        mode: '0600'
```

### Vault Best Practices

1. Use separate vault files for different environments
2. Never commit vault passwords to version control
3. Use vault-id for different environments
4. Rotate vault passwords regularly
5. Restrict vault file access permissions

## SSH Security

### SSH Configuration

```bash
# Generate SSH key
ssh-keygen -t ed25519 -C "ansible@control"

# Copy SSH key
ssh-copy-id -i ~/.ssh/id_ed25519.pub ansible@managed-node

# SSH config for Ansible
# ~/.ssh/config
Host managed-*
    User ansible
    IdentityFile ~/.ssh/id_ed25519
    StrictHostKeyChecking no
```

## RBAC

### AWX/AAP RBAC

| Role | Permissions |
|------|-------------|
| **Admin** | Full access |
| **Contributor** | Create/edit projects, inventories |
| **Operator** | Run jobs, view credentials |
| **Auditor** | View only |

## Compliance

### Security Checklist

1. Use vault for all secrets
2. Restrict SSH access to managed nodes
3. Implement RBAC in AWX/AAP
4. Regular security audits
5. Encrypt sensitive data at rest
6. Follow least privilege principle

## References

- Ansible Security: https://docs.ansible.com/ansible/latest/user_guide/
- Ansible Vault: https://docs.ansible.com/ansible/latest/user_guide/vault.html
- Ansible Security Best Practices: https://docs.ansible.com/ansible/latest/