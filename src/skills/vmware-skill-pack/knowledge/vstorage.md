# VMware vSphere — Storage Management Knowledge

## Storage Stack

```
Virtual Machines
    ↓
Virtual Disks (VMDK, RDM)
    ↓
Datastore (VMFS, NFS, vSAN)
    ↓
Storage Protocols (FC, iSCSI, NFS, vSAN)
    ↓
Storage Arrays (SAN, NAS, Direct-Attach)
```

## Storage Technologies

### VMFS (VMware File System)
| Version | Max Size | Max VMs | Block Size | Features |
|---------|----------|---------|------------|----------|
| VMFS-3 | 64 TB | 2048 | 1, 8, 64 MB | Legacy |
| VMFS-5 | 64 TB | 2048 | 1 MB | Transition |
| VMFS-6 | 64 TB | 2048 | 1 MB | Concurrency lock, sparse |

### VMFS-6 Features
- **Concurrency Lock**: Multiple hosts can access same datastore
- **Sparse Provisioning**: Thin provisioning at filesystem level
- **Online Resize**: Extend VMFS without downtime
- **Block Size**: 1 MB default (cannot change after creation)

### NFS (Network File System)
| Version | Port | Protocol | Features |
|---------|------|----------|----------|
| NFS 3 | 2049 | TCP/UDP | Legacy support |
| NFS 4.1 | 2049 | TCP | Sessions, multicast, vSAN |

### vSAN (vSphere Storage Area Network)
| Component | Description |
|-----------|-------------|
| Cache Tier | SSD for read/write caching |
| Capacity Tier | HDD or SSD for data storage |
| Disk Group | 1 flash + 5 HDD (hybrid) or 1 flash + 1 SSD (all-flash) |
| Stretched Cluster | 2 sites + witness for DR |

### vSAN Configuration
| Setting | Hybrid | All-Flash |
|---------|--------|-----------|
| Cache Size | 10% of total RAM or 32 GB | 5% of total RAM or 16 GB |
| Cache to Capacity | 1:4 | 1:7 |
| FT Level | 1 | 1 or 2 |
| Read Cache | Enabled | Enabled |
| Write Buffer | 5% of cache | Enabled |

## Storage Operations

### Datastore Management
```bash
# List all datastores
esxcli storage filesystem list

# Create VMFS datastore
esxcli storage vmfs extent -a -s 512 -d naa.<uuid>

# Extend VMFS datastore
esxcli storage vmfs extend -d <datastore-name> -e naa.<uuid>

# Unmount datastore
esxcli storage vmfs unmount -d <datastore-name>

# Delete VMFS datastore
esxcli storage vmfs destroy -d <datastore-name>
```

### NFS Datastore
```bash
# Add NFS datastore
esxcli storage nfs add \
    --volume-name nfs-datastore \
    --host 192.168.1.50 \
    --share /export/datastore \
    --readonly false

# Mount NFS datastore
esxcli storage nfs mount \
    --volume-name nfs-datastore \
    --host 192.168.1.50 \
    --share /export/datastore

# Remove NFS datastore
esxcli storage nfs remove --volume-name nfs-datastore
```

### iSCSI Storage
```bash
# Enable software iSCSI adapter
esxcli iscsi software adapter set --enabled true

# Add iSCSI target
esxcli iscsi adapter discovery isns add --address <isns-server>

# Rescan HBAs
esxcli storage core adapter rescan --all

# List iSCSI targets
esxcli iscsi adapter target list --adapter <adapter>
```

### FC (Fibre Channel) Storage
```bash
# List FC HBAs
esxcli storage fc adapter list

# Rescan HBAs
esxcli storage core adapter rescan --all

# List devices
esxcli storage core device list
```

## Storage Multipathing

### Path Selection Policies (PSP)
| Policy | Description | Use Case |
|--------|-------------|----------|
| VMW_PSP_RR | Round Robin | Active-active arrays |
| VMW_PSP_RR_IO_LIMIT | Round Robin with I/O limit | Active-active with limits |
| VMW_PSP_FIXED | Fixed (preferred path) | Active-passive arrays |
| VMW_PSP_OFF | Fixed (no path selection) | Special cases |

