# Understanding Systemd Service States

## Common States

- **active (running)**: Service is running normally
- **active (exited)**: One-shot service completed successfully
- **failed**: Service failed (check `journalctl -u <service>`)
- **inactive (dead)**: Service is not running
- **activating**: Service is starting up

## Troubleshooting Failed Services

```bash
# 1. Check the current status
systemctl status <service>

# 2. View recent logs
journalctl -u <service> -n 100 --no-pager

# 3. Check for specific error patterns
journalctl -u <service> -p err --no-pager

# 4. Check if the service exists and is enabled
systemctl list-unit-files | grep <service>

# 5. Reload systemd after config changes
systemctl daemon-reload

# 6. Restart the service
systemctl restart <service>

# 7. Verify recovery
systemctl status <service>
```

## Common Failure Causes

1. **Configuration syntax errors**: Check for typos in service unit files
2. **Missing dependencies**: Verify all required services are running
3. **Permission issues**: Ensure the service user can access required files
4. **Port conflicts**: Check if another service is using the same port
5. **Disk space**: Verify sufficient disk space for logs and runtime data

## Example: Troubleshooting Nginx

```bash
systemctl status nginx
journalctl -u nginx -n 50
nginx -t  # Test configuration syntax
systemctl restart nginx
```