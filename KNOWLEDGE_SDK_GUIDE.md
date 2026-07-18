# Knowledge SDK Guide

**Version:** 0.5.0-alpha  
**Phase:** 8

## Overview

The Knowledge SDK provides developer tools for creating, validating, packaging, and testing knowledge packs in the Wiki Labs AI Copilot.

## Quick Start

```bash
# Create a new knowledge pack
create-knowledge-pack openshift

# Validate a pack
validate-knowledge-pack ./packages/openshift

# Package a pack
package-knowledge-pack -i ./packages/openshift -o openshift.wkl

# Test a pack
test-knowledge-pack ./packages/openshift
```

## Installation

The SDK is included in the Wiki Labs CLI tool:

```bash
# From source
cargo install --path src/cli

# Or use the bundled CLI from the desktop app
```

## Commands

### create-knowledge-pack

Creates a new knowledge pack from a template.

```bash
create-knowledge-pack <name> [options]

Options:
  -d, --display-name <name>  Human-readable display name
  -v, --version <version>    Initial version (default: 1.0.0)
  -n, --vendor <name>        Vendor name
  -p, --product <product>    Product name
  -t, --technology <tech>    Technology tag (can be repeated)
  -c, --category <cat>       Category (can be repeated)
  --schema-version <ver>     Schema version (default: 1.0.0)
  --priority <n>             Priority 0-1000 (default: 100)
  -o, --output <dir>         Output directory (default: ./packages)
```

Example:
```bash
create-knowledge-pack openshift \
  --display-name "OpenShift Documentation" \
  --vendor "Red Hat" \
  --product "OpenShift" \
  --technology openshift \
  --technology kubernetes \
  --technology containers
```

Generates:
```
packages/openshift/
├── manifest.yaml
├── metadata.yaml
├── documents/
│   └── .keep
├── embeddings/
│   └── .keep
├── indexes/
│   └── .keep
├── relationships/
│   └── .keep
├── tests/
│   ├── test_manifest.yaml
│   └── test_metadata.yaml
└── documentation/
    └── README.md
```

### validate-knowledge-pack

Validates a knowledge pack against all rules.

```bash
validate-knowledge-pack <path> [options]

Options:
  -f, --format <format>  Output format: text, json (default: text)
  -v, --verbose          Show detailed validation output
  --strict               Treat warnings as errors
```

Validation checks:

1. **Manifest schema** — Validates manifest.yaml against schema
2. **Metadata schema** — Validates metadata.yaml against schema
3. **Document existence** — All documents listed in metadata.yaml exist
4. **Embedding compatibility** — Documents are embeddable
5. **Schema version** — Manifest version is supported
6. **Duplicate identifiers** — No duplicate document IDs
7. **Dependencies** — All dependencies are installed
8. **Broken references** — All internal references resolve
9. **Version compatibility** — Versions are semantically valid

Output:
```
✓ manifest.yaml — Valid
✓ metadata.yaml — Valid
✓ 245 documents — All exist and readable
✓ embedding compatibility — All compatible
✓ schema version — 1.0.0 (supported)
✓ identifiers — No duplicates
✓ dependencies — All resolved
✓ references — No broken references
✓ versions — All valid semver

Result: 12 checks passed, 0 failed
```

JSON output:
```json
{
  "valid": true,
  "checks": {
    "manifest": "pass",
    "metadata": "pass",
    "documents": "pass",
    "embedding": "pass",
    "schema_version": "pass",
    "duplicate_id": "pass",
    "dependencies": "pass",
    "broken_refs": "pass",
    "version_compat": "pass"
  },
  "errors": [],
  "warnings": [],
  "validated_at": "2025-01-15T10:30:00Z"
}
```

### package-knowledge-pack

Packages a knowledge pack into a `.wkl` (Wiki Labs Knowledge) archive.

```bash
package-knowledge-pack -i <input-dir> -o <output.wkl> [options]

Options:
  -i, --input <dir>        Input pack directory
  -o, --output <file>      Output file path
  --include-embeddings     Include embeddings in archive
  --include-indexes        Include indexes in archive
  --compress               Compress the archive (default: true)
  --checksum               Generate checksum
```

Archive contents:
```
openshift.wkl (zip archive)
├── manifest.yaml
├── metadata.yaml
├── documents/
│   └── ...
├── embeddings/
│   └── ...
├── indexes/
│   └── ...
├── relationships/
│   └── ...
├── tests/
│   └── ...
├── documentation/
│   └── ...
└── .wkl-manifest
```

