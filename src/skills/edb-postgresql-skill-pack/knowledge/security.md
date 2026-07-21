# EDB PostgreSQL Security Configuration

## Overview

EDB PostgreSQL security configuration covers authentication, authorization, network security, encryption, and compliance.

## Authentication

### Authentication Methods

| Method | Description | Use Case |
|--------|-------------|----------|
| **md5/scram-sha-256** | Password-based | Standard authentication |
| **certificate** | SSL certificates | High-security environments |
| **ldap** | LDAP directory | Enterprise integration |
| **kerberos** | Kerberos SSO | Enterprise SSO |
| **pam** | PAM modules | System integration |

### Authentication Configuration

```ini
# pg_hba.conf
host    all             all             10.0.0.0/8            scram-sha-256
hostssl all             all             10.0.0.0/8            cert
local   all             all             peer
```

## Authorization

### Role-Based Access Control

| Role | Permissions |
|------|-------------|
| **superuser** | Full system access |
| **db_owner** | Database administration |
| **db_user** | Read/write access |
| **db_readonly** | Read-only access |
| **replication** | Replication access |

### Grant Examples

```sql
-- Create role
CREATE ROLE app_user WITH LOGIN PASSWORD 'secure_password';

-- Grant privileges
GRANT CONNECT ON DATABASE mydb TO app_user;
GRANT USAGE ON SCHEMA public TO app_user;
GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO app_user;

-- Revoke excessive privileges
REVOKE ALL ON ALL TABLES IN SCHEMA public FROM PUBLIC;
```

## Network Security

### SSL/TLS Configuration

```ini
# postgresql.conf
ssl = on
ssl_cert_file = 'server.crt'
ssl_key_file = 'server.key'
ssl_ca_file = 'ca.crt'
```

### Firewall Rules

```bash
# Allow PostgreSQL port from trusted networks only
iptables -A INPUT -p tcp --dport 5432 -s 10.0.0.0/8 -j ACCEPT
iptables -A INPUT -p tcp --dport 5432 -j DROP
```

## Data Protection

### Encryption

| Type | Description | Configuration |
|------|-------------|---------------|
| **At Rest** | pgcrypto, TDE | Column-level encryption |
| **In Transit** | SSL/TLS | ssl = on |
| **Backup** | Encrypted backups | pg_dump with encryption |

### Audit Logging

```ini
# postgresql.conf
logging_collector = on
log_directory = 'log'
log_filename = 'postgresql-%Y-%m-%d.log'
log_statement = 'all'
log_min_duration_statement = 1000
```

## Compliance

### Security Checklist

1. Use scram-sha-256 authentication
2. Enable SSL for all connections
3. Implement strong password policies
4. Regular role access reviews
5. Enable audit logging
6. Regular security patches
7. Encrypt sensitive data
8. Follow least privilege principle

## References

- EDB PostgreSQL Security: https://www.enterprisedb.com/docs/
- PostgreSQL Security: https://www.postgresql.org/docs/current/security.html
- PostgreSQL Authentication: https://www.postgresql.org/docs/current/auth-methods.html