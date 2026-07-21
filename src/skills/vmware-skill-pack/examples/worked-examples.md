# VMware vSphere — Worked Examples

## Example 1: vCenter Service Failure After Certificate Expiry

### Scenario
After a planned maintenance window, vCenter Server fails to start. The vSphere Client shows "Cannot connect to the server" error.

### Symptoms
- vSphere Client returns "Cannot connect to the server"
- Users unable to access vCenter
- Monitoring alert: "vCenter Server unreachable"

### Evidence
```bash
# Check vCenter services
$ service-control --status
FAILED           vmware-vpxd
FAILED           vmware-vpxd-svcs
STOPPED          vmware-vsphere-client
STOPPED          vmware-sso

# Check disk space
$ df -h
Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1        50G   49G  1.0G  98% /
vmfs/volumes     10T  8.5T  1.5T  85% /vmfs/volumes

# Check vpxd logs
$ tail -50 /var/log/vmware/vpxd/vpxd.log
2026-07-21T10:15:00.000Z [error] vpxd[12345] [Originator@6876.1] Exception: SAML token validation failed
2026-07-21T10:15:00.000Z [error] vpxd[12345] [Originator@6876.1] Certificate expired for sso service
2026-07-21T10:15:00.000Z [error] vpxd[12345] [Originator@6876.1] SSL handshake failed: certificate has expired
```

### Analysis
The certificate for the SSO service has expired, causing SSL handshake failures. The vpxd service cannot validate SAML tokens and fails to start. This is a common issue when certificates are not monitored for expiry.

### Resolution
```bash
# 1. Stop all vCenter services
sudo service-control --stop --all

# 2. Renew certificates using VCSA certificate manager
sudo certificate-manager --renew --domain sso

# 3. Restart vCenter services
sudo service-control --start --all

# 4. Verify services are running
sudo service-control --status
```

### Verification
```bash
$ service-control --status
RUNNING          vmware-vpxd
RUNNING          vmware-vpxd-svcs
RUNNING          vmware-vsphere-client
RUNNING          vmware-sso

$ curl -k https://localhost
<html>
  <title>vSphere Web Client</title>
...
```

### Lessons
- Always monitor certificate expiry in vCenter (set alerts 30 days before expiry)
- Renew certificates during maintenance windows
- Consider custom CA for better certificate lifecycle management

---

## Example 2: ESXi Host Disconnected from vCenter

### Scenario
An ESXi host suddenly disconnects from vCenter Server. The host shows red in the vSphere Client.

### Symptoms
- Host shows red/disconnected status in vSphere Client
- VMs on host become unmanaged
- No VM operations possible on the host
- Alarms: "Host has been disconnected"

### Evidence
```bash
# Check network connectivity
$ esxcli network ping -d 192.168.1.100  # vCenter IP
ping: sendmsg: Network is unreachable
ping: sendmsg: Network is unreachable
--- 192.168.1.100 ping statistics ---
3 packets transmitted, 0 received, 100% packet loss

# Check management network
$ esxcli network ip interface list
name   type    mac           mtu   ipv4         ipv6   gateway
vmk0   user   00:50:56:xx   1500  192.168.1.50 --     192.168.1.1
vmk1   user   00:50:56:yy   1500  10.0.0.50    --     --

# Check management network port group
$ esxcli network vswitch standard portgroup list
...
vmk0 uses vmnic1 (uplink)
```

### Analysis
The management network (vmk0) is unreachable from the ESXi host. Looking at the network configuration, vmk0 is using vmnic1 as its uplink. Checking the physical switch shows vmnic1 has been unplugged or the port has gone down.

### Resolution
```bash
# 1. Check physical connectivity (through console)
# vmnic1 appears to be unplugged at the physical switch

# 2. If possible, re-connect the cable to vmnic1
# OR reconfigure vmk0 to use vmnic0

# 3. Reconfigure management network
$ esxcli network vswitch standard portgroup set \
    --portgroup-name "Management Network" \
    --vmknic vmnic0

# 4. Verify connectivity
$ esxcli network ping -d 192.168.1.100
--- 192.168.1.100 ping statistics ---
3 packets transmitted, 3 received, 0% packet loss
```

### Verification
```bash
$ esxcli network ping -d 192.168.1.100
--- 192.168.1.100 ping statistics ---
3 packets transmitted, 3 received, 0% packet loss

$ esxcli network ip interface list
name   type    mac           mtu   ipv4         ipv6   gateway
vmk0   user   00:50:56:xx   1500  192.168.1.50 --     192.168.1.1
```

