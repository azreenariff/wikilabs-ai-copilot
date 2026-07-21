# Windows Engineering Foundation

## Architecture

### Windows Architecture Layers

**Hardware Abstraction Layer (HAL)**
- Abstracts hardware differences from the kernel
- Provides standard interfaces for hardware access

**Kernel Layer**
- **Windows NT Kernel:** Microkernel hybrid, handles memory management, process scheduling, interrupt handling
- **Executive Services:** Object manager, security reference monitor, I/O manager, process/thread manager
- **Kernel-Mode Drivers:** Device drivers running in ring 0 (highest privilege)

**User-Mode Subsystem**
- **Win32 API:** Primary application interface
- **Subsys processes:** csrss.exe (client-server runtime), winlogon.exe (login manager)
- **Services Control Manager:** Manages Windows services (equivalent to systemd)

**Applications**
- GUI applications, console applications, services, UWP apps

### Key Components

| Component | Purpose |
|-----------|---------|
| Windows Kernel (ntoskrnl.exe) | Core OS functionality |
| Win32k.sys | Window manager, GDI |
| Services Control Manager | Service lifecycle management |
| Event Log Service | System and application logging |
| DHCP Client | Network address assignment |
| DNS Client | Name resolution |
| Windows Firewall | Packet filtering |
| Windows Update | Patch management |
| Windows Defender | Antivirus/security |
| Task Scheduler | Scheduled task execution |

---

## Core Concepts

### Services Model
- **Services:** Background processes managed by SCM (Services Control Manager)
- **Service Types:** 
  - Interactive (has desktop session)
  - Non-interactive (no desktop session)
  - Share (shares process with other services)
- **Startup Types:** Automatic, Automatic (Delayed Start), Manual, Disabled
- **Service Dependencies:** Services can depend on other services starting first

### Process Model
- **Process ID (PID):** Unique identifier
- **Thread:** Lightweight execution unit within a process
- **Session:** User login session (Session 0 = services, Session 1+ = interactive users)
- **Privilege Levels:** User (standard), Administrator (elevated)
- **UAC (User Account Control):** Prompts for elevation when admin privileges needed

### Registry
- **Hierarchy:** HKCR, HKCU, HKLM, HKU, HKCC
- **Data Types:** REG_SZ (string), REG_DWORD (32-bit), REG_QWORD (64-bit), REG_BINARY, REG_MULTI_SZ
- **Purpose:** Configuration storage, service settings, application preferences
- **Warning:** Registry edits can break the system — always backup before changes

### File System
- **NTFS:** Primary file system with ACLs, encryption (EFS), compression, quotas
- **FAT32/ExFAT:** Older formats, limited permissions
- **ReFS:** Resilient file system (Server 2012+), checksums, self-healing
- **FHS vs NTFS:** Windows doesn't follow FHS — uses drive letters (C:\, D:\)

### Event Logging
- **Event Logs:** Application, Security, Setup, System, Forwarded Events, PowerShell
- **Event Levels:** Error, Warning, Information, Verbose, Critical
- **Event IDs:** Numeric identifiers for specific events
- **Event Viewer:** GUI and PowerShell (Get-WinEvent) tools for log analysis

### Active Directory Fundamentals
- **Domain:** Group of objects (users, computers, groups) managed as a unit
- **Domain Controller:** Server running AD DS role
- **Forest:** Collection of one or more domains with shared schema
- **OU (Organizational Unit):** Container for organizing objects
- **GPO (Group Policy Object):** Configuration settings applied to users/computers
- **DNS:** AD relies heavily on DNS for service location

---

## Common Components

### Essential Tools
| Tool | Purpose |
|------|---------|
| Event Viewer | Log analysis |
| Task Manager | Process/resource monitoring |
| Performance Monitor (PerfMon) | Detailed performance metrics |
| Resource Monitor | Real-time resource usage |
| Services (services.msc) | Service management |
| Registry Editor (regedit) | Registry configuration |
| Disk Management | Disk/volume management |
| Device Manager | Hardware/driver management |
| Computer Management | MMC snap-in collection |
| Group Policy Editor | GPO management |

### PowerShell
- **Cmdlet naming:** Verb-Noun pattern (Get-Service, Stop-Process, Set-Item)
- **Providers:** Registry, FileSystem, Certificate, Variable
- **Pipeline:** Cmdlets pass objects (not text) between each other
- **Splatting:** Pass parameters via hash table for readability
- **Remoting:** Invoke-Command, Enter-PSSession for remote management

### Networking Stack
- **TCP/IP:** Primary network protocol
- **Winsock (WSA):** Windows sockets API for networking
- **DNS Client:** Name resolution (hosts file, DNS servers, NetBIOS)
- **DHCP Client:** Automatic IP assignment
- **Windows Firewall:** Stateful packet inspection
- **ICS:** Internet Connection Sharing

### Security Model
- **SID (Security Identifier):** Unique ID for every user/group
- **ACL (Access Control List):** Permissions on objects
- **ACE (Access Control Entry):** Individual allow/deny entries
- **Authentication:** NTLM, Kerberos, Negotiate
- **Authorization:** DACL ( discretionary ACL), SACL (system ACL)
- **Local Security Policy:** Local account policies, audit policies

---

## Common Failures

### Service Failures
| Symptom | Possible Cause |
|---------|----------------|
| Service fails to start | Dependency not running, incorrect binary path, permissions |
| Service stops unexpectedly | Application crash, memory leak, dependency failure |
| Service starts then stops | Misconfigured startup parameters, conflicting software |
| Service stuck "starting" | Timeout, deadlock, hanging dependency |

