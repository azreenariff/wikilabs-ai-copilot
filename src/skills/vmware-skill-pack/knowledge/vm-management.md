# VMware vSphere — Virtual Machine Management Knowledge

## VM Architecture

### VM Components
```
Virtual Machine
├── VMX (Configuration File)
├── VMDK (Virtual Disk Files)
├── NVRAM (BIOS Settings)
├── Snapshots (Delta Files)
├── Logs (.log files)
├── Swap File (.vswp)
└── VMtools (Guest Agent)
```

### VM Hardware Versions
| vSphere Version | Latest Hardware Version | ESXi Compatibility |
|----------------|------------------------|-------------------|
| vSphere 8.0 | v21 | ESXi 8.0+ |
| vSphere 7.0 | v19 | ESXi 7.0+ |
| vSphere 6.7 | v15 | ESXi 6.7+ |

### Virtual Hardware Components
| Component | Description | Default | Notes |
|-----------|-------------|---------|-------|
| vCPU | Virtual processor | 1 | Max 320 per VM |
| vRAM | Virtual memory | 1 GB | Depends on guest OS |
| Virtual Disk | Storage for VM | 40 GB | Thin or thick provisioned |
| CD/DVD Drive | Virtual optical drive | None | ISO file attachment |
| Network Adapter | Virtual NIC | vmxnet3 | VMware paravirtual driver |
| SCSI Controller | Disk controller | LSI Logic SAS | Para-virtual recommended |
| Sound Card | Audio device | None | Rarely used |
| Serial/Parallel | Legacy ports | None | Rarely used |

## VM Lifecycle

### VM Creation Process
1. **Specify VM name and location** — Name, folder, datacenter
2. **Select compute resource** — Host, cluster, or resource pool
3. **Choose compatibility** — Hardware version, guest OS
4. **Configure hardware** — CPU, RAM, disks, network
5. **Customize** — Guest OS customization specification
6. **Deploy** — Power on and install guest OS
7. **Install VMtools** — Always install after deployment
8. **Configure snapshots** — Set retention policy

### VM State Transitions
```
Powered Off → Power On → Running → Suspended → Powered Off
    ↑                                         ↓
    └──── Snapshot Restore ←───── Snapshot Create ──┘
```

### VM Operations
| Operation | Description | Downtime | Risk |
|-----------|-------------|----------|------|
| Power On | Start VM | Brief (seconds) | Low |
| Power Off | Stop VM | Brief (seconds) | Low |
| Reset | Force restart | Brief (seconds) | Medium (data loss) |
| Suspend | Save state | Brief (seconds) | Low |
| Resume | Restore state | Brief (seconds) | Low |
| Migrate (vMotion) | Live move to host | Zero | Low |
| Storage vMotion | Live move storage | Zero | Low |
| Clone | Create copy | Brief | Low |
| Deploy from Template | Rapid deployment | Brief | Low |

## VM Configuration

### Resource Allocation
| Setting | Description | Best Practice |
|---------|-------------|---------------|
| Reservation | Guaranteed resources | Set for critical VMs |
| Limit | Maximum resources | Leave unlimited for most |
| Shares | Priority during contention | Higher for critical VMs |

### Resource Allocation Example
```yaml
web-server-vm:
  cpu:
    reservation: 2 GHz
    limit: unlimited
    shares: high
  memory:
    reservation: 4 GB
    limit: unlimited
    shares: high
  disk:
    provision: thin
    size: 100 GB
    eager_zero: false
```

### VM Network Configuration
| Adapter Type | Driver | Performance | Notes |
|-------------|--------|-------------|-------|
| E1000 | Emulated Intel | Moderate | Compatibility with older OS |
| E1000E | Emulated Intel Xeon | Moderate | Modern emulated adapter |
| vmxnet2 | VMware para-virtual | Good | Old para-virtual driver |
| vmxnet3 | VMware para-virtual | Best | Requires VMtools |

### VM Disk Configuration
| Provisioning | Description | Performance | Notes |
|-------------|-------------|-------------|-------|
| Thin | Allocates on demand | Slightly slower | Saves space |
| Thick Zeroed | Pre-allocates, zeroed | Fastest | Recommended for prod |
| Thick Eager | Pre-allocates, eagerly zeroed | Fastest | Required for some features |