### Lessons
- Always configure dual uplinks for management network
- Use vSphere vMotion to migrate VMs before maintenance
- Monitor physical switch port status as part of monitoring
- Configure HA with multiple datastore heartbeats

---

## Example 3: VM Running Slow Due to CPU Ready Time

### Scenario
Users report a web application VM is running very slowly. Response times have increased from 100ms to several seconds.

### Symptoms
- Web application response time degraded
- Users experiencing timeouts
- No changes to application or database
- Other VMs on same host running normally

### Evidence
```bash
# Check VM processes
$ esxcli vm process list | grep -A 10 -i "webapp01"
World ID: 123456
   VM ID: 987654
   Name: webapp01
   UUID: 501a2b3c-4d5e-6f7a-8b9c-0d1e2f3a4b5c
   Display Name: webapp01
   Guest OS: Ubuntu Linux (64-bit)
   State: Running
   CPU Count: 4
   Memory Size: 16384 MB

# Check resource usage
$ esxtop -b -n 1 | grep -i "CPU READY"
VM Name     CPU   %RDY  %IDLE  %SWP  %MDT  %MLD
webapp01    0     35.2    0.1   0.0   0.0   0.0
webapp01    1     28.7    0.3   0.0   0.0   0.0
webapp01    2     42.1    0.0   0.0   0.0   0.0
webapp01    3     38.9    0.2   0.0   0.0   0.0

# Check host resource usage
$ esxtop -b -n 1 | head -10
Hostname: esxi01.example.com
Power State: On
Uptime: 45 days, 3:20
Total CPUs: 16
Total Cores: 32
Total vCPUs: 48

CPU Used: 85.2%
CPU Ready: 28.4%
CPU Co-stop: 12.3%
CPU Swap: 0.0%
```

### Analysis
The VM's CPU Ready time is extremely high (28-42%), indicating vCPU contention. The host has 48 vCPUs allocated but only 16 physical CPUs, resulting in severe overcommitment. The webapp01 VM is starving for CPU cycles.

### Resolution
```bash
# Option 1: Migrate VM to less loaded host (vMotion)
# Via vSphere Client or PowerCLI:
# Move-VM -VM "webapp01" -Destination "esxi02.example.com"

# Option 2: Increase CPU allocation (if capacity exists)
# Via vSphere Client: VM Settings → Hardware → CPU → Increase vCPUs

# Option 3: Reduce overcommitment by migrating other VMs

# Best option: vMotion to less loaded host
```

### Verification
```bash
# After vMotion to less loaded host:
$ esxtop -b -n 1 | grep -i "CPU READY"
VM Name     CPU   %RDY  %IDLE  %SWP  %MDT  %MLD
webapp01    0      2.1   95.3   0.0   0.0   0.0
webapp01    1      1.8   96.1   0.0   0.0   0.0
webapp01    2      3.4   93.2   0.0   0.0   0.0
webapp01    3      2.5   94.8   0.0   0.0   0.0
```

### Lessons
- Monitor CPU Ready time continuously (alert at >5% average)
- Avoid overcommitting vCPUs beyond 4:1 ratio for production VMs
- Use DRS to automatically balance workloads
- Consider EVC mode for migration flexibility

---

## Example 4: Datastore Nearly Full

### Scenario
A monitoring alert reports a datastore is at 95% capacity. Multiple VMs are at risk of running out of disk space.

### Symptoms
- Alert: "Datastore production-ds01 at 95% capacity"
- Some VMs experiencing disk full errors
- No new backups can be written

### Evidence
```bash
# Check datastore capacity
$ esxcli storage filesystem list
Mount Point                          Volume Name    UUID                               Type    Capacity(MB)    Free Space(MB)
/vmfs/volumes/5f8a9b2c-1d3e-4f5a    production-ds01  5f8a9b2c-1d3e-4f5a-6b7c-8d9e0f1a2b3c  VMFS-6  104857600    5242880
# 100TB datastore, 95TB used, 5TB free

# Check VM disk usage on datastore
$ find /vmfs/volumes/production-ds01 -name "*.vmdk" -exec ls -lh {} \; 2>/dev/null | sort -k 5 -h | tail -10
-rw------- 1 root root  500G  webapp01/webapp01.vmdk
-rw------- 1 root root  400G  db01/db01.vmdk
-rw------- 1 root root  350G  app01/app01.vmdk
...

# Check snapshot sizes
$ esxcli storage vmfs snapshot list
UUID                             Name       Disk Size(MB)    Datastore
abc123-def456-789            webapp01-20260701    51200           production-ds01
abc123-def456-789            webapp01-20260715    76800           production-ds01
# 128GB of snapshots from webapp01

# Check VM log sizes
$ find /vmfs/volumes/production-ds01 -name "*.log" -exec du -sh {} \; 2>/dev/null | sort -k 1 -h | tail -5
2.1G    /vmfs/volumes/production-ds01/db01/vmware.log
1.8G    /vmfs/volumes/production-ds01/app01/vmware.log
```

