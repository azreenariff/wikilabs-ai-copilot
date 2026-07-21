# Linux Engineering — Commands Reference

## Purpose

This directory contains the complete command reference for Linux engineering skill packs. Commands are organized by category and include purpose, risk classification, and usage examples.

## Command Categories

| Category | Description | File |
|----------|-------------|------|
| Service Management | systemctl, journalctl, systemd | commands/service.md |
| Process Monitoring | ps, top, htop, ps aux | commands/process.md |
| System Information | uname, hostnamectl, lscpu, lspci | commands/info.md |
| Storage Management | df, du, lsblk, mount, fdisk | commands/storage.md |
| Network Management | ip, ss, ping, nslookup, traceroute | commands/network.md |
| Security Management | ssh, firewall-cmd, getfacl, chown, chmod | commands/security.md |
| Package Management | dnf, apt, rpm, dpkg, yum | commands/packages.md |
| Performance Tuning | iostat, vmstat, strace, lsof | commands/performance.md |
| Scheduling | cron, crontab, at, systemd-timer | commands/schedule.md |
| Kernel Management | sysctl, modprobe, insmod, lsmod | commands/kernel.md |

## Risk Classification

| Risk Level | Description | Examples |
|------------|-------------|----------|
| Low | Read-only, non-destructive | ps, top, df, ip addr |
| Medium | Modifies system state but reversible | systemctl restart, mount, chown |
| High | Can cause system instability or data loss | kill -9, rm -rf, sysctl write, fsck |

## Command Metadata

Each command entry includes:
- **name**: Command name
- **description**: What the command does
- **syntax**: Usage syntax
- **purpose**: When to use it
- **risk**: Risk classification
- **examples**: Common usage patterns
- **notes**: Important caveats or prerequisites