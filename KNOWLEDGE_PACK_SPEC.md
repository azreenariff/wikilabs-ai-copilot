# Knowledge Pack Specification

**Version:** 0.5.0-alpha  
**Phase:** 8

## Overview

Knowledge Packs are independent, versioned, installable modules that contain engineering knowledge. They are the fundamental unit of knowledge in the Enterprise Knowledge Platform.

Each pack is a self-contained directory structure with a standardized schema, making them portable, versionable, and independently manageable.

## Directory Structure

```
<pack_name>/
├── manifest.yaml          # Required. Pack metadata and configuration
├── metadata.yaml          # Required. Detailed pack metadata
├── documents/             # Raw document files
│   ├── .keep
├── embeddings/            # Generated embeddings
│   ├── .keep
├── indexes/               # Search indexes
│   ├── .keep
├── relationships/         # Future graph relationships
│   ├── .keep
├── tests/                 # Validation tests
│   ├── .keep
└── documentation/         # Pack documentation
    ├── .keep
    └── README.md
```

## manifest.yaml Schema

Required metadata about the knowledge pack:

```yaml
# Schema version (enforces backward compatibility)
schema_version: "1.0.0"

# Pack name (unique identifier)
name: "openshift"

# Human-readable display name
display_name: "OpenShift Documentation"

# Pack version (semantic versioning)
version: "1.2.0"

# One-line description
description: "OpenShift container platform documentation, commands, and best practices"

# Vendor information
vendor:
  name: "Red Hat"
  url: "https://www.redhat.com"

# Vendor product
product: "OpenShift"

# Primary technology domain
technology:
  - "openshift"
  - "kubernetes"
  - "containers"

# Document categories
categories:
  - "documentation"
  - "reference"
  - "troubleshooting"

# Schema version for documents within this pack
document_schema: "1.0.0"

# Priority ranking (higher = ranked higher in retrieval)
priority: 100

# Supported platforms
platforms:
  - "linux"

# Minimum SDK version required
min_sdk_version: "0.5.0"

# Dependencies on other knowledge packs
dependencies:
  - name: "kubernetes"
    version: ">=1.0.0"
  - name: "linux"
    version: ">=1.0.0"

# Release history
changelog:
  - version: "1.2.0"
    date: "2025-01-15"
    changes:
      - "Added new troubleshooting guides"
      - "Updated command reference"
  - version: "1.1.0"
    date: "2024-11-20"
    changes:
      - "Added new OpenShift 4.14 documentation"
  - version: "1.0.0"
    date: "2024-09-01"
    changes:
      - "Initial release"

# Security classification
classification: "internal"

# License
license: "CC-BY-4.0"

# Maintainer contact
maintainer:
  name: "Wiki Labs Team"
  email: "team@wikilabs.io"
```

## metadata.yaml Schema

Detailed metadata about the pack's contents:

```yaml
# Last time the pack was fully indexed
last_indexed: "2025-01-15T10:30:00Z"

# Number of documents in the pack
document_count: 245

# Total size in bytes
size_bytes: 52428800

# Provider used to populate this pack
provider: "filesystem"

# Character encoding of all documents
encoding: "UTF-8"

# Document listing
documents:
  - id: "doc-001"
    title: "OpenShift Container Platform Installation Guide"
    source: "documents/openshift-install-guide.md"
    format: "markdown"
    word_count: 15000
    last_modified: "2025-01-10T08:00:00Z"
    embeddable: true
  - id: "doc-002"
    title: "OpenShift Networking Configuration"
    source: "documents/openshift-networking.md"
    format: "markdown"
    word_count: 8500
    last_modified: "2025-01-08T14:30:00Z"
    embeddable: true

# Embedding configuration
embedding:
  provider: "local"
  model: "all-MiniLM-L6-v2"
  dimension: 384
  batch_size: 100
  enabled: true
  last_embedded: "2025-01-15T10:30:00Z"

# Index configuration
index:
  vector_store: "sqlite-vss"
  namespace: "openshift"
  enabled: true
  last_indexed: "2025-01-15T10:30:00Z"

# Validation status
validation:
  status: "pass"
  last_validated: "2025-01-15T10:30:00Z"
  errors: []
  warnings: []

# Relationship status
relationships:
  total: 15
  last_updated: "2025-01-15T10:30:00Z"
  ready: true
```

