# VMware vSphere — Networking Management Knowledge

## vSphere Networking Architecture

### Networking Layers
```
Virtual Machines
    ↓
vNIC (Virtual Network Adapter)
    ↓
Port Group (Standard or Distributed)
    ↓
vSwitch (vSS or vDS)
    ↓
Uplinks (Physical NICs)
    ↓
Physical Switch (Access Switch)
    ↓
Core/Edge Network
```

### Network Object Hierarchy
```
Datacenter
├── Network Folder
│   ├── Standard Switches (per-host)
│   │   ├── vSwitch0 (Management, vMotion)
│   │   │   ├── Port Group: Management Network
│   │   │   └── Port Group: vMotion Network
│   │   └── vSwitch1 (VM Network)
│   │       └── Port Group: VM Network
│   └── Distributed Switches (cluster-wide)
│       └── vDS0 (Production)
│           ├── Port Group: Production VMs
│           ├── Port Group: Management
│           └── Port Group: vMotion
```

## Standard vSwitch (vSS)

### vSS Components
| Component | Description |
|-----------|-------------|
| vSwitch | Virtual layer-2 switch |
| Port Group | Logical grouping of ports |
| Uplink | Physical NIC connected to vSwitch |
| VMkernel Adapter (vmk) | Specialized host network interface |
| VM Network Adapter | VM network connection |

### vSS Configuration
```bash
# List vSwitches
esxcli network vswitch standard list

# List port groups
esxcli network vswitch standard portgroup list

# List uplinks
esxcli network vswitch standard uplink list --vswitch-name vSwitch0

# List security policy
esxcli network vswitch standard policy security get --vswitch-name vSwitch0
```

### vSS Creation
```bash
# Create vSwitch
esxcli network vswitch standard add --vswitch-name vSwitch2

# Add uplink
esxcli network vswitch standard uplink add --uplink-name vmnic2 --vswitch-name vSwitch2

# Add port group
esxcli network vswitch standard portgroup add --portgroup-name "Production" --vswitch-name vSwitch2

# Set VLAN
esxcli network vswitch standard portgroup set --portgroup-name "Production" --vlans 100
```

## Distributed vSwitch (vDS)

### vDS Components
| Component | Description |
|-----------|-------------|
| vDS | Virtual switch spanning multiple hosts |
| Port Group | Centralized port configuration |
| Uplink Group | Physical NIC groups |
| VMkernel Adapter | Host network interface |
| NIO | Network I/O Control (QoS) |

### vDS Configuration
```bash
# List vDS
esxcli network vswitch distributed list

# List port groups
esxcli network vswitch distributed portgroup list --dvs <dvs-name>

# List uplinks
esxcli network vswitch distributed uplink list --dvs <dvs-name>

# List vmkernel adapters
esxcli network ip interface list
```

### vDS Features
| Feature | Description |
|---------|-------------|
| NetFlow | Traffic monitoring and analysis |
| sFlow | Sampled traffic monitoring |
| NIO | QoS for different traffic types |
| Private VLAN | Network isolation |
| Teaming | NIC load balancing and failover |
| Promiscuous Mode | All traffic allowed (security risk) |

## VMkernel Adapters

### VMkernel Adapter Purposes
| Adapter | Purpose | Typical Port Group | Port |
|---------|---------|-------------------|------|
| vmk0 | Management | Management Network | 443 (HTTPS) |
| vmk1 | vMotion | vMotion Network | 8000 (TCP) |
| vmk2 | iSCSI | iSCSI Storage Network | 3260 (TCP) |
| vmk3 | NFS | NFS Storage Network | 2049 (TCP/UDP) |
| vmk4 | vSAN | vSAN Network | 2371 (TCP) |
| vmk5 | Fault Tolerance | FT Network | 5000 (UDP) |
| vmk6 | Replication | Replication Network | Varies |

### VMkernel Configuration
```bash
# List all vmkernel adapters
esxcli network ip interface list

# Add vmkernel adapter
esxcli network ip interface add --interface-name vmk2 --port-group "Storage Network"

# Set IPv4
esxcli network ip interface ipv4 set --interface-name vmk2 --ipv4 10.0.1.10 --netmask 255.255.255.0 --type static

# Enable vMotion
esxcli network ip interface set --interface-name vmk1 --enabled true

# Enable TCP Offload
esxcli network ip interface tcp-offload set --interface-name vmk2 --enabled true
```

