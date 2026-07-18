# Technology Recognition Engine

> Wiki Labs AI Copilot v0.4.0-alpha

## Purpose

Determine which technologies the engineer is actively working with, with evidence and confidence scores.

## Supported Domains

### Infrastructure

- Linux
- Windows
- OpenShift
- Kubernetes
- VMware vSphere
- Red Hat Virtualization
- Docker
- Ansible

### Monitoring

- Nagios XI
- Nagios Log Server
- Checkmk
- Grafana

### Database

- MySQL
- EDB PostgreSQL
- Microsoft SQL Server

### Development

- Git

## Evidence Sources

The engine matches observation events against DetectionRules loaded from Skill packages:

| Detection Rule Type | Evidence Source |
|---|---|
| `browser_url` | Browser URL patterns |
| `browser_title` | Browser page titles |
| `terminal_command` | Executed terminal commands |
| `active_application` | Foreground application name |
| `file_pattern` | Opened file paths |
| `workspace_context` | Workspace metadata |
| `conversation_keyword` | Conversation content |

## Output

```yaml
technology: OpenShift
confidence: 0.96
evidence:
  - "oc command detected"
  - "OpenShift console detected"
```

## Architecture

DetectionRules are loaded from Skill packages:

```
skill-name/
  detection_rules.yaml
```

The engine does not contain any technology-specific logic. Technology knowledge comes entirely from Skills.

## Implementation

- `src/technology_recognition/src/engine.rs` — DetectionEngine core
- `src/technology_recognition/src/pipeline.rs` — Evidence aggregation pipeline
- `src/technology_recognition/src/aggregator.rs` — Multi-source evidence fusion
- `src/technology_recognition/src/lib.rs` — Public API

## Testing

See `src/technology_recognition/src/pipeline.rs` and `src/technology_recognition/src/aggregator.rs` for test coverage.