## Pack Lifecycle

### 1. Creation

Packs are created using the Knowledge SDK:

```bash
create-knowledge-pack openshift
```

This generates the standard directory structure with manifest.yaml and metadata.yaml templates.

### 2. Population

Documents are added to the pack's `documents/` directory. This can be done via:

- Manual placement of documents
- Import from a Knowledge Provider
- Import of a previously exported pack (.wkl file)
- Automated scraping from documentation portals

### 3. Validation

The pack is validated against:

- Manifest schema
- Metadata schema
- Document existence
- Embedding compatibility
- Schema version compatibility
- Duplicate identifiers
- Dependencies
- Broken references
- Version compatibility

### 4. Indexing

Documents are processed through the ingestion pipeline and indexed in the vector store.

### 5. Installation

The pack is installed by placing it in the knowledge packs directory and registering it with the pack repository.

### 6. Enable/Disable

Packs can be enabled or disabled per workspace. Disabled packs do not participate in retrieval.

### 7. Update

Packs are updated by:

- Replacing documents in the documents/ directory
- Updating manifest.yaml metadata
- Running incremental indexing
- Updating metadata.yaml

### 8. Export

Packs are exported as `.wkl` (Wiki Labs Knowledge) archive files:

```bash
export-knowledge-pack openshift -o openshift.wkl
```

The archive contains:
- manifest.yaml
- metadata.yaml
- All documents
- Embeddings
- Indexes (optional)
- Relationships (if any)

### 9. Deletion

Packs are removed by:

1. Disabling the pack in all workspaces
2. Removing from the pack repository
3. Deleting the pack directory
4. Cleaning up associated data (embeddings, indexes)

## Pack Names

Pack names are:

- Unique identifiers within the platform
- Lowercase, hyphen-separated
- Alphanumeric plus hyphens only
- Minimum 3 characters
- Maximum 64 characters

Examples:

- `openshift`
- `linux`
- `windows`
- `vmware`
- `nagios-xi`
- `ansible`
- `grafana`
- `wiki-labs-sop`
- `customer-sop`
- `vendor-kb`

## Schema Versions

The schema version in manifest.yaml ensures backward compatibility:

- `1.0.0` — Initial schema
- Incremental updates add fields but never remove existing ones
- Older manifests are validated against their declared schema version
- The platform can read all schema versions it supports

## Pack Repository

The pack repository maintains:

- List of all installed packs
- Pack statuses (installed, enabled, disabled)
- Enabled/disabled state per workspace
- Version history
- Checksums for integrity verification

## File Formats

Supported document formats within packs:

- `.md` — Markdown
- `.pdf` — PDF
- `.html` — HTML
- `.htm` — HTML
- `.docx` — Microsoft Word
- `.txt` — Plain text
- `.yaml` — YAML
- `.yml` — YAML
- `.json` — JSON
- `.xml` — XML

## Dependencies

Packs can declare dependencies on other packs:

```yaml
dependencies:
  - name: "kubernetes"
    version: ">=1.0.0"
```

When a pack is installed:

1. Dependencies are resolved
2. Dependent packs must be installed first
3. Version constraints are validated
4. Circular dependencies are rejected

## Versioning

Packs follow semantic versioning:

- `MAJOR` — Breaking changes to manifest/metadata schema
- `MINOR` — New features, new documents
- `PATCH` — Bug fixes, minor updates

## Validation Rules

1. `name` must be unique across all packs
2. `version` must follow semantic versioning
3. `schema_version` must be supported
4. `vendor.name` is required
5. `technology` array must not be empty
6. `priority` must be between 0 and 1000
7. Dependencies must reference installed packs
8. All documents listed in metadata.yaml must exist
9. Document IDs must be unique within the pack
10. No broken internal references

## Error States

Packs can enter error states:

```rust
pub enum PackStatus {
    Installed,
    Enabled,
    Disabled,
    Error(String),        // Validation/indexing error
    Incomplete,           // Missing required files
    Outdated,             // Schema version mismatch
    Corrupted,            // Integrity check failed
}
```