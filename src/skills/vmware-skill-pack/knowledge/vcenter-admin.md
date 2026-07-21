# VMware vSphere — vCenter Administration Knowledge

## vCenter Architecture

### vCenter Topologies
| Topology | Description | Max Hosts | Max VMs | Use Case |
|----------|-------------|-----------|---------|----------|
| Single | One vCenter, one SSO | 4000 | 32000 | Small deployments |
| Linked | Multiple vCenters, shared SSO | 8 vCenters | 32000 each | Multi-site, large |
| Cross-vCenter | Migration between vCenters | N/A | N/A | vCenter migration |

### VCSA (VMware vCenter Server Appliance)
```
VCSA Appliance (4 appliances combined)
├── vCenter Server (vpxd) — 192.168.1.100
├── Platform Services Controller (PSC) — embedded
├── Certificate Authority (VMCA) — embedded
├── Embedded DB (PostgreSQL) — embedded
└── HTML5 Web Client — embedded
```

### VCSA Deployment Options
| Option | Description | RAM | vCPU | Disk |
|--------|-------------|-----|------|------|
| Tiny | Small scale (<5 hosts, <50 VMs) | 4 GB | 2 | 537 GB |
| Small | Medium scale (<100 hosts, <300 VMs) | 12 GB | 4 | 633 GB |
| Medium | Large scale (<400 hosts, <3000 VMs) | 20 GB | 8 | 668 GB |
| Large | Enterprise scale (>400 hosts, >3000 VMs) | 36 GB | 12 | 668 GB |

## vCenter Service Management

### VCSA Service List
| Service | Purpose | Port | Default State |
|---------|---------|------|---------------|
| vpxd | Core vCenter management | 443 | Running |
| vpxd-svcs | vCenter application services | N/A | Running |
| vsphere-client | HTML5 web client | 443 | Running |
| sso | Single Sign-On | 7444 | Running |
| vpxa | vCenter agent (on hosts) | 902 | Running |
| vmware-sts-idmd | Identity Management | N/A | Running |
| vmware-vpxd-vids | vCenter Identity | N/A | Running |

### Service Control Commands
```bash
# Check all services
service-control --status

# Restart specific service
service-control --restart vpxd

# Stop all services
service-control --stop --all

# Start all services
service-control --start --all

# Restart all services (stop then start)
service-control --restart --all
```

### Service Control Troubleshooting
```bash
# Check service logs
tail -50 /var/log/vmware/vpxd/vpxd.log
tail -50 /var/log/vmware/vpxd-svcs/vpxd-svcs.log
tail -50 /var/log/vmware/vsphere-client/vsphere-client.log

# Check database connectivity
db-tool --check
```

## vCenter Configuration

### SSO (Single Sign-On)
| Setting | Default | Notes |
|---------|---------|-------|
| Domain | vsphere.local | Default SSO domain |
| Admin User | administrator@vsphere.local | Default admin account |
| Password Policy | Default | Can customize |
| Identity Sources | None | Add AD/LDAP |

### Identity Sources
| Type | Description | Use Case |
|------|-------------|----------|
| Active Directory | Microsoft AD integration | Enterprise environments |
| LDAP | Generic LDAP directory | Non-Microsoft directories |
| Windows Active Directory | AD integration with groups | Large organizations |
| VMware Identity Manager | VMware Workspace ONE | Cloud identity |

### Adding AD Identity Source
```bash
# Via vSphere Client
# 1. Administration → Single Sign-On → Configuration
# 2. Add Identity Source
# 3. Select "Windows Active Directory"
# 4. Enter domain name: example.com
# 5. Enter DC IP: 192.168.1.10
# 6. Enter bind account: administrator@example.com
# 7. Test connection
# 8. Save
```

### Certificate Management

#### VMCA (VMware Certificate Authority)
| Mode | Description | Use Case |
|------|-------------|----------|
| Custom (default) | VMCA manages all certificates | Simple deployments |
| Custom (mixed) | Mix VMCA and custom certs | Transitioning to custom CA |
| Custom (all) | All custom certificates | Enterprise PKI |

#### Certificate Operations
```bash
# View certificate info
certificate-manager --show

# View certificate expiration
certificate-manager --show --days-before-expiry 30

# Renew all certificates
certificate-manager --renew

# Replace specific certificate
certificate-manager --replace --server --domain vpxd

# View pending requests
certificate-manager --pending
```

