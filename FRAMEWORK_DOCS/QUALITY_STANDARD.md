# Quality Standard for Engineering Skill Packs

## Overview

The Quality Standard defines the minimum requirements for all Engineering Skill Packs. Every skill pack must meet these criteria to be considered production-ready.

## Quality Dimensions

Each skill pack is evaluated across 10 quality dimensions:

| Dimension | Score Range | Target |
|-----------|------------|--------|
| **Knowledge Coverage** | 0-10 | ≥8/10 |
| **Workflow Coverage** | 0-10 | ≥8/10 |
| **Reasoning Coverage** | 0-10 | ≥8/10 |
| **Detection Coverage** | 0-10 | ≥8/10 |
| **Command Coverage** | 0-10 | ≥8/10 |
| **Guidance Quality** | 0-10 | ≥8/10 |
| **Safety Compliance** | 0-10 | ≥9/10 |
| **Documentation** | 0-10 | ≥7/10 |
| **Examples** | 0-10 | ≥6/10 |
| **Testing** | 0-10 | ≥7/10 |

## Dimension Definitions

### 1. Knowledge Coverage

**Definition**: How comprehensively the skill pack covers the technology's domain.

**Criteria**:
- Core architecture documented
- Key components explained
- Common operations covered
- Configuration management included
- Security practices documented
- Backup and recovery procedures
- Performance optimization guidance
- High availability patterns

### 2. Workflow Coverage

**Definition**: Number and quality of state-machine troubleshooting workflows.

**Criteria**:
- ≥5 state machine workflows for common failure patterns
- Each workflow includes: states, transitions, evidence requirements, decision trees
- Workflows cover: detection → diagnosis → remediation → verification
- Risk assessment included in each workflow
- Rollback strategies provided

### 3. Reasoning Coverage

**Definition**: Quality of diagnostic reasoning patterns.

**Criteria**:
- ≥3 diagnostic reasoning patterns (decision trees)
- Confidence scoring methodology defined
- Evidence quality assessment included
- Common reasoning pitfalls documented
- Version-aware reasoning considered

### 4. Detection Coverage

**Definition**: Number and accuracy of detection rules for context signals.

**Criteria**:
- ≥10 detection rules per skill pack
- Rules cover CLI commands, browser URLs, window titles, text patterns
- Confidence scoring for each detection rule
- Detection rules validated against real-world patterns
- Detection rules categorized by signal type

### 5. Command Coverage

**Definition**: Completeness of command knowledge base.

**Criteria**:
- ≥50 command entries per skill pack
- Commands cover all major categories
- Each command includes: purpose, parameters, risk level, usage examples
- Verification steps provided for each command
- Documentation references included

### 6. Guidance Quality

**Definition**: Quality and usefulness of engineering guidance.

**Criteria**:
- Engineering reasoning principles documented
- Safety rules clearly defined
- Risk assessment framework provided
- Command explanation standards set
- Operational guidelines included

### 7. Safety Compliance

**Definition**: Adherence to safety constraints and guidelines.

**Criteria**:
- Never execute commands — advisory only
- Risk levels defined for all commands
- Rollback strategies included
- Warning messages provided for high-risk operations
- Human-in-the-loop enforcement
- Audit trail documentation

### 8. Documentation

**Definition**: Quality and completeness of external documentation.

**Criteria**:
- External documentation references included
- Architecture documentation provided
- Configuration reference available
- Troubleshooting guide complete
- Best practices documented

### 9. Examples

**Definition**: Quality and quantity of worked examples.

**Criteria**:
- ≥3 worked examples per skill pack
- Examples cover common scenarios
- Examples show evidence → reasoning → recommendation flow
- Examples include before/after states
- Examples reference real-world situations

### 10. Testing

**Definition**: Validation procedures and test coverage.

**Criteria**:
- Validation tests defined
- Detection rules tested
- Knowledge accuracy verified
- Workflow completeness checked
- Safety rules compliance verified

## Quality Scoring

### Overall Score

```
Overall Score = Average of all 10 dimensions
Minimum Threshold: ≥7.0 for production release
Recommended: ≥8.0 for enterprise-ready
```

### Quality Tiers

| Tier | Score | Description |
|------|-------|-------------|
| **Bronze** | 5.0-6.9 | Minimum viable — limited coverage |
| **Silver** | 7.0-7.9 | Production-ready — adequate coverage |
| **Gold** | 8.0-8.9 | Enterprise-ready — comprehensive coverage |
| **Platinum** | 9.0+ | Reference quality — full coverage |

## Quality Review Process

1. **Self-Review**: Skill pack author reviews against quality checklist
2. **Peer Review**: Another engineer reviews and provides feedback
3. **Automated Testing**: Detection rules and command coverage tested
4. **Stakeholder Review**: Subject matter expert validates knowledge accuracy
5. **Production Gate**: Quality score ≥7.0 required for release

## Quality Metrics Tracking

Track quality metrics over time:
- Detection rule accuracy rate
- User feedback on guidance quality
- Time to first recommendation
- Recommendation acceptance rate
- False positive rate on detections
- Knowledge base coverage gaps

## References

- Quality Standards: https://docs.wikilabs.ai/quality/
- Skill Pack Quality: https://docs.wikilabs.ai/skills/quality/
- Engineering Quality Framework: https://docs.wikilabs.ai/framework/