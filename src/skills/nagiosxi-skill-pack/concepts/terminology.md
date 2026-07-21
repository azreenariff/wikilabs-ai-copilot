# Nagios XI Terminology Glossary

## Monitoring Terms

| Term | Definition |
|------|-----------|
| **Host** | A network device or server being monitored |
| **Service** | A specific function or protocol on a host (HTTP, SSH, etc.) |
| **Check** | A test performed on a host or service to determine status |
| **Plugin** | A script that performs a specific check |
| **Command** | A predefined check configuration with parameters |
| **Contact** | A person or group notified of alerts |
| **Contact Group** | A group of contacts for notification routing |
| **Host Group** | A logical grouping of hosts |
| **Service Group** | A logical grouping of services |

## Status States

| State | Value | Description |
|-------|-------|-------------|
| **OK** | 0 | Service is functioning normally |
| **WARNING** | 1 | Service is degraded but operational |
| **CRITICAL** | 2 | Service is not functioning properly |
| **UNKNOWN** | 3 | Check could not be completed |
| **UP** | 0 | Host is reachable |
| **DOWN** | 1 | Host is not reachable |
| **UNREACHABLE** | 2 | Host status unknown (parent is down) |

## Alert Terms

| Term | Definition |
|------|-----------|
| **Hard State** | A state confirmed after max attempts |
| **Soft State** | A transient state during check attempts |
| **Notification** | An alert sent to contacts about state changes |
| **Acknowledgement** | A host/service issue that has been acknowledged |
| **Downtime** | Scheduled maintenance window suppressing notifications |
| **Flapping** | Rapid state changes causing notification storms |
| **Escalation** | Notification to additional contacts based on conditions |

## Configuration Terms

| Term | Definition |
|------|-----------|
| **cfg_file** | A configuration file containing host/service definitions |
| **contact_notification_options** | When to send notifications (d,u,r,f,s) |
| **check_period** | Time period when checks are active |
| **notification_period** | Time period when notifications are active |
| **max_check_attempts** | Number of checks before hard state |
| **normal_check_interval** | Standard check interval |
| **retry_check_interval** | Interval during soft state |
| **host_notifications_enabled** | Enable/disable host notifications |
| **service_notifications_enabled** | Enable/disable service notifications |

## Architecture Terms

| Term | Definition |
|------|-----------|
| **NDOUtil** | Nagios Data Out — database synchronizer |
| **NRPE** | Nagios Remote Plugin Executor |
| **NSClient++** | Nagios client for Windows |
| **Nagios Core** | The base monitoring engine |
| **Nagios XI** | Enterprise management layer |
| **NDODB** | Nagios database (MySQL) |
| **Event Broker** | Plugin architecture for event processing |
| **External Commands** | API for programmatic control of Nagios |

## Check Plugin Terms

| Term | Definition |
|------|-----------|
| **Exit Code** | Return code indicating status (0-3) |
| **Plugin Output** | Text output after the exit code |
| **Perfdata** | Performance data for graphing (|value=unit) |
| **Check Interval** | How often a check runs |
| **Timeout** | Maximum execution time for a check |
| **Freshness** | Whether check results are stale |
| **Passive Check** | Check result received from external source |
| **Active Check** | Check initiated by Nagios |

## Notification Terms

| Term | Definition |
|------|-----------|
| **Notification Methods** | Email, SMS, Webhook, custom commands |
| **Escalation Levels** | Additional contacts after threshold |
| **Notification Dampening** | Limiting notification frequency |
| **Host-Services-Host** | Escalation path configuration |
| **Notification Commands** | Predefined notification methods |

## References

- Nagios XI Glossary: https://assets.nagios.com/downloads/nagiosxi/docs/
- Nagios Core Documentation: https://docs.nagios.com/nagioscore/
- Monitoring Terminology: https://www.monitoringportal.org/glossary/