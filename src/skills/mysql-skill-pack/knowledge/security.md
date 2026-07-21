# MySQL Security

## Overview

MySQL security encompasses authentication, authorization, encryption, and audit capabilities. Implementing comprehensive security measures protects data integrity, confidentiality, and availability. This document covers user account management, authentication plugins, password policies, privilege models, roles, encryption at rest and in transit, key management, audit logging, network security, and production hardening.

## Authentication

### User Account Management

```sql
-- Create user with specific host
CREATE USER 'appuser'@'192.168.1.%' IDENTIFIED BY 'secure_password';

-- Grant privileges
GRANT SELECT, INSERT, UPDATE ON mydb.* TO 'appuser'@'192.168.1.%';

-- Apply changes
FLUSH PRIVILEGES;

-- Check user authentication method
SELECT user, host, plugin, account_locked, password_expired FROM mysql.user;

-- Create user with multiple host patterns
CREATE USER 'admin'@'localhost' IDENTIFIED BY 'admin_password';
CREATE USER 'admin'@'10.0.0.%' IDENTIFIED BY 'admin_password';
```

**User Account Best Practices**:

| Practice | Command | Rationale |
|----------|---------|-----------|
| Specific host restriction | `@'10.0.0.%'` | Prevent connections from unauthorized hosts |
| Minimum privilege grant | `GRANT SELECT, INSERT ON db.*` | Principle of least privilege |
| Disable unused accounts | `ACCOUNT LOCK` | Reduce attack surface |
| Regular credential audit | `SELECT user, host, plugin FROM mysql.user` | Verify account state |
| Remove anonymous users | `DROP USER ''@'localhost'` | Eliminate unauthorized access |

### Authentication Plugins

| Plugin | Description | Security Level | Compatibility |
|--------|-------------|---------------|--------------|
| `caching_sha2_password` | Default in MySQL 8.0, SHA-256 hashing with client-side caching | High | Most modern drivers (8.0+) |
| `mysql_native_password` | Legacy MySQL password hashing (MD5) | Medium | All drivers including older versions |
| `sha256_password` | SHA-256 with mandatory SSL | High | Drivers with SSL support |
| `auth_socket` | Unix socket authentication (no password) | Very High | Only local Unix socket connections |
| `connect_encrypted` | Requires encrypted connection | High | SSL/TLS connections only |
| `windows` | Windows native authentication | Medium | Windows environments |
| `ldap_sasl` | LDAP SASL authentication | High | Directory integration |

**Recommendation**: Use `caching_sha2_password` as default for new users. Only use `mysql_native_password` for legacy client compatibility.

**Plugin Migration Strategy**:

```sql
-- Migrate user from native_password to caching_sha2_password
ALTER USER 'appuser'@'192.168.1.%'
  IDENTIFIED WITH caching_sha2_password BY 'new_secure_password';

-- Check all users and their plugins
SELECT user, host, plugin, authentication_string 
FROM mysql.user 
WHERE user NOT IN ('mysql.sys', 'mysql.session', 'mysql.infoschema');

-- Set global default authentication plugin
SET GLOBAL default_authentication_plugin = 'caching_sha2_password';
```

### Password Security

**Password Policy Configuration**:

```sql
-- Set password policy to STRONG
SET GLOBAL validate_password.policy = 'STRONG';
SET GLOBAL validate_password.length = 14;
SET GLOBAL validate_password.mixed_case_count = 2;
SET GLOBAL validate_password.number_count = 2;
SET GLOBAL validate_password.special_char_count = 2;
SET GLOBAL validate_password.numeral_count = 2;
SET GLOBAL validate_password.dictionary_file = '/etc/mysql/dictionary.txt';

-- Enable the password validation plugin
INSTALL COMPONENT 'file:///component_validate_password';
```

**Password Policy Levels**:

