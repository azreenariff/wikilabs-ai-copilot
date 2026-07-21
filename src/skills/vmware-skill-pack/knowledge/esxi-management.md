# VMware vSphere — ESXi Management Knowledge

## ESXi Architecture

### ESXi Hypervisor Layers
```
VMs (Guest OS)
├── Virtual Hardware (vCPU, vRAM, vDisk, vNIC)
├── Virtual Machine Monitor (VMM)
├── VMkernel (Core)
│   ├── Hardware Abstraction Layer
│   ├── Memory Manager
│   ├── Scheduler
│   ├── Networking Stack
│   └── Storage Stack
├── Management Agents (hostd, vpxa)
└── Direct Hardware Access
```

### ESXi Components
| Component | Description | Service |
|-----------|-------------|---------|
| VMkernel | Core hypervisor | N/A (kernel) |
| hostd | Host management daemon | `services.sh restart` |
| vpxa | vCenter agent | `services.sh restart` |
| rbd | Remote syslog | syslog |
| vmauditd | Audit daemon | auditd |
| sfcbd | CIM provider | sfcbd |
| sshd | SSH daemon | `services.sh restart` |

## ESXi Host Management

### Host State Transitions
```
Connected → Maintenance Mode → Disconnected → Not Responding → Not Responding → Connected
```

### State Management Commands
```bash
# Enter maintenance mode
vim-cmd hostsvc/maintenance_mode_enter

# Exit maintenance mode
vim-cmd hostsvc/maintenance_mode_exit

# Check if in maintenance mode
vim-cmd hostsvc/net/host Nat get
```

### ESXi Service Management
```bash
# Start all services
services.sh start

# Stop all services
services.sh stop

# Restart all services
services.sh restart

# Check service status
esxcli system syslog config get

# Enable SSH
vim-cmd hostsvc/enable_ssh

# Disable SSH
vim-cmd hostsvc/disable_ssh
```

### ESXi System Information
```bash
# Get host hostname
esxcli system hostname get

# Get hardware info
esxcli hardware platform get

# Get uptime
esxcli system uptime get

# Get version info
esxcli system version get

# Get license info
esxcli system license get
```

## ESXi Configuration

### Network Configuration
```bash
# List all network interfaces
esxcli network ip interface list

# Add VMkernel adapter
esxcli network ip interface add --interface-name vmk2 --port-group "Storage Network" --ip 10.0.1.10 --netmask 255.255.255.0

# Set IPv4
esxcli network ip interface ipv4 set --interface-name vmk2 --ipv4 10.0.1.10 --netmask 255.255.255.0 --type static

# Set default gateway
esxcli network ip route ipv4 add --gateway 10.0.1.1 --network 0.0.0.0/0
```

### Storage Configuration
```bash
# List all devices
esxcli storage core device list

# Rescan HBAs
esxcli storage core adapter rescan --all

# Add iSCSI software adapter
esxcli iscsi software adapter set --enabled true

# Discover iSCSI targets
esxcli iscsi adapter discovery isns add --address <isns-server>

# Set multipathing policy
esxcli storage nmp device set --device <device-id> --policy VMW_PSP_RR
```

### Advanced Settings
```bash
# List advanced settings
esxcli system settings advanced list

# Get specific setting
esxcli system settings advanced list --option "/Userdata/HAHeartBeatDatastoreSelection"

# Set advanced setting
esxcli system settings advanced set --option "/Userdata/HAHeartBeatDatastoreSelection" --int-value 1
```

## ESXi Troubleshooting

### Common ESXi Issues

| Issue | Symptoms | Resolution |
|-------|----------|------------|
| Purple Screen of Death (PSOD) | ESXi crash with purple screen | Analyze crash dump, check drivers |
| Host Disconnected | vCenter shows red status | Check network, restart hostd |
| Service Failure | hostd or vpxa not running | Restart services.sh |
| Storage Unavailable | Datastore shows red | Check LUN connectivity, rescan |
| High CPU Usage | Host CPU at 95%+ | Identify VM consuming CPU |

### Diagnostic Procedures

#### 1. Service Issues
```bash
# Check service status
esxcli system module list | grep -i "hostd\|vpxa"

# Restart services
services.sh restart

# Check logs
tail -100 /var/log/vmware/hostd.log
tail -100 /var/log/vmware/vpxa.log
```

#### 2. Network Issues
```bash
# Check interface status
esxcli network ip interface list

# Test connectivity
esxcli network ping -d <destination>

# Check routing
esxcli network ip route list

# Check DNS
esxcli network ip dns server list
```

#### 3. Storage Issues
```bash
# Rescan storage
esxcli storage core adapter rescan --all

# Check device status
esxcli storage core device list | grep -i state

# Check multipathing
esxcli storage nmp device list

# Check path selection
esxcli storage nmp device path list --device <device>
```

### ESXi Crash Analysis

When ESXi crashes (Purple Screen of Death):
1. **Capture the screen** — photograph the PSOD
2. **Check crash dump** — `/var/log/vmware/vmware.log`
3. **Identify cause** — driver failure, hardware error, or bug
4. **Collect dump** — copy `/var/log/vmware/vmkdump` to a safe location
5. **Analyze** — Use ESXcore or VMware Support to analyze dump

```bash
# Check vmkernel log for crash
grep -i panic /var/log/vmware/vmkernel.log

# Check recent crashes
esxcli system coredump get

# List crash dumps
ls -la /var/log/vmware/
```

## ESXi Security

### Lockdown Mode
| Mode | Direct Access | vCenter Access |
|------|--------------|----------------|
| Disabled | Full access via console | Full access |
| Enabled | No access | Full access |
| Enhanced | No access | Full access |

### Lockdown Mode Commands
```bash
# Check lockdown status
esxcli system settings advanced list --option="/Lockdown/Enabled"

# Enable lockdown
esxcli system settings advanced set --option="/Lockdown/Enabled" --bool-value true
```

### Network Security
| Setting | Default | Recommendation |
|---------|---------|---------------|
| Promiscuous Mode | Disabled | Keep disabled |
| MAC Address Changes | Disabled | Keep disabled |
| Forged Transmits | Disabled | Keep disabled |

### Firewall Rules
```bash
# List firewall rules
esxcli network firewall ruleset list

# Enable vpxClient ruleset
esxcli network firewall ruleset set --enabled true --ruleset-id vpxClient

# Enable SSH ruleset
esxcli network firewall ruleset set --enabled true --ruleset-id sshServer
```

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial ESXi management knowledge |