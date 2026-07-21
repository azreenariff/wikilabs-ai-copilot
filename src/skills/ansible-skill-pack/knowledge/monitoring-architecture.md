# Ansible Monitoring Architecture

## Overview

Ansible monitoring covers monitoring agent deployment, health checks, and integration with monitoring platforms.

## Monitoring Agent Deployment

### Node-Exporter (Linux)

```yaml
---
- name: Deploy Node Exporter
  hosts: monitoring
  become: true
  tasks:
    - name: Install Node Exporter
      ansible.builtin.apt:
        name: prometheus-node-exporter
        state: present
    - name: Configure Node Exporter
      ansible.builtin.lineinfile:
        path: /etc/default/prometheus-node-exporter
        regexp: '^ARGS='
        line: 'ARGS="--web.listen-address=:9100"'
    - name: Start Node Exporter
      ansible.builtin.service:
        name: prometheus-node-exporter
        state: started
        enabled: true
```

### Windows Exporter

```yaml
---
- name: Deploy Windows Exporter
  hosts: windows_servers
  tasks:
    - name: Install Windows Exporter
      win_chocolatey:
        name: windows-exporter
        state: present
    - name: Start Windows Exporter
      win_service:
        name: windows_exporter
        start_mode: auto
```

## Health Check Playbooks

### Basic Health Check

```yaml
---
- name: System Health Check
  hosts: all
  become: true
  tasks:
    - name: Check Disk Space
      ansible.builtin.shell: df -h / | awk 'NR==2 {print $5}'
      register: disk_usage
    - name: Check CPU Load
      ansible.builtin.shell: uptime | awk -F'load average:' '{print $2}'
      register: cpu_load
    - name: Check Memory
      ansible.builtin.shell: free -m | awk 'NR==2 {print $3/$2 * 100}'
      register: memory_usage
```

## Monitoring Integration

### Nagios XI Integration

```yaml
---
- name: Deploy NRPE Agent
  hosts: all
  become: true
  tasks:
    - name: Install NRPE
      ansible.builtin.package:
        name: nagios-nrpe-server
        state: present
    - name: Configure NRPE
      ansible.builtin.copy:
        src: nrpe.cfg
        dest: /etc/nagios/nrpe.cfg
        owner: nagios
        group: nagios
        mode: '0644'
    - name: Start NRPE
      ansible.builtin.service:
        name: nagios-nrpe-server
        state: started
        enabled: true
```

## References

- Ansible Monitoring: https://docs.ansible.com/ansible/latest/user_guide/
- Ansible Monitoring Architecture: https://docs.ansible.com/ansible/latest/
- Ansible Monitoring Integration: https://docs.ansible.com/ansible/latest/