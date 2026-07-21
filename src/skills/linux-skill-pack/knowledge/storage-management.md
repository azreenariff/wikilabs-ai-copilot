# Linux Knowledge — Storage Management

## Storage Stack

```
Application
    ↓
Filesystem (ext4, XFS, Btrfs, ZFS)
    ↓
Logical Volume Manager (LVM)
    ↓
Block Devices (/dev/sda, /dev/nvme0n1)
    ↓
Physical Disks (HDD, SSD, NVMe)
```

## Disk Types

| Type | Interface | Speed | Use Case |
|------|-----------|-------|----------|
| HDD | SATA, SAS | 100-250 MB/s | Bulk storage, backups |
| SSD | SATA | 500 MB/s | General purpose |
| NVMe | PCIe | 2-7 GB/s | Databases, VMs |
| NVMe-oF | RoCE, TCP | 10+ GB/s | Storage arrays |

## Filesystems

### ext4

```bash
# Create filesystem
mkfs.ext4 /dev/sdb1

# Mount
mount /dev/sdb1 /mnt/data

# Options
mount -o noatime,nodiratime /dev/sdb1 /mnt/data

# Check
fsck.ext4 -n /dev/sdb1

# Resize
resize2fs /dev/sdb1 100G
```

### XFS

```bash
# Create filesystem
mkfs.xfs /dev/sdb1

# Mount
mount /dev/sdb1 /mnt/data

# Check
xfs_repair -n /dev/sdb1

# Grow
xfs_growfs /mnt/data
```

### Btrfs

```bash
# Create
mkfs.btrfs /dev/sdb1

# Mount with features
mount -o compress=zstd /dev/sdb1 /mnt/data

# Snapshots
btrfs subvolume create /mnt/data/snapshot
btrfs subvolume snapshot /mnt/data /mnt/data/snap-before-update
```

### ZFS

```bash
# Pool creation
zpool create mypool /dev/sdb /dev/sdc

# Filesystem
zfs create mypool/data

# Features
zfs set compression=on mypool/data
zfs set reservation=100G mypool/data
```

## LVM

### Concepts

| Component | Description |
|-----------|-------------|
| PV (Physical Volume) | Physical disk or partition |
| VG (Volume Group) | Pool of PVs |
| LV (Logical Volume) | Formatted LV from VG |
| PE (Physical Extent) | Smallest allocatable unit in PV |

### LVM Commands

```bash
# Create
pvcreate /dev/sdb
vgcreate vg0 /dev/sdb
lvcreate -L 50G -n lv0 vg0
mkfs.xfs /dev/vg0/lv0
mount /dev/vg0/lv0 /mnt/data

# Extend
lvextend -L +100G /dev/vg0/lv0
xfs_growfs /mnt/data

# Reduce
lvreduce -L -50G /dev/vg0/lv0
resize2fs /dev/vg0/lv0

# Remove
umount /mnt/data
lvremove /dev/vg0/lv0
vgremove vg0
pvremove /dev/sdb
```

## Storage Monitoring

```bash
# Disk usage
df -h
df -i

# Directory sizes
du -sh /var/log/*
du -sh /* 2>/dev/null | sort -rh | head -10

# Large files
find / -xdev -type f -size +100M 2>/dev/null

# I/O stats
iostat -x 1 3
iotop -o

# SMART health
smartctl -a /dev/sda

# RAID status
cat /proc/mdstat
mdadm --detail /dev/md0
```

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial storage management knowledge |