### Multipathing Configuration
```bash
# View current policy
esxcli storage nmp device list --device <device>

# Set policy
esxcli storage nmp device set --device <device> --policy VMW_PSP_RR

# View paths
esxcli storage nmp device path list --device <device>

# Set path state
esxcli storage nmp device path set --state active --path <path-name>
```

## Storage Performance

### Storage Metrics
| Metric | Description | Good | Warning | Critical |
|--------|-------------|------|---------|----------|
| Latency | Time for I/O to complete | < 10ms | 10-20ms | > 20ms |
| IOPS | I/O operations per second | Depends on array | — | — |
| Throughput | Data transfer rate | Depends on array | — | — |
| Queue Depth | Pending I/Os | < 32 | 32-64 | > 64 |

### Storage Performance Monitoring
```bash
# Check storage performance
esxtop -b -n 1 | grep -i "DISK"

# Check device latency
esxcli storage core device stats get --device <device>

# Check datastore free space
esxcli storage filesystem list
```

### Storage Optimization
| Optimization | Description | Impact |
|-------------|-------------|--------|
| Thin provisioning | Allocate on demand | Space savings |
| Eager zero | Pre-allocate fully | Better performance |
| SSD caching | Use flash for cache | Read performance |
| Storage DRS | Auto-balance workloads | Balanced I/O |
| Storage I/O Control | Prioritize critical VMs | QoS |

## Storage Troubleshooting

### Common Storage Issues

| Issue | Symptoms | Root Cause | Resolution |
|-------|----------|------------|------------|
| Datastore full | VMs cannot write | No space left | Delete files or expand |
| LUN lost | Datastore inaccessible | SAN connectivity issue | Check fabric, rescan |
| Path failure | Single path active | Multipathing issue | Check cable, HBA |
| Snapshot full | Snapshot fails | No space for delta | Delete old snapshots |
| NFS timeout | Datastore inaccessible | Network/storage issue | Check network, retry |
| VMFS corruption | Datastore inaccessible | Filesystem error | Repair or recreate |
| vSAN failure | Capacity reduced | Disk group failure | Replace disk, rebuild |

### Storage Diagnostic Procedures

#### 1. Datastore Issues
```bash
# Check datastore capacity
esxcli storage filesystem list

# Check datastore health
esxcli storage vmfs extent list

# Check for errors
dmesg | grep -i "vmfs\|nfs\|iscsi"

# Check VM files on datastore
ls -la /vmfs/volumes/<datastore-name>/
```

#### 2. Path Issues
```bash
# Check path status
esxcli storage nmp device path list

# Check active paths
esxcli storage nmp device list | grep -i "state"

# Rescan storage
esxcli storage core adapter rescan --all

# Check HBAs
esxcli storage san iscsi adapter list
esxcli storage san fc adapter list
```

#### 3. Performance Issues
```bash
# Check storage latency
esxtop -b -n 10 | grep -i "CONC\|DLMIT"

# Check IOPS
esxtop -b -n 10 | grep -i "IOPS"

# Check queue depth
esxtop -b -n 10 | grep -i "Q"

# Check device stats
esxcli storage core device stats get --device <device>
```

### Storage Repair Procedures

#### VMFS Repair
```bash
# Check VMFS consistency
esxcli storage vmfs snapshot list

# Repair VMFS (emergency only)
# Note: May cause data loss — consult VMware Support first
esxcli storage vmfs recover --volume-uuid <uuid>
```

#### Snapshot Consolidation
```bash
# Check snapshot state
esxcli storage vmfs snapshot list

# Consolidate snapshots
# Via vSphere Client: VM → Snapshots → Consolidate

# Manual consolidation (ESXi level)
# Note: Use vSphere Client preferred; CLI risky
```

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial storage management knowledge |