| Level | Description | Requirements |
|-------|-------------|-------------|
| LOW | No validation | Length >= 8 characters only |
| MEDIUM | Standard validation | Meets policy length, numeric, mixed case, special chars |
| STRONG | Strict validation | MEDIUM plus dictionary check, no common passwords |

**Password Rotation and Lifecycle**:

```sql
-- Set password expiration per user
ALTER USER 'appuser'@'192.168.1.%' PASSWORD EXPIRE INTERVAL 90 DAY;

-- Rotate password
ALTER USER 'appuser'@'192.168.1.%' IDENTIFIED BY 'new_password_2024';

-- Check password status
SELECT user, host, password_last_changed, password_lifetime, password_expired
FROM mysql.user WHERE user != 'mysql.sys';

-- Set global default password lifetime
SET GLOBAL default_password_lifetime = 90;

-- Disable password expiration for service accounts
ALTER USER 'svc_backup'@'localhost' PASSWORD EXPIRE NEVER;
```

**Password Expiration Policy**:

| User Type | Expiration | Retention | Reason |
|-----------|-----------|-----------|--------|
| Application | 90 days | 5 reuse | Automated rotation via secret management |
| Developer | 90 days | 3 reuse | Regular credential refresh |
| Admin | 60 days | 5 reuse | Higher security requirement |
| Service account | NEVER | N/A | Automated systems cannot rotate |
| Backup | 90 days | 3 reuse | Infrastructure access |

### Account Management

```sql
-- Lock inactive accounts
ALTER USER 'appuser'@'192.168.1.%' ACCOUNT LOCK;

-- Unlock accounts
ALTER USER 'appuser'@'192.168.1.%' ACCOUNT UNLOCK;

-- Set password history (prevent reuse)
SET GLOBAL password_history = 5;
SET GLOBAL password_reuse_interval = 365;

-- Set password rotation
SET GLOBAL default_password_lifetime = 90;

-- Drop inactive user
DROP USER 'old_user'@'%';

-- Rename user
RENAME USER 'old_name'@'host' TO 'new_name'@'host';
```

## Authorization

### Privilege Model

MySQL uses a hierarchical privilege system that operates at four levels:

| Scope | Examples | Syntax |
|-------|----------|--------|
| Global | *.* privileges for all databases | `GRANT SELECT ON *.*` |
| Database | db_name.* privileges | `GRANT SELECT ON mydb.*` |
| Table | db_name.table privileges | `GRANT SELECT ON mydb.users` |
| Column | Specific column privileges | `GRANT SELECT(id, name) ON mydb.users` |
| Routine | Stored procedure/function privileges | `GRANT EXECUTE ON PROCEDURE mydb.sp_calc` |

**Privilege Hierarchy**: Global > Database > Table > Column > Routine

**Key Privilege Categories**:

| Category | Privileges | Description |
|----------|-----------|-------------|
| Data Access | SELECT, INSERT, UPDATE, DELETE, SHOW DATABASES, LOCK TABLES | CRUD operations |
| Schema Management | CREATE, DROP, ALTER, INDEX, CREATE TEMPORARY TABLES, CREATE VIEW, SHOW VIEW | Structure changes |
| Administrative | PROCESS, RELOAD, SHUTDOWN, REPLICATION CLIENT, REPLICATION SLAVE, SUPER, FILE | Server operations |
| Security | GRANT OPTION | Grant privileges to others |
| Replication | REPLICATION CLIENT, REPLICATION SLAVE | Replication management |
| Data Loading | FILE, LOAD DATA | File-based data operations |

### Principle of Least Privilege

Grant only the minimum required privileges:

