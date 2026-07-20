# Known Issues and Limitations

## Detection Limitations

- Command-based detection has lower confidence than file-based detection
- Cannot detect in-container services without container access
- Remote systems require SSH access for full detection
- Some patterns may have false positives on similar tools

## Performance Considerations

- Deep file scanning can be slow on large filesystems
- Command execution requires sudo privileges for full coverage
- Pattern matching may miss obfuscated configurations
- Real-time monitoring requires additional setup

## Environment Restrictions

- Alpine Linux uses openrc instead of systemd (limited coverage)
- Some distributions use alternative package managers
- Custom kernel configurations may not match detection patterns
- Containerized environments may not have traditional file layout

## Recovery Procedures

### If SSH Lockout Occurs

1. Access server via console or recovery mode
2. Reset sshd_config: `sed -i 's/^PermitRootLogin no/PermitRootLogin yes/' /etc/ssh/sshd_config`
3. Restart SSH: `systemctl restart sshd`
4. Regain access and re-apply hardening

### If Disk Full

1. Identify largest consumers: `du -sh /* | sort -rh | head`
2. Clean temp files: `rm -rf /tmp/*`
3. Vacuum journal: `journalctl --vacuum-size=500M`
4. Clean package cache: `apt-get clean` or `dnf clean all`

### If Service Won't Start

1. Check config syntax: `systemd-analyze verify <service>.service`
2. Check permissions: Ensure service user can read config
3. Check dependencies: Verify all required services are running
4. Check logs: `journalctl -u <service> -n 100 --no-pager`
5. Try starting manually: `systemctl start <service>`
6. If still failing, check resource limits: `ulimit -a`