# Nagios XI Common Failure Patterns

## Overview

This reference documents common Nagios XI failure patterns, their symptoms, diagnosis methods, and recommended actions.

## Common Failures

### NDOUtil Database Connection Failure

**Symptoms**:
- Web interface shows no data
- NDOUtil process stopped
- MySQL connection errors in logs

**Diagnosis**:
```bash
# Check NDOUtil process
ps aux | grep ndo2db

# Check NDOUtil log
tail -n 50 /var/log/nagiosxi/ndo2db.log

# Check database connectivity
mysql -u root -p nagiosxi -e "SELECT COUNT(*) FROM nagios_hosts;"
```

**Remediation**:
1. Check MySQL service status
2. Verify database credentials in `/etc/nagiosxi/db.php`
3. Check NDOUtil configuration in `/etc/nagios/nagios.cfg`
4. Restart NDOUtil service
5. Verify database table integrity

### NRPE Connection Failure

**Symptoms**:
- Service shows CRITICAL or UNKNOWN
- NRPE check timeout
- Connection refused errors

**Diagnosis**:
```bash
# Test NRPE connectivity
telnet <host> 5666

# Test NRPE check
/usr/lib/nagios/plugins/check_nrpe -H <host> -c check_load

# Check NRPE configuration on remote host
cat /etc/nagios/nrpe.cfg
```

**Remediation**:
1. Verify NRPE service is running on remote host
2. Check firewall rules allow port 5666
3. Verify NRPE allowed hosts configuration
4. Restart NRPE service on remote host
5. Check NRPE log for errors

### Nagios Configuration Error

**Symptoms**:
- Configuration apply fails
- Service restart fails
- Validation errors in web interface

**Diagnosis**:
```bash
# Validate configuration
/usr/local/nagios/bin/nagios -v /usr/local/nagios/etc/nagios.cfg

# Check error log
tail -n 100 /var/log/nagios/nagios.log
```

**Remediation**:
1. Fix configuration syntax errors
2. Remove duplicate object definitions
3. Verify all referenced objects exist
4. Reapply configuration
5. Restart Nagios service

### Database Growth Issue

**Symptoms**:
- Slow web interface
- MySQL disk space warnings
- NDOUtil sync delays

**Diagnosis**:
```sql
-- Check table sizes
SELECT table_name, ROUND(data_length/1024/1024,2) AS 'Size MB'
FROM information_schema.tables
WHERE table_schema = 'nagios'
ORDER BY data_length DESC;

-- Check for large log tables
SELECT table_name, table_rows
FROM information_schema.tables
WHERE table_schema = 'nagios'
ORDER BY table_rows DESC;
```

**Remediation**:
1. Clean old data from nagios_log table
2. Implement log rotation policies
3. Archive old historical data
4. Optimize database indexes
5. Consider adding more disk space

### Notification Failure

**Symptoms**:
- Alerts not reaching contacts
- Notification log shows errors
- Contact groups misconfigured

**Diagnosis**:
```bash
# Check notification queue
tail -n 50 /var/log/nagios/nagios.log | grep NOTIFICATION

# Check SMTP configuration
postconf mydestination

# Test email delivery
sendmail -t << EOF
Subject: Test
To: admin@example.com
From: nagios@monitoring

Test notification
EOF
```

**Remediation**:
1. Verify SMTP configuration
2. Check contact email addresses
3. Verify contact groups are properly configured
4. Test notification command
5. Check notification permissions

### Flapping Detection Issue

**Symptoms**:
- Service marked as flapping
- Notifications suppressed
- Rapid state changes

**Diagnosis**:
```bash
# Check for flapping services
tail -n 100 /var/log/nagios/nagios.log | grep FLAP

# Check flapping detection configuration
grep flap_detection /etc/nagios/nagios.cfg
```

**Remediation**:
1. Review check intervals for affected services
2. Adjust flapping detection thresholds
3. Check for network instability
4. Investigate root cause of state changes
5. Clear flap detection manually if needed

## References

- Nagios XI Troubleshooting: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios XI Common Issues: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios XI Support: https://support.nagios.com/