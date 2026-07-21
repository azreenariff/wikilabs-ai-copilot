# Linux Engineering Foundation

## Architecture

Linux is a Unix-like, monolithic kernel operating system. The architecture consists of:

### Kernel Layer
- **Core kernel:** Process scheduling, memory management, device drivers, file systems, networking stack
- **System calls:** Interface between user space and kernel space
- **Modules:** Loadable kernel modules (LKMs) for dynamic functionality

### User Space
- **Systemdaemon:** Init system, service manager, logging (journald)
- **Shell:** Bash, Zsh, or other POSIX shells
- **Utilities:** Core GNU utilities, package managers, networking tools
- **Applications:** Daemons, user applications, containers

### Key Design Principles
- Everything is a file
- Small, single-purpose tools composed together
- Plain text configuration
- Open source, community-driven
- POSIX compliance

---

## Core Concepts

### Init Systems
- **systemd:** Modern init system with parallel startup, service management, logging
- **SysVinit:** Legacy init system (deprecated in most distributions)
- **Upstart:** Transitional init system (deprecated)

### Filesystem Hierarchy Standard (FHS)
| Path | Purpose |
|------|---------|
| / | Root filesystem |
| /bin | Essential user binaries |
| /etc | Configuration files |
| /home | User home directories |
| /var | Variable data (logs, cache, spool) |
| /tmp | Temporary files |
| /dev | Device files |
| /proc | Process and kernel information (virtual FS) |
| /sys | Kernel and hardware information (virtual FS) |
| /opt | Optional/additional software |
| /usr | User programs and read-only data |

### Process Model
- **PID (Process ID):** Unique identifier for each process
- **PPID (Parent PID):** Parent process identifier
- **PPID 1 (init/systemd):** Parent of all user-space processes
- **Zombie processes:** Terminated but not reaped by parent
- **Orphan processes:** Parent died, adopted by init

### Permissions Model
- **Owners:** User (owner), Group, Other
- **Modes:** Read (r=4), Write (w=2), Execute (x=1)
- **setuid/setgid:** Execute with owner/group privileges
- **sticky bit:** Prevent deletion by non-owners (e.g., /tmp)
- **ACLs:** Fine-grained access control beyond basic permissions

### User Management
- **Root (UID 0):** Superuser with full privileges
- **System users:** Services and daemons (UID 1-999 on modern systems)
- **Regular users:** Human accounts (UID 1000+)
- **Groups:** Organize users, control access via group permissions

---

## Common Components

### System Services
| Component | Purpose |
|-----------|---------|
| systemd | Init system, service manager |
| journald | System logging |
| cron | Scheduled task execution |
| systemd-resolved | DNS resolution |
| NetworkManager | Network configuration |
| firewalld / iptables | Firewall management |
| sshd | SSH daemon |
| udev | Device management |
| logrotate | Log rotation |
| auditd | Security auditing |

### Package Management
| Distribution | Package Manager | Format |
|--------------|-----------------|--------|
| RHEL/CentOS/Alma/Rocky | dnf/yum | RPM |
| Debian/Ubuntu | apt | DEB |
| SUSE/openSUSE | zypper | RPM |
| Arch | pacman | PKGBUILD |
| Alpine | apk | APK |

### Key Utilities
- **Process:** ps, top, htop, systemctl, kill, nice, ionice
- **Filesystem:** df, du, lsblk, mount, mount, find, stat, blkid
- **Memory:** free, vmstat, slabtop, /proc/meminfo
- **Network:** ip, ss, netstat, traceroute, ping, curl, wget
- **Storage:** fdisk, lsblk, blkid, lvs, pvs, vgs, smartctl
- **Security:** id, whoami, groups, su, sudo, getfacl, setfacl
- **Logs:** journalctl, dmesg, less, tail, grep, awk, sed

---

## Common Failures

### Boot Failures
| Symptom | Possible Cause |
|---------|----------------|
| Kernel panic | Hardware failure, driver issue, corrupted kernel |
| Boot loop | Corrupted initramfs, filesystem errors |
| Drop to emergency mode | Filesystem mount failure, missing device |
| Grub not found | Bootloader corruption, disk failure |

### Service Failures
| Symptom | Possible Cause |
|---------|----------------|
| Service not starting | Configuration error, port conflict, missing dependency |
| Service fails after boot | Timeout, resource exhaustion, SELinux blocking |
| Service crashes | Application bug, memory exhaustion, segfault |
| Service stops unexpectedly | OOM killer, signal received, resource limit |

### Resource Exhaustion
| Symptom | Possible Cause |
|---------|----------------|
| High CPU usage | Runaway process, crypto mining, misconfiguration |
| High memory usage | Memory leak, too many processes, insufficient RAM |
| Disk full (/var, /tmp, /boot) | Log accumulation, temp files, package cache |
| No inodes | Many small files exhausting inode table |
| Swap thrashing | Insufficient RAM, memory leak |