### test-knowledge-pack

Runs validation tests on a knowledge pack.

```bash
test-knowledge-pack <path> [options]

Options:
  -v, --verbose            Verbose output
  --list                   List available tests
  --filter <pattern>       Only run tests matching pattern
```

Tests include:

- Manifest schema validation
- Metadata schema validation
- Document format validation
- Cross-reference validation
- Embedding compatibility
- Version consistency
- Dependency resolution

### list-knowledge-packs

Lists all installed knowledge packs.

```bash
list-knowledge-packs [options]

Options:
  --enabled                Show only enabled packs
  --disabled               Show only disabled packs
  --all                    Show all packs
  --json                   JSON output
```

Output:
```
Package                  Version   Status    Docs    Last Indexed
────────────────────────────────────────────────────────────
openshift                1.2.0     Enabled   245     2025-01-15 10:30
linux                    2.1.0     Enabled   189     2025-01-14 08:15
windows                  1.0.0     Disabled  95      2025-01-10 14:00
vmware                   3.0.1     Enabled   312     2025-01-15 09:00
```

### import-knowledge-pack

Imports a knowledge pack from a `.wkl` archive.

```bash
import-knowledge-pack <file.wkl> [options]

Options:
  -d, --destination <dir>  Destination directory
  --force                  Overwrite existing pack
```

### export-knowledge-pack

Exports a knowledge pack to a `.wkl` archive.

```bash
export-knowledge-pack <pack-name> [options]

Options:
  -o, --output <file>      Output file path
  --include-embeddings     Include embeddings
  --include-indexes        Include indexes
```

### reindex-knowledge-pack

Re-indexes a knowledge pack.

```bash
reindex-knowledge-pack <pack-name> [options]

Options:
  --embeddings             Regenerate embeddings
  --incremental            Only re-index changed documents
  --verbose                Verbose output
```

## API Reference

### Creating a Pack Programmatically

```rust
use wikilabs_knowledge::sdk::{
    KnowledgePackCreator,
    KnowledgePackManifest,
    KnowledgePackMetadata,
    KnowledgePackConfig,
};

// Create a new pack
let creator = KnowledgePackCreator::new("./packages");

let manifest = KnowledgePackManifest {
    name: "openshift".to_string(),
    display_name: "OpenShift Documentation".to_string(),
    version: "1.0.0".parse().unwrap(),
    description: "OpenShift container platform docs".to_string(),
    vendor: VendorInfo {
        name: "Red Hat".to_string(),
        url: Some("https://www.redhat.com".to_string()),
    },
    product: "OpenShift".to_string(),
    technologies: vec!["openshift".into(), "kubernetes".into()],
    categories: vec!["documentation".into(), "reference".into()],
    priority: 100,
    schema_version: "1.0.0".to_string(),
    dependencies: vec![],
};

let metadata = KnowledgePackMetadata {
    last_indexed: None,
    document_count: 0,
    size_bytes: 0,
    provider: "filesystem".to_string(),
    encoding: "UTF-8".to_string(),
    documents: vec![],
    embedding: EmbeddingConfig {
        provider: "local".to_string(),
        model: "all-MiniLM-L6-v2".to_string(),
        dimension: 384,
        enabled: false,
    },
    index: IndexConfig {
        enabled: false,
        namespace: "openshift".to_string(),
    },
    validation: ValidationStatus {
        status: "not_validated".to_string(),
        errors: vec![],
        warnings: vec![],
    },
};

let pack = creator.create(&manifest, &metadata).await?;
```

### Validating a Pack Programmatically

```rust
use wikilabs_knowledge::validate::PackValidator;

let validator = PackValidator::new();
let result = validator.validate(&pack_path).await?;

if result.is_valid() {
    println!("Pack is valid!");
} else {
    for error in &result.errors {
        eprintln!("Validation error: {}", error);
    }
    for warning in &result.warnings {
        eprintln!("Warning: {}", warning);
    }
}
```

### Packaging a Pack Programmatically

```rust
use wikilabs_knowledge::sdk::Packager;

let packager = Packager::new();
let result = packager.package(
    &pack_path,
    "openshift.wkl",
    PackageOptions {
        include_embeddings: true,
        include_indexes: false,
        compress: true,
    }
).await?;

println!("Packaged {} documents into {}", result.document_count, result.output_path);
```