```sql
-- Read-only application user
GRANT SELECT ON mydb.* TO 'readonly_app'@'192.168.1.%';

-- Read-write application user
GRANT SELECT, INSERT, UPDATE, DELETE ON mydb.* TO 'rw_app'@'192.168.1.%';

-- User for specific table operations
GRANT SELECT, INSERT ON mydb.users TO 'insert_only'@'192.168.1.%';

-- Backup user (minimum required for mysqldump)
GRANT SELECT, RELOAD, LOCK TABLES, REPLICATION CLIENT, SHOW VIEW, EVENT, TRIGGER ON *.* TO 'backup_user'@'192.168.1.%';

-- Monitoring user (minimum for performance_schema access)
GRANT SELECT ON performance_schema.* TO 'monitor_user'@'192.168.1.%';
GRANT SELECT ON sys.* TO 'monitor_user'@'192.168.1.%';
```

**Privilege Review Schedule**:

| Frequency | Review | Scope |
|-----------|--------|-------|
| Monthly | GRANT audit | All active users |
| Quarterly | Privilege drift check | Compare actual vs expected grants |
| Annually | Full access review | All accounts, roles, privileges |

### Role Management

MySQL 8.0 introduced full role support for group-based privilege management.

**Creating and Managing Roles**:

```sql
-- Create roles for different access levels
CREATE ROLE 'app_reader', 'app_writer', 'app_admin';

-- Assign privileges to roles
GRANT SELECT ON mydb.* TO 'app_reader';
GRANT SELECT, INSERT, UPDATE, DELETE ON mydb.* TO 'app_writer';
GRANT ALL PRIVILEGES ON mydb.* TO 'app_admin';

-- Create a role for database administrators
CREATE ROLE 'dba_role';
GRANT SELECT, INSERT, UPDATE, DELETE, CREATE, DROP, ALTER, INDEX,
  CREATE TEMPORARY TABLES, LOCK TABLES, CREATE VIEW, SHOW VIEW,
  CREATE ROUTINE, ALTER ROUTINE, EXECUTE, EVENT, TRIGGER
  ON mydb.* TO 'dba_role';

-- Assign roles to users
GRANT 'app_reader', 'app_writer' TO 'appuser'@'192.168.1.%';

-- Set default roles (roles automatically activated on login)
SET DEFAULT ROLE ALL TO 'appuser'@'192.168.1.%';

-- Make role persistent (survives session)
SET DEFAULT ROLE ALL FOR 'appuser'@'192.168.1.%';
```

**Role Best Practices**:

1. Use roles for application users instead of individual grants
2. Avoid direct user grants when roles exist
3. Review roles periodically for privilege creep
4. Use role hierarchy for complex access patterns
5. Set appropriate default roles for each user

### Revoking Privileges

```sql
-- Revoke specific privileges
REVOKE INSERT, UPDATE ON mydb.* FROM 'appuser'@'192.168.1.%';

-- Revoke all privileges
REVOKE ALL PRIVILEGES, GRANT OPTION FROM 'appuser'@'192.168.1.%';

-- Drop user
DROP USER 'appuser'@'192.168.1.%';

-- Revoke role from user
REVOKE 'app_reader' FROM 'appuser'@'192.168.1.%';

-- Drop role entirely
DROP ROLE 'app_reader';
```

## Encryption

### Data in Transit

**SSL/TLS Configuration for MySQL 8.0**:

```ini
[mysqld]
# SSL/TLS configuration
ssl-ca = /etc/mysql/ssl/ca.pem
ssl-cert = /etc/mysql/ssl/server-cert.pem
ssl-key = /etc/mysql/ssl/server-key.pem
ssl-cipher = 'ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384'
require_secure_transport = ON
tls_version = 'TLSv1.2,TLSv1.3'

# X.509 authentication (optional, high security)
x509_identity = ''
```

**SSL Requirement Per User**:

```sql
-- Force SSL connections
GRANT SELECT ON mydb.* TO 'secure_user'@'192.168.1.%' REQUIRE SSL;

-- Require X.509 certificates
GRANT SELECT ON mydb.* TO 'cert_user'@'192.168.1.%' REQUIRE X509;

-- Require specific cipher
GRANT SELECT ON mydb.* TO 'cipher_user'@'192.168.1.%' REQUIRE CIPHER 'AES256-SHA';
```