### Network Failures
| Symptom | Possible Cause |
|---------|----------------|
| No network connectivity | Interface down, DHCP failure, routing error |
| DNS resolution fails | DNS server unreachable, /etc/resolv.conf misconfigured |
| Can ping but can't access services | Firewall blocking, service not listening, SELinux |
| Intermittent connectivity | Network hardware, cable, wireless interference |

### Permission Failures
| Symptom | Possible Cause |
|---------|----------------|
| Permission denied | Incorrect ownership, restrictive permissions, ACLs |
| Can't read config file | SELinux context, restrictive permissions, AppArmor |
| Sudo not working | User not in sudoers, misconfigured sudoers file |

---

## Troubleshooting Philosophy

### Step-by-Step Diagnostic Approach

**Phase 1: Understand the Symptom**
- What is the user seeing?
- When did it start?
- What changed recently?
- Is it intermittent or consistent?

**Phase 2: Gather Evidence**
- Check system status: `systemctl status <service>`
- Check logs: `journalctl -u <service> --since "1 hour ago"`
- Check resources: `top`, `df -h`, `free -m`, `vmstat`
- Check network: `ip addr`, `ss -tlnp`, `ping`, `traceroute`
- Check events: `journalctl --priority=err`, `dmesg | tail`

**Phase 3: Hypothesis**
- Based on evidence, form a likely cause
- Rank hypotheses by likelihood and impact
- Consider recent changes (config, software, hardware)

**Phase 4: Validate**
- Test hypothesis with targeted commands
- Check logs for confirmation
- Verify no other symptoms contradict hypothesis

**Phase 5: Remediate**
- Apply fix based on confirmed root cause
- Document changes made
- Verify fix with monitoring

**Phase 6: Verify**
- Confirm issue is resolved
- Monitor for recurrence
- Update documentation if new pattern discovered

### Evidence Priority Order
1. **System status** (systemctl, uptime, load average)
2. **Logs** (journalctl, dmesg, application logs)
3. **Resource usage** (CPU, memory, disk, network)
4. **Events** (recent errors, warnings, criticals)
5. **Configuration** (relevant config files, services)

---

## Best Practices

### System Administration
- **Always check logs first** — errors are often in journalctl before symptoms appear
- **Use systemd services** — proper restart policies, logging, dependency management
- **Monitor resource usage** — set up alerts for CPU, memory, disk thresholds
- **Document changes** — log what was changed, why, and when
- **Back up configurations** — keep /etc backups before making changes
- **Use configuration management** — ansible, puppet, or chef for consistency
- **Keep systems updated** — regular security and bug fix updates
- **Test in staging** — verify changes before production deployment

### Security
- **Principle of least privilege** — grant minimum required access
- **Regular audits** — review users, groups, permissions, sudoers
- **SSH hardening** — disable password auth, use keys, change default port
- **Firewall by default** — deny all, allow explicitly
- **SELinux in enforcing mode** — on RHEL-family systems
- **Fail2ban** — protect SSH and other services from brute force
- **Regular updates** — apply security patches promptly
- **Audit logging** — enable auditd for compliance

### Performance
- **Baseline normal behavior** — know what "good" looks like
- **Monitor trends** — track resource usage over time
- **Avoid over-provisioning** — right-size to actual usage
- **Clean up regularly** — remove old logs, cache, temp files
- **Use monitoring tools** — Prometheus, Grafana, Nagios, Zabbix
- **Set alert thresholds** — proactive notification before issues

### Backup & Recovery
- **3-2-1 rule** — 3 copies, 2 different media, 1 off-site
- **Test restores** — backups are useless if you can't restore
- **Automate backups** — cron or systemd timers
- **Document recovery procedures** — know what to do in an emergency
- **Keep bootable media** — for recovery scenarios

---

## Risk Awareness

### High-Risk Operations
| Operation | Risk Level | Warning |
|-----------|------------|---------|
| `rm -rf /` | Critical | Destroys entire filesystem |
| `dd` to disk | Critical | Overwrites data, can destroy filesystems |
| `chmod 777` | High | Removes all access controls |
| Modifying /etc/passwd | High | Can lock out all users |
| Kernel parameter changes (/etc/sysctl.conf) | Medium | Can cause instability |
| Modifying /etc/fstab | Medium | Can prevent boot if incorrect |
| Stopping critical services | Medium | Can cause service disruption |
| Package updates | Low | Usually safe, but check release notes |
| SSH config changes | Low-Medium | Can lock out remote access |

### Safe Commands (Read-Only)
- `systemctl status`
- `journalctl`
- `ps`, `top`, `htop`
- `df`, `du`
- `free`
- `ip`, `ss`
- `cat` (with caution on large files)
- `ls`, `find`
- `grep`
- `wc`
- `head`, `tail`