## Networking Security

### vSwitch Security Policies
| Setting | Default | Description | Safe Default |
|---------|---------|-------------|--------------|
| Promiscuous Mode | Disabled | All traffic allowed | Disabled |
| MAC Address Changes | Disabled | MAC changes allowed | Disabled |
| Forged Transmits | Disabled | NIC MAC changes | Disabled |

### Security Policy Configuration
```bash
# Get security policy
esxcli network vswitch standard policy security get --vswitch-name vSwitch0

# Set security policy
esxcli network vswitch standard policy security set \
    --vswitch-name vSwitch0 \
    --allow-forged-transmits false \
    --allow-mac-change false \
    --allow-promiscuous false
```

## vMotion Networking

### vMotion Requirements
| Requirement | Description |
|-------------|-------------|
| vmk interface | Dedicated vmk for vMotion |
| Same subnet | Source and destination must be same subnet |
| Network latency | < 1ms recommended |
| Bandwidth | 10 Gbps minimum, 25 Gbps recommended |
| MTU | 1500 minimum, 9000 for jumbo frames |
| Security | Same security policy on both hosts |

### vMotion Network Configuration
```bash
# Configure vMotion network
esxcli network ip interface add --interface-name vmk1 --port-group "vMotion Network"
esxcli network ip interface ipv4 set --interface-name vmk1 --ipv4 10.10.10.10 --netmask 255.255.255.0 --type static

# Enable vMotion on vmk
esxcli network ip interface set --interface-name vmk1 --enabled true

# Configure vMotion routing
esxcli network ip route ipv4 add --gateway 10.10.10.1 --network 10.10.0.0/16
```

## Network Troubleshooting

### Common Networking Issues

| Issue | Symptoms | Root Cause | Resolution |
|-------|----------|------------|------------|
| Port Group missing | VM cannot connect | Port group deleted or misconfigured | Recreate port group |
| VLAN mismatch | VM cannot reach network | VLAN ID mismatch | Correct VLAN ID |
| vMotion network down | vMotion fails | vmk1 misconfigured | Reconfigure vmk1 |
| Uplink failure | Host loses connectivity | Physical NIC or cable issue | Replace cable/switch port |
| Security policy | VM cannot communicate | Promiscuous mode disabled | Adjust policy or use correct adapter |
| MTU mismatch | Fragmented packets | MTU mismatch on path | Align MTU across path |
| vDS sync issue | Host not on vDS | vDS configuration out of sync | Re-attach host to vDS |

### Network Diagnostic Commands

#### vSS Diagnostics
```bash
# List vSwitches
esxcli network vswitch standard list

# List port groups
esxcli network vswitch standard portgroup list

# Test connectivity
esxcli network ping -d <destination>

# Check interface status
esxcli network ip interface list
```

#### vDS Diagnostics
```bash
# List vDS
esxcli network vswitch distributed list

# List vDS port groups
esxcli network vswitch distributed portgroup list --dvs <dvs-name>

# Check vDS host membership
esxcli network vswitch distributed host list --dvs <dvs-name>

# Check uplink status
esxcli network vswitch distributed uplink list --dvs <dvs-name>
```

#### General Network Diagnostics
```bash
# Check routing
esxcli network ip route list

# Check DNS
esxcli network ip dns server list

# Check firewall rules
esxcli network firewall ruleset list

# Check packet loss
esxcli network ping -d <destination> -c 100

# List active connections
esxcli network connection list | head -20
```

## Network Best Practices

### vSS Best Practices
- Use separate vSwitches for different traffic types
- Configure redundant uplinks
- Set appropriate VLANs
- Review security policies
- Document port group naming

### vDS Best Practices
- Use vDS for clusters with 4+ hosts
- Configure NIO for QoS
- Enable NetFlow for monitoring
- Regularly review vDS configuration
- Document vDS changes

### VMkernel Best Practices
- Dedicated vmk for each traffic type
- Same subnet for vMotion
- Jumbo frames for storage/vSAN if supported
- Proper MTU alignment
- Redundant network paths

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial networking management knowledge |