**Client-Side SSL Verification**:

```sql
-- Connect with SSL verification
mysql --ssl-ca=/etc/mysql/ssl/ca.pem \
      --ssl-cert=/etc/mysql/ssl/client-cert.pem \
      --ssl-key=/etc/mysql/ssl/client-key.pem \
      -h mysql.example.com -u appuser -p

-- Verify SSL connection
SHOW SESSION STATUS LIKE 'Ssl_cipher';
```

**SSL Certificate Management**:

| Task | Command | Frequency |
|------|---------|-----------|
| Create CA | openssl req -new -x509 -keyout ca-key.pem -out ca.pem -days 3650 | One-time |
| Create server cert | openssl req -newkey rsa:2048 -nodes -keyout server-key.pem -out server-req.pem | One-time |
| Create client cert | Same process with client-specific CN | Per client |
| Renew certificates | Re-run with updated expiry | Before expiry |
| Verify certificates | openssl x509 -in cert.pem -noout -dates | Before deployment |

### Data at Rest

**InnoDB Tablespace Encryption**:

1. **Tablespace Encryption**: InnoDB tablespace encryption with per-table or per-database keys
2. **Key Ring Plugin**: External key management integration (HashiCorp Vault, AWS KMS)
3. **Disk Encryption**: LUKS/file-level encryption for backup storage
4. **Volume Encryption**: Cloud provider volume encryption (AWS EBS, Azure managed disks)

```sql
-- Enable tablespace encryption
CREATE TABLE encrypted_table (
    id INT PRIMARY KEY,
    data VARCHAR(255)
) ENCRYTION='Y';

-- Alter existing table
ALTER TABLE existing_table ENCRYPTION='Y';

-- Check encryption status
SELECT table_name, table_schema, encryption 
FROM information_schema.tables 
WHERE table_schema = 'mydb' AND engine = 'InnoDB';
```

### Key Management

**Key Ring Plugin Configuration**:

```sql
-- Create keyring file for development
INSTALL PLUGIN keyring_file SONAME 'keyring_file.so';

-- Configure keyring in my.cnf
[keyring_file]
file_location = /var/lib/mysql-keyring/keyring
```

**Key Management Best Practices**:

1. Use external key management (AWS KMS, HashiCorp Vault) for production
2. Never store encryption keys in the same location as encrypted data
3. Implement key rotation procedures
4. Backup encryption keys securely before any server migration
5. Test key recovery procedures before relying on encrypted data

## Audit and Monitoring

### Audit Events

MySQL provides multiple levels of audit capability:

| Method | Coverage | Overhead | Use Case |
|--------|----------|---------|----------|
| Performance Schema | Event-level (statements, waits, transactions) | Low (10-30%) | Performance monitoring |
| General Log | All statements | Very High (disabled in production) | Debugging |
| Binary Log | Transaction-level (ROW format) | Medium | Audit trail, replication |
| MySQL Enterprise Audit | All events (commercial plugin) | Medium | Compliance (PCI-DSS, HIPAA) |
| Proxy-level Audit | All queries through proxy | Low (per proxy) | Enterprise environments |

**Performance Schema Security Monitoring**:

```sql
-- Monitor authentication events
SELECT * FROM performance_schema.events_transactions_current;

-- Monitor statement events
SELECT * FROM performance_schema.events_statements_current;

-- Monitor table access
SELECT * FROM performance_schema.table_io_waits_summary_by_table;

-- Count failed authentication attempts
SELECT COUNT(*) AS auth_failures 
FROM performance_schema.events_statements_summary_by_digest 
WHERE DIGEST_TEXT LIKE '%GRANT%' OR DIGEST_TEXT LIKE '%ALTER USER%';
```

### Audit Log

For compliance requirements (PCI-DSS, HIPAA, SOX):

