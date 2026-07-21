# Security Engineering Foundation

## Architecture

### Security Layers

| Layer | Domain | Mechanisms |
|-------|--------|------------|
| Physical | Access control | Locks, badges, surveillance |
| Network | Transport security | Firewalls, VPNs, TLS, IDS/IPS |
| System | Host security | SELinux, AppArmor, hardening, patches |
| Application | Code security | Input validation, auth, encryption |
| Identity | Access management | LDAP, AD, OAuth, SAML, MFA |
| Data | Information security | Encryption, classification, DLP |

### Defense in Depth

```
Physical → Network → Host → Application → Data → Identity
     │         │         │          │         │        │
     ▼         ▼         ▼          ▼         ▼        ▼
  Badge     Firewall  SELinux   Auth     Encrypt  MFA
  Sensors   IDS/IPS   Hardening  Tokens   KMS     SSO
  Cameras   VPN       Patches   RBAC     Certs   Audit
```

---

## Core Concepts

### Authentication vs Authorization

| Concept | Question | Examples |
|---------|----------|----------|
| **Authentication** | Who are you? | Passwords, certificates, tokens, biometrics |
| **Authorization** | What can you do? | RBAC, ACLs, permissions, policies |

### Identity Models

| Model | Description | Use Case |
|-------|-------------|----------|
| Local | OS-level users/groups | Single server |
| Centralized | LDAP/AD directory | Enterprise environment |
| Federated | SAML, OAuth, OIDC | Cross-organization |
| Service Mesh | mTLS, service identity | Microservices |
| Cloud IAM | Cloud provider identity | Cloud-native apps |

### RBAC (Role-Based Access Control)

**Components**
- **User:** Person or system account
- **Role:** Collection of permissions
- **Permission:** Specific action allowed (read, write, execute)
- **Policy:** Rules defining who can do what

**RBAC Model**
```
User ──assigns──> Role ──has──> Permission
```

**Linux RBAC**
- Groups and file permissions
- sudoers for elevated privileges
- SELinux policies for mandatory access control

### Encryption

| Type | Description | Examples |
|------|-------------|----------|
| Symmetric | Same key for encrypt/decrypt | AES-256, ChaCha20 |
| Asymmetric | Public/private key pair | RSA, ECC, Ed25519 |
| Hash | One-way function | SHA-256, SHA-3 |
| HSM | Hardware security module | Physical key storage |
| KMS | Key management service | AWS KMS, Azure Key Vault |

### Certificates

**PKI (Public Key Infrastructure)**
- **Root CA:** Trust anchor (self-signed)
- **Intermediate CA:** Signs server certs
- **Server Cert:** End-entity certificate
- **Chain of Trust:** Server → Intermediate → Root

**Certificate Fields**
- Subject (CN, O, C)
- SAN (Subject Alternative Names)
- Validity period
- Issuer
- Serial number
- Fingerprint (SHA-256)

### Least Privilege

**Principle:** Grant minimum permissions necessary to perform a task.

**Implementation:**
- Service accounts with only required permissions
- Principle of least privilege applies to ALL accounts
- Regular permission reviews
- Just-in-time access for elevated operations

### Audit and Compliance

**Audit Trail**
- Who did what, when, and from where
- Authentication and authorization events
- Configuration changes
- Security-relevant events

**Compliance Frameworks**
- **PCI DSS** — Payment card data
- **HIPAA** — Healthcare data
- **SOC 2** — Service organization controls
- **GDPR** — Personal data protection
- **ISO 27001** — Information security management

---

## Common Components

### Authentication Tools
| Tool | Purpose |
|------|---------|
| SSH keys | Key-based authentication |
| TLS certificates | Transport encryption |
| PAM | Pluggable authentication modules |
| sudo | Privilege escalation |
| fail2ban | Brute-force protection |
| LDAP/AD | Directory authentication |

### Security Tools
| Tool | Purpose |
|------|---------|
| SELinux/AppArmor | Mandatory access control |
| iptables/nftables | Packet filtering |
| firewalld | Firewall management |
| fail2ban | Intrusion prevention |
| rkhunter/chkrootkit | Rootkit detection |
| AIDE/Tripwire | File integrity monitoring |

### Monitoring Tools
| Tool | Purpose |
|------|---------|
| auditd | System call auditing |
| syslog/rsyslog | Centralized logging |
| OSSEC | Host-based intrusion detection |
| Wazuh | XDR platform |
| Splunk/ELK | Log analysis |

---

## Common Failures

### Authentication Failures
| Symptom | Possible Cause |
|---------|----------------|
| Login fails repeatedly | Wrong password, account locked, expired credentials |
| SSH key rejected | Key permissions too open, wrong authorized_keys |
| TLS handshake fails | Expired cert, wrong hostname, protocol mismatch |
| Kerberos ticket fails | Time skew >5 min, wrong principal, DC unreachable |