### Performance Issues
| Symptom | Possible Cause |
|---------|----------------|
| System slow | High disk usage, insufficient RAM, too many startup programs |
| Blue Screen (BSOD) | Driver failure, hardware failure, memory corruption |
| Application crashes | Missing dependencies, incompatible software, memory issues |
| High CPU usage | Malware, runaway process, Windows Update |
| High disk usage | Page file, hibernation file, Windows Search indexing |

### Network Failures
| Symptom | Possible Cause |
|---------|----------------|
| No internet access | DHCP failure, DNS misconfigured, firewall blocking |
| Can't resolve DNS | DNS server unreachable, hosts file override |
| Can ping but can't connect | Firewall, port blocked, service not listening |
| Slow network | Network congestion, bandwidth throttling, faulty hardware |

### Authentication Failures
| Symptom | Possible Cause |
|---------|----------------|
| Can't log in | Wrong password, account locked, expired credentials |
| AD connection fails | DNS misconfigured, time sync issue, DC unreachable |
| Kerberos ticket fails | Time skew >5 minutes, wrong SPN, clock sync issue |
| Permission denied | Insufficient ACL permissions, group membership missing |

---

## Troubleshooting Philosophy

### Evidence Collection Priority
1. **Event Viewer** — System and Application logs for errors/warnings
2. **Task Manager** — Real-time process and resource usage
3. **Performance Monitor** — Historical performance data (if counters configured)
4. **Service status** — Running/stopped/dead services
5. **Disk space** — Available space on all volumes
6. **Network configuration** — IP, gateway, DNS, adapter status

### Diagnostic Commands

```powershell
# Event logs
Get-EventLog -LogName System -Newest 20 -EntryType Error
Get-WinEvent -LogName "Application" -MaxEvents 50 | Where-Object {$_.Level -eq 2}

# Process and resource
Get-Process | Sort-Object CPU -Descending | Select-Object -First 10
Get-Process | Sort-Object WorkingSet -Descending | Select-Object -First 10

# Service status
Get-Service | Where-Object {$_.Status -ne "Running"}
Get-Service -Name <service> | Format-List *

# Network
Get-NetIPAddress
Get-NetAdapter
Test-NetConnection <host> -Port <port>
Resolve-DnsName <domain>

# Disk
Get-PSDrive | Where-Object {$_.Free -lt 1GB}
Get-Volume

# Memory
Get-CimInstance -ClassName Win32_OperatingSystem | Select-Object TotalVisibleMemorySize, FreePhysicalMemory
```

### Troubleshooting Flow
1. **Identify** — What's broken? When did it start?
2. **Observe** — Check Event Viewer, Task Manager, services
3. **Hypothesize** — What's the most likely cause?
4. **Validate** — Test hypothesis with targeted checks
5. **Remediate** — Apply fix based on confirmed cause
6. **Verify** — Confirm resolution, monitor for recurrence

---

## Best Practices

### System Administration
- **Monitor Event Logs** — Set up alerts for critical and error events
- **Regular updates** — Apply Windows Update patches on schedule
- **Backup registry** — Before any registry changes
- **Document GPOs** — Track Group Policy changes and their impact
- **Use PowerShell** — For automation and consistency
- **Least privilege** — Don't run as Administrator unless necessary
- **Service accounts** — Dedicated accounts for services, not user accounts
- **Document configurations** — Keep records of changes made

### Security
- **UAC enabled** — Never disable User Account Control
- **Windows Defender** — Keep real-time protection enabled
- **Windows Firewall** — Configure explicit allow rules, not open ports
- **Regular audits** — Review local users, groups, permissions
- **Patch management** — Stay current on security updates
- **Disable unnecessary services** — Reduce attack surface
- **Audit policies** — Enable security event logging
- **BitLocker** — Enable drive encryption on laptops and servers

### Performance
- **Baseline performance** — Know normal CPU, memory, disk usage
- **Disable startup programs** — Reduce boot time and memory usage
- **Manage page file** — Ensure adequate page file size
- **Disk maintenance** — Regular defragmentation (HDD only), SSD TRIM
- **Windows Search** — Tune indexing for performance
- **Power plan** — Use "High performance" for servers, "Balanced" for workstations

---

## Risk Awareness

### High-Risk Operations
| Operation | Risk Level | Warning |
|-----------|------------|---------|
| Registry editing | Critical | Can break Windows, always backup first |
| Modifying system services | High | Can prevent boot or critical functionality |
| Modifying firewall rules | High | Can expose system to attacks |
| Disabling Windows Update | High | Security vulnerabilities |
| Deleting system files | Critical | System instability or failure |
| UAC bypass | Critical | Security compromise |
| Domain controller changes | Critical | Can break entire AD forest |

### Safe Operations
- Viewing logs (Event Viewer, Get-WinEvent)
- Checking service status (Get-Service)
- Checking disk space (Get-Volume, Get-PSDrive)
- Checking network status (Get-NetIPAddress, Test-NetConnection)
- Restarting non-critical services
- Clearing cache/temp files
- Running Windows Update

---

## References

- [Microsoft Windows Server Documentation](https://learn.microsoft.com/en-us/windows-server/)
- [PowerShell Documentation](https://learn.microsoft.com/en-us/powershell/)
- [Windows Event IDs Reference](https://learn.microsoft.com/en-us/windows/win32/wes/event-log-entry-levels)
- [Active Directory Documentation](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/get-started/virtual-dc/active-directory-domain-services-overview)
- [Windows Security Documentation](https://learn.microsoft.com/en-us/windows/security/)

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial Windows Engineering Foundation |