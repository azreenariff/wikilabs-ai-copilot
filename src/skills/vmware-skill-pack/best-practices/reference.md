# VMware vSphere Engineering — Best Practices

## General Best Practices

### Change Management
1. **Always test in non-production first**
2. **Document changes before implementation**
3. **Use maintenance windows for production changes**
4. **Have a rollback plan for every change**
5. **Notify stakeholders before changes**

### Backup and Recovery
1. **Regular VM backups with tested restore**
2. **Snapshot rotation policy (max 7 days)**
3. **VCSA backup schedule (daily recommended)**
4. **Document recovery procedures**
5. **Test disaster recovery annually**

### Performance Management
1. **Monitor CPU Ready time (<5% target)**
2. **No memory ballooning or swapping**
3. **Disk latency <20ms**
4. **Balance VM workloads with DRS**
5. **Regular capacity planning reviews**

### Security Best Practices
1. **Regular certificate rotation**
2. **Role-based access control (RBAC)**
3. **Enable audit logging**
4. **Patch ESXi and vCenter regularly**
5. **Network segmentation for management**

## vCenter Best Practices

### Deployment
1. **Use VCSA for new deployments**
2. **Select appropriate size (tiny/small/medium/large)**
3. **Deploy in active directory domain**
4. **Configure redundant network paths**
5. **Use custom certificates for production**

### Maintenance
1. **Regular patch updates (quarterly)**
2. **Certificate monitoring (alert 30 days before expiry)**
3. **Database backup and verify**
4. **Review and clean up orphaned objects**
5. **Performance baseline monitoring**

### Certificate Management
1. **Rotate certificates annually**
2. **Use VMware Certificate Authority for simplicity**
3. **Document certificate inventory**
4. **Test certificate renewal in non-production**
5. **Backup certificates before renewal**

## ESXi Best Practices

### Host Configuration
1. **Standardize ESXi version across cluster**
2. **Enable SSH only for troubleshooting**
3. **Configure syslog server**
4. **Set proper NTP configuration**
5. **Use lockdown mode for security**

### Storage
1. **Monitor datastore capacity (alert at 80%)**
2. **Use SSD caching where possible**
3. **Configure redundant storage paths**
4. **Regular VMFS health checks**
5. **Use Storage DRS for workload balancing**

### Networking
1. **Dedicate VMkernel adapters for specific traffic**
2. **Configure redundant uplinks**
3. **Use jumbo frames consistently (if supported)**
4. **Document network configuration**
5. **Monitor network utilization**

### Patching
1. **Test patches in non-production first**
2. **Use vSphere Lifecycle Manager for centralized patching**
3. **Maintain support contracts for VMware guidance**
4. **Schedule maintenance windows for updates**
5. **Verify cluster health after patching**

## VM Best Practices

### Provisioning
1. **Use templates for standard deployments**
2. **Customize VMs with guest OS specifications**
3. **Set appropriate resource allocations**
4. **Install VMtools immediately after deployment**
5. **Enable encryption for sensitive workloads**

### Lifecycle
1. **Regular VM patching (quarterly)**
2. **Review and clean up old snapshots**
3. **Monitor performance and adjust resources**
4. **Deprecate unused VMs (archival policy)**
5. **Document VM ownership and purpose**

### Resource Management
1. **Avoid vCPU overcommitment (>4:1)**
2. **Set memory reservations for critical VMs**
3. **Use storage vMotion for workload rebalancing**
4. **Enable DRS for automatic load balancing**
5. **Regular review of resource pools**

## HA/DRS Best Practices

### HA Configuration
1. **Enable HA for all production clusters**
2. **Configure admission control (20% reserved)**
3. **Set VM restart priorities**
4. **Use multiple datastore heartbeats**
5. **Test failover scenarios annually**

### DRS Configuration
1. **Fully automated for large clusters**
2. **Partially automated for medium clusters**
3. **Configure anti-affinity rules for critical VMs**
4. **Monitor DRS recommendations**
5. **Regular review of DRS automation level**

### EVC Configuration
1. **Enable EVC for migration flexibility**
2. **Match EVC to oldest host CPU in cluster**
3. **Upgrade hosts uniformly for EVC**
4. **Test vMotion compatibility after changes**
5. **Document EVC levels per cluster**

## Storage Best Practices

### Capacity Planning
1. **Monitor and forecast capacity monthly**
2. **Plan for 30% headroom**
3. **Use thin provisioning with monitoring**
4. **Regular cleanup of orphaned files**
5. **Document storage growth trends**

### Performance
1. **Monitor storage latency continuously**
2. **Configure Storage I/O Control for critical VMs**
3. **Use SSD caching for performance tiers**
4. **Balance workloads with Storage DRS**
5. **Test storage performance regularly**

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial best practices guide |