# Knowledge Architecture — Wiki Labs AI Copilot

## Overview

The knowledge system provides semantic and keyword search over documentation stored locally on the engineer's laptop. It uses SQLite VSS (vector search) and FTS5 (full-text search) for hybrid search.

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                  KNOWLEDGE SYSTEM                             │
│                                                               │
│  Import Pipeline: File/URL → Parser → Chunker → Embedder    │
│                         │              │                      │
│                    ┌─────┘              └─────┐               │
│                    ▼                      ▼                │
│              ┌─────────────┐     ┌─────────────────┐       │
│              │ Knowledge   │     │ SQLite VSS       │       │
│              │ Chunks      │     │ Vector Index      │       │
│              │ (SQLite)    │     │ (dim=384)         │       │
│              └─────────────┘     └─────────────────┘       │
│                    │                                         │
│                    ▼                                         │
│              ┌─────────────────┐                            │
│              │ SQLite FTS5     │                            │
│              │ Keyword Index   │                            │
│              └─────────────────┘                            │
│                                                               │
│  Embedding Model: all-MiniLM-L6-v2 (ONNX Runtime)            │
│  - 384-dimensional embeddings                                │
│  - CPU inference, < 100ms per query                          │
│  - ~50 MB model file, loaded on demand                       │
│  - Decoupled from AI provider (offline-capable)              │
└──────────────────────────────────────────────────────────────┘
```

## Hybrid Search

Search results are weighted: **70% vector similarity + 30% FTS5 relevance**.

```
HybridScore = 0.7 * VectorScore + 0.3 * FTS5Score
```

VectorScore: Cosine similarity between query embedding and document chunk embedding.
FTS5Score: BM25 relevance score from SQLite FTS5 full-text search.

## Embedding Model

| Property | Value |
|----------|-------|
| Model | all-MiniLM-L6-v2 |
| Dimensions | 384 |
| Inference | CPU (ONNX Runtime) |
| Latency | < 100ms per query |
| Model Size | ~50 MB |
| Offline | Yes |

## Storage

All data stored in a single SQLite database at `~/.local/share/wikilabs/wikilabs.db`.

| Table | Purpose |
|-------|---------|
| `knowledge_documents` | Document metadata (title, source, workspace_id) |
| `knowledge_chunks` | Chunk content + vector embedding (VSS indexed) |
| `knowledge_chunks_fts` | Full-text index (FTS5) |