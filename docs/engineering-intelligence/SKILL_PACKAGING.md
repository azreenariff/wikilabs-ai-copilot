# Skill Packaging

> Wiki Labs AI Copilot v0.8.0-alpha  
> Phase 11 — Enterprise Skill Platform

## Purpose

Skill packaging defines how skills are bundled, distributed, installed, and updated. The packaging system ensures skills are portable, versioned, and verifiable.

## Package Format

Skills are packaged as `.wls` (Wiki Labs Skill) archives. This is a gzip-compressed tar archive containing the full skill directory structure.

```bash
# Create a skill package
skill-sdk package /path/to/skill/ -o /output/skill.wls

# The resulting .wls file contains:
skill-package/
  manifest.yaml
  technology.yaml
  workflows.yaml
  detection_rules.yaml
  commands.yaml
  best_practices.yaml
  known_issues.yaml
  prompts/
  examples/
  documentation/
  tests/
  mcp/          (optional)
  metadata.json (auto-generated)
```

## Package Metadata

Each `.wls` file includes a `metadata.json` auto-generated during packaging:

```json
{
  "package_version": "1",
  "created_at": "2026-07-20T10:00:00Z",
  "packager": "skill-sdk 0.8.0",
  "checksum": "sha256:<hash>",
  "skill_id": "linux-engineering",
  "skill_version": "1.0.0",
  "vendor": "Wiki Labs",
  "manifest_schema_version": "1.0"
}
```

## Installation

Skills can be installed from `.wls` packages or from local directories.

### Install from package

```bash
skill-sdk install /path/to/skill.wls --target /path/to/skills/
```

This:
1. Extracts the archive to a skill directory
2. Validates the manifest against the schema
3. Resolves dependencies
4. Registers the skill in the Skill Runtime
5. Sets the skill to `Enabled` state

### Install from directory

```bash
skill-sdk install /path/to/skill-directory --target /path/to/skills/
```

Same process as above, but reads from an existing directory instead of extracting an archive.

### Install with validation only

```bash
skill-sdk validate /path/to/skill/
```

Validates without installing. Returns exit code 0 if valid, non-zero with error details if invalid.

## Uninstall

```bash
skill-sdk uninstall linux-engineering --target /path/to/skills/
```

This:
1. Deactivates the skill if currently active
2. Removes the skill directory
3. Removes the skill from the runtime registry
4. Updates the enabled skills list

## Updating Skills

```bash
skill-sdk update linux-engineering --package /path/to/skill-v2.wls
```

Update process:
1. Validates the new package manifest
2. Checks version compatibility (new version ≥ current version)
3. Deactivates the current skill
4. Extracts new files, preserving any custom overrides
5. Validates the new installation
6. Re-enables the skill
7. Notifies the runtime of the version change

### Version Compatibility

- The new version must be ≥ the installed version (semver comparison)
- If the new manifest has a higher `schema_version`, the runtime must support it
- Dependencies are re-resolved after update

## Validation

The `validate` command checks a skill package before installation:

```bash
skill-sdk validate /path/to/skill/
```

Checks performed:

| Check | What it validates |
|-------|-------------------|
| Manifest exists | `manifest.yaml` is present |
| Schema compliance | Manifest matches `manifest.schema.json` |
| Required fields | All required manifest fields are present and valid |
| File structure | Expected subdirectories exist (knowledge/, workflows/, etc.) |
| YAML syntax | All `.yaml`/`.yml` files are valid YAML |
| Dependency references | Each dependency ID resolves to a valid skill |
| Detection rules | Rules have valid `id`, `pattern`, `confidence` |
| Workflows | States have valid `initial`/`terminal` flags |
| Commands | Commands have valid `syntax` and `id` |
| Tests exist | If a `tests/` directory exists, at least one test file is present |
| Duplicate IDs | No duplicate IDs within any YAML array |

### Output

```bash
$ skill-sdk validate /tmp/my-skill/
Validating /tmp/my-skill/...
  ✓ manifest.yaml - Valid
  ✓ technology.yaml - Valid
  ✓ workflows.yaml - Valid
  ✓ detection_rules.yaml - Valid
  ✓ commands.yaml - Valid
  ✓ No duplicate IDs found
  ✓ All dependency references resolve
Result: VALID
```

## Packaging Commands

The Skill SDK exposes these packaging commands:

```bash
# Create a skill from template
skill-sdk create-skill <name> --template <template-type>

# Validate a skill
skill-sdk validate <path-to-skill>

# Package a skill into .wls
skill-sdk package <path-to-skill> -o <output.wls>

# Extract a .wls package
skill-sdk extract <package.wls> -o <output-directory>

# List available templates
skill-sdk list-templates

# List templates for a specific type
skill-sdk list-templates --type <type>
```

## Template Types

The SDK supports 8 template types for `create-skill`:

| Template | Description | Files Generated |
|----------|-------------|-----------------|
| `technology` | Full technology skill | manifest, technology, intents, workflows, detection, commands, best_practices, known_issues |
| `workflow` | Workflow-only skill | manifest, workflows, best_practices |
| `command` | Command reference skill | manifest, commands |
| `detection` | Detection rules only | manifest, detection_rules |
| `intent` | Intent definitions only | manifest, intents |
| `knowledge` | Knowledge base only | manifest, knowledge/ (with README) |
| `policy` | Policy/guidelines skill | manifest, best_practices, known_issues |
| `guidance` | AI guidance skill | manifest, guidance/rules.md, prompts/ |

## Custom Skills

Custom skills follow the same format as built-in skills:

```yaml
# Custom skill manifest
id: acme-customer-internal
name: ACME Internal Procedures
version: 1.0.0
author: ACME Engineering Team
vendor: acme-customer       # Customer-specific vendor
category: Engineering
dependencies:
  - linux-engineering
schema_version: 1.0
enabled: true
```

Partner skills use:

```yaml
vendor: partner-<name>       # e.g., "partner-datadog"
```

## Security

- `.wls` archives use gzip compression (no encryption)
- Manifest validation prevents malformed or malicious skills
- Skills cannot execute commands by design
- The `metadata.json` includes a SHA-256 checksum of the archive
- Package installation verifies the checksum matches

## Versioning

Skills use semantic versioning:

```
MAJOR.MINOR.PATCH
```

| Change | Version bump |
|--------|-------------|
| Breaking manifest schema change | `MAJOR` |
| New skill content (new workflow, command, etc.) | `MINOR` |
| Bug fix in existing content | `PATCH` |