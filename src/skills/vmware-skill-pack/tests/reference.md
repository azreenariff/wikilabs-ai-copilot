# VMware vSphere Engineering — Test Reference

## Purpose

This directory contains test specifications and validation criteria for the VMware vSphere engineering skill pack.

## Test Categories

### 1. Detection Rule Testing
- Verify detection rules match expected patterns
- Test rule priority and severity assignment
- Validate false positive rate

### 2. Workflow Completeness
- Verify all workflows have complete phase definitions
- Check command syntax for correctness
- Validate phase transitions

### 3. Knowledge Base Accuracy
- Verify technical accuracy of content
- Cross-reference with VMware documentation
- Test examples against known scenarios

### 4. Integration Testing
- Verify YAML files parse correctly
- Test command references for accuracy
- Validate workflow state machines

## Test Matrix

| Test Area | Pass Criteria | Notes |
|-----------|--------------|-------|
| YAML Validation | All .yaml files parse | No syntax errors |
| Detection Rules | >95% accuracy | Low false positive rate |
| Workflows | All phases defined | Commands reference valid |
| Knowledge Base | Cross-referenced | Matches VMware docs |
| Examples | Realistic scenarios | Solutions verified |

## Version Information

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-07-21 | Initial test reference |