# Enterprise Knowledge Platform

**Version:** 0.5.0-alpha  
**Phase:** 8  
**Status:** Active

## Overview

The Enterprise Knowledge Platform is the authoritative engineering knowledge layer used by every component of the Wiki Labs AI Copilot. It provides a production-grade foundation for knowledge retrieval, indexing, organization, and provenance — not reasoning.

Reasoning, recommendations, and advisory functions are handled by the Advisor Engine in a later phase. The Knowledge Platform's sole responsibility is to retrieve, index, and organize information with full provenance.

## Architecture

```
Observation Framework
  ↓
Engineering Intelligence
  ↓
Workspace Context
  ↓
Knowledge Platform
  ↓
Advisor Engine (future)
  ↓
Skill Runtime
  ↓
Optional MCP Tools
```

## Core Philosophy

Knowledge in this platform must be:

- **Modular** — packaged as independent Knowledge Packs
- **Versioned** — every pack and document carries version history
- **Traceable** — full provenance from retrieval to original source
- **Explainable** — every result includes confidence and metadata
- **Provider-independent** — supports multiple document sources
- **Workspace-aware** — per-workspace pack enablement
- **Engineering-aware** — retrieval ranking considers engineering context
- **Future graph-enabled** — metadata designed for knowledge graph expansion

Knowledge must never be hardcoded. Every piece of knowledge must retain its provenance.

## Feature Inventory

| # | Feature | Status |
|---|---------|--------|
| 1 | Knowledge Pack Framework | Implemented |
| 2 | Knowledge Provider Framework | Implemented |
| 3 | Document Ingestion Pipeline | Implemented |
| 4 | Document Processing | Implemented |
| 5 | Embedding Pipeline | Implemented |
| 6 | Vector Storage | Implemented |
| 7 | Metadata Store | Implemented |
| 8 | Context-Aware Retrieval | Implemented |
| 9 | Citation & Provenance Engine | Implemented |
| 10 | Knowledge SDK | Implemented |
| 11 | Knowledge Validation | Implemented |
| 12 | Workspace Knowledge Association | Implemented |
| 13 | Graph-Ready Metadata Architecture | Implemented |
| 14 | Knowledge Management UI | Implemented |
| 15 | Performance Architecture | Implemented |
| 16 | Comprehensive Testing | Implemented |

## What This Platform Does NOT Do

The Knowledge Platform explicitly does NOT implement:

- Advisor Engine
- Recommendation generation
- MCP execution
- Automation
- Technology-specific recommendations
- Autonomous reasoning

## Key Components

### Knowledge Packs

Independent, installable, versioned modules containing engineering knowledge:

```
openshift/
├── manifest.yaml
├── metadata.yaml
├── documents/
├── embeddings/
├── indexes/
├── relationships/
├── tests/
└── documentation/
```

### Knowledge Providers

Sources that supply documents to the platform:

- Filesystem
- PDF
- Markdown
- HTML
- DOCX
- TXT
- YAML
- JSON
- XML
- Git Repository

Future providers (stubbed):

- Confluence
- SharePoint
- Vendor Documentation Portal
- Wiki
- REST API
- Cloud Storage

### Document Pipeline

A chainable, extensible ingestion pipeline with 12 stages:

1. Document Discovery
2. Validation
3. Duplicate Detection
4. Incremental Updates
5. Parsing
6. Cleaning
7. Normalization
8. Language Detection
9. Chunk Generation
10. Metadata Extraction
11. Version Detection
12. Index Preparation

### Embedding Pipeline

- Embedding provider abstraction
- Configurable embedding models
- Incremental and batch embedding
- Embedding versioning
- Provider-independent generation

### Vector Storage

- SQLite VSS backend
- Namespace isolation (per pack, per workspace)
- Incremental indexing
- Background indexing
- Migration support

### Context-Aware Retrieval

Retrieval considers:

- Technology domain
- User intent
- Engineering workflow
- Workspace configuration
- Conversation context
- Current engineering state
- Knowledge pack priority
- Customer-specific knowledge
- Vendor documentation

### Citation & Provenance

Every result includes:

- Knowledge Pack
- Document
- Section
- Heading
- Version
- Author (when available)
- Publication date
- Index timestamp
- Confidence score
- Original source

### Knowledge SDK

Developer tools for:

- Creating knowledge packs: `create-knowledge-pack openshift`
- Validating pack integrity
- Packaging packs (.wkl format)
- Testing knowledge content
- Schema management

### Validation Framework

Validates:

- Manifest schema
- Metadata schema
- Document existence
- Embedding compatibility
- Schema version
- Duplicate identifiers
- Dependencies
- Broken references
- Version compatibility

### Graph-Ready Metadata

Metadata designed to support future knowledge graph relationships:

- Technology → Workflow
- Technology → Commands
- Technology → Documentation
- Technology → Vendor KB
- Workflow → SOP
- Workflow → Skill
- Command → Troubleshooting Guide
- Error → Vendor KB
- Error → Workflow
- Error → Best Practice

## Workspace Association

Each workspace enables/disables specific knowledge packs:

```
Workspace: ABC Bank
Enabled:
  ✓ OpenShift Documentation
  ✓ Linux Documentation
  ✓ Wiki Labs SOP
  ✓ Customer SOP
Disabled:
  ✗ VMware
  ✗ SQL Server
```

Only enabled packs participate in retrieval.

## Performance

- Lazy loading of knowledge packs
- Incremental indexing (only changed documents)
- Background indexing tasks
- LRU caching of retrieval results
- Operation cancellation support
- Configurable memory limits
- Progress reporting

## Testing

Comprehensive tests cover:

- Knowledge Providers
- Knowledge Packs
- Document parsing
- Metadata extraction
- Chunking
- Embedding pipeline
- Vector indexing
- Retrieval
- Citation generation
- Validation
- Workspace association
- SDK generation

## Dependencies

- anyhow
- async-trait
- chrono
- futures
- once_cell
- regex
- rusqlite
- serde
- serde_json
- serde_yaml
- thiserror
- tokio
- tracing
- uuid
- globset