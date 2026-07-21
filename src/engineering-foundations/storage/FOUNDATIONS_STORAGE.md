# Storage Engineering Foundation

## Architecture

### Storage Hierarchy

| Layer | Technology | Description |
|-------|-----------|-------------|
| Physical | HDD, SSD, NVMe, SSD | Actual storage media |
| Block | RAID, LVM, SAN | Block-level storage abstraction |
| Filesystem | ext4, XFS, NTFS, Btrfs | File and directory management |
| Network | NFS, SMB, iSCSI | Remote storage access |
| Application | Databases, VMs, Containers | Storage consumers |

### Disk Types

| Type | Speed | Use Case |
|------|-------|----------|
| HDD (7200 RPM) | Medium | Cold storage, backups, NAS |
| HDD (10/15K RPM) | Medium-High | Enterprise server storage |
| SSD (SATA) | High | General-purpose, desktop |
| SSD (NVMe) | Very High | Databases, VMs, performance-critical |
| NVMe-oF | Highest | Datacenter, storage arrays |

---

## Core Concepts

### RAID Levels

| Level | Min Disks | Description | Pros | Cons |
|-------|-----------|-------------|------|------|
| 0 | 2+ | Striped | Max speed, max capacity | No redundancy |
| 1 | 2 | Mirrored | Full redundancy | 50% capacity loss |
| 5 | 3+ | Striping + parity | Good balance of speed/capacity/redundancy | Write penalty, rebuild risk |
| 6 | 4+ | Dual parity | Can survive 2 disk failures | Higher write penalty |
| 10 | 4+ | Mirror + stripe | Best speed + redundancy | 50% capacity loss |
| 50 | 6+ | Stripe + mirror + parity | High speed + redundancy | Complex, expensive |
| 0+1 | 4+ | Stripe + mirror | Good speed | No redundancy on stripe level |

### LVM (Logical Volume Manager)

**Components**
| Component | Description |
|-----------|-------------|
| PV (Physical Volume) | Actual disk or partition |
| VG (Volume Group) | Pool of PVs |
| LV (Logical Volume) | Formatted filesystem from VG |

**LVM Workflow**
```
PV → VG → LV → Filesystem → Mount Point
```

**Common Operations**
- `pvcreate`, `vgcreate`, `lvcreate` — Create
- `vgextend`, `lvextend` — Extend
- `vgreduce`, `lvreduce` — Shrink (risky)
- `pvs`, `vgs`, `lvs` — List
- `df -h` — Check usage

### Network Storage

| Protocol | Layer | Use Case | Port |
|----------|-------|----------|------|
| NFS | File-level | Linux/Unix file sharing | 2049 |
| SMB/CIFS | File-level | Windows file sharing | 445 |
| iSCSI | Block-level | Linux/Windows block storage | 3260 |
| FC (Fibre Channel) | Block-level | Enterprise SAN | N/A (dedicated fabric) |
| NVMe-oF | Block-level | High-performance remote storage | 4420 |

### Filesystems

| FS | Linux | Windows | Features |
|----|-------|---------|----------|
| ext4 | ✓ | | Journaling, extensible |
| XFS | ✓ | | High performance, large files |
| Btrfs | ✓ | | Copy-on-write, snapshots, RAID |
| ZFS | ✓ | | Copy-on-write, snapshots, RAID-Z, integrity |
| NTFS | | ✓ | ACLs, encryption, compression |
| ReFS | | ✓ | Integrity, self-healing |

---

## Common Components

### Disk Utilities
| Tool | Purpose |
|------|---------|
| fdisk, parted | Partition management |
| lsblk, blkid | List disks and block devices |
| smartctl | SMART health monitoring |
| dd | Disk-to-disk copy, image creation |
| mkfs.ext4, mkfs.xfs | Create filesystems |
| tune2fs | Tune filesystem parameters |

### Storage Monitoring
| Tool | Purpose |
|------|---------|
| df -h | Disk space usage |
| du -sh | Directory size |
| iostat -x 1 | I/O statistics |
| iotop | Real-time I/O by process |
| smartctl -a | Disk health |
| zpool status | ZFS pool health |

