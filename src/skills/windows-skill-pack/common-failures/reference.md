# Windows Engineering — Common Failures

## Failure Catalog

### 1. Windows Service Failure

**Severity:** Critical
**Impact:** Service unavailable, dependent services affected
**Frequency:** High

**Symptoms:**
- Service shows "Stopped" in services.msc
- Event ID 7024 or 7023 in System log
- Application not responding
- Dependent services failing

**Root Causes:**
- Service dependency not running
- Service account password expired
- Missing executable or DLL
- Configuration corruption
- Insufficient permissions

**Resolution:**
```powershell
# Check service status
Get-Service -Name <service-name>

# Check dependencies
Get-WinEvent -FilterHashtable @{LogName='System'; ProviderName='Service Control Manager'}

# Restart service
Start-Service -Name <service-name>

# Check service account
sc qc <service-name>
```

### 2. DNS Resolution Failure

**Severity:** High
**Impact:** Name resolution broken, AD operations fail
**Frequency:** Medium

**Symptoms:**
- Cannot resolve hostnames
- DNS queries timing out
- Event ID 4011 in DNS log
- Applications failing to connect

**Root Causes:**
- DNS Server service stopped
- Corrupt DNS cache
- Zone configuration error
- DNS forwarder unreachable
- Firewall blocking UDP 53

**Resolution:**
```powershell
# Flush DNS cache
ipconfig /flushdns

# Restart DNS service
Restart-Service -Name DNS

# Test resolution
Resolve-DnsName <hostname>

# Check DNS server status
Get-Service -Name DNS
```

### 3. Active Directory Replication Failure

**Severity:** Critical
**Impact:** Inconsistent AD state, login failures
**Frequency:** Medium

**Symptoms:**
- Repadmin shows errors
- Event ID 1006 or 1083
- Users cannot login with recent changes
- GPO updates not applied

**Root Causes:**
- Network connectivity between DCs
- DNS misconfiguration
- KDC service failure
- Database corruption
- Time sync issues

**Resolution:**
```powershell
# Check replication status
Repadmin /showrepl

# Force replication
Repadmin /syncall /AdeP

# Check time sync
w32tm /query /status

# Verify DC health
Test-ComputerSecureChannel
```

### 4. Disk Space Exhaustion

**Severity:** Critical
**Impact:** System instability, service failures
**Frequency:** Medium

**Symptoms:**
- Event ID 2019 (low disk space)
- Applications failing to write
- Windows Update unable to install
- Eventual system crash

**Root Causes:**
- Log files growing unbounded
- Temp file accumulation
- Windows Update cache
- No growth monitoring

**Resolution:**
```powershell
# Check volume status
Get-Volume

# Find large files
Get-ChildItem C:\ -Recurse -ErrorAction SilentlyContinue | Sort-Object Length -Descending | Select-Object -First 20

# Clean Windows Update cache
Stop-Service -Name wuauserv
Remove-Item C:\Windows\SoftwareDistribution\Download\* -Recurse
Start-Service -Name wuauserv

# Clean component store
DISM /Online /Cleanup-Image /StartComponentCleanup
```

### 5. IIS Failure

**Severity:** High
**Impact:** Web services unavailable
**Frequency:** Medium

**Symptoms:**
- Website returns 502/503
- W3SVC service stopped
- Application pool crashed
- Event ID 1000 or 1001

**Root Causes:**
- Application pool crash
- SSL certificate expired
- Configuration error
- Port conflict
- Resource exhaustion

**Resolution:**
```powershell
# Check IIS service
Get-Service -Name W3SVC

# Restart IIS
iisreset

# Restart application pool
Restart-WebAppPool -Name <pool-name>

# Check sites
Get-WebSite
```

### 6. PowerShell Execution Policy Block

**Severity:** Medium
**Impact:** Scripts cannot run
**Frequency:** High

**Symptoms:**
- "Cannot be loaded because running scripts is disabled"
- Execution policy shows "Restricted"
- Script won't execute

**Root Causes:**
- Execution policy set to Restricted
- Script not digitally signed
- Group Policy blocking scripts

**Resolution:**
```powershell
# Check current policy
Get-ExecutionPolicy

# Set to RemoteSigned
Set-ExecutionPolicy RemoteSigned -Scope LocalMachine

# Verify
Get-ExecutionPolicy
```

### 7. Windows Update Failure

**Severity:** Medium
**Impact:** Missing security patches
**Frequency:** Medium

**Symptoms:**
- Event ID 1058 in System log
- Update stuck at "Installing"
- Update rollbacks on restart

**Root Causes:**
- Corrupt SoftwareDistribution folder
- Missing prerequisites
- Conflicting updates
- WMI service issues

**Resolution:**
```powershell
# Stop update services
Stop-Service -Name wuauserv
Stop-Service -Name bits

# Remove corrupt cache
Remove-Item C:\Windows\SoftwareDistribution\* -Recurse

# Restart services
Start-Service -Name wuauserv
Start-Service -Name bits

# Run DISM repair
DISM /Online /Cleanup-Image /RestoreHealth
```

### 8. Windows Performance Degradation

**Severity:** Medium
**Impact:** System responsiveness reduced
**Frequency:** High

**Symptoms:**
- Slow application response
- High CPU or memory usage
- Slow disk I/O
- Frequent paging

**Root Causes:**
- Resource exhaustion
- Memory leaks
- Disk bottleneck
- Too many services running
- Malware or unwanted processes

**Resolution:**
```powershell
# Check resource usage
Get-Process | Sort-Object WorkingSet64 -Descending | Select-Object -First 10
Get-CimInstance -ClassName Win32_Processor | Select-Object LoadPercentage

# Check memory
Get-CimInstance -ClassName Win32_OperatingSystem | Select-Object FreePhysicalMemory, TotalVisibleMemorySize

# Identify top consumers
Get-Counter '\Processor(_Total)\% Processor Time'
Get-Counter '\Memory\Pages/sec'
```

## Failure Matrix

| Failure | Severity | Frequency | Auto-Recovery | Manual Intervention |
|---------|----------|-----------|---------------|-------------------|
| Service Failure | Critical | High | No | Service restart |
| DNS Resolution | High | Medium | Partial | DNS restart/flush |
| AD Replication | Critical | Medium | No | Repadmin fix |
| Disk Full | Critical | Medium | No | Space cleanup |
| IIS Failure | High | Medium | Partial | IIS restart |
| PowerShell Block | Medium | High | No | Policy change |
| Update Failure | Medium | Medium | Partial | Cache cleanup |
| Performance | Medium | High | No | Resource fix |

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial common failures catalog |