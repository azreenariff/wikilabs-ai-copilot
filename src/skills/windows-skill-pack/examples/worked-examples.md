# Windows Engineering — Worked Examples

## Example 1: IIS Not Serving Websites

### Scenario
After a Windows Server update, IIS stops serving web content. The website returns 502 Bad Gateway.

### Symptoms
- Website returns 502 Bad Gateway or connection refused
- W3SVC service shows as stopped
- Application pools not starting

### Evidence
```powershell
# Check IIS service status
PS C:\> Get-Service W3SVC

Name               Status    DisplayName
----               ------    -----------
W3SVC              Stopped   World Wide Web Publishing Service

# Check Event Log
PS C:\> Get-EventLog -LogName System -EntryType Error -Newest 10 | Where-Object {$_.Source -like "*W3SVC*"}

TimeWritten         EntryType Source       EventID Message
-----------         --------- ------       ------- -------
7/21/2026 10:15AM   Error     W3SVC        1000    The World Wide Web Publishing Service failed due to...
7/21/2026 10:15AM   Error     Service Control Manager 7024  The World Wide Web Publishing Service terminated...

# Check application pools
PS C:\> Get-WebAppPoolState

Name                      State
----                      -----
DefaultAppPool            Stopped
MyAppPool                 Stopped
```

### Analysis
The W3SVC service is stopped. Event ID 1000 indicates an application crash. The application pools are also stopped, suggesting a broader IIS issue. This is common after Windows updates that require service restart.

### Resolution
```powershell
# 1. Restart IIS service
PS C:\> net start W3SVC
The World Wide Web Publishing Service service is starting.
The World Wide Web Publishing Service service was started successfully.

# 2. If net start fails, try iisreset
PS C:\> iisreset
Attempting stop...
Internet services successfully stopped
Attempting restart...
Internet services successfully restarted

# 3. Verify sites are running
PS C:\> Get-WebSite

Name      ID State  PhysicalPath
----      -- -----  ------------
Default Web Site  1 Running C:\inetpub\wwwroot
MyAppSite         2 Running C:\inetpub\myapp
```

### Verification
```powershell
PS C:\> Get-Service W3SVC

Name               Status    DisplayName
----               ------    -----------
W3SVC              Running   World Wide Web Publishing Service

PS C:\> curl http://localhost  # Test HTTP access
StatusCode: 200
StatusDescription: OK
Content: <html>...</html>
```

### Lessons
- Always restart services after Windows updates
- Use iisreset as a comprehensive restart method
- Check Event Viewer for specific error messages
- Test website accessibility after restart

---

## Example 2: Active Directory Replication Failure

### Scenario
A secondary Domain Controller is not replicating changes from the primary DC.

### Symptoms
- Users on secondary DC cannot login with recent changes
- Group Policy updates not applied on secondary DC
- Replication status shows errors in AD Replication Status

### Evidence
```powershell
# Check replication status
PS C:\> Repadmin /showrepl

Default-First-Site-Name\DC02
  DSA Options: IS_GC
  Product Version: 10.0.17763
  OS Version: 10.0.17763.0

  KCC Connection Object:
    Source: DC01
    DSA Object: CN=NTDS Settings,...
    Connection Status: FAILED
    Last error: 1753 (The directory service is unavailable)

# Check DNS resolution
PS C:\> nslookup DC01.example.com
Server:  DC01.example.com
Address:  192.168.1.10
Name:     DC01.example.com
Address:  192.168.1.10

# Check domain controller status
PS C:\> Get-ADDomainController
Name             ReplicaPartition   Domain               OperationMasterRoles
----             ----------------   ------               ----------------------
DC01             DomainDnsZones     example.com          PDC, RID, Infrastructure
DC02             DomainDnsZones     example.com          (none)
```

### Analysis
Replication is failing from DC01 to DC02 with error 1753 (directory service unavailable). DNS resolution works, so the issue is at the AD service level on DC01. The PDC Emulator role is on DC01, so all time sync depends on it.

### Resolution
```powershell
# 1. Check if KDC service is running on DC01
PS C:\> Get-Service Kdc, Netlogon, NTDS -ComputerName DC01

# 2. Restart AD DS service on DC01 (if accessible)
PS C:\> Invoke-Command -ComputerName DC01 -ScriptBlock { Restart-Service NTDS }

# 3. Force replication
PS C:\> repadmin /syncall /AdeP

# 4. Verify replication
PS C:\> repadmin /showrepl DC02
```

### Verification
```powershell
# Check replication status
PS C:\> Repadmin /showrepl

Default-First-Site-Name\DC02
  Last attempt at <DC01>'s change was successful.
  Last replication result: 0 (The operation completed successfully)

# Verify FSMO roles
PS C:\> Get-ADDomainController | Format-Table Name, OperationMasterRoles
```

### Lessons
- Regularly monitor AD replication with repadmin
- Ensure both DCs have identical time (PDC is NTP source)
- Check DNS first — replication requires DNS resolution
- Keep redundancy in FSMO roles planning

---

## Example 3: Disk C: is Nearly Full

### Scenario
The C: drive on a file server is at 95% capacity, threatening system stability.

### Symptoms
- Low disk space warnings
- Applications failing to write logs
- Windows Update unable to install

### Evidence
```powershell
# Check volume status
PS C:\> Get-Volume

DriveLetter FriendlyName FileSystemType HealthStatus SizeRemaining Size
----------- ------------ -------------- ------------ ----------- ----
C:                      NTFS           Healthy      5.2 GB      100 GB
D:        Data          NTFS           Healthy      250 GB      500 GB

# Check large files
PS C:\> Get-ChildItem C:\ -Recurse -ErrorAction SilentlyContinue | Sort-Object Length -Descending | Select-Object -First 10 | Format-Table FullName, Length -AutoSize

FullName                                                  Length
--------                                                  ------
C:\Windows\Temp\large_file.tmp                           8.5 GB
C:\Windows\SoftwareDistribution\Download\...             4.2 GB
C:\ProgramData\Microsoft\Windows\WER\...                 2.1 GB
...

# Check Windows Update cache
PS C:\> Get-ChildItem C:\Windows\SoftwareDistribution\Download | Measure-Object -Property Length -Sum
Count    : 15
Average  :
Sum      : 4200000000
Maximum  :
Minimum  :
Property :
```

### Analysis
The C: drive is nearly full due to accumulated temp files and Windows Update download cache. The SoftwareDistribution folder has over 4GB of update files that may be stale.

### Resolution
```powershell
# 1. Stop Windows Update service
PS C:\> net stop wuauserv

# 2. Clear Update cache
PS C:\> Remove-Item C:\Windows\SoftwareDistribution\Download\* -Recurse -Force

# 3. Clean temp files
PS C:\> Remove-Item C:\Windows\Temp\* -Recurse -Force

# 4. Clean component store
PS C:\> Dism /Online /Cleanup-Image /StartComponentCleanup

# 5. Restart Windows Update service
PS C:\> net start wuauserv

# 6. Clear DNS cache
PS C:\> ipconfig /flushdns

# 7. Empty Recycle Bin
PS C:\> Clear-RecycleBin -Force
```

### Verification
```powershell
# Check remaining space
PS C:\> Get-Volume

DriveLetter FriendlyName FileSystemType HealthStatus SizeRemaining Size
----------- ------------ -------------- ------------ ----------- ----
C:                      NTFS           Healthy      25.3 GB     100 GB
D:        Data          NTFS           Healthy      250 GB      500 GB
```

### Lessons
- Implement regular disk cleanup schedules
- Monitor disk space with alerts at 80% and 90%
- Configure log rotation for application logs
- Consider moving Windows Update cache to D: drive

---

## Example 4: PowerShell Script Fails Due to Execution Policy

### Scenario
A user tries to run a signed PowerShell script but gets blocked by execution policy.

### Symptoms
- "Cannot be loaded because running scripts is disabled on this system"
- Script won't execute
- User reports "I can't run any scripts"

### Evidence
```powershell
# Check current execution policy
PS C:\> Get-ExecutionPolicy
Restricted

# Check execution policy for all scopes
PS C:\> Get-ExecutionPolicy -List

Scope       ExecutionPolicy
-----       ---------------
MachinePolicy       Undefined
UserPolicy          Undefined
Process             Restricted
CurrentUser         Undefined
LocalMachine        Restricted

# Try running script
PS C:\> .\deploy.ps1
.\deploy.ps1 : File C:\scripts\deploy.ps1 cannot be loaded. The file C:\scripts\deploy.ps1 is not digitally signed.
The script is not allowed by the execution policy.
```

### Analysis
The LocalMachine execution policy is set to "Restricted", which blocks all scripts. The script is not digitally signed, so even "RemoteSigned" would block it without proper signing.

### Resolution
```powershell
# 1. Set execution policy to RemoteSigned (allows local scripts, requires signature for downloaded)
PS C:\> Set-ExecutionPolicy RemoteSigned -Scope LocalMachine
Execution Policy Change
The execution policy helps protect you from scripts that you do not trust.
Changing the execution policy might expose you to the security risks described...
Do you want to change the execution policy?
[Y] Yes  [A] Yes to All  [N] No  [L] No to All  [S] Suspend  [?] Help: default is "N"
Y

# 2. Verify the change
PS C:\> Get-ExecutionPolicy
RemoteSigned

# 3. Run the script
PS C:\> .\deploy.ps1
[Script runs successfully]
```

### Verification
```powershell
PS C:\> Get-ExecutionPolicy -List

Scope       ExecutionPolicy
-----       ---------------
MachinePolicy       Undefined
UserPolicy          Undefined
Process             Undefined
CurrentUser         Undefined
LocalMachine        RemoteSigned
```

### Lessons
- Use "RemoteSigned" instead of "AllSigned" for most environments
- Sign scripts for production deployments
- Document execution policy configuration in security baseline
- Consider using Code Signing Certificates for script authentication

---

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial worked examples |