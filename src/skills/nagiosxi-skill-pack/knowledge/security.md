# Nagios XI Security Configuration

## Overview

Nagios XI security configuration covers authentication, authorization, network security, and compliance requirements for monitoring infrastructure.

## User Authentication

### Web Interface Authentication

Nagios XI uses database-driven authentication with password policies.

**Configuration**:
- Database: nagiosxi.users table
- Password hashing: MD5 or SHA-256
- Session management: PHP sessions
- Account lockout: After failed attempts

### User Management

```sql
-- List all users
SELECT * FROM users;

-- Add new user
INSERT INTO users (username, password, fullname, email, notify_by_email, contactalias)
VALUES ('newuser', 'password_hash', 'New User', 'user@example.com', 1, 'New User');

-- Modify user privileges
UPDATE users SET admin = 1 WHERE username = 'adminuser';

-- Disable user account
UPDATE users SET disabled = 1 WHERE username = 'inactiveuser';
```

### Password Policy

| Setting | Description | Recommended |
|---------|-------------|-------------|
| **Minimum Length** | Minimum password characters | 12 |
| **Complexity** | Require mixed case, numbers, symbols | Enabled |
| **Expiration** | Password rotation interval | 90 days |
| **History** | Prevent password reuse | 5 |
| **Lockout** | Account lockout after failures | 5 attempts, 15 min |

## Network Security

### Firewall Configuration

```bash
# Allow Nagios XI web interface
iptables -A INPUT -p tcp --dport 80 -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -j ACCEPT

# Allow NRPE from monitoring servers only
iptables -A INPUT -p tcp --dport 5666 -s 192.168.1.0/24 -j ACCEPT

# Allow SNMP from network devices
iptables -A INPUT -p udp --dport 161 -s 10.0.0.0/8 -j ACCEPT
iptables -A INPUT -p udp --dport 162 -s 10.0.0.0/8 -j ACCEPT

# Allow database access from Nagios server only
iptables -A INPUT -p tcp --dport 3306 -s 127.0.0.1 -j ACCEPT
```

### SSL/TLS Configuration

```bash
# Generate self-signed certificate
openssl req -newkey rsa:2048 -x509 -days 365 -nodes \
  -keyout /etc/pki/tls/private/nagios.key \
  -out /etc/pki/tls/certs/nagios.crt

# Configure Apache for SSL
# /etc/httpd/conf.d/ssl.conf
SSLCertificateFile /etc/pki/tls/certs/nagios.crt
SSLCertificateKeyFile /etc/pki/tls/private/nagios.key
```

### HTTPS Redirect

```apache
# /etc/httpd/conf.d/redirect.conf
RewriteEngine On
RewriteCond %{HTTPS} off
RewriteRule ^(.*)$ https://%{HTTP_HOST}%{REQUEST_URI} [R=301,L]
```

## Authorization and Roles

### User Roles

| Role | Permissions |
|------|-------------|
| **Admin** | Full access to all functions |
| **Operator** | View status, acknowledge alerts, manage downtime |
| **Viewer** | View status and reports only |
| **API User** | API access only |

### Role Configuration

```sql
-- Set admin role
UPDATE users SET admin = 1 WHERE username = 'admin';

-- Set operator role
UPDATE users SET admin = 0, operator = 1 WHERE username = 'operator';

-- Set viewer role
UPDATE users SET admin = 0, operator = 0, viewer = 1 WHERE username = 'viewer';
```

## Security Best Practices

### 1. Use Strong Passwords

- Minimum 12 characters
- Mix of upper/lower case, numbers, symbols
- Avoid dictionary words
- Regular rotation (90 days)

### 2. Enable Two-Factor Authentication

- Configure 2FA for admin accounts
- Use TOTP-based authenticators
- Backup 2FA codes securely

### 3. Restrict Network Access

- Firewall rules limiting access
- VPN for remote access
- No direct internet exposure

### 4. Regular Security Audits

- Review user access regularly
- Audit configuration changes
- Monitor for unauthorized access
- Patch operating system and software

### 5. Secure Database Access

- Use strong database passwords
- Restrict MySQL access to localhost
- Regular database backups
- Encrypt database at rest

### 6. Logging and Monitoring

- Enable Nagios XI audit logging
- Monitor web server access logs
- Monitor database access logs
- Alert on suspicious activity

## Compliance Considerations

### Audit Trail

Nagios XI maintains audit logs for:
- User login attempts
- Configuration changes
- Alert acknowledgements
- Downtime scheduling
- Notification changes

### Log Retention

| Log Type | Retention |
|----------|-----------|
| **Nagios Core** | 90 days (rotated) |
| **NDOUtil** | 90 days |
| **Apache** | 30 days |
| **MySQL** | 30 days |
| **System** | 90 days |

## References

- Nagios XI Security Guide: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios XI Administration: https://assets.nagios.com/downloads/nagiosxi/docs/
- Apache SSL Configuration: https://httpd.apache.org/docs/2.4/ssl/
- Firewall Best Practices: https://linux.die.net/man/8/iptables