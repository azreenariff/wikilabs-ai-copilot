# Linux Knowledge — Security Hardening

## Security Model

Linux security is built on multiple layers: authentication, authorization, access control, encryption, and auditing.

### Authentication

| Method | Description | Security Level |
|--------|-------------|----------------|
| Password | Traditional password authentication | Medium |
| SSH Keys | Public/private key pair authentication | High |
| PAM | Pluggable Authentication Modules for extensible auth | Varies |
| Kerberos | Network authentication service | High |
| Certificate | X.509 certificate authentication | High |
| Biometric | Fingerprint, face recognition | High |

### SSH Security

```bash
# Best practices for SSH
/etc/ssh/sshd_config

# Recommended settings
PermitRootLogin no
PasswordAuthentication no
PubkeyAuthentication yes
MaxAuthTries 3
ClientAliveInterval 300
ClientAliveCountMax 2
AllowGroups ssh-users
```

### SSH Key Management

```bash
# Generate SSH key pair
ssh-keygen -t ed25519 -C "user@host"

# Copy to remote host
ssh-copy-id user@remote-host

# Key permissions
chmod 700 ~/.ssh
chmod 600 ~/.ssh/authorized_keys
chmod 600 ~/.ssh/id_ed25519
chmod 644 ~/.ssh/id_ed25519.pub
```

## Access Control

### File Permissions

```bash
# Permission types
rwxr-xr--  user   group   file
^   ^   ^
|   |   └── Others (read)
|   └────── Group (read, execute)
└────────── User (read, write, execute)

# Change permissions
chmod 755 script.sh      # rwxr-xr-x
chmod 644 file.txt       # rw-r--r--
chmod 600 sensitive.key  # rw-------

# Change ownership
chown user:group file
chown -R user:group directory/

# Setuid/setgid/sticky
chmod 4755 script      # Setuid
chmod 2755 directory   # Setgid
chmod 1777 /tmp        # Sticky bit
```

### Access Control Lists (ACLs)

```bash
# View ACLs
getfacl file.txt

# Set ACLs
setfacl -m u:user:rwx file.txt
setfacl -m g:group:rx file.txt
setfacl -d -m u:user:rw file.txt    # Default ACL
```

### SELinux

```bash
# Check SELinux status
sestatus
getenforce

# SELinux modes
permissive  # Log denials but don't block
enforcing   # Block and log denials
disabled    # SELinux off

# Manage SELinux contexts
chcon -t httpd_sys_content_t /var/www/html
restorecon -v /var/www/html

# View SELinux contexts
ls -Z /var/www/html
ps -Z -t httpd_t
```

## Firewall Configuration

### firewalld (RHEL/Fedora)

```bash
# Status
sudo firewall-cmd --state

# Zones
sudo firewall-cmd --get-active-zones
sudo firewall-cmd --list-all

# Ports
sudo firewall-cmd --add-port=80/tcp --permanent
sudo firewall-cmd --add-service=https --permanent
sudo firewall-cmd --reload

# Services
sudo firewall-cmd --list-services
sudo firewall-cmd --add-service=http
```

### ufw (Ubuntu/Debian)

```bash
# Status
sudo ufw status

# Rules
sudo ufw allow 22/tcp
sudo ufw allow from 192.168.1.0/24
sudo ufw deny 3306
sudo ufw enable
```

### iptables

```bash
# View rules
sudo iptables -L -n -v

# Add rule
sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT

# Save rules
sudo iptables-save > /etc/iptables/rules.v4
```

## Security Auditing

### auditd

```bash
# Service management
sudo systemctl status auditd
sudo systemctl enable auditd

# Audit rules
sudo ausearch -m SYSCALL -ts recent
sudo aureport -au

# Add rule
sudo auditctl -w /etc/ssh/sshd_config -p wa -k ssh_config

# List rules
sudo auditctl -l
```

### fail2ban

```bash
# Status
sudo systemctl status fail2ban

# Ban info
sudo fail2ban-client status sshd

# Unban IP
sudo fail2ban-client set sshd unbanip 1.2.3.4
```

## Patch Management

### RHEL/CentOS

```bash
# View updates
dnf check-update

# Apply updates
sudo dnf update

# Security-only updates
sudo dnf update --security

# View installed security packages
rpm -qa --qf '%{NAME}-%{VERSION}-%{RELEASE}\n' | sort
```

### Debian/Ubuntu

```bash
# View updates
apt list --upgradable

# Apply updates
sudo apt update
sudo apt upgrade

# Security-only updates
sudo unattended-upgrades
```

### Automated Updates

```bash
# Ubuntu automatic updates
sudo dpkg-reconfigure -plow unattended-upgrades

# RHEL automatic updates
sudo dnf install dnf-automatic
sudo systemctl enable --now dnf-automatic.timer
```

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial security hardening knowledge |