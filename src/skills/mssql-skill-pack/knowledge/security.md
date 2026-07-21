# MSSQL Security Configuration

## Overview

Microsoft SQL Server security configuration covers authentication, authorization, network security, encryption, and compliance.

## Authentication

### Authentication Methods

| Method | Description | Use Case |
|--------|-------------|----------|
| **Windows Authentication** | Kerberos/NTLM | Domain environments |
| **SQL Server Authentication** | Username/password | Mixed mode |
| **Azure AD Authentication** | Azure identity | Cloud integration |
| **Integrated Authentication** | OAuth2/OIDC | Modern apps |

### Authentication Configuration

```sql
-- Check authentication mode
EXEC sp_configure 'show advanced options', 1;
RECONFIGURE;
EXEC sp_configure 'xp_cmdshell';
```

## Authorization

### Role-Based Access Control

| Role | Permissions |
|------|-------------|
| **sysadmin** | Full system access |
| **db_owner** | Database administration |
| **db_datareader** | Read access |
| **db_datawriter** | Write access |
| **db_ddladmin** | DDL access |

### Grant Examples

```sql
-- Create user
CREATE LOGIN app_user WITH PASSWORD = 'secure_password';
GO
USE mydb;
GO
CREATE USER app_user FOR LOGIN app_user;
GO

-- Grant privileges
GRANT SELECT, INSERT, UPDATE ON schema.table TO app_user;
GO
```

## Network Security

### SSL/TLS Configuration

```sql
-- Configure SSL certificate
CREATE SERVER CERTIFICATE ssl_cert
    WITH SUBJECT = 'SQL Server SSL Certificate';
GO

-- Force encryption
EXEC sp_configure 'show advanced options', 1;
RECONFIGURE;
EXEC sp_configure 'force encryption', 1;
RECONFIGURE;
```

### Firewall Rules

```powershell
# Allow SQL Server port
New-NetFirewallRule -DisplayName "SQL Server" -Direction Inbound -LocalPort 1433 -Protocol TCP -Action Allow
```

## Data Protection

### Encryption

| Type | Description | Configuration |
|------|-------------|---------------|
| **TDE** | Transparent Data Encryption | COLUMN_ENCRYPTION |
| **Always Encrypted** | Column-level encryption | ALWAYS_ENCRYPTED |
| **Cell-Level** | Application-level encryption | ENCRYPTED BY |
| **Backup Encryption** | Encrypted backups | BACKUP ENCRYPTION |

### Audit Logging

```sql
-- Create server audit
CREATE SERVER AUDIT SQLAudit
TO FILE (FILEPATH = 'C:\Audit\');
GO

-- Enable audit
ALTER SERVER AUDIT SQLAudit WITH (STATE = ON);
GO
```

## Compliance

### Security Checklist

1. Use Windows Authentication where possible
2. Enable TDE for sensitive data
3. Implement strong password policies
4. Regular role access reviews
5. Enable audit logging
6. Regular security patches
7. Encrypt backups securely
8. Follow least privilege principle

## References

- SQL Server Security: https://learn.microsoft.com/en-us/sql/
- SQL Server Authentication: https://learn.microsoft.com/en-us/sql/
- SQL Server Encryption: https://learn.microsoft.com/en-us/sql/