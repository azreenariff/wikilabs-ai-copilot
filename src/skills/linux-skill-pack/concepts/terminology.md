# Linux Engineering — Terminology

## System Terms

| Term | Definition |
|------|------------|
| Kernel | Core of the operating system that manages hardware and software resources |
| Init System | System and service manager that starts at boot (PID 1) |
| Daemon | Background process that runs without interactive control |
| Process | Running instance of a program |
| Thread | Lightweight process within a process |
| PID | Process ID — unique identifier for each process |
| PPID | Parent Process ID — the process that started this one |
| Zombie | Terminated process whose exit status has not been collected by parent |
| Orphan | Process whose parent has terminated |

## Filesystem Terms

| Term | Definition |
|------|------------|
| Mount Point | Directory where a filesystem is attached to the hierarchy |
| Block Device | Device file representing storage hardware (/dev/sda) |
| inode | Data structure storing file metadata (permissions, owner, timestamps) |
| Superblock | Data structure storing filesystem metadata |
| Journal | Mechanism that logs changes before applying them (crash recovery) |
| Symlink | Symbolic link — pointer to another file or directory |
| Hardlink | Direct reference to an inode (multiple names, same data) |
| FHS | Filesystem Hierarchy Standard — Linux directory structure specification |

## Service Terms

| Term | Definition |
|------|------------|
| Unit | Managed resource in systemd (service, target, mount, etc.) |
| Target | Group of units for system state (multi-user.target, graphical.target) |
| Service Unit | Configuration file for a daemon (e.g., nginx.service) |
| WantedBy | Target that enables a unit (similar to init.d runlevels) |
| Requires | Hard dependency — unit fails if dependency fails |
| Wants | Soft dependency — unit starts with dependency but doesn't fail if it fails |
| After | Ordering dependency — start this unit after listed units |
| Before | Ordering dependency — start this unit before listed units |
| Daemon-reload | Reload systemd configuration after unit file changes |

## Package Terms

| Term | Definition |
|------|------------|
| Package | Software bundle including binaries, libraries, and metadata |
| Repository | Remote location containing packages and metadata |
| Dependency | Package required by another package |
| Dependency Conflict | Two packages require incompatible versions |
| Transaction | Atomic set of package operations (install, remove, update) |
| RPM | Red Hat Package Manager — package format for RHEL-family |
| DEB | Debian package format — package format for Debian/Ubuntu |
| APK | Alpine Package Keeper — package format for Alpine Linux |
| GPG Signature | Cryptographic signature verifying package authenticity |
| Changelog | Record of changes in package versions |

## Network Terms

| Term | Definition |
|------|------------|
| Interface | Network device (eth0, wlan0, docker0) |
| IP Address | Unique identifier for a network interface |
| Subnet | Network segment defined by IP range and mask |
| Gateway | Router connecting different network segments |
| Route | Path to a destination network |
| Port | Number identifying a service on a host (0-65535) |
| Socket | Network endpoint (IP + port) |
| VLAN | Virtual LAN — logical network segmentation |
| NAT | Network Address Translation — translating IP addresses |
| DNS | Domain Name System — resolving domain names to IPs |

## Security Terms

| Term | Definition |
|------|------------|
| Permission | Access rights (read, write, execute) for user/group/others |
| ACL | Access Control List — extended permission system |
| UID | User ID — numeric identifier for a user |
| GID | Group ID — numeric identifier for a group |
| Sudo | Privilege escalation to root |
| PAM | Pluggable Authentication Modules — extensible authentication |
| SELinux | Security-Enhanced Linux — mandatory access control |
| AppArmor | Mandatory access control (alternative to SELinux) |
| SSH Key | Cryptographic key pair for SSH authentication |
| Certificate | X.509 certificate for TLS/SSL authentication |
| Firewall | Network security system filtering traffic |

## Storage Terms

| Term | Definition |
|------|------------|
| Partition | Defined section of a disk |
| LVM | Logical Volume Manager — flexible storage allocation |
| PV | Physical Volume — disk or partition in LVM |
| VG | Volume Group — pool of PVs in LVM |
| LV | Logical Volume — formatted volume from VG |
| PE | Physical Extent — smallest allocatable unit in LVM |
| RAID | Redundant Array of Independent Disks — data redundancy |
| SMART | Self-Monitoring, Analysis and Reporting Technology — disk health |
| LUKS | Linux Unified Key Setup — disk encryption |
| Mount Options | Configuration options for filesystem mounting |

## Performance Terms

| Term | Definition |
|------|------------|
| Load Average | Average number of processes waiting for CPU (1, 5, 15 min) |
| Context Switch | CPU switching between processes |
| I/O Wait | CPU time waiting for I/O operations |
| Swap | Disk space used as virtual memory |
| Page Cache | RAM caching of disk reads |
| Buffer Cache | RAM caching of disk writes |
| Throughput | Amount of data processed per unit time |
| Latency | Time between request and response |
| Contention | Multiple processes competing for the same resource |
| Governor | CPU frequency scaling policy (ondemand, performance, powersave) |

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial terminology glossary |