### Safe Commands (Low-Risk Write)
- `systemctl restart` (service restart)
- `systemctl reload` (config reload without restart)
- `journalctl --vacuum-size` (clean old logs)
- `yum clean all` / `apt clean` (clear package cache)
- `systemctl enable` (enable at boot)
- `systemctl disable` (disable at boot)

### Commands Requiring Human Review
- Any `rm` command
- Any `dd` command
- Any `chmod` or `chown`
- Any `fsck` command
- Any network configuration change
- Any kernel module load/unload
- Any fstab modification
- Any firewall rule change

---

## Decision Trees

### System Slow — Diagnostic Flow

```
System slow
  │
  ├─→ Check CPU usage (top)
  │   ├─→ High (>80%) → Identify top process
  │   │   ├─→ Expected (batch job) → Let it finish
  │   │   └─→ Unexpected → Check if runaway process
  │   │       └─→ Consider killing or resource limiting
  │   │
  │   └─→ Low (<50%) → Check memory
  │
  ├─→ Check memory (free -m)
  │   ├─→ Low free + high swap → Insufficient RAM
  │   │   └─→ Consider adding RAM or reducing workloads
  │   │
  │   └─→ High free → Check I/O
  │
  └─→ Check I/O (iostat, df, dmesg)
      ├─→ Disk 100% busy → Find blocking process (iotop)
      │   └─→ Check for log storms, backups, or failing disk
      │
      └─→ Disk OK → Check network or application layer
```

### Service Not Starting — Diagnostic Flow

```
Service not starting
  │
  ├─→ systemctl status <service>
  │   ├─→ Exit code >0 → Check journalctl for errors
  │   │   └─→ Parse error message for root cause
  │   │
  │   └─→ Exit code 0 → Check if listening on expected port
  │
  ├─→ journalctl -u <service> --since "5 min ago"
  │   ├─→ Configuration error → Check service config file
  │   │   └─→ Validate syntax: `systemctl daemon-reload`
  │   │
  │   ├─→ Port conflict → `ss -tlnp | grep <port>`
  │   │   └─→ Change port or stop conflicting service
  │   │
  │   ├─→ Permission denied → Check SELinux/AppArmor
  │   │   └─→ Check file permissions and ownership
  │   │
  │   └─→ No obvious errors → Check resource limits
  │       └─→ ulimit, cgroups, memory limits
  │
  └─→ Still failing → Check dependencies
      └─→ systemctl list-dependencies <service>
```

---

## References

- [Red Hat Enterprise Linux Documentation](https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/)
- [Debian Administrator's Handbook](https://debian-handbook.info/)
- [Linux System Administrator's Guide](https://linux.die.net/gentoo-linux-admin-guide)
- [Systemd Documentation](https://www.freedesktop.org/software/systemd/man/)
- [GNU Coreutils Manual](https://www.gnu.org/software/coreutils/manual/)
- [NIXPedia - Linux Administration](https://www.nixcraft.com/)
- [Server Fault Linux Tag](https://serverfault.com/questions/tagged/linux)

---

## Examples

### Example 1: High CPU Diagnosis

**Symptom:** Server responding slowly

**Evidence:**
```bash
# Check load average
$ uptime
  14:32:01 up 45 days,  3:14,  2 users,  load average: 8.5, 6.2, 4.1

# Identify top processes
$ top -bn1 | head -20
  PID USER      PR  NI    VIRT    RES    SHR S  %CPU  %MEM     TIME+ COMMAND
 12345 app       20   0 2048000 512000  4096 S  85.2   3.2  12:34.56 java

# Check if expected
$ ps aux | grep java | grep -v grep
  app  12345 85.2  3.2 2048000 512000 ?  Ssl  Jul01  12:34 /usr/bin/java -jar app.jar
```

**Diagnosis:** Java application consuming 85% CPU — expected batch processing job.

**Action:** Monitor completion. If unexpected, check application logs for errors.

### Example 2: Disk Full Diagnosis

**Symptom:** Application errors, unable to write files

**Evidence:**
```bash
$ df -h
  /dev/sda1       50G   50G     0  100% /
  /dev/sdb1      500G  400G  100G   80% /var

$ df -i
  /dev/sda1       3M    3M     0  100% /
```

**Diagnosis:** Both disk space and inodes exhausted on /

**Action:**
```bash
# Find large files
$ find / -type f -size +100M 2>/dev/null | sort -k5 -rn | head -10

# Find many small files (inodes)
$ for dir in /var/log /tmp /home; do
    echo "$dir: $(find $dir -maxdepth 2 | wc -l) files"
done

# Clean up
$ journalctl --vacuum-size=1G
$ rm -f /tmp/*.tmp
$ yum clean all
```

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial Linux Engineering Foundation |