## Templates

### Default Template

When creating a pack without a template, the default template is used:

```yaml
# manifest.yaml
schema_version: "1.0.0"
name: "{{pack_name}}"
display_name: "{{display_name}}"
version: "{{version}}"
description: ""
vendor:
  name: "{{vendor}}"
  url: ""
product: "{{product}}"
technology: []
categories: []
document_schema: "1.0.0"
priority: 100
platforms: []
min_sdk_version: "0.5.0"
dependencies: []
changelog: []
classification: "internal"
license: "CC-BY-4.0"
maintainer:
  name: "Wiki Labs Team"
  email: "team@wikilabs.io"
```

### Custom Templates

Create custom templates in `~/.config/wikilabs/templates/`:

```
templates/
├── default/
│   ├── manifest.yaml
│   ├── metadata.yaml
│   └── README.md
├── vendor-docs/
│   ├── manifest.yaml
│   ├── metadata.yaml
│   └── README.md
└── internal-sop/
    ├── manifest.yaml
    ├── metadata.yaml
    └── README.md
```

Use with:
```bash
create-knowledge-pack mypack --template vendor-docs
```

## Testing Utilities

### Test Fixtures

```rust
use wikilabs_knowledge::sdk::testing::{
    TestDocument,
    TestPack,
    TestContext,
};

// Create a test pack with sample documents
let test_pack = TestPack::builder("test-openstack")
    .with_manifest(|m| {
        m.name = "test-openstack".into();
        m.version = "1.0.0".parse().unwrap();
    })
    .with_documents(vec![
        TestDocument::new("test1.md", "# OpenStack Test\nSome content."),
        TestDocument::new("test2.md", "## Network Config\nMore content."),
    ])
    .build();

// Run validation
let result = test_pack.validate().await?;
assert!(result.is_valid());

// Run indexing test
let indexed = test_pack.index(&test_config).await?;
assert_eq!(indexed.chunks_generated, 4);
```

### Test Matchers

```rust
use wikilabs_knowledge::sdk::testing::matchers::*;

assert_that(&result.pack_name).is_equal_to("openshift");
assert_that(&result.document_count).is_greater_than(0);
assert_that(&result.embedding_status).is_equal_to(EmbeddingStatus::Ready);
assert_that(&result.validation_status).is_equal_to(ValidationStatus::Valid);
```

## Schema Reference

### manifest.yaml Schema

See KNOWLEDGE_PACK_SPEC.md for full schema.

### metadata.yaml Schema

See KNOWLEDGE_PACK_SPEC.md for full schema.

## Best Practices

1. **Always validate** before deploying a pack
2. **Version consistently** — follow semver
3. **Include changelog** — document every change
4. **Keep packs small** — split large topics
5. **Use priorities wisely** — higher priority = ranked higher
6. **Document everything** — include documentation/README.md
7. **Test thoroughly** — use the testing utilities
8. **Check dependencies** — ensure all dependencies are met
9. **Update metadata** — after every indexing run
10. **Backup before reindex** — before major reindexing

## Error Handling

Common SDK errors:

```rust
pub enum SDKError {
    /// Pack directory not found
    PackNotFound { path: String },

    /// Manifest invalid or missing
    InvalidManifest { reason: String },

    /// Metadata invalid or missing
    InvalidMetadata { reason: String },

    /// Document parse failed
    DocumentParseFailed { file: String, reason: String },

    /// Embedding failed
    EmbeddingFailed { reason: String },

    /// Indexing failed
    IndexingFailed { reason: String },

    /// Packaging failed
    PackagingFailed { reason: String },

    /// Invalid version
    InvalidVersion { version: String },

    /// Circular dependency
    CircularDependency { pack1: String, pack2: String },

    /// Schema version unsupported
    UnsupportedSchema { version: String },
}
```

## Contributing

To add a new SDK command:

1. Add the command to the CLI entry point
2. Implement the command in the SDK module
3. Add tests for the command
4. Update this guide
5. Update the CHANGELOG

## See Also

- KNOWLEDGE_PLATFORM.md — Platform overview
- KNOWLEDGE_PACK_SPEC.md — Pack specification
- KNOWLEDGE_VALIDATION.md — Validation specification
- DOCUMENT_PIPELINE.md — Document pipeline
- KNOWLEDGE_PROVIDER_SPEC.md — Provider specification