#### Custom Certificate Deployment
```bash
# 1. Generate CSR on VCSA
certificate-manager --generate --csr --domain vpxd --output /tmp/vpxd.csr

# 2. Sign CSR with CA
#    Copy CSR to CA server, sign it, copy signed cert back

# 3. Deploy signed certificate
certificate-manager --replace --server --domain vpxd \
    --certificate /tmp/vpxd.crt \
    --private-key /tmp/vpxd.key \
    --ca-certificate /tmp/ca.crt

# 4. Restart services
service-control --restart --all
```

## vCenter Operations

### vCenter Inventory Management
```bash
# List all hosts
vim-cmd hostsvc/get_all_host_view

# List all VMs
vim-cmd vmsvc/get_allvms

# Get VM info by ID
vim-cmd vmsvc/get.summary <vm-id>

# Power on VM
vim-cmd vmsvc/power.on <vm-id>

# Power off VM
vim-cmd vmsvc/power.off <vm-id>

# Reset VM
vim-cmd vmsvc/power.reset <vm-id>
```

### vCenter Database
| Component | Default | Notes |
|-----------|---------|-------|
| Database | PostgreSQL | Embedded in VCSA |
| Port | 5432 | Internal only |
| Backup | Built-in API | Regular backup recommended |
| Retention | Configurable | Adjust based on compliance |

### vCenter Backup
```bash
# Backup VCSA via API
curl -k -X POST https://<vcenter>/rest/vcenter/appliance/backup -H "Authorization: Bearer <token>"

# Scheduled backup via vSphere Client
# 1. vSphere Client → Administration → System Configuration
# 2. Select VCSA → Backup and Restore
# 3. Configure backup schedule
# 4. Select backup location (NFS, SMB, HTTP)
# 5. Test backup

# Restore VCSA
# 1. Boot VCSA deployment appliance
# 2. Select "Restore"
# 3. Enter backup location and credentials
# 4. Follow restore wizard
```

### vCenter Logging
| Log File | Purpose | Default Location |
|----------|---------|-----------------|
| vpxd.log | vCenter core operations | /var/log/vmware/vpxd/ |
| vpxd-svcs.log | Application services | /var/log/vmware/vpxd-svcs/ |
| vsphere-client.log | Web client | /var/log/vmware/vsphere-client/ |
| identity.log | SSO/Identity | /var/log/vmware/sso/ |
| vpxd-provisioning.log | Provisioning | /var/log/vmware/vpxd/ |
| audit.log | Security audit | /var/log/vmware/sso/ |

## vCenter Troubleshooting

### Common vCenter Issues

| Issue | Symptoms | Root Cause | Resolution |
|-------|----------|------------|------------|
| vpxd not starting | vpxd service failed | Database error, disk full, cert expired | Check logs, fix root cause |
| SSO not responding | Cannot login to vCenter | PSC service down, cert expired | Restart PSC, renew cert |
| Web client slow | UI loading slowly | DB performance, network latency | Optimize DB, check network |
| Linked vCenter issues | Sync failures between vCenters | Network connectivity, PSC issues | Fix network, check PSC |
| Certificate expiry | Services failing to start | Certificates expired | Renew certificates |
| Database growth | Performance degradation | Logs too large, retention too high | Clean logs, adjust retention |
| License issues | Features unavailable | License expired, server unreachable | Renew license, check network |

### vCenter Diagnostic Procedures

#### 1. Service Issues
```bash
# Check service status
service-control --status

# Check vpxd logs
tail -100 /var/log/vmware/vpxd/vpxd.log

# Check if vpxd is responsive
curl -k https://localhost/rest/com/vmware/cis/session -H "vmware-api-session-id: <token>"

# Check memory usage
free -m
vmware-vpxd-memory
```

#### 2. Database Issues
```bash
# Check database status
db-tool --check

# Check database size
du -sh /storage/db/

# Check database logs
tail -100 /storage/db/logs/postgresql.log

# Check connection count
db-tool --connections
```

#### 3. Network Issues
```bash
# Check DNS resolution
nslookup vcenter.example.com

# Check connectivity to hosts
ping <esxi-host>

# Check port availability
esxcli network firewall ruleset ruleset info --ruleset-id vpxClient
```

### vCenter Performance Tuning

| Parameter | Default | Recommended | Notes |
|-----------|---------|-------------|-------|
| vpxd threads | 100 | 200-400 | Based on VM count |
| vpxd max threads | 500 | 800-1000 | Max concurrent requests |
| DB connection pool | 50 | 100-200 | Based on activity |
| Log retention | 7 days | 30 days | Based on compliance |
| vpxd heap size | 2 GB | 4-8 GB | Based on inventory |

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial vCenter administration knowledge |