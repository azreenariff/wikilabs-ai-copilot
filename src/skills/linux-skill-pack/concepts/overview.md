# Linux Engineering — Concepts Overview

## Core Concepts

This skill pack covers the fundamental concepts of Linux system engineering, organized by domain.

## Domains

### 1. System Architecture
Linux is built on a monolithic kernel with a rich user-space ecosystem. Key concepts include:
- **Kernel**: The core that manages hardware abstraction, process scheduling, and memory management
- **Init System**: systemd manages services, logging, and system state transitions
- **Filesystem**: Hierarchical directory structure following FHS standards
- **Process Model**: UNIX process hierarchy with parent-child relationships and signals

### 2. Service Management
systemd provides the foundation for service lifecycle management:
- **Units**: Resources managed by systemd (services, targets, mounts, sockets, timers)
- **Dependencies**: Units can depend on other units for ordering and activation
- **State Machine**: Services transition through active, inactive, failed states
- **Logging**: journalctl provides persistent, searchable log management

### 3. Package Management
Each distribution family has its own package ecosystem:
- **RHEL family**: dnf/rpm with RPM Package Manager
- **Debian family**: apt/dpkg with DEB Package Manager
- **Alpine**: apk with Alpine Package Keeper
- **Package lifecycle**: Install, update, remove, query, verify

### 4. Network Management
Linux networking follows the TCP/IP stack with extensive tooling:
- **Interfaces**: Network devices (eth0, wlan0, docker0)
- **Routing**: IP routing tables and gateway management
- **DNS**: Name resolution through resolv.conf and systemd-resolved
- **Firewall**: Packet filtering through firewalld, ufw, or iptables

### 5. Security Management
Linux provides layered security:
- **Authentication**: Passwords, SSH keys, PAM, Kerberos
- **Authorization**: File permissions, ACLs, SELinux/AppArmor
- **Auditing**: auditd, journald, log management
- **Hardening**: Firewall configuration, SSH hardening, patch management

### 6. Storage Management
Linux storage spans from physical disks to logical volumes:
- **Disk types**: HDD, SSD, NVMe with different performance characteristics
- **Filesystems**: ext4, XFS, Btrfs, ZFS with different features
- **LVM**: Logical Volume Manager for flexible storage allocation
- **Monitoring**: Disk usage, I/O performance, SMART health

### 7. Performance Management
System performance spans multiple dimensions:
- **CPU**: Process scheduling, load averages, CPU governors
- **Memory**: RAM usage, swap, OOM killer, page cache
- **Disk I/O**: Block device throughput, I/O wait, disk latency
- **Network**: Bandwidth, latency, packet loss, connection tracking

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial concepts overview |