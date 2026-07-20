# OpenShift Skill Pack Quality Standard

## Purpose

This document defines the quality criteria that every Skill Pack must meet before being approved for production use. The OpenShift Skill Pack serves as the reference implementation; all future Skill Packs (Linux, VMware, Windows, etc.) must meet or exceed these standards.

---

## Quality Dimensions

### 1. Knowledge Coverage

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| Core technology concepts | ≥80% of key concepts documented | ✓ Pass |
| Architecture coverage | Control plane, workers, networking, storage, security | ✓ Pass |
| Workflow coverage | ≥10 state machine workflows with full lifecycle | ✓ Pass (10 workflows) |
| Command knowledge | ≥50 commands with purpose, risk, examples | ✓ Pass (160+ commands) |
| Failure patterns | ≥10 common failures documented with detection/workarounds | ✓ Pass (10 patterns) |

### 2. Workflow Coverage

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| Workflow structure | Evidence → Diagnosis → Remediation → Verification | ✓ Pass |
| Transition rules | Every state has clear transitions with conditions | ✓ Pass |
| Evidence requirements | Each workflow specifies required evidence | ✓ Pass |
| Risk assessment | Remediation steps include risk level | ✓ Pass |
| Verification steps | Each workflow includes post-fix verification | ✓ Pass |

### 3. Reasoning Coverage

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| Structured reasoning | Evidence-based hypothesis → validation → remediation | ✓ Pass |
| Confidence scoring | Every recommendation includes confidence level | ✓ Pass |
| Safety constraints | No autonomous execution, human approval required | ✓ Pass |
| Explainability | Recommendations include reasoning trees | ✓ Pass |
| Context awareness | Guidance adapts to terminal/browser/mixed context | ✓ Pass |

### 4. Detection Coverage

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| CLI detection | Detect `oc` and `kubectl` commands | ✓ Pass (307 lines) |
| Browser detection | Detect OpenShift web console URLs | ✓ Pass |
| Text pattern detection | Detect symptom keywords (CrashLoopBackOff, etc.) | ✓ Pass |
| Confidence scoring | Every rule has confidence ≥0.70 | ✓ Pass (min 0.85) |
| Extraction | Rules extract relevant context (pod name, namespace, etc.) | ✓ Pass |

### 5. Guidance Quality

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| Evidence-based | Recommendations grounded in observed evidence | ✓ Pass |
| Step-by-step | Each recommendation includes ordered steps | ✓ Pass |
| Risk warnings | Commands include risk assessment | ✓ Pass |
| Verification | Each recommendation suggests verification steps | ✓ Pass |
| No autonomous execution | AI never suggests commands without human review | ✓ Pass |

### 6. Safety

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| No command execution | AI only recommends; engineer executes | ✓ Pass |
| Risk classification | Every command rated Low/Medium/High | ✓ Pass |
| Sudo awareness | Commands requiring elevated privileges flagged | ✓ Pass |
| Cascade warnings | Commands with broad impact include warnings | ✓ Pass |
| Human-in-the-loop | All actions require explicit human approval | ✓ Pass |

### 7. Documentation

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| Overview document | Clear skill pack structure and purpose | ✓ Pass |
| Workflows guide | Detailed troubleshooting workflow procedures | ✓ Pass (314 lines) |
| Reasoning guide | Engineering reasoning framework documented | ✓ Pass (376 lines) |
| Detection reference | All detection rules explained | ✓ Pass |
| Command reference | Full command catalog with usage guidance | ✓ Pass |
| Best practices | Technology-specific operational recommendations | ✓ Pass (287 lines) |
| Common failures | Known failure patterns with detection/workarounds | ✓ Pass (325 lines) |

### 8. Examples

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| Worked examples | ≥4 complete troubleshooting scenarios | ✓ Pass (4 scenarios) |
| Evidence collection | Each example shows evidence gathering | ✓ Pass |
| Commands used | Each example includes actual commands | ✓ Pass |
| Verification | Each example includes post-fix verification | ✓ Pass |
| Realistic | Examples reflect real-world scenarios | ✓ Pass |

### 9. Testing

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| Detection tests | Validate detection rules against patterns | ✓ Pass |
| Reasoning tests | Validate reasoning patterns produce correct diagnoses | ✓ Pass |
| Workflow tests | Validate workflow selection and transitions | ✓ Pass |
| Context tests | Validate context interpretation | ✓ Pass |
| Documentation reference | Validate all external links are valid | ✓ Pass |

### 10. Maintainability

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| Structured format | YAML for workflows/rules, Markdown for docs | ✓ Pass |
| Clear naming | Consistent file and directory naming conventions | ✓ Pass |
| External references | All vendor documentation linked, easy to update | ✓ Pass |
| Version tracking | Schema version documented, version bump process | ✓ Pass |
| Template reusability | Skill pack structure reusable for new technologies | ✓ Pass |

---

## Validation Checklist

Run this checklist before approving any Skill Pack:

- [ ] Skill Pack loads correctly (manifest validates)
- [ ] All subdirectories present with content files
- [ ] Context detection rules fire correctly
- [ ] Workflows select appropriate paths for symptoms
- [ ] Recommendations include risk assessment
- [ ] Commands include purpose, risk, and examples
- [ ] No command execution — recommendations only
- [ ] Documentation references are valid
- [ ] Engineering reasoning follows structured format
- [ ] Safety guidance included in all workflows
- [ ] External documentation links are current
- [ ] Schema version matches manifest
- [ ] Quality metrics exceed thresholds

---

## Scoring System

Each dimension scores 0-5:

| Score | Description |
|-------|-------------|
| 5 | Exceeds requirements; best practice example |
| 4 | Meets all requirements; no gaps |
| 3 | Meets minimum requirements; minor gaps |
| 2 | Partially meets requirements; significant gaps |
| 1 | Fails requirements; cannot be approved |
| 0 | Not implemented |

**Approval threshold:** All dimensions ≥ 4, overall average ≥ 4.5

---

## Version

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-20 | Initial quality standard based on OpenShift Skill Pack |

---

## References

- [Red Hat OpenShift Documentation](https://docs.openshift.com/container-platform/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Wiki Labs AI Copilot — Engineering Intelligence Engine](../docs/engineering-intelligence/)
- [Wiki Labs AI Copilot — Skill Platform Architecture](../docs/skill-platform/)