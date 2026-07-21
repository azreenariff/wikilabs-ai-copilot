# Nagios Log Server Security Configuration

## Overview

Nagios Log Server security configuration covers authentication, authorization, network security, and data protection.

## Authentication

### Web Interface Authentication

Nagios Log Server uses Nagios XI authentication with:
- Database-driven user accounts
- Password policies and rotation
- Role-based access control

### User Management

```sql
-- List users
SELECT * FROM nagiosxi.users;

-- Add user
INSERT INTO nagiosxi.users (username, password, fullname, email)
VALUES ('loguser', 'password_hash', 'Log User', 'user@example.com');

-- Modify role
UPDATE nagiosxi.users SET admin = 1 WHERE username = 'logadmin';
```

## Authorization

### Roles and Permissions

| Role | Permissions |
|------|-------------|
| **Admin** | Full access to all functions |
| **Operator** | Search, alerts, configuration |
| **Viewer** | Search and reports only |
| **API User** | API access only |

### Elasticsearch Access Control

```json
{
  "roles": {
    "log_viewer": {
      "indices": [
        {
          "names": ["nagioslog-*"],
          "privileges": ["read", "view_index_metadata"]
        }
      ]
    },
    "log_admin": {
      "indices": [
        {
          "names": ["nagioslog-*"],
          "privileges": ["all"]
        }
      ]
    }
  }
}
```

## Network Security

### Firewall Configuration

```bash
# Allow Nagios Log Server web interface
iptables -A INPUT -p tcp --dport 80 -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -j ACCEPT

# Allow Logstash input from log sources
iptables -A INPUT -p tcp --dport 5544 -s 10.0.0.0/8 -j ACCEPT
iptables -A INPUT -p udp --dport 5544 -s 10.0.0.0/8 -j ACCEPT

# Allow Elasticsearch from Logstash only
iptables -A INPUT -p tcp --dport 9200 -s 127.0.0.1 -j ACCEPT
iptables -A INPUT -p tcp --dport 9300 -s 127.0.0.1 -j ACCEPT

# Allow MySQL from Nagios Log Server only
iptables -A INPUT -p tcp --dport 3306 -s 127.0.0.1 -j ACCEPT
```

### SSL/TLS Configuration

```bash
# Generate Elasticsearch SSL certificates
openssl req -newkey rsa:2048 -x509 -days 365 -nodes \
  -keyout /etc/elasticsearch/certs/elasticsearch.key \
  -out /etc/elasticsearch/certs/elasticsearch.crt

# Configure Elasticsearch SSL
# /etc/elasticsearch/elasticsearch.yml
xpack.security.transport.ssl.enabled: true
xpack.security.transport.ssl.verification_mode: certificate
xpack.security.transport.ssl.keystore.path: certs/elasticsearch.p12
xpack.security.transport.ssl.truststore.path: certs/elasticsearch.p12
```

## Data Protection

### Log Data Encryption

1. **At Rest**: Elasticsearch encryption at rest
2. **In Transit**: SSL/TLS for all communications
3. **Backup Encryption**: Encrypted backup files

### Sensitive Data Handling

| Data Type | Protection |
|-----------|-----------|
| **Passwords** | Never stored in plain text |
| **API Keys** | Encrypted in configuration |
| **Personal Data** | Redaction or anonymization |
| **Financial Data** | Extra protection and access control |

### Audit Logging

Nagios Log Server maintains audit logs for:
- User login attempts
- Configuration changes
- Alert modifications
- Search queries (optionally)

## Security Best Practices

1. Use HTTPS for web interface access
2. Implement strong password policies
3. Regular user access reviews
4. Restrict Logstash access by IP
5. Enable audit logging
6. Regular security patches
7. Backup configuration securely
8. Monitor for unauthorized access

## References

- Nagios Log Server Security: https://assets.nagios.com/downloads/nagiosxi/docs/
- Elasticsearch Security: https://www.elastic.co/guide/en/elasticsearch/reference/
- SSL/TLS Configuration: https://httpd.apache.org/docs/2.4/ssl/