### Analysis
The datastore is nearly full due to two main factors:
1. **Accumulated snapshots** on webapp01 (128GB) that were never consolidated
2. **Large log files** from VMs that aren't being rotated

### Resolution
```bash
# 1. Delete old snapshots on webapp01
# Via vSphere Client: VM → Snapshots → Delete All...

# 2. Clean old VM log files
$ rm /vmfs/volumes/production-ds01/db01/vmware.log.1
$ rm /vmfs/volumes/production-ds01/db01/vmware.log.2
$ rm /vmfs/volumes/production-ds01/app01/vmware.log.1

# 3. Verify space freed
$ esxcli storage filesystem list
```

### Verification
```bash
$ esxcli storage filesystem list
Mount Point                          Volume Name    UUID                               Type    Capacity(MB)    Free Space(MB)
/vmfs/volumes/5f8a9b2c-1d3e-4f5a    production-ds01  5f8a9b2c-1d3e-4f5a-6b7c-8d9e0f1a2b3c  VMFS-6  104857600    26214400
# Now 25TB free (25% of 100TB)
```

### Lessons
- Establish a snapshot retention policy (max 7 days, auto-delete)
- Implement log rotation on guest OS level
- Monitor datastore capacity with alerts at 70%, 80%, 90%
- Consider thin provisioning with regular cleanup

---

## Example 5: HA Cluster Failure Due to Management Network Issue

### Scenario
vSphere HA shows red status on one host in the cluster. VMs on that host are not restarting automatically.

### Symptoms
- HA status shows red on host cluster01-host03
- VMs on host03 not recovering after manual host failure
- Other hosts in cluster showing green

### Evidence
```bash
# Check HA cluster status
$ esxcli system module get --module-name=ha-agent
Name: ha-agent
Is Loadable: Yes
Loaded: Yes
Description: vSphere HA agent

# Check HA agent logs
$ tail -50 /var/log/vmware/fdm/vmware-fdm.log
2026-07-21T11:00:00.000Z [error] Host heartbeating failed - no management network
2026-07-21T11:00:00.000Z [error] Lost heartbeat on vmk0
2026-07-21T11:00:00.000Z [error] Datastore heartbeats also failed

# Check management network
$ esxcli network ip interface list
name   type    mac           mtu   ipv4         ipv6   gateway
vmk0   user   00:50:56:xx   1500  192.168.1.30 --     192.168.1.1
vmk1   user   00:50:56:yy   1500  192.168.2.30 --     --

# Check vSwitch
$ esxcli network vswitch standard list
```

### Analysis
The management network (vmk0) has lost connectivity. The HA agent has lost heartbeat with vCenter, causing the host to be marked as isolated. Datastore heartbeats also failed because the datastore was accessed through the same failed network path.

### Resolution
```bash
# 1. Check physical connectivity
# vmnic0 appears to have cable issue or switch port failure

# 2. Reconfigure management to use alternate uplink
$ esxcli network vswitch standard portgroup set \
    --portgroup-name "Management Network" \
    --vmknic vmnic1

# 3. If vmk1 doesn't have correct IP configuration
$ esxcli network ip interface ipv4 set \
    --interface-name vmk0 \
    --ipv 192.168.1.30 \
    --netmask 255.255.255.0 \
    --type static

# 4. Verify connectivity
$ esxcli network ping -d 192.168.1.100  # vCenter
$ esxcli network ping -d 192.168.1.1     # Default gateway
```

### Verification
```bash
$ esxcli network ping -d 192.168.1.100
--- 192.168.1.100 ping statistics ---
3 packets transmitted, 3 received, 0% packet loss

$ tail -20 /var/log/vmware/fdm/vmware-fdm.log
2026-07-21T11:15:00.000Z [info] Host heartbeat restored
2026-07-21T11:15:00.000Z [info] HA agent connected to vCenter
```

### Lessons
- Always configure dual uplinks for management network
- Use redundant network paths for HA heartbeats
- Monitor management network latency and packet loss
- Test failover scenarios regularly

---

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial worked examples |