---

## Common Failures

### Disk Failures
| Symptom | Possible Cause |
|---------|----------------|
| Disk not found | Loose cable, controller failure, disk failure |
| Read/write errors | Bad sectors, failing disk, filesystem corruption |
| SMART warnings | Predictive disk failure, replace ASAP |
| Disk full | No quota management, log accumulation, temp files |

### RAID Failures
| Symptom | Possible Cause |
|---------|----------------|
| RAID degraded | One or more disks failed, hot spare activated |
| RAID failed | Multiple disk failures, controller failure |
| RAID rebuild slow | Large disks, concurrent I/O load |
| RAID rebuild fails | Second disk also failing during rebuild |

### Filesystem Issues
| Symptom | Possible Cause |
|---------|----------------|
| Filesystem not mounting | Corrupted superblock, fstab error, missing device |
| Slow I/O | Disk nearing failure, heavy fragmentation (HDD) |
| Inode exhaustion | Many small files exhausting inode table |
| Permission issues | Incorrect ownership, restrictive permissions |

### Network Storage Issues
| Symptom | Possible Cause |
|---------|----------------|
| NFS mount fails | NFS server down, firewall blocking, export not configured |
| SMB share inaccessible | Authentication failure, network issue, service down |
| iSCSI target unreachable | Network partition, target service down, CHAP auth failure |

---

## Troubleshooting Philosophy

### Storage Diagnostic Flow

```
Storage issue
  │
  ├─→ Disk visible? (lsblk, fdisk -l)
  │   └─→ No → Check cables, controllers, BIOS/UEFI
  │
  ├─→ SMART health? (smartctl -a /dev/sdX)
  │   └─→ Bad → Backup immediately, replace disk
  │
  ├─→ Mounted? (mount, df -h)
  │   └─→ No → Check fstab, mount command, device name
  │
  ├─→ Space available? (df -h, du -sh)
  │   └─→ No → Clean files, extend volume, add disk
  │
  ├─→ I/O performance? (iostat, iotop)
  │   └─→ Slow → Check for bad disk, heavy load, fragmentation
  │
  └─→ Filesystem integrity? (fsck, xfs_repair)
      └─→ Corrupted → Run repair (unmount first!), restore from backup
```

---

## Best Practices

### Capacity Management
- **Monitor usage trends** — Alert at 70%, 80%, 90% thresholds
- **Plan growth** — Add capacity before running out
- **Clean regularly** — Remove old logs, cache, temp files
- **Use quotas** — Prevent runaway disk usage
- **Archive old data** — Move infrequently accessed data to cold storage

### Performance
- **SSD for active data** — HDD for cold/archive
- **RAID 10 for databases** — Best speed + redundancy
- **RAID 6 for file servers** — Balance of redundancy + capacity
- **Monitor I/O latency** — Alert on high latency
- **Avoid fragmentation** — Trim SSDs, defrag HDDs (infrequently)

### Reliability
- **3-2-1 backup rule** — 3 copies, 2 media types, 1 off-site
- **Test restores** — Verify backup integrity regularly
- **Monitor SMART** — Alert on predictive failure indicators
- **Replace before failure** — Act on SMART warnings immediately
- **Document storage topology** — RAID level, capacity, usage

### Security
- **Encrypt sensitive data** — LUKS, BitLocker, ZFS encryption
- **Restrict access** — Proper ownership and permissions
- **Audit changes** — Log mount/unmount events
- **Secure network storage** — NFSv4 with Kerberos, SMB with NTLMv2/NTLM
- **Verify backups** — Checksums for backup integrity

---

## References

- [Red Hat Storage Documentation](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/)
- [ZFS Documentation](https://openzfs.github.io/openzfs-docs/)
- [LVM Documentation](https://man.archlinux.org/man/lvm.8.en)
- [RAID Levels Comparison](https://en.wikipedia.org/wiki/Standard_RAID_levels)
- [Filesystem Comparison](https://en.wikipedia.org/wiki/Comparison_of_file_systems)

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial Storage Engineering Foundation |