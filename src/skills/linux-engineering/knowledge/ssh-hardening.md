# SSH Security Hardening Guide

## Quick Security Checklist

```bash
# 1. Check current configuration
cat /etc/ssh/sshd_config | grep -E "^Permit|^\s*Password|^\s*Port|^\s*Protocol"

# 2. Recommended secure settings
PermitRootLogin no
PasswordAuthentication no
PubkeyAuthentication yes
X11Forwarding no
MaxAuthTries 3
AllowUsers admin deployer  # Explicit user whitelist
Port 2222  # Non-standard port (optional but recommended)
```

## Implementation Steps

### 1. Disable Root Login
```bash
sed -i 's/^#PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config
```

### 2. Enable Key-Based Authentication Only
```bash
sed -i 's/^PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
```

### 3. Create SSH Keys for Deployment User
```bash
ssh-keygen -t ed25519 -C "deploy@server"
cp ~/.ssh/id_ed25519.pub ~/.ssh/authorized_keys
chmod 600 ~/.ssh/authorized_keys
chmod 700 ~/.ssh
```

### 4. Apply Changes
```bash
# Test config first
sshd -t

# Restart SSH
systemctl restart sshd
```

### 5. Verify
```bash
# Test key-based login from another terminal
ssh -i ~/.ssh/id_ed25519 user@server
ssh -o PasswordAuthentication=no user@server
```

## Advanced Hardening

### Fail2Ban Integration
```bash
apt install fail2ban
# Configure /etc/fail2ban/jail.local for SSH protection
```

### TCP Wrappers
```bash
# /etc/hosts.allow
sshd: 192.168.1.0/24

# /etc/hosts.deny
sshd: ALL
```

### SSH Audit
```bash
# Install ssh-audit (optional)
pip install ssh-audit
ssh-audit server_ip
```

## Security Best Practices

1. **Use strong key algorithms**: Ed25519 > RSA 4096-bit
2. **Rotate keys regularly**: Every 90 days minimum
3. **Monitor auth logs**: `journalctl -u sshd -f`
4. **Disable unused protocols**: SSHv2 only
5. **Use SSH agent forwarding**: For multi-hop deployments
6. **Restrict by IP**: Use firewall rules or AllowGroups

## Recovery: If You Lock Yourself Out

```bash
# From server console or recovery mode:
sed -i 's/^PermitRootLogin no/PermitRootLogin yes/' /etc/ssh/sshd_config
systemctl restart sshd
```