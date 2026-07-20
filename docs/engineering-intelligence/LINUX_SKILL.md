# Linux Engineering Skill

> Wiki Labs AI Copilot v0.8.0-alpha  
> Phase 11 — Enterprise Skill Platform  
> Reference Skill Implementation

## Overview

The Linux Engineering Skill is the first reference skill shipped with Wiki Labs AI Copilot. It covers system administration, troubleshooting, kernel management, networking, storage, and security for all major Linux distributions.

**Why Linux?** Linux is foundational for most enterprise infrastructure. Nearly every other technology (OpenShift, VMware, databases, monitoring) runs on Linux. This skill provides the base knowledge layer that other skills build upon.

## What This Skill Covers

- System architecture and initialization (systemd)
- Process management (lifecycle, signals, cgroups)
- Filesystem management (mounts, LVM, disk space)
- Network configuration (interfaces, DNS, firewall)
- Service management (systemd units, cron, services)
- Package management (apt, yum, dnf, pacman)
- Kernel management (modules, parameters, boot process)
- Logging and monitoring (journald, rsyslog, dmesg)
- Security (SELinux, AppArmor, SSH, auditing)
- User and permissions management

## Detection Rules

The skill activates when the discovery engine detects Linux technology signals:

| Detection ID | Signal Type | Pattern | Confidence |
|-------------|-------------|---------|------------|
| `linux-detect-systemd` | File | `/etc/systemd/system/` | 0.9 |
| `linux-detect-apt` | File | `/etc/apt/` | 0.85 |
| `linux-detect-ssh` | File | `/etc/ssh/sshd_config` | 0.9 |
| `linux-detect-lvm` | File | `/etc/lvm/` | 0.85 |
| `linux-detect-firewall` | File | `/etc/iptables/` | 0.8 |
| `linux-detect-cron` | File | `/etc/cron` | 0.85 |
| `linux-detect-kernel` | Command | `uname -r` | 0.9 |
| `linux-detect-network` | File | `/etc/network/` | 0.8 |

When 4+ signals are detected, confidence approaches 1.0 and the skill activates automatically.

## Knowledge Base

### System Architecture

The Linux kernel manages hardware resources through:
- **Process scheduler** — Time-sharing CPU execution across processes
- **Memory manager** — Virtual memory, paging, swap, cgroups
- **File system layer** — VFS abstraction for ext4, xfs, btrfs, etc.
- **Network stack** — TCP/IP, sockets, routing tables, netfilter
- **Device drivers** — Kernel modules for hardware interaction
- **Security modules** — LSM frameworks (SELinux, AppArmor)

### Systemd Service Management

systemd is the init system and service manager for most modern distributions:

```bash
# View unit status
systemctl status <service>

# Check unit file
systemctl cat <service>

# List all units with failed state
systemctl list-units --state=failed

# View unit dependencies
systemctl list-dependencies <service>

# Mask (disable + prevent manual enable)
systemctl mask <service>

# Reload daemon config
systemctl daemon-reload
```

**Unit file structure:**
```ini
[Unit]
Description=My Service
After=network.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/bin/my-service --config /etc/my-service/config.yaml
Restart=on-failure
RestartSec=5
User=my-service
Group=my-service
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

### Journal Logging

```bash
# View service logs
journalctl -u <service>

# Follow logs in real-time
journalctl -u <service> -f

# Last 50 lines, no pager
journalctl -u <service> -n 50 --no-pager

# From specific time
journalctl -u <service> --since "1 hour ago"

# System boot journal
journalctl -b

# Kernel ring buffer
journalctl -k

# Log to file for external analysis
journalctl -u <service> --output=short-iso > /tmp/service.log
```

### Process Management

```bash
# List all processes with resource usage
ps aux --sort=-%cpu

# Real-time process monitoring
top -o %CPU
htop

# Kill process by PID
kill -SIGTERM <pid>    # Graceful shutdown
kill -SIGKILL <pid>    # Force kill

# Find processes using a port
ss -tlnp | grep :<port>
lsof -i :<port>

# Check resource limits
ulimit -a
cat /proc/<pid>/limits
```

### Filesystem and Storage

```bash
# Disk usage overview
df -h