1. **MySQL Enterprise Audit**: Commercial audit plugin with detailed event logging
2. **ProxySQL Audit**: Proxy-level auditing for all queries passing through
3. **Binary Log Analysis**: Post-hoc audit capability with ROW format
4. **Third-Party Tools**: Percona audit plugins, MySQL Router audit
5. **General Log**: Basic audit (use sparingly due to performance impact)

**Security Monitoring Queries**:

```sql
-- Failed login attempts (check error log)
-- Search error log for "Access denied" entries

-- Privilege changes
SELECT * FROM mysql.event WHERE db = 'mysql';

-- User activity
SELECT user, host FROM mysql.user 
WHERE plugin = 'caching_sha2_password';

-- Active connections by user
SELECT user, host, command, time, state, info 
FROM information_schema.processlist 
ORDER BY time DESC;
```

## Network Security

### Host-Based Restrictions

```sql
-- Restrict user to specific hosts
CREATE USER 'restricted_user'@'10.0.0.%' IDENTIFIED BY 'password';
CREATE USER 'restricted_user'@'localhost' IDENTIFIED BY 'password';

-- Create user for specific IP (most restrictive)
CREATE USER 'app_user'@'192.168.1.100' IDENTIFIED BY 'password';

-- Wildcard for subnet
CREATE USER 'dev_user'@'10.0.%' IDENTIFIED BY 'password';
```

**Host Restriction Strategy**:

| Access Type | Host Pattern | Example |
|------------|-------------|---------|
| Application server | Specific IP | `@'192.168.1.100'` |
| Application subnet | CIDR | `@'10.0.1.%'` |
| Monitoring server | Specific IP | `@'10.0.5.50'` |
| Admin access | localhost only | `@'localhost'` |
| Remote admin | VPN subnet | `@'172.16.%'` |

### MySQL Configuration Security

```ini
[mysqld]
# Bind to specific interface only
bind-address = 127.0.0.1

# Disable local_infile for security
local_infile = OFF

# Restrict secure file import
secure_file_priv = /var/lib/mysql-files

# Disable symbolic links
skip-symbolic-links = ON

# Disable old password compatibility
show_compatibility_56 = OFF
```

### Connection Resource Limits

```sql
-- Limit connections per user
CREATE USER 'limited_user'@'%' 
IDENTIFIED BY 'password'
WITH MAX_USER_CONNECTIONS 10;

-- Set connection timeouts
SET GLOBAL connect_timeout = 10;
SET GLOBAL net_read_timeout = 30;
SET GLOBAL net_write_timeout = 30;
```

**Resource Limits by User Type**:

| User Type | Max Connections | Max Queries/hr | Max Updates/hr |
|-----------|----------------|----------------|----------------|
| Application | 20-50 | 10,000-50,000 | 5,000-20,000 |
| Monitoring | 2-5 | 1,000-5,000 | 500-2,000 |
| Backup | 1 | 500 | 500 |
| Developer | 5-10 | 2,000 | 1,000 |
| Admin | 10-20 | 5,000 | 2,000 |

## Secret Management

**MySQL User Credentials Storage**:

1. Never hardcode credentials in application code
2. Use environment variables or secret management systems (HashiCorp Vault, AWS Secrets Manager)
3. Use `.myloginpath` for mysql client command-line authentication
4. Use `--defaults-extra-file` for application configuration
5. Rotate credentials regularly using automated rotation tools

**Creating a .myloginpath**:

```sql
-- Create encrypted login path file
mysql_config_editor set --login-path=client \
  --host=localhost --user=appuser --password

-- Verify login path
mysql_config_editor print --login-path=client
```

**Secret Rotation Procedure**:

1. Generate new credential in secret management system
2. Update MySQL user account with new password
3. Update application configuration with new credential
4. Verify application connectivity
5. Delete old credential from secret management
6. Audit all systems using the old credential

## Security Hardening Checklist

