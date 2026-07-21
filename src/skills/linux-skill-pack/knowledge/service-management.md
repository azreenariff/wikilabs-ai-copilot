# Linux Knowledge — Service Management

## systemd Architecture

systemd is the init system and service manager for most modern Linux distributions.

### Units

| Unit Type | Extension | Example |
|-----------|-----------|---------|
| Service | .service | nginx.service |
| Target | .target | multi-user.target |
| Mount | .mount | /dev/sda1.mount |
| Path | .path | /var/log/nginx.path |
| Socket | .socket | nginx.socket |
| Timer | .timer | backup.timer |
| Device | .device | sda.device |
| Snapshot | .snapshot | rollback.snapshot |

### Unit File Structure

```ini
[Unit]
Description=The nginx HTTP and reverse proxy server
After=network.target
Requires=network.target

[Service]
Type=forking
ExecStart=/usr/sbin/nginx
ExecReload=/bin/kill -s HUP $MAINPID
ExecStop=/bin/kill -s QUIT $MAINPID
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

### Key Directives

| Directive | Purpose |
|-----------|---------|
| Description | Human-readable description |
| After | Start this unit after listed units |
| Before | Start this unit before listed units |
| Requires | Hard dependency on listed units |
| Wants | Soft dependency on listed units |
| ExecStart | Command to start the service |
| ExecReload | Command to reload the service |
| ExecStop | Command to stop the service |
| Restart | When to restart (always, on-failure, on-abort) |
| RestartSec | Seconds to wait before restarting |
| WantedBy | Target(s) for Enable |

### Service Management Commands

```bash
# Start/stop/restart
systemctl start nginx
systemctl stop nginx
systemctl restart nginx
systemctl reload nginx

# Enable/disable at boot
systemctl enable nginx
systemctl disable nginx

# Check status
systemctl status nginx
systemctl is-active nginx
systemctl is-enabled nginx

# View dependencies
systemctl list-dependencies nginx

# View unit configuration
systemctl show nginx
systemctl show nginx --property=ExecStart

# Reload configuration
systemctl daemon-reload

# Mask/unmask (prevent from starting)
systemctl mask nginx
systemctl unmask nginx
```

### Common Service Issues

| Issue | Cause | Fix |
|-------|-------|-----|
| Failed to start | Configuration error, missing dependencies | Check logs, fix config |
| Failed to reload | SIGHUP not supported | Use restart instead |
| Activation timed out | Service takes too long to start | Increase TimeoutStartSec |
| Out of memory | OOM killer terminated service | Increase memory, optimize |
| Permission denied | SELinux, AppArmor, or file permissions | Fix permissions, adjust policy |

### Journal Log Management

```bash
# View logs
journalctl -u nginx
journalctl -u nginx --since "1 hour ago"
journalctl -u nginx --since "2026-07-21" --until "2026-07-22"

# Monitor in real-time
journalctl -f -u nginx

# Persistent journal
sudo mkdir -p /var/log/journal
sudo systemctl restart systemd-journald

# Vacuum logs
journalctl --vacuum-time=2d
journalctl --vacuum-size=500M
```

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial service management knowledge |