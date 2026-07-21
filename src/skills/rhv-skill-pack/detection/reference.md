# Red Hat Virtualization (RHV) — Detection Reference

## Purpose

This document defines the detection rules for the Red Hat Virtualization engineering skill pack.

## Foundation Reference

Detection rules in this document apply RHV-specific patterns (ovirt-engine, VDSM, Gluster, cockpit, rhev-*, virsh) to the shared detection framework defined by the [Virtualization Engineering Foundation](docs/virtualization/VIRTUALIZATION_FOUNDATION.md).

## Architecture Overview

RHV is detected through:

- RHV Manager web interface and API
- Cockpit web console integration
- CLI tools (rhev-admin, rhev-mkiso, vdsm-tool, virt-who)
- Log patterns and system commands
- Browser URLs and window titles
- Window titles showing RHV interfaces
- Common terminology in terminal output

## Detection Rule Catalog

### RHV Manager Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| `rhv-manager-detect` | RHV Manager UI detected | ovirt-engine-web UI or cockpit detected | Informational |
| `rhv-engine-not-running` | ovirt-engine service not running | Engine service failure | Critical |
| `rhv-engine-db-error` | PostgreSQL engine errors | Database connectivity or query errors | Critical |
| `rhv-engine-api-error` | REST API failures | Engine API returning errors | High |
| `rhv-engine-cert-error` | Engine certificate issues | TLS certificate expired or mismatch | Critical |

### RHV Host Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| `rhv-host-detect` | Hosts detected in inventory | Hosts registered with RHV Manager | Informational |
| `rhv-host-unreachable` | Host not responding | Host lost connection to Engine | Critical |
| `rhv-host-degraded` | Host in maintenance or warning | Host performance degraded | High |
| `rhv-host-kernel-error` | KVM/libvirt errors | Hypervisor-level issues | High |
| `rhv-host-storage-error` | Storage domain access failure | Host cannot access storage | Critical |
| `rhv-host-memory-pressure` | Host memory exhaustion | Host running out of memory | Critical |

### RHV VM Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| `rhv-vm-detect` | VMs in inventory | VMs managed by RHV Manager | Informational |
| `rhv-vm-not-running` | VM not running as expected | Unexpected VM state | Medium |
| `rhv-vm-slow` | VM performance degraded | High CPU ready, ballooning, I/O | Medium |
| `rhv-vm-crashed` | VM unexpectedly stopped | Guest OS crash or OOM | High |
| `rhv-vm-disk-full` | VM guest disk full | Guest OS running out of space | High |
| `rhv-vm-tools-outdated` | Virtio drivers outdated | Missing or old Virtio drivers | Medium |

### RHV Storage Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| `rhv-storage-domain-detect` | Storage domains detected | ISO, Data, Export, Backup domains | Informational |
| `rhv-storage-domain-maintenance` | Domain in maintenance mode | Domain being modified or repaired | Medium |
| `rhv-storage-domain-down` | Domain is down | Domain inaccessible to all hosts | Critical |
| `rhv-storage-domain-readonly` | Domain becomes read-only | Quorum loss or error recovery | Critical |
| `rhv-storage-domain-full` | Domain approaching capacity | Storage domain near full | High |
| `rhv-gluster-volume-down` | Gluster volume inactive | Gluster volume down or degraded | Critical |

### RHV Network Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| `rhv-network-detect` | Networks in inventory | Virtual and physical networks detected | Informational |
| `rhv-network-isolation` | Network isolated | Network partition detected | High |
| `rhv-network-vnic-error` | vNIC errors | Virtual NIC configuration errors | High |
| `rhv-network-bond-failure` | Bond failover detected | NIC bond failover occurred | Medium |
| `rhv-network-dns-error` | DNS resolution failure | DNS not resolving for hosts/VMs | High |

### RHV Cluster and HA Detection Rules