# Directory size analysis
du -sh /* 2>/dev/null | sort -rh | head -20

# Find large files
find / -type f -size +100M -exec ls -lh {} \; 2>/dev/null | head -30

# Mount points
mount | column -t
findmnt -t ext4,xfs

# LVM status
vgs
pvs
lvs

# Inode usage
df -i

# Block device info
lsblk
blkid
```

### Network Configuration

```bash
# Interface configuration
ip addr show
ip -br addr

# Routing table
ip route show
ip -br route

# DNS resolution
cat /etc/resolv.conf
nslookup <domain>
dig +short <domain>

# Network connections
ss -tlnp                    # Listening TCP
ss -tnp                     # Established TCP
ss -ulnp                    # UDP sockets

# Packet routing trace
ip rule show
iptables -L -n -v

# DNS query
curl -s https://ifconfig.me
traceroute <host>
```

### Security Fundamentals

```bash
# SSH configuration
cat /etc/ssh/sshd_config
ss -tlnp | grep :22

# File permissions
ls -la /etc/shadow
stat /etc/ssh/sshd_config

# User and group info
id <username>
groups <username>
getent passwd
getent group

# Audit logging
auditctl -l
ausearch -m SYSCALL --success yes

# SELinux status
getenforce
sestatus

# AppArmor status
aa-status

# Check for world-writable files
find / -perm -o+w -type f 2>/dev/null | head -20
```

### Package Management

**Debian/Ubuntu:**
```bash
apt update
apt install <package>
apt remove <package>
apt list --installed | grep <package>
dpkg -l | grep <package>
```

**RHEL/CentOS/Fedora:**
```bash
dnf check-update
dnf install <package>
dnf remove <package>
rpm -qa | grep <package>
```

**Common across distributions:**
```bash
uname -r          # Kernel version
cat /etc/os-release  # Distribution info
```

## Workflows

### SSH Security Hardening

State machine for securing SSH server configuration:

**States:**
1. **Assessment** — Analyze current SSH config, check PermitRootLogin, PasswordAuthentication, Port, allowed ciphers
2. **Remediation** — Apply hardening changes (key-based auth, disable root login, change default port)
3. **Verification** — Test key-based auth, confirm root login disabled, verify access

**Commands (suggested, not executed):**
- `cat /etc/ssh/sshd_config`
- `ss -tlnp | grep ssh`
- `last -f /var/log/wtmp`
- `systemctl reload sshd`

**Evidence required:** Current sshd_config, active SSH processes, authentication logs, last login records

### Disk Space Remediation

State machine for resolving disk space issues:

**States:**
1. **Discovery** — Identify consuming filesystems with `df`, `du`, `find`
2. **Cleanup** — Remove old logs, caches, temp files; rotate logs
3. **Monitoring** — Configure logrotate, set up disk space alerts

**Commands (suggested, not executed):**
- `df -h`
- `du -sh /* 2>/dev/null | sort -rh | head -20`
- `journalctl --vacuum-time=3d`
- `find /var/log -name "*.gz" -delete`

**Evidence required:** Current disk usage, large files/directories identified, log file sizes

### Service Troubleshooting

State machine for diagnosing systemd service failures:

**States:**
1. **Diagnosis** — Examine logs and status with `systemctl status` and `journalctl`
2. **Root Cause Analysis** — Analyze failure cause from logs, check dependencies, config, resources
3. **Resolution** — Fix configuration, reload systemd, restart service

**Commands (suggested, not executed):**
- `systemctl status <service>`
- `journalctl -u <service> -n 50 --no-pager`
- `systemctl list-units --state=failed`
- `systemctl daemon-reload`
- `systemctl restart <service>`

**Evidence required:** Service status output, journal logs, service configuration file

## Common Commands Reference

| Category | Command | Purpose |
|----------|---------|---------|
| Service | `systemctl status <svc>` | Check service status |
| Service | `systemctl restart <svc>` | Restart a service |
| Logging | `journalctl -u <svc> -n 50 --no-pager` | View service logs |
| Logging | `journalctl -f -u <svc>` | Follow logs in real-time |
| Process | `ps aux --sort=-%cpu` | Top processes by CPU |
| Process | `top` / `htop` | Real-time process monitor |
| Disk | `df -h` | Disk usage |
| Disk | `du -sh /var/log` | Directory size |
| Disk | `find / -type f -size +100M` | Find large files |
| Network | `ip addr show` | Network interfaces |
| Network | `ss -tlnp` | Listening sockets |
| Network | `iptables -L -n -v` | Firewall rules |
| Package | `apt install <pkg>` | Install package (Debian) |
| Package | `dnf install <pkg>` | Install package (RHEL) |
| Package | `dpkg -l \| grep <pkg>` | Check installed package |
| User | `useradd -m -s /bin/bash <user>` | Create user |
| User | `chmod <mode> <file>` | Change permissions |
| User | `chown <user>:<group> <file>` | Change ownership |
| Security | `getenforce` | Check SELinux status |
| Security | `cat /etc/ssh/sshd_config` | SSH configuration |

## Best Practices

1. Always verify the current state before making changes
2. Use version control for configuration files
3. Automate repetitive tasks with scripts
4. Keep systems updated with security patches
5. Monitor disk space, memory, and CPU usage
6. Implement log rotation for all services
7. Use SSH keys instead of passwords for remote access
8. Document all manual interventions
9. Test changes in staging environments first
10. Maintain regular backup schedules
11. Implement proper access controls (least privilege)
12. Use configuration management tools (Ansible, Puppet, Chef)
13. Set up centralized logging (ELK, Grafana Loki)
14. Implement network segmentation for security
15. Keep kernel and critical packages updated

## Known Issues

### Systemd Service Restart Loops

When a service keeps restarting, check:
1. Service exit code: `journalctl -u <service> -n 20 --no-pager | grep "Main process exited"`
2. Missing dependencies: `systemctl list-dependencies <service> --reverse`
3. Resource limits: `journalctl -u <service> | grep -i "oom\|memory\|resource"`
4. Port conflicts: `ss -tlnp | grep :<port>`

### Disk Full on /var

If `/var` fills up:
1. Check journal size: `journalctl --disk-usage`
2. Vacuum old journals: `journalctl --vacuum-time=3d`
3. Check for large log files: `find /var/log -type f -size +100M`
4. Check for large packages: `du -sh /var/cache/* 2>/dev/null | sort -rh`
5. Check for core dumps: `find /var/lib/systemd/coredump -type f`

### High CPU Load

Investigation steps:
1. Identify top processes: `top -o %CPU` or `ps aux --sort=-%cpu | head -20`
2. Check for runaway threads: `ps -T -p <pid> -o tid,%cpu,cmd`
3. Check for spin loops: `dmesg | grep -i "softlockup\|hardlockup"`
4. Check I/O wait: `iostat -x 1` — high `%iowait` indicates disk bottleneck
5. Check for kernel threads: `ps aux | grep '\[.*\]'` — many idle kthreads are normal

### SSH Connection Refused

Troubleshooting SSH connectivity:
1. Check service: `systemctl status sshd`
2. Check listening port: `ss -tlnp | grep :22`
3. Check firewall: `iptables -L -n \| grep 22` or `firewall-cmd --list-ports`
4. Check SELinux: `getenforce` — if Enforcing, check `audit.log`
5. Check SSH config: `sshd -t` for syntax errors, `sshd -T \| grep Port`
6. Check logs: `journalctl -u sshd \| grep -i "error\|refused\|failed"`

## Safety Constraints

This skill **never**:
- Executes commands on the engineer's system
- Modifies configuration files
- Performs system remediation
- Automatically restarts services
- Changes firewall rules
- Installs or removes packages

This skill **always**:
- Suggests investigation commands for the engineer to run
- Provides recommended commands with clear explanations
- Shows expected output for context
- Requires engineer approval before any action
- Includes safety warnings for destructive operations
- States confidence levels for all recommendations

## Supported Environments

- Ubuntu (20.04, 22.04, 24.04)
- Debian (11, 12)
- CentOS (7, 8, Stream)
- RHEL (8, 9)
- Fedora (38+)
- Arch Linux
- Alpine Linux
- SUSE/SLES (15)

## Version

| Field | Value |
|-------|-------|
| Skill ID | `linux-engineering` |
| Version | `1.0.0` |
| Manifest Schema | `1.0` |
| Vendor | `Wiki Labs` |
| Category | `Engineering` |
| Dependencies | None (base skill) |
| Enabled | `true` |