### Production Security Hardening

1. **Remove default accounts**: Drop `test` database and anonymous users
2. **Bind address**: Set `bind-address` to specific interface or `127.0.0.1`
3. **Disable local infile**: Set `local_infile = OFF`
4. **Secure file privilege**: Set `secure_file_priv` to specific directory
5. **Disable performance_schema**: Reduce overhead in production (only when not needed)
6. **Enable log_bin**: For replication and audit trail
7. **Set root password**: Ensure root has strong password
8. **Remove sample data**: Delete any default test data
9. **Disable X Protocol**: If not using MySQL Shell
10. **Set secure tmpdir**: Ensure /tmp is on separate partition with noexec

**Hardening Configuration**:

```ini
[mysqld]
# Security hardening
bind-address = 127.0.0.1
local_infile = OFF
secure_file_priv = /var/lib/mysql-files
skip-symbolic-links = ON
show_compatibility_60 = OFF
show_compatibility_56 = OFF
# Performance
performance_schema = ON
# Logging
general_log = OFF
slow_query_log = ON
log_queries_not_using_indexes = ON
```

## Vulnerability Management

**Security Update Strategy**:

1. **Subscribe to MySQL Security Advisories**: https://www.mysql.com/security/
2. **Patch Testing**: Test security patches in staging before production
3. **Emergency Patching**: Critical vulnerabilities require immediate action
4. **Version Lifecycle**: Stay within Oracle-supported versions
5. **Third-Party Patches**: Percona and MariaDB also release security patches

**CVE Response Procedure**:

1. **Assess**: Determine if CVE affects your deployment
2. **Test**: Apply patch in staging environment
3. **Schedule**: Plan maintenance window
4. **Deploy**: Apply patch during maintenance window
5. **Verify**: Test application functionality post-patch
6. **Monitor**: Watch for unusual behavior for 48 hours

## Security Best Practices

1. **Use `caching_sha2_password`** as default authentication
2. **Enforce SSL/TLS** for all connections in production
3. **Apply least privilege** principle for all users
4. **Regular password rotation** with strong complexity requirements
5. **Disable unused accounts** and anonymous users
6. **Monitor security events** continuously via Performance Schema
7. **Keep MySQL updated** with latest security patches
8. **Use roles** for scalable privilege management
9. **Encrypt sensitive data** at rest (tablespace) and in transit (SSL/TLS)
10. **Regular security audits** and penetration testing
11. **Implement audit logging** for compliance requirements
12. **Use secret management** for credential storage and rotation
13. **Network segmentation** for database servers
14. **Resource limits** per user to prevent abuse

## References

- MySQL 8.0 Security: https://dev.mysql.com/doc/refman/8.0/en/security.html
- MySQL 8.0 Authentication: https://dev.mysql.com/doc/refman/8.0/en/authentication-plugins.html
- MySQL 8.0 Encryption: https://dev.mysql.com/doc/refman/8.0/en/encrypted-connection-protocol-cryptografic-stream-ciphers.html
- MySQL 8.0 Authorization: https://dev.mysql.com/doc/refman/8.0/en/privileges.html
- MySQL 8.0 Password Management: https://dev.mysql.com/doc/refman/8.0/en/password-management.html
- MySQL 8.0 Password Validation Plugin: https://dev.mysql.com/doc/refman/8.0/en/password-validation-plugin.html
- MySQL 8.0 User Resources: https://dev.mysql.com/doc/refman/8.0/en/user-resources.html
- MySQL 8.0 Audit Log Plugin: https://dev.mysql.com/doc/refman/8.0/en/audit-log-plugin.html
- MySQL 8.0 Secure Installation: https://dev.mysql.com/doc/refman/8.0/en/secure-default-scenarios.html
- MySQL 8.0 Tablespace Encryption: https://dev.mysql.com/doc/refman/8.0/en/innodb-tablespace-encryption.html