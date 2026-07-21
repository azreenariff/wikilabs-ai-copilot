# Ansible Architecture Reference

## Overview

This document provides architectural details for Ansible deployments.

## Core Architecture

### Components

1. **Control Node**: Machine running Ansible
2. **Managed Nodes**: Target hosts
3. **Inventory**: Host definitions
4. **Playbooks**: Automation definitions
5. **Modules**: Automation units
6. **Plugins**: Extension mechanisms

### Data Flow

```
Control Node → Inventory → Playbook → SSH → Managed Nodes → Modules → Results
```

### Communication Protocols

| Protocol | Use Case | Transport |
|----------|----------|-----------|
| **SSH** | Linux/Unix | TCP port 22 |
| **WinRM** | Windows | TCP port 5985/5986 |
| **API** | AAP/AWX | HTTP/HTTPS |
| **Local** | Control node | Local execution |

### Plugin Architecture

| Plugin Type | Purpose | Examples |
|-------------|---------|----------|
| **Connection** | Node connectivity | ssh, winrm, local |
| **Strategy** | Execution strategy | linear, free, host_pinned |
| **Lookup** | External data | file, ini, env |
| **Filter** | Data transformation | json_query, default |
| **Callback** | Event handling | timer, log_plays |

## References

- Ansible Architecture: https://docs.ansible.com/ansible/latest/getting_started/
- Ansible Core Architecture: https://docs.ansible.com/ansible/latest/
- Ansible Plugin System: https://docs.ansible.com/ansible/latest/plugins.html