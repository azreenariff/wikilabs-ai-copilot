# Nagios Log Server Terminology Glossary

## Logging Terms

| Term | Definition |
|------|-----------|
| **Log Source** | A system or application generating logs |
| **Log File** | A file containing log entries |
| **Log Entry** | A single line or record in a log |
| **Log Level** | Severity of a log entry (DEBUG, INFO, WARN, ERROR, FATAL) |
| **Log Rotation** | Automatic management of log file size and retention |

## Collection Terms

| Term | Definition |
|------|-----------|
| **Log Server Agent** | Nagios Log Server agent for remote log collection |
| **Logstash** | Log collection and parsing engine |
| **Input** | Logstash input plugin for log collection |
| **Filter** | Logstash filter plugin for log parsing |
| **Output** | Logstash output plugin for data destination |
| **Index** | Elasticsearch index for log storage |
| **Shipper** | Component that ships logs to Logstash |

## Search Terms

| Term | Definition |
|------|-----------|
| **Query** | Search expression for finding log entries |
| **Filter** | Search filter for narrowing results |
| **Date Range** | Time-based search filter |
| **Source** | Log source filter |
| **Level** | Log level filter |
| **Host** | Host name filter |
| **Pattern** | Log pattern for alerting |

## Alert Terms

| Term | Definition |
|------|-----------|
| **Log Alert** | Alert triggered by log pattern matching |
| **Threshold** | Count threshold for alert triggering |
| **Pattern Match** | Regular expression match on log content |
| **Escalation** | Additional notification for recurring alerts |
| **Dampening** | Limiting alert frequency |

## Elasticsearch Terms

| Term | Definition |
|------|-----------|
| **Index** | Collection of documents with similar characteristics |
| **Document** | Single log entry stored in Elasticsearch |
| **Cluster** | Collection of Elasticsearch nodes |
| **Node** | Single Elasticsearch server instance |
| **Shard** | Horizontal partition of an index |
| **Replica** | Duplicate shard for redundancy |
| **Mapping** | Definition of field types in an index |
| **Template** | Index template for automatic index creation |

## Logstash Terms

| Term | Definition |
|------|-----------|
| **Pipeline** | Complete log processing chain (input→filter→output) |
| **Grok** | Log pattern matching and parsing |
| **Mutate** | Field manipulation and transformation |
| **Date** | Timestamp parsing and normalization |
| **GeoIP** | Geographic enrichment of IP addresses |
| **User Agent** | Browser/application identification |
| **Filter** | Log transformation step |

## Administration Terms

| Term | Definition |
|------|-----------|
| **Log Server Settings** | Nagios Log Server configuration options |
| **Search Index** | Elasticsearch index configuration |
| **Log Parser** | Log parsing pattern configuration |
| **Alert Config** | Alert definition and threshold settings |
| **Backup** | Elasticsearch cluster and configuration backup |
| **Restore** | Elasticsearch cluster and configuration restoration |

## References

- Nagios Log Server Documentation: https://assets.nagios.com/downloads/nagiosxi/docs/
- Elasticsearch Terminology: https://www.elastic.co/guide/en/elasticsearch/reference/
- Logstash Terminology: https://www.elastic.co/guide/en/logstash/