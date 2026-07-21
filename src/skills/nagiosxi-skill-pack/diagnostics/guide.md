# Nagios XI Diagnostic Procedures

## Overview

Systematic diagnostic procedures for common Nagios XI issues. Each procedure follows: identify symptoms, collect evidence, analyze data, recommend actions.

## Nagios Core Issues

### Service Not Running

**Symptoms**:
- Nagios XI web interface unavailable
- Services not being monitored
- Status showing stale data

**Diagnostic Steps**:
```bash
# Check Nagios process
ps aux | grep nagios

# Check service status
/etc/init.d/nagios status

# Check error log
tail -n 100 /var/log/nagios/nagios.log

# Validate configuration
/usr/local/nagios/bin/nagios -v /usr/local/nagios/etc/nagios.cfg
```

**Evidence Required**:
- Process status
- Error log entries
- Configuration validation result

**Remediation**:
1. Fix configuration errors if any
2. Restart Nagios service
3. Monitor startup log for errors
4. Verify web interface is accessible

### High CPU Usage

**Symptoms**:
- Slow response times
- Check execution delays
- High CPU utilization

**Diagnostic Steps**:
```bash
# Check CPU usage
top -bn1 | head -20

# Check Nagios processes
ps aux --sort=-%cpu | head -20

# Check check execution times
tail -n 200 /var/log/nagios/nagios.log | grep "EXECUTE"
```

**Remediation**:
1. Review check intervals
2. Optimize plugin execution
3. Check for stuck checks
4. Consider distributed monitoring

## NDOUtil Issues

### NDOUtil Not Syncing

**Symptoms**:
- Web interface shows stale data
- NDOUtil process running but not writing
- Database tables not updating

**Diagnostic Steps**:
```bash
# Check NDOUtil process
ps aux | grep ndo2db

# Check NDOUtil log
tail -n 100 /var/log/nagiosxi/ndo2db.log

# Check database updates
mysql -u root -p nagiosxi -e "SELECT NOW(), UNIX_TIMESTAMP(last_update) FROM nagios_programstatus;"

# Compare with actual Nagios time
date +%s
```

**Evidence Required**:
- NDOUtil log entries
- Database last_update time
- System current time

**Remediation**:
1. Check database connectivity
2. Verify NDOUtil configuration
3. Check MySQL performance
4. Restart NDOUtil service

### Database Performance Issues

**Symptoms**:
- Slow web interface
- NDOUtil sync delays
- MySQL CPU high

**Diagnostic Steps**:
```sql
-- Check slow queries
SHOW PROCESSLIST;

-- Check table sizes
SELECT table_name, ROUND(data_length/1024/1024,2) AS 'Size MB'
FROM information_schema.tables
WHERE table_schema = 'nagios'
ORDER BY data_length DESC;

-- Check indexes
SHOW INDEX FROM nagios_log;
```

**Evidence Required**:
- Query execution times
- Table sizes
- Index status

**Remediation**:
1. Optimize database indexes
2. Clean old data
3. Increase buffer pool size
4. Consider database partitioning

## Web Interface Issues

### Login Problems

**Symptoms**:
- Login failures
- Session timeouts
- Authentication errors

**Diagnostic Steps**:
```bash
# Check Apache logs
tail -n 100 /var/log/httpd/error_log

# Check PHP logs
tail -n 100 /var/log/httpd/php_error.log

# Check database connection
mysql -u root -p nagiosxi -e "SELECT COUNT(*) FROM users;"
```

**Remediation**:
1. Check Apache service status
2. Verify database credentials
3. Check PHP session configuration
4. Clear browser cache and cookies

### Reports Not Generating

**Symptoms**:
- Reports showing no data
- Report generation failing
- CSV/PNG export failing

**Diagnostic Steps**:
```bash
# Check report generation logs
tail -n 100 /var/log/httpd/error_log

# Check database connectivity
mysql -u root -p nagiosxi -e "SELECT COUNT(*) FROM nagios_log;"

# Check disk space
df -h
```

**Remediation**:
1. Check database connectivity
2. Verify report configuration
3. Check disk space
4. Restart Apache service

## References

- Nagios XI Diagnostics: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios XI Troubleshooting: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios XI Support: https://support.nagios.com/