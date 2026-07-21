# Version Awareness Guidelines

## Overview

Version Awareness ensures the AI copilot provides accurate, version-specific guidance for technologies with multiple supported versions. Different versions may have different features, behaviors, and known issues.

## Version Tracking

### Version Sources

| Source | Description | Update Frequency |
|--------|-------------|-----------------|
| **Vendor Releases** | Official vendor release notes | As released |
| **Skill Pack Manifest** | Skill pack version scope | Per version bump |
| **Technology YAML** | Feature matrix per version | Per skill update |
| **Knowledge Base** | Version-specific knowledge | As discovered |

### Version Detection

The copilot detects versions through:
- CLI version output (`--version`, `-v`, `version()` functions)
- Configuration file metadata
- Browser version detection (for web-based tools)
- Active session context

```
Detection → Version String → Parse → Match → Select Guidance
```

## Version-Aware Guidance

### Guidance Selection Rules

1. **Primary Match**: Select guidance matching the detected version exactly
2. **Fallback Match**: If exact version not found, select guidance for nearest major version
3. **Cross-Version**: Note differences between detected version and guidance scope
4. **Unknown Version**: If version cannot be detected, provide generic guidance with version caveat

### Version Scope Examples

| Technology | Skill Pack Version | Guidance Scope |
|-----------|-------------------|---------------|
| MySQL | 8.0+, 8.4+ | MySQL 8.0 features, 8.4-specific changes noted |
| PostgreSQL | 15+, 16+ | PostgreSQL 15 features, 16 improvements |
| SQL Server | 2017+, 2019+, 2022+ | Per-version features and deprecations |
| Checkmk | 2.2+ | Core 2.2 features, Enterprise extras noted |
| Nagios XI | 5.x | Standard XI functionality |
| Ansible | Core 2.14+, AAP 2.4+ | Core vs Platform feature differences |

### Version-Specific Knowledge

Each skill pack should include:

1. **Feature Matrix**: What features exist in which versions
2. **Deprecation Notices**: Features removed or changed between versions
3. **Known Issues**: Version-specific bugs and workarounds
4. **Migration Notes**: Changes when upgrading between versions
5. **Breaking Changes**: Changes that break backward compatibility

## Version Detection Examples

### MySQL

```
Command: mysql --version
Output: mysql  Ver 8.0.35 for Linux on x86_64 (MySQL Community Server)
Parsed: Major=8, Minor=0, Patch=35
Scope: MySQL 8.0+ ✅
```

### PostgreSQL

```
Command: psql --version
Output: psql (PostgreSQL) 16.1
Parsed: Major=16, Minor=1, Patch=1
Scope: PostgreSQL 15+ ✅
```

### SQL Server

```
Command: SELECT @@VERSION
Output: Microsoft SQL Server 2022 (RTM) - 16.0.xxxx
Parsed: Version=2022, Build=16.0
Scope: SQL Server 2017+ ✅
```

### Checkmk

```
Command: check_mk --version
Output: check_mk 2.2.0p11
Parsed: Major=2, Minor=2, Patch=0
Scope: Checkmk 2.2+ ✅
```

## Version Handling in Workflows

Workflows should include version-specific decision paths:

```
IF version >= 8.4 THEN
    → Use MySQL 8.4-specific guidance
    → Note new features (e.g., data dictionary improvements)
ELIF version >= 8.0 THEN
    → Use MySQL 8.0 standard guidance
    → Note JSON improvements, common table expressions
ELSE
    → Use legacy guidance with caveats
    → Warn about deprecated features
END IF
```

## Version Update Process

When a new technology version is released:

1. **Identify**: Detect new version release (vendor announcement, changelog)
2. **Evaluate**: Assess impact on existing skill pack
3. **Update**: Add version-specific knowledge, update detection rules
4. **Test**: Verify detection rules work with new version
5. **Document**: Update changelog and version scope
6. **Communicate**: Alert users via changelog and release notes

## References

- Version Management Best Practices: https://semver.org/
- Technology Version Tracking: https://docs.wikilabs.ai/version-aware/
- Wiki Labs Version Awareness: https://docs.wikilabs.ai/framework/