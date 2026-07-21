# Linux Knowledge — System Architecture

## Architecture Overview

Linux is a UNIX-like, open-source operating system built on the Linux kernel with a rich ecosystem of user-space tools and libraries.

## Key Components

### Kernel Layer
| Component | Description |
|-----------|-------------|
| Linux Kernel | Core of the operating system — process management, memory management, device drivers |
| sysctl | Runtime kernel parameter configuration |
| cgroups | Control groups for resource isolation and limits |
| namespaces | Process, network, mount, and user namespace isolation |

### Init System
| Component | Description |
|-----------|-------------|
| systemd | Modern init system — service management, logging, unit files |
| SysVinit | Legacy init system (still supported for compatibility) |
| OpenRC | Lightweight init system used by Alpine, Gentoo |

### User Space
| Component | Description |
|-----------|-------------|
| glibc/musl | C standard library implementations |
| bash/zsh/fish | Shell interpreters |
| coreutils | Fundamental commands (ls, cp, mv, rm, mkdir) |
| sysstat | System monitoring tools (iostat, mpstat, sar) |

### Package Managers
| Distro Family | Package Manager | Repository Format |
|--------------|-----------------|-------------------|
| RHEL/Rocky/Alma/CentOS | dnf/rpm | RPM packages, yum repos |
| Debian/Ubuntu | apt/dpkg | DEB packages, apt repos |
| Alpine | apk | APK packages, Alpine repos |
| SUSE | zypper/rpm | RPM packages, zypper repos |
| Arch | pacman | PKGBUILD, Arch repos |

## Process Model

### Process Hierarchy
```
init (PID 1)
├── systemd (PID 1 on modern systems)
│   ├── dbus-daemon
│   ├── sshd
│   ├── nginx
│   └── systemd-journald
├── cron
└── rsyslogd
```

### Process States
| State | Description |
|-------|-------------|
| R (Running) | Process is executing or ready to execute |
| S (Sleeping) | Process is waiting for an event |
| D (Uninterruptible Sleep) | Process is waiting for I/O |
| T (Stopped) | Process is stopped (e.g., debugged) |
| Z (Zombie) | Process terminated but parent hasn't collected exit status |

### Resource Limits
```bash
# View limits
ulimit -a

# Set limits
ulimit -n 65536    # File descriptors
ulimit -u 4096     # User processes
ulimit -m unlimited # Memory
```

## Filesystem Hierarchy Standard (FHS)

| Path | Purpose |
|------|---------|
| `/` | Root directory |
| `/bin` | Essential user binaries |
| `/sbin` | Essential system binaries |
| `/etc` | Configuration files |
| `/home` | User home directories |
| `/var` | Variable data (logs, cache, spools) |
| `/tmp` | Temporary files |
| `/dev` | Device files |
| `/proc` | Process and system information (virtual filesystem) |
| `/sys` | Kernel and device information (virtual filesystem) |
| `/usr` | User programs and data |
| `/opt` | Add-on application software |
| `/boot` | Boot loader files |
| `/lib` | Shared libraries |
| `/run` | Runtime data (PID files, sockets) |

## Virtual Filesystems

### /proc
```bash
# CPU information
cat /proc/cpuinfo

# Memory information
cat /proc/meminfo

# Mount information
cat /proc/mounts

# Running processes
ls /proc
```

### /sys
```bash
# Device information
ls /sys/class

# Power management
cat /sys/power/state

# CPU governor
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
```

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial system architecture overview |