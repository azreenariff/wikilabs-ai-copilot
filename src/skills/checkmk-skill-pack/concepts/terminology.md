# Checkmk Terminology Glossary

## Core Terms

| Term | Definition |
|------|-----------|
| **Site** | An isolated Checkmk instance under OMD |
| **Host** | A monitored network device or server |
| **Service** | A specific function or metric on a host |
| **Check** | A monitoring action that evaluates a service |
| **Plugin** | Code that performs a specific check |
| **Agent** | Software installed on hosts for data collection |
| **WATO** | Web-based configuration interface |
| **Ruleset** | Configuration rule for check parameters |

## Micro Core Terms

| Term | Definition |
|------|-----------|
| **Micro Core (CMC)** | High-performance check engine |
| **Parallelization** | Concurrent check execution |
| **Status Engine** | Check result processing |
| **Check Plugin** | CMC check implementation |
| **Agent Plugin** | Data collection from agent |
| **Check Result** | Output from a check execution |
| **Performance Data** | Numeric metrics from checks |

## Livestatus Terms

| Term | Definition |
|------|-----------|
| **Livestatus** | Unix socket API for status data |
| **Query** | Livestatus request for monitoring data |
| **Column** | Field in a Livestatus response |
| **Filter** | Livestatus query filter |
| **Sort** | Livestatus response sorting |
| **Limit** | Maximum results in response |

## Agent Terms

| Term | Definition |
|------|-----------|
| **Agent** | check_mk_agent on managed hosts |
| **Agent Plugin** | Script that provides monitoring data |
| **Agent Output** | Formatted output from agent |
| **Agent Cache** | Cached agent data (TTL-based) |
| **Piggyback** | Cluster data from parent to child |
| **IPV4/IPv6** | IP protocol support |
| **TLS** | Secure agent communication |

## Notification Terms

| Term | Definition |
|------|-----------|
| **Notification** | Alert sent to contact |
| **Contact** | Recipient of notifications |
| **Escalation** | Additional notification rules |
| **Dampening** | Alert frequency reduction |
| **Notification Rules** | Conditions for sending notifications |
| **Contact Groups** | Groups of contacts |
| **Service Notifications** | Service-specific alerts |
| **Host Notifications** | Host-specific alerts |

## Cluster Terms

| Term | Definition |
|------|-----------|
| **Cluster** | Group of related hosts/services |
| **Piggyback** | Cluster parent-child data relay |
| **Kubernetes** | Kubernetes cluster monitoring |
| **VMware** | VMware vCenter monitoring |
| **SAP HANA** | SAP HANA database monitoring |
| **Active-Active** | Multi-node cluster monitoring |

## Performance Terms

| Term | Definition |
|------|-----------|
| **RRD** | Round-robin database for metrics |
| **Perfdata** | Performance data from checks |
| **Metric** | Numeric measurement |
| **Threshold** | Warning/critical boundaries |
| **Trending** | Metric trend analysis |
| **Visualization** | Metric display in web UI |
| **Export** | Data export (CSV, PNG) |

## References

- Checkmk Glossary: https://docs.checkmk.com/master/en/glossary.html
- Checkmk Documentation: https://docs.checkmk.com/
- Checkmk Terminology: https://docs.checkmk.com/master/en/