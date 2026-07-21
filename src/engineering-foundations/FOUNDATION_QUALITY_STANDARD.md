# Foundation Quality Standard

## Purpose

Quality criteria for Engineering Foundations and Skill Packs. Ensures consistency, reusability, and production readiness.

---

## Foundation Quality Criteria

| Criterion | Requirement |
|-----------|-------------|
| Architecture | Complete architecture description with key components |
| Core Concepts | All essential concepts documented with clear explanations |
| Common Components | Standard tools and utilities listed and described |
| Common Failures | ≥10 typical failure modes with symptoms and causes |
| Troubleshooting | Structured methodology with decision trees |
| Best Practices | ≥10 operational recommendations |
| Risk Awareness | High-risk operations documented with warnings |
| References | ≥5 external documentation links |
| Examples | ≥2 worked troubleshooting scenarios |
| Self-Contained | Foundation can be understood independently |

## Skill Pack Quality Criteria

| Criterion | Requirement |
|-----------|-------------|
| Manifest | Valid manifest.yaml with all required fields |
| Technology | Comprehensive technology definitions |
| Detection | ≥10 detection rules with confidence scoring |
| Workflows | ≥8 state machine workflows with full lifecycle |
| Commands | ≥50 commands with purpose, risk, examples |
| Knowledge | References foundations instead of duplicating |
| Guidance | Evidence-based recommendations with risk warnings |
| Best Practices | Technology-specific operational recommendations |
| Documentation | ≥8 markdown docs covering all aspects |
| Examples | ≥3 worked troubleshooting scenarios |
| Tests | Coverage for detection, reasoning, workflow, context |

## Validation Checklist

- [ ] Foundations are self-contained
- [ ] Skill Packs reference foundations
- [ ] No duplicated foundational knowledge
- [ ] All detection rules have confidence ≥0.70
- [ ] All workflows include risk assessment
- [ ] All commands include risk classification
- [ ] Cross-foundation relationships documented
- [ ] Loading order validated
- [ ] All references are valid
- [ ] Quality standards met

## Scoring

| Score | Description |
|-------|-------------|
| 5 | Exceeds requirements |
| 4 | Meets all requirements |
| 3 | Meets minimum |
| 2 | Partially meets |
| 1 | Fails requirements |
| 0 | Not implemented |

**Approval threshold:** All criteria ≥ 4, average ≥ 4.5

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial standard |