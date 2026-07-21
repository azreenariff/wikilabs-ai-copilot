# VMware vSphere Engineering — Knowledge Base

## Cluster Architecture

### vSphere Cluster Design
A vSphere cluster provides centralized management and resource sharing for ESXi hosts.

```
Datacenter
└── Cluster (Cluster01)
    ├── HA: Enabled (vm restart priority: High)
    ├── DRS: Fully Automated
    ├── EVC: Intel Haswell
    ├── Hosts (3x)
    │   ├── esxi01 (128GB RAM, 32 vCPU)
    │   ├── esxi02 (128GB RAM, 32 vCPU)
    │   └── esxi03 (128GB RAM, 32 vCPU)
    ├── Datastores (2x)
    │   ├── production-ds01 (100TB VMFS-6)
    │   └── backup-ds01 (50TB VMFS-6)
    └── Networks (2x)
        ├── Management vSwitch (vmk0)
        └── vMotion vSwitch (vmk1)
```

### Cluster Configuration Requirements
| Setting | Recommended Value | Notes |
|---------|-------------------|-------|
| HA | Enabled | Required for production |
| DRS | Fully Automated | For large clusters |
| Admission Control | 20% reserved | At least 1 host for failover |
| EVC Mode | Match oldest host CPU | Ensures vMotion compatibility |
| Storage DRS | Enabled (if vSAN) | Balances storage workloads |

### Cluster Sizing
| Cluster Size | Max Hosts | Max VMs | Notes |
|-------------|-----------|---------|-------|
| Small | 8 | 64 | < 500 VMs |
| Medium | 32 | 256 | < 2000 VMs |
| Large | 64 | 512 | < 4000 VMs |

## ESXi Management

### ESXi Service Management
| Service | Description | Port | Restart Command |
|---------|-------------|------|-----------------|
| hostd | ESXi management daemon | 443 | `services.sh restart` |
| vpxa | vCenter agent on host | 902 | `services.sh restart` |
| vmkernel | Hypervisor kernel | N/A | Requires reboot |
| SSH | Remote access | 22 | `services.sh restart` |

### ESXi Host Maintenance
1. **Maintenance Mode**: `vim-cmd hostsvc/maintenance_mode_enter`
2. **Check VMs on host**: `vim-cmd vmsvc/getallvms`
3. **vMotion VMs off**: Use vSphere Client or PowerCLI
4. **Enter maintenance mode**: `vim-cmd hostsvc/maintenance_mode_enter`
5. **Perform maintenance**
6. **Exit maintenance mode**: `vim-cmd hostsvc/maintenance_mode_exit`

### ESXi Diagnostic Access
- **DCUI**: Direct Console User Interface (F2 for configuration)
- **SSH**: Enable via vSphere Client or DCUI
- **ESXi Shell**: Local CLI via console (disabled by default)
- **Support Mode**: `supportMode` for VMware support access

## vCenter Administration

### VCSA Architecture
```
VCSA Appliance
├── VMware vCenter Server (vpxd)
├── VMware ESXi Host Agent (vpxa)
├── VMware Certificate Authority (VMCA)
├── VMware Identity Manager (optional)
├── VMware Platform Services Controller (PSC)
├── Embedded Database (PostgreSQL)
└── HTML5 Web Client
```

### VCSA Service Management
| Command | Purpose |
|---------|---------|
| `service-control --status` | List all services and status |
| `service-control --stop --all` | Stop all services |
| `service-control --start --all` | Start all services |
| `service-control --restart <service>` | Restart specific service |
| `shell` | Access VCSA shell |

### VCSA Certificate Management
```bash
# View certificate status
certificate-manager --show

# Renew certificates
certificate-manager --renew

# Custom certificates
certificate-manager --import --domain sso --certificate /path/to/cert --private-key /path/to/key
```

### VCSA Backup and Restore
- **API-based backup**: `backup.vcsa` endpoint
- **Scheduled backup**: vSphere Client → Administration → Backup
- **Restore**: VCSA deployment appliance with restore option

## VM Management

### VM Configuration Standards
| Setting | Recommendation | Notes |
|---------|---------------|-------|
| Hardware Version | Latest supported | Ensure ESXi compatibility |
| CPU | Match physical cores | Avoid >4:1 overcommit |
| RAM | Allocate as needed | Leave room for ballooning |
| Disk | Thin provision for dev, Thick for prod | VMFS-6 supports both |
| VMtools | Always updated | Required for snapshots, HA, vMotion |
| Snapshots | Max 7 days | Performance impact with each |

### VM Lifecycle Management
1. **Create**: Via vSphere Client, CLI, or PowerCLI
2. **Configure**: CPU, RAM, disks, network adapters
3. **Deploy**: Power on and install guest OS
4. **Monitor**: Performance metrics and health
5. **Maintain**: Updates, patches, backups
6. **Decommission**: Snapshot cleanup, disk deletion, registration removal

### VM Troubleshooting Checklist
- [ ] VMtools installed and running
- [ ] Hardware version compatible with ESXi
- [ ] No excessive snapshots (>3)
- [ ] CPU ready time <5%
- [ ] Memory ballooning/swapping minimal
- [ ] Disk latency <20ms
- [ ] Network connectivity established
- [ ] Power state correct
- [ ] Resource allocation appropriate

## vStorage

### VMFS-6 Storage
- **Max VMFS size**: 64 TB
- **Max block size**: 1 MB, 8 MB, 64 MB
- **Max VMs per datastore**: 2048
- **Features**: Concurrency lock, online resize, sparse provisioning

### vSAN Storage
| Setting | Recommended | Notes |
|---------|-------------|-------|
| Cache Size | 10% of total RAM | For read caching |
| Disk Groups | 1 per host | 1 flash + 5 HDD or all flash |
| FT | 1 | Fault tolerance level |
| Capacity | 3x for FT-1 | 1 for data + 2 for replicas |

### Storage Operations
| Operation | Command/Method | Risk |
|-----------|---------------|------|
| Extend VMFS | LUN resize → `esxcli storage vmfs extend` | Medium |
| Add datastore | `esxcli storage nfs add` or LUN mapping | Medium |
| Delete datastore | `esxcli storage vmfs unmount` | High |
| Storage vMotion | vSphere Client → Move → Change datastore | Low |
| Snapshot consolidate | vSphere Client → VM → Snapshots → Consolidate | Medium |

## vNetworking

### Standard vSwitch (vSS)
- Per-host configuration
- Up to 4096 ports
- VLAN support (0-4094)
- Teaming and failover
- Security policies

### Distributed vSwitch (vDS)
- Centralized management
- Up to 64000 ports per switch
- Private VLAN support
- NetFlow/sFlow monitoring
- NIO (Network I/O Control)

### VMkernel Adapters
| Adapter | Purpose | Port Group |
|---------|---------|------------|
| vmk0 | Management | Management Network |
| vmk1 | vMotion | vMotion Network |
| vmk2 | iSCSI Storage | iSCSI Storage Network |
| vmk3 | NFS Storage | NFS Storage Network |
| vmk4 | vSAN | vSAN Network |

### vNetwork Troubleshooting
- **Port Group misconfiguration**: Check vSphere Client → Network → Port Groups
- **VLAN mismatch**: Verify VLAN ID on vSwitch and physical switch
- **Teaming issues**: Check active uplinks and failover order
- **vMotion network**: Verify vmk1 configuration on both hosts

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial knowledge base |