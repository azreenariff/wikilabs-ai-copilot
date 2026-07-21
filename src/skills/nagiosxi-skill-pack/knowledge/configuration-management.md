# Nagios XI Configuration Management

## Overview

Nagios XI configuration is managed through both the web interface (WATO) and direct file editing. Understanding the configuration model is critical for reliable monitoring operations.

## Configuration Model

### File Structure

```
/etc/nagiosxi/
├── components/           # Component configurations
│   ├── nagioscore/
│   ├── ndoutils/
│   └── nagiosql/
├── cgi.cfg              # CGI configuration
├── nmis.cfg             # NMIS bridge config
├── config.php           # PHP configuration
├── db.php               # Database credentials
└── settings.cfg         # XI settings

/etc/nagios/
├── nagios.cfg           # Main Nagios Core config
├── cgi.cfg              # CGI configuration
├── commands.cfg         # Command definitions
├── timeperiods.cfg      # Time period definitions
├── contacts.cfg         # Contact definitions
├── contactgroups.cfg    # Contact group definitions
├── hostgroups.cfg       # Host group definitions
├── servicegroups.cfg    # Service group definitions
├── hosts/               # Host configuration directory
│   └── *.cfg
├── services/            # Service configuration directory
│   └── *.cfg
├── services/            # Service configuration directory
│   └── *.cfg
└── conf/                # Custom configuration snippets
```

### Configuration Flow

```
Web Interface (WATO)
       │
       ├──→ nagiosql database
       │
       ├──→ config generation
       │
       └──→ /etc/nagios/*.cfg
              │
              └──→ Nagios Core restart/reload
```

## Configuration Best Practices

### 1. Use WATO for Primary Configuration

- WATO provides validation and error checking
- Configuration is stored in the database for backup
- Changes are tracked in the configuration history
- Rollback is supported through configuration versions

### 2. Backup Before Changes

```bash
# Backup Nagios XI configuration
tar czf /backup/nagiosxi-config-$(date +%Y%m%d).tar.gz /etc/nagios /etc/nagiosxi

# Backup database
mysqldump -u root -p nagiosxi > /backup/nagiosxi-db-$(date +%Y%m%d).sql
```

### 3. Apply Configuration Changes

```bash
# From web interface:
# Admin → Manage Config → Apply Configuration

# From command line:
/etc/init.d/nagios restart
/etc/init.d/ndo2db restart
```

### 4. Validate Configuration

```bash
# Test configuration before applying
/usr/local/nagios/bin/nagios -v /usr/local/nagios/etc/nagios.cfg

# Check for errors in output
# Should show: "Total Warnings: 0" and "Total Errors: 0"
```

## Configuration Elements

### Host Configuration

```
define host {
    host_name               webserver01
    alias                   Web Server 01
    address                 192.168.1.100
    check_period            24x7
    notification_period     24x7
    max_check_attempts      5
    normal_check_interval   5
    retry_check_interval    1
    contact_groups          admins
    notifications_enabled   1
    check_command           check-host-alive
}
```

### Service Configuration

```
define service {
    host_name               webserver01
    service_description     HTTP
    check_period            24x7
    notification_period     24x7
    max_check_attempts      3
    normal_check_interval   5
    retry_check_interval    1
    contact_groups          admins
    notifications_enabled   1
    check_command           check_http!-H webserver01 -w 5 -c 10
}
```

### Contact Configuration

```
define contact {
    contact_name            admin
    alias                   System Admin
    email                   admin@example.com
    host_notifications_enabled  1
    service_notifications_enabled 1
    notification_options    d,u,r,f,s
    service_notification_options d,u,r,f
    notification_interval   30
    first_notification_delay  0
}
```

### Time Period Configuration

```
define timeperiod {
    timeperiod_name 24x7
    alias           24 Hours A Day, 7 Days A Week
    sunday          00:00-24:00
    monday          00:00-24:00
    tuesday         00:00-24:00
    wednesday       00:00-24:00
    thursday        00:00-24:00
    friday          00:00-24:00
    saturday        00:00-24:00
}

define timeperiod {
    timeperiod_name workhours
    alias           Business Hours
    monday          08:00-18:00
    tuesday         08:00-18:00
    wednesday       08:00-18:00
    thursday        08:00-18:00
    friday          08:00-18:00
}
```

## Advanced Configuration

### Command Customization

```
define command {
    command_name    check_custom
    command_line    $USER1$/check_custom -H $HOSTADDRESS$ -w $ARG1$ -c $ARG2$
}
```

### Dependency Configuration

```
define servicedependency {
    host_name                   webserver01
    service_description         HTTP
    dependent_host_name         database01
    dependent_service_description MySQL
    failure_prediction_enabled  1
}
```

### Dependency Trigger

```
define serviceescalation {
    host_name               webserver01
    service_description     HTTP
    escalation_period       workhours
    first_notification_delay  0
    notification_interval   15
    contact_groups          senior-admins
}
```

### Scheduler Downtime

```
define scheduleddowntime {
    host_name               webserver01
    service_description     HTTP
    author                  admin
    comment                 Maintenance window
    start_time              2024-01-15 02:00:00
    end_time                2024-01-15 04:00:00
}
```

## Troubleshooting Configuration Issues

### Common Configuration Errors

1. **Syntax errors**: Check config files for typos and missing brackets
2. **Missing objects**: Ensure referenced hosts/services exist
3. **Circular dependencies**: Check for circular host/service dependencies
4. **Duplicate names**: Verify unique host_name and service_description
5. **Invalid commands**: Verify command definitions exist

### Configuration Validation

```bash
# Validate all configuration files
/usr/local/nagios/bin/nagios -v /usr/local/nagios/etc/nagios.cfg

# Check specific file
/usr/local/nagios/bin/nagios -v /usr/local/nagios/etc/hosts/webserver01.cfg

# Test with specific object
/usr/local/nagios/libexec/check_http -H webserver01 -w 5 -c 10
```

### Debugging Tips

1. Check `/var/log/nagios/nagios.log` for errors
2. Verify NDOUtil is running: `ps aux | grep ndo2db`
3. Check MySQL database connectivity: `mysql -u root -p nagiosxi`
4. Test web interface logs: `/var/log/nagiosxi/components/nagioscore/nagios.log`
5. Verify permissions on config files and directories

## References

- Nagios XI Configuration Guide: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios Core Configuration: https://docs.nagios.com/nagioscore/
- Nagios XI Administration: https://assets.nagios.com/downloads/nagiosxi/docs/