### Authorization Failures
| Symptom | Possible Cause |
|---------|----------------|
| Permission denied | Insufficient ACL, wrong group, SELinux denial |
| Can't access file | Incorrect ownership, restrictive permissions |
| Can't run command | Not in sudoers, insufficient RBAC role |
| API returns 403 | Invalid token, insufficient scopes, IP blocked |

### Certificate Issues
| Symptom | Possible Cause |
|---------|----------------|
| Certificate expired | Renewal automation missed |
| Wrong certificate | Multiple certs on same IP/SAN, wrong order |
| Self-signed in production | Should use CA-signed cert |
| Certificate chain broken | Missing intermediate certificate |

### Security Configuration Issues
| Symptom | Possible Cause |
|---------|----------------|
| Service accessible from internet | No firewall, open port, missing ACL |
| Weak encryption used | TLS 1.0/1.1, RC4, MD5 still active |
| Default credentials | Factory defaults not changed |
| Unencrypted data | Sensitive data not encrypted at rest |

---

## Troubleshooting Philosophy

### Security Diagnostic Flow

```
Security issue
  │
  ├─→ Authentication failure
  │   ├─→ Check credentials/expiry
  │   ├─→ Check account lockout
  │   ├─→ Check certificate validity
  │   └─→ Check time sync (Kerberos)
  │
  ├─→ Authorization failure
  │   ├─→ Check group membership
  │   ├─→ Check ACL/RBAC
  │   ├─→ Check SELinux/AppArmor denials
  │   └─→ Check service account permissions
  │
  ├─→ Access from unexpected source
  │   ├─→ Check firewall rules
  │   ├─→ Check service binding (0.0.0.0 vs specific IP)
  │   ├─→ Check for unauthorized VPN access
  │   └─→ Check for credential theft
  │
  └─→ Security event detected
      ├─→ Check audit logs
      ├─→ Check authentication logs
      ├─→ Identify affected systems
      └─→ Assess scope and impact
```

### Essential Commands
- `id`, `groups`, `whoami` — Identity checks
- `getfacl`, `setfacl` — ACL inspection
- `sudo -l` — User sudo privileges
- `journalctl -u sshd` — SSH audit logs
- `cat /var/log/auth.log` or `/var/log/secure` — Authentication events
- `sestatus`, `ausearch` — SELinux audit
- `openssl x509 -in cert.pem -text` — Certificate inspection
- `ss -tlnp` — Listening ports and processes

---

## Best Practices

### Identity and Access
- **Principle of least privilege** — Grant minimum required permissions
- **Regular access reviews** — Audit permissions quarterly
- **MFA for remote access** — Require multi-factor authentication
- **Service accounts** — Dedicated accounts, not shared passwords
- **Just-in-time access** — Elevated access only when needed
- **Remove stale accounts** — Inactive accounts are attack surface

### Network Security
- **Default deny** — Block all, explicitly allow
- **Segment networks** — Isolate sensitive systems
- **Encrypt in transit** — TLS everywhere, SSH for management
- **Monitor traffic** — IDS/IPS, anomaly detection
- **Patch promptly** — Apply security updates within SLA

### Data Security
- **Encrypt at rest** — LUKS, BitLocker, database encryption
- **Classify data** — Know what's sensitive
- **Back up securely** — Encrypted backups, tested restores
- **DLP monitoring** — Detect data exfiltration
- **Minimize data retention** — Delete data when no longer needed

### Monitoring and Auditing
- **Centralized logging** — Collect logs from all systems
- **Alert on security events** — Auth failures, privilege changes
- **Regular audits** — Check configurations, permissions, certificates
- **Incident response plan** — Know what to do when things go wrong
- **Penetration testing** — Regular security assessments

---

## Risk Awareness

### High-Risk Operations
| Operation | Risk Level | Warning |
|-----------|------------|---------|
| Opening firewall ports | High | Can expose services to internet |
| Adding users to sudo | High | Privilege escalation risk |
| Modifying SELinux policies | Medium | Can break applications |
| Disabling security features | Critical | Exposes system to attacks |
| Sharing credentials | Critical | Authentication compromise |
| Default credentials | Critical | Immediate attack target |
| Weak encryption | Medium-High | Data interception possible |
| Self-signed certs in prod | Medium | Man-in-the-middle risk |

### Safe Operations
- Reading authentication logs
- Checking certificate expiration
- Reviewing firewall rules
- Checking user permissions
- Running security scanners (read-only)
- Verifying TLS configuration

---

## References

- [CIS Benchmarks](https://www.cisecurity.org/benchmarks)
- [NIST Cybersecurity Framework](https://www.nist.gov/cybersecurity)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Mozilla SSL Configuration Generator](https://ssl-config.mozilla.org/)
- [Red Hat SELinux Guide](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/)
- [SSH Security Best Practices](https://www.ssh.com/academy/ssh/security)
- [Certificate Authority Best Practices](https://cabforum.org/baseline-requirements/)

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial Security Engineering Foundation |