| Rule ID | Condition | Description | Severity |
|---------|-----------|-------------|----------|
| `rhv-cluster-detect` | Clusters in inventory | Clusters with HA configured | Informational |
| `rhv-cluster-ha-failed` | HA failover occurred | VM migrated due to host failure | High |
| `rhv-cluster-compat-issue` | Compatibility version mismatch | Host version doesn't match cluster | High |
| `rhv-cluster-no-capacity` | Insufficient cluster capacity | Cluster cannot run required VMs | Critical |
| `rhv-hosted-engine-failover` | Engine VM moved | Hosted Engine migrated to another host | Critical |
| `rhv-hosted-engine-degraded` | Engine VM unhealthy | Engine VM running but degraded | Critical |

## Confidence Scoring

| Confidence | Description | Example |
|------------|-------------|---------|
| High (0.9+) | Clear error, specific cause | "ovirt-engine service failed: disk full" |
| Medium (0.7-0.9) | Multiple possible causes | "Host unreachable from Engine" |
| Low (0.5-0.7) | Vague symptoms, needs more info | "RHV Manager acting slow" |
| Very Low (<0.5) | Insufficient data | "Something wrong with RHV" |

## Detection Patterns

### Terminal Command Patterns

```yaml
patterns:
  # RHV CLI tools
  - pattern: '^rhev-.*'
    confidence: 0.95
    technology_domain: RHV
  
  # VDSM tools
  - pattern: '^vdsm-tool|^vdsm-.*'
    confidence: 0.9
  
  # Cockpit RHV integration
  - pattern: '^cockpit.*rhv|^virt-.*'
    confidence: 0.85
  
  # Libvirt/KVM commands
  - pattern: '^virsh.*|^virt-.*'
    confidence: 0.8
  
  # Engine CLI
  - pattern: '^engine-.*|^engine-setup|^engine-cleanup|^engine-upgrade'
    confidence: 0.95
  
  # RHV Manager services
  - pattern: 'systemctl.*(ovirt-engine|vdsm|rhevm|cockpit)'
    confidence: 0.9
  
  # VDSM configuration
  - pattern: 'vdsm.conf|^config.*vdsm'
    confidence: 0.9
  
  # Gluster storage
  - pattern: 'gluster.*(volume|peer|volume.*status)'
    confidence: 0.85
```

### Browser URL Patterns

```yaml
browser_patterns:
  # RHV Manager web UI
  - pattern: '.*ovirt-engine.*webadmin.*|.*ovirt.org.*webadmin.*'
    confidence: 0.95
  
  # Cockpit RHV
  - pattern: '.*cockpit.*rhev.*|.*cockpit.*rhv.*'
    confidence: 0.9
  
  # RHV Portal
  - pattern: '.*engine.*portal.*|.*ovirt.*portal.*'
    confidence: 0.9
```

### Window Title Patterns

```yaml
window_patterns:
  - pattern: '.*Red Hat Virtualization.*|.*RHV Manager.*'
    confidence: 0.95
  
  - pattern: '.*ovirt-engine.*web.*'
    confidence: 0.9
  
  - pattern: '.*Cockpit.*RHV.*'
    confidence: 0.9
```

### Text Pattern Matches

```yaml
text_patterns:
  # Engine-related
  - pattern: 'ovirt-engine.*error|ovirt-engine.*fail'
    confidence: 0.9
  
  # VDSM-related
  - pattern: 'vdsm.*error|vdsm.*fail|vdsm.*exception'
    confidence: 0.9
  
  # Host-related
  - pattern: 'host.*unreachable|host.*not responding|host.*disconnected'
    confidence: 0.85
  
  # Storage-related
  - pattern: 'storage.*domain.*down|storage.*domain.*error|gluster.*down'
    confidence: 0.9
  
  # VM-related
  - pattern: 'VM.*stopped unexpectedly|VM.*migration.*failed|VM.*power.*off'
    confidence: 0.85
  
  # HA-related
  - pattern: 'HA.*failover|HA.*migrate|non-responsive.*host'
    confidence: 0.9
  
  # Cluster-related
  - pattern: 'cluster.*compatibility.*version|cluster.*upgrade'
    confidence: 0.85
```

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial RHV detection rules |