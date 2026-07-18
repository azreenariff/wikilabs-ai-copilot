-- Knowledge metadata schema for SQLite.
--
-- Stores structured metadata for knowledge documents, supporting
-- full-text search, tag filtering, and graph-ready relationships.

-- Knowledge metadata table — the core of the metadata store.
CREATE TABLE IF NOT EXISTS knowledge_metadata (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL,

    -- Document identity
    title TEXT NOT NULL DEFAULT '',
    knowledge_pack TEXT NOT NULL DEFAULT '',
    vendor TEXT NOT NULL DEFAULT '',
    product TEXT NOT NULL DEFAULT '',
    version TEXT NOT NULL DEFAULT '',
    technology TEXT NOT NULL DEFAULT '',

    -- Author and publication info
    author TEXT NOT NULL DEFAULT '',
    publication_date TEXT,
    last_indexed TEXT NOT NULL DEFAULT (datetime('now')),

    -- Classification
    security_classification TEXT NOT NULL DEFAULT 'public',
    customer_scope TEXT NOT NULL DEFAULT '',
    language TEXT NOT NULL DEFAULT 'en',

    -- Embedding info
    embedding_version TEXT NOT NULL DEFAULT '',

    -- Tags (comma-separated for fast filtering)
    tags TEXT NOT NULL DEFAULT '',

    -- Graph-ready fields (future knowledge graph)
    node_type TEXT NOT NULL DEFAULT 'documentation',
    relationship_type TEXT NOT NULL DEFAULT '',
    source_node_id TEXT DEFAULT '',
    target_node_id TEXT DEFAULT '',
    edge_properties TEXT DEFAULT 'null',

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Full-text search virtual table for metadata content.
CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_metadata_fts USING fts5(
    title,
    vendor,
    technology,
    author,
    tags,
    content=knowledge_metadata,
    content_rowid=rowid
);

-- Triggers to keep FTS index in sync on INSERT.
CREATE TRIGGER IF NOT EXISTS knowledge_metadata_ai AFTER INSERT ON knowledge_metadata BEGIN
    INSERT INTO knowledge_metadata_fts(rowid, title, vendor, technology, author, tags)
    VALUES (new.rowid, new.title, new.vendor, new.technology, new.author, new.tags);
END;

-- Triggers to keep FTS index in sync on UPDATE.
CREATE TRIGGER IF NOT EXISTS knowledge_metadata_au AFTER UPDATE ON knowledge_metadata BEGIN
    INSERT INTO knowledge_metadata_fts(knowledge_metadata_fts, rowid, title, vendor, technology, author, tags)
    VALUES ('delete', old.rowid, old.title, old.vendor, old.technology, old.author, old.tags);
    INSERT INTO knowledge_metadata_fts(rowid, title, vendor, technology, author, tags)
    VALUES (new.rowid, new.title, new.vendor, new.technology, new.author, new.tags);
END;

-- Triggers to keep FTS index in sync on DELETE.
CREATE TRIGGER IF NOT EXISTS knowledge_metadata_ad AFTER DELETE ON knowledge_metadata BEGIN
    INSERT INTO knowledge_metadata_fts(knowledge_metadata_fts, rowid, title, vendor, technology, author, tags)
    VALUES ('delete', old.rowid, old.title, old.vendor, old.technology, old.author, old.tags);
END;

-- Indexes for common query patterns.
CREATE INDEX IF NOT EXISTS idx_metadata_pack_name ON knowledge_metadata(knowledge_pack);
CREATE INDEX IF NOT EXISTS idx_metadata_technology ON knowledge_metadata(technology);
CREATE INDEX IF NOT EXISTS idx_metadata_vendor ON knowledge_metadata(vendor);
CREATE INDEX IF NOT EXISTS idx_metadata_node_type ON knowledge_metadata(node_type);
CREATE INDEX IF NOT EXISTS idx_metadata_security ON knowledge_metadata(security_classification);
CREATE INDEX IF NOT EXISTS idx_metadata_tags ON knowledge_metadata(tags);
CREATE INDEX IF NOT EXISTS idx_metadata_source_node ON knowledge_metadata(source_node_id);
CREATE INDEX IF NOT EXISTS idx_metadata_target_node ON knowledge_metadata(target_node_id);
CREATE INDEX IF NOT EXISTS idx_metadata_relationship ON knowledge_metadata(relationship_type);
CREATE INDEX IF NOT EXISTS idx_metadata_product ON knowledge_metadata(product);
CREATE INDEX IF NOT EXISTS idx_metadata_version ON knowledge_metadata(version);