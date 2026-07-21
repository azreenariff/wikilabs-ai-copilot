# Ansible Architecture Overview

## Overview

Ansible is an open-source automation engine that provides configuration management, application deployment, and orchestration capabilities.

## Core Architecture

### Components

1. **Control Node**: The machine running Ansible, requires Python and SSH access
2. **Managed Nodes**: Target hosts managed by Ansible
3. **Inventory**: Host and group definitions
4. **Playbooks**: YAML automation definitions
5. **Modules**: Reusable automation code
6. **Plugins**: Extension mechanisms (filters, strategies, lookup)

### Communication

```
Control Node → SSH → Managed Nodes → Modules → Results
```

### Key Technologies

| Component | Technology | Version |
|-----------|-----------|---------|
| Ansible Core | ansible-core | 2.14+ |
| Ansible Engine | Red Hat Ansible Engine | 2.14+ |
| Automation Platform | AAP | 2.4+ |
| AWX | AWX | 23+ |
| Python on Control Node | Python | 3.8+ |
| Python on Managed Nodes | Python | 3.8+ |

### Architecture Layers

1. **Task Layer**: Playbooks and tasks
2. **Module Layer**: Individual automation units
3. **Transport Layer**: SSH, WinRM, API
4. **Host Layer**: Inventory and dynamic inventory
5. **Plugin Layer**: Filters, lookup, callbacks
6. **Integration Layer**: Galaxy, AWX, AAP

## References

- Ansible Architecture: https://docs.ansible.com/ansible/latest/getting_started/
- Ansible Core Documentation: https://docs.ansible.com/ansible/latest/
- Ansible Architecture Overview: https://docs.ansible.com/ansible/latest/