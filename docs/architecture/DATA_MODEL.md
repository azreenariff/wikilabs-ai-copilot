# Data Model — Wiki Labs AI Copilot

## Database Schema

All data stored in a single SQLite database at `~/.local/share/wikilabs/wikilabs.db`.

### Tables

#### workspaces

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT (UUID) | Primary key |
| name | TEXT | Workspace name |
| customer_name | TEXT | Customer name |
| technology_stack | TEXT | JSON array of technology names |
| created_at | TEXT | ISO 8601 timestamp |
| updated_at | TEXT | ISO 8601 timestamp |

#### chat_messages

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT (UUID) | Primary key |
| workspace_id | TEXT (FK) | References workspaces.id |
| role | TEXT | "user", "assistant", "system" |
| content | TEXT | Message content |
| created_at | TEXT | ISO 8601 timestamp |

#### knowledge_documents

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT (UUID) | Primary key |
| title | TEXT | Document title |
| source | TEXT | File path or URL |
| workspace_id | TEXT (FK) | References workspaces.id |
| author | TEXT | Document author |
| created_at | TEXT | ISO 8601 timestamp |
| updated_at | TEXT | ISO 8601 timestamp |

#### knowledge_chunks (VSS indexed)

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT (UUID) | Primary key |
| document_id | TEXT (FK) | References knowledge_documents.id |
| content | TEXT | Chunk text content |
| embedding | VECTOR(384) | 384-dim embedding vector |
| vector_id | TEXT | VSS index identifier |

#### audit_log

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT (UUID) | Primary key |
| timestamp | TEXT | ISO 8601 timestamp |
| action | TEXT | Action description |
| actor | TEXT | User or system identifier |
| hash | TEXT | SHA-256 of previous entry (hash chain) |
| signature | TEXT | Ed25519 signature (optional) |

### Indexes

| Table | Index | Purpose |
|-------|-------|---------|
| chat_messages | workspace_id + created_at | Efficient per-workspace queries |
| knowledge_documents | workspace_id | Efficient per-workspace knowledge |
| audit_log | timestamp | Time-range queries |

### Relationships

```
workspaces (1) ──< (N) chat_messages
workspaces (1) ──< (N) knowledge_documents
knowledge_documents (1) ──< (N) knowledge_chunks
```