## VM Troubleshooting

### VM Performance Issues

#### CPU Issues
| Symptom | Cause | Solution |
|---------|-------|----------|
| High CPU ready | vCPU contention | Reduce vCPUs or migrate |
| High CPU usage | Application bottleneck | Optimize application |
| CPU limit reached | Limit set too low | Increase or remove limit |
| CPU reservation 100% | Reservation exceeded | Adjust reservations |

#### Memory Issues
| Symptom | Cause | Solution |
|---------|-------|----------|
| Memory ballooning | Host memory pressure | Add RAM or migrate |
| Memory swapping | Host memory exhausted | Add RAM or migrate |
| Memory reservation 100% | Reservation exceeded | Adjust reservations |
| OOM killer active | Host out of memory | Emergency: migrate or shutdown |

#### Disk Issues
| Symptom | Cause | Solution |
|---------|-------|----------|
| High disk latency | Storage overload | Migrate to faster storage |
| Disk full | Guest OS disk full | Clean up or expand disk |
| Disk IOPS limit | Storage IOPS capped | Expand or optimize |
| Snapshot performance | Too many snapshots | Consolidate or delete snapshots |

### VM Troubleshooting Checklist
- [ ] VMtools installed and up-to-date
- [ ] CPU ready time < 5%
- [ ] No memory ballooning or swapping
- [ ] Disk latency < 20ms
- [ ] No excessive snapshots (> 3)
- [ ] Network connectivity established
- [ ] Guest OS patched and updated
- [ ] Proper resource allocation set
- [ ] Not in maintenance mode
- [ ] HA/DRS configured correctly

### VM Diagnostic Commands

#### ESXi Level
```bash
# List all VMs
esxcli vm process list

# Get VM summary
esxcli vm process list | grep -A 5 <vm-name>

# Check CPU ready time
esxtop -b -n 1 | grep <vm-name>

# Check memory usage
esxtop -b -n 1 | grep -i <vm-name>

# Check disk usage
esxcli storage vmfs snapshot list
```

#### Guest OS Level
```bash
# Check disk space
df -h

# Check memory usage
free -m

# Check CPU usage
top -bn1 | head -20

# Check VMtools status
vmware-toolbox-cmd -v

# Check network interfaces
ip addr
```

## VM Operations

### Snapshot Management
| Operation | Description | Risk |
|-----------|-------------|------|
| Create | Capture current state | Low |
| Revert | Return to snapshot | Medium (data loss) |
| Delete | Remove snapshot | Low |
| Consolidate | Merge delta files | Medium |
| Export | Export VM with snapshots | Low |

### Snapshot Best Practices
- **Retention**: Max 7 days, auto-delete after
- **Frequency**: Only when needed, not for backups
- **Testing**: Always test revert before using in production
- **Monitoring**: Alert on snapshot chain > 3

### VM Cloning
| Type | Description | Use Case |
|------|-------------|----------|
| Full Clone | Independent copy | New VM deployment |
| Linked Clone | Depends on parent | Testing, dev environments |
| Template | Read-only base | Standardized deployment |

### VM Templates
```bash
# Create template from VM
# 1. vSphere Client → VM → Templates → Convert to Template

# Deploy VM from template
# 1. vSphere Client → Right-click template → Deploy Template

# Customize template
# 1. Clone to VM
# 2. Customize (hostname, IP, software)
# 3. Convert back to template
```

## VM Security

### VM Security Settings
| Setting | Default | Recommendation |
|---------|---------|---------------|
| Encrypt at rest | Disabled | Enable for sensitive data |
| Secure boot | Disabled | Enable for security |
| TPM 2.0 | Disabled | Enable for BitLocker |
| Nested virtualization | Disabled | Enable only if needed |

### VM Isolation
| Setting | Purpose | Location |
|---------|---------|----------|
| Network ACL | Control VM network access | vSwitch/Port Group |
| Storage permissions | Control datastore access | vSphere Client |
| Resource controls | Limit VM resource usage | Resource Pool |
| Lockdown mode | Restrict host access | Host Settings |

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial VM management knowledge |