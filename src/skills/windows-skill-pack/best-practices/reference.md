# Windows Engineering — Best Practices

## General Best Practices

### Service Management
1. **Document all services and their dependencies** — Use `sc qc` to document configurations
2. **Set appropriate startup types** — Automatic for critical, Manual for on-demand
3. **Configure recovery actions** — Restart service on first failure
4. **Use dedicated service accounts** — Never run services as LocalSystem if avoidable
5. **Monitor service health** — Use Event Viewer and performance counters

### PowerShell
1. **Follow verb-noun naming convention** — Get-Service, Set-Service, Start-Service
2. **Use functions for reusable logic** — Encapsulate common operations
3. **Validate input** — Use Parameter validation attributes
4. **Error handling** — Use Try/Catch/Finally blocks
5. **Logging** — Write to Event Log or file for audit trail
6. **Execution policy** — Use RemoteSigned for most environments

### Active Directory
1. **Regular replication checks** — Use `repadmin /showrepl`
2. **Monitor FSMO roles** — Know which DC holds each role
3. **Test GPOs before deployment** — Use gpresult /r for validation
4. **Document domain functional levels** — Track upgrade paths
5. **Backup AD regularly** — System State backups from DCs

### Security
1. **Regular Windows Updates** — Keep patches current
2. **Configure auditing** — Enable security event logging
3. **Least privilege principle** — Users have minimal required access
4. **Regular password audits** — Check for expired/weak passwords
5. **Network segmentation** — Separate management from production

### Disk Management
1. **Monitor disk space** — Alert at 80% capacity
2. **Regular cleanup** — Remove temp files, old logs, update cache
3. **Plan for growth** — Leave 20% headroom on volumes
4. **Document volume layout** — Which data is on which drive
5. **Test backups** — Verify restore capability

## Windows Server Specific

### Domain Controller
1. **Minimum 2 DCs** — Always for redundancy
2. **Spread FSMO roles** — Know who holds each role
3. **Regular SYSVOL replication check** — Use DFS-R, not FRS
4. **Time sync** — Ensure PDC Emulator syncs to reliable NTP source
5. **Database backup** — Regular System State backups

### Web Server (IIS)
1. **Regular application pool recycling** — Prevent memory leaks
2. **SSL certificates monitoring** — Alert 30 days before expiry
3. **Log rotation** — Manage IIS log file sizes
4. **Security headers** — Configure proper HTTP security headers
5. **Performance monitoring** — Track requests/sec, active connections

### File Server
1. **NTFS permissions review** — Regular access audits
2. **DFS replication monitoring** — Check replication status
3. **Shadow Copies** — Enable for user data recovery
4. **Quotas** — Set disk quotas for user profiles
5. **Backup verification** — Regular restore testing

### PowerShell Best Practices
1. **Use advanced functions** — Parameter validation, pipeline support
2. **Module development** — Use proper module manifests
3. **Comment-based help** — Include help documentation
4. **Type acceleration** — Use appropriate .NET types
5. **Remote management** — Use Invoke-Command, PSSession

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial best practices |