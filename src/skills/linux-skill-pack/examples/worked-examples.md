# Linux Engineering — Worked Examples

## Example 1: Nginx Service Not Starting

### Scenario
After a package update, nginx fails to start on a production server.

### Symptoms
- Website returns 502 Bad Gateway
- `systemctl status nginx` shows "failed"
- Users report the website is down

### Evidence
```bash
# Check service status
$ systemctl status nginx
● nginx.service - The nginx HTTP and reverse proxy server
   Loaded: loaded (/usr/lib/systemd/system/nginx.service; enabled)
   Active: failed (Result: exit-code) since Mon 2026-07-21 10:15:00 UTC
   Process: 12345 ExecStart=/usr/sbin/nginx (code=exited, status=1/FAILURE)

# Check error logs
$ journalctl -u nginx --no-pager | tail -20
Jul 21 10:15:00 server01 nginx[12345]: nginx: [emerg] bind() to 0.0.0.0:80 failed (98: Address already in use)
Jul 21 10:15:00 server01 nginx[12345]: nginx: [emerg] socket() failed (97: Address family not supported by protocol)
Jul 21 10:15:00 server01 nginx[12345]: nginx: [emerg] still could not bind()

# Check what's using port 80
$ ss -tlnp | grep :80
tcp   LISTEN   0   128   0.0.0.0:80   0.0.0.0:*   users:(("apache2",pid=8765,fd=4))
```

### Analysis
The error message clearly states "Address already in use" on port 80. The `ss` command shows that `apache2` (process 8765) is already listening on port 80. This likely happened because:
1. Both nginx and Apache were installed on the same server
2. After the update, nginx started before Apache stopped
3. Both tried to bind to the same port

### Resolution
```bash
# Stop Apache
sudo systemctl stop apache2

# Restart nginx
sudo systemctl restart nginx

# Verify nginx is running
sudo systemctl status nginx
```

### Verification
```bash
$ systemctl status nginx
● nginx.service - The nginx HTTP and reverse proxy server
   Active: active (running) since Mon 2026-07-21 10:20:00 UTC

# Test website
$ curl -I http://localhost
HTTP/1.1 200 OK
Server: nginx/1.24.0
```

### Lessons
- Avoid running two HTTP servers on the same port
- If switching from Apache to nginx, always stop Apache first
- Use `ss -tlnp` before starting any service to check for port conflicts

---

## Example 2: Disk Space Exhaustion

### Scenario
A monitoring alert reports disk space at 98% on the root filesystem.

### Symptoms
- Alert: "Disk space critical on /dev/sda1 (98% used)"
- Applications starting to fail
- Cannot create new log files

### Evidence
```bash
# Check filesystem usage
$ df -h
Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1        50G   49G  1.0G  98% /
tmpfs           7.8G     0  7.8G   0% /dev/shm

# Find large files
$ du -sh /* 2>/dev/null | sort -rh | head -10
8.5G    /var
3.2G    /usr
2.1G    /opt
1.5G    /home
1.2G    /tmp
...

$ du -sh /var/log/* 2>/dev/null | sort -rh | head -5
5.2G    /var/log/syslog
3.8G    /var/log/auth.log
2.1G    /var/log/kern.log
1.5G    /var/log/dpkg.log

# Check journal size
$ journalctl --disk-usage
Archived and active journals take up 8.5G in the file system.
```

### Analysis
The root cause is accumulated log files in `/var/log/`. The syslog alone is 5.2GB and auth.log is 3.8GB. These logs are not being rotated properly or the retention policy is too aggressive.

### Resolution
```bash
# Vacuum journal logs to 1GB
sudo journalctl --vacuum-size=1G

# Truncate oversized log files (after confirming no active processes are writing to them)
sudo truncate -s 0 /var/log/auth.log

# Ensure logrotate is configured and running
sudo logrotate -f /etc/logrotate.conf

# Verify space freed
df -h
```

### Verification
```bash
$ df -h
Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1        50G   39G  11G   78% /
```

### Lessons
- Always check journalctl disk usage before looking at individual log files
- Set up automated log rotation with size limits
- Monitor disk usage with alerts at 70%, 80%, and 90% thresholds

---

## Example 3: High CPU Usage on Production Server

### Scenario
A production database server shows CPU usage at 95%+.

### Symptoms
- Application response times degraded
- Monitoring shows CPU at 95%+
- Users reporting slow queries

### Evidence
```bash
# Check load average
$ uptime
 10:30:00 up 45 days, 3:20, 2 users, load average: 24.50, 18.30, 12.80

# Check top CPU consumers
$ ps aux --sort=-%cpu | head -10
USER       PID %CPU %MEM    VSZ   RSS TTY  STAT START   TIME COMMAND
postgres  4521 45.2  8.5  98524 67234 ?    Sl   Jul19  120:45 postgres: worker process
postgres  4522 22.1  4.2  98524 35123 ?    Ss   Jul19   45:30 postgres: background writer
www-data  8901  8.5  2.1  45632 17234 ?    S    Jul20   12:45 php-fpm: pool www
apache2   8902  6.2  1.8  34521 14523 ?    S    Jul20    8:30 apache2 -k start

# Check iostat
$ iostat -x 1 3
Device    r/s    w/s   rkB/s   wkB/s  await  svctm  %util
sda       15.2   45.2  120.5   356.2   8.5   2.1   45.2
sdb        0.0    0.0    0.0     0.0   0.0   0.0    0.0
```

### Analysis
PostgreSQL process 4521 is consuming 45.2% CPU and has been running for 120+ minutes. This suggests a long-running query or query planning issue. The load average of 24.5 on a likely 8-core system indicates significant queuing.

### Resolution
```bash
# Identify the long-running query
sudo -u postgres psql -c "SELECT pid, now() - pg_stat_activity.query_start AS duration, query FROM pg_stat_activity WHERE state = 'active' ORDER BY duration DESC LIMIT 5;"

# Terminate the problematic query if needed
sudo -u postgres psql -c "SELECT pg_terminate_backend(<PID>);"

# Analyze and optimize the query
EXPLAIN ANALYZE <query>;

# Add appropriate index if needed
CREATE INDEX idx_table_column ON table(column);
```

### Verification
```bash
$ uptime
 11:15:00 up 45 days, 4:05, 2 users, load average: 3.20, 2.80, 2.10

$ ps aux --sort=-%cpu | head -5
USER       PID %CPU %MEM    VSZ   RSS TTY  STAT START   TIME COMMAND
www-data  8901  2.1  2.1  45632 17234 ?    S    Jul20    1:45 php-fpm: pool www
apache2   8902  1.8  1.8  34521 14523 ?    S    Jul20    0:30 apache2 -k start
```

### Lessons
- Monitor PostgreSQL for long-running queries
- Set up `pg_stat_statements` extension for query performance tracking
- Add indexes before queries become problematic
- Consider connection pooling with PgBouncer