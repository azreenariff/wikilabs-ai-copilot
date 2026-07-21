# Checkmk Security Configuration

## Overview

Checkmk security configuration covers authentication, authorization, network security, and data protection.

## Authentication

### Authentication Methods

| Method | Description | Use Case |
|--------|-------------|----------|
| **Internal** | Built-in user database | Basic setup |
| **LDAP** | Directory integration | Enterprise |
| **OAuth2** | Modern authentication | Cloud integration |
| **SAML** | SSO integration | Enterprise SSO |

### User Management

```bash
# Add user via CLI
omd sitename adduser username

# Remove user
omd sitename rmuser username

# List users
omd sitename listusers
```

## Authorization

### Role-Based Access Control

| Role | Permissions |
|------|-------------|
| **Admin** | Full access |
| **Manager** | Configure hosts/services |
| **Monitor** | View only |
| **API User** | API access only |

### Ruleset Permissions

| Setting | Permission |
|---------|-----------|
| **WATO Access** | Configuration access |
| **Site Access** | Site management |
| **Ruleset Edit** | Ruleset modification |
| **Host Config** | Host configuration |

## Network Security

### Firewall Configuration

```bash
# Allow Checkmk agent port
iptables -A INPUT -p tcp --dport 6556 -s 10.0.0.0/8 -j ACCEPT

# Allow Web interface
iptables -A INPUT -p tcp --dport 5000 -s 10.0.0.0/8 -j ACCEPT

# Allow Livestatus socket
# (Unix socket, no firewall needed)
```

### TLS/SSL Configuration

```bash
# Configure HTTPS
omd sitename config check_mk_web.cfg

# SSL certificate
/etc/ssl/certs/checkmk.pem
/etc/ssl/private/checkmk.key
```

## Data Protection

### Data Encryption

1. **In Transit**: HTTPS/TLS for web interface
2. **At Rest**: Encrypted backups
3. **Agent Communication**: TLS for agent data
4. **Database**: Encryption for sensitive data

### Audit Logging

Checkmk maintains audit logs for:
- User login attempts
- Configuration changes
- Rule modifications
- Notification changes

## Security Best Practices

1. Use HTTPS for web interface
2. Implement strong password policies
3. Regular user access reviews
4. Restrict agent access by IP
5. Enable audit logging
6. Regular security patches
7. Encrypt backups securely
8. Follow least privilege principle

## References

- Checkmk Security: https://docs.checkmk.com/master/en/
- Checkmk Authentication: https://docs.checkmk.com/master/en/
- Checkmk Network Security: https://docs.checkmk.com/master/en/