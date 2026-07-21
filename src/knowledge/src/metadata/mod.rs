//! Metadata Store — SQLite-backed storage for knowledge document metadata.
//!
//! Provides CRUD operations, full-text search, and tag-based filtering.
//! The schema is designed to support future knowledge graph relationships.

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

pub mod models;
use models::MetadataEntry;

/// The SQL schema for the metadata store, embedded at compile time.
pub const SCHEMA_SQL: &str = include_str!("schema.sql");

/// Metadata store using SQLite for structured document metadata.
///
/// Wraps a `Connection` in `Arc<Mutex<>>` for interior mutability
/// and thread safety.
#[derive(Clone)]
pub struct MetadataStore {
    connection: Arc<Mutex<Connection>>,
}

impl MetadataStore {
    /// Create a new metadata store, initializing the schema if needed.
    pub fn new(path: &str) -> Result<Self> {
        debug!(path, "Opening metadata store");
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open metadata store at {}", path))?;

        // Enable WAL mode and foreign keys
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "on")?;

        // Initialize schema
        conn.execute_batch(SCHEMA_SQL)
            .with_context(|| "Failed to initialize metadata schema")?;

        info!(path, "Metadata store opened and initialized");
        Ok(Self {
            connection: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open or create a metadata store from a connection (useful for in-memory).
    pub fn from_conn(conn: Connection) -> Result<Self> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "on")?;
        conn.execute_batch(SCHEMA_SQL)
            .context("Failed to initialize metadata schema")?;
        Ok(Self {
            connection: Arc::new(Mutex::new(conn)),
        })
    }

    /// Insert a metadata entry.
    pub fn insert(&self, entry: &MetadataEntry) -> Result<()> {
        let conn = self
            .connection
            .lock()
            .expect("Metadata store lock poisoned");

        conn.execute(
            r#"INSERT INTO knowledge_metadata (
                id, document_id, title, knowledge_pack, vendor, product,
                version, technology, author, publication_date, last_indexed,
                security_classification, customer_scope, language,
                embedding_version, tags, node_type, relationship_type,
                source_node_id, target_node_id, edge_properties,
                created_at, updated_at
            ) VALUES (
                :id, :document_id, :title, :knowledge_pack, :vendor, :product,
                :version, :technology, :author, :publication_date, :last_indexed,
                :security_classification, :customer_scope, :language,
                :embedding_version, :tags, :node_type, :relationship_type,
                :source_node_id, :target_node_id, :edge_properties,
                :created_at, :updated_at
            )"#,
            params! {
                &entry.id,
                &entry.document_id,
                &entry.title,
                &entry.knowledge_pack,
                &entry.vendor,
                &entry.product,
                &entry.version,
                &entry.technology,
                &entry.author,
                entry.publication_date.as_deref(),
                &entry.last_indexed,
                &entry.security_classification,
                &entry.customer_scope,
                &entry.language,
                &entry.embedding_version,
                &entry.tags,
                &entry.node_type,
                &entry.relationship_type,
                &entry.source_node_id,
                &entry.target_node_id,
                &entry.edge_properties,
                &entry.created_at,
                &entry.updated_at,
            },
        )
        .context("Failed to insert metadata entry")?;

        debug!(id = %entry.id, "Metadata entry inserted");
        Ok(())
    }

    /// Update an existing metadata entry.
    pub fn update(&self, entry: &MetadataEntry) -> Result<usize> {
        let conn = self
            .connection
            .lock()
            .expect("Metadata store lock poisoned");

        let _updated_at = Utc::now().to_rfc3339();
        let updated = entry.updated_at.clone();

        let rows = conn
            .execute(
                r#"UPDATE knowledge_metadata SET
                title = :title,
                knowledge_pack = :knowledge_pack,
                vendor = :vendor,
                product = :product,
                version = :version,
                technology = :technology,
                author = :author,
                publication_date = :publication_date,
                last_indexed = :last_indexed,
                security_classification = :security_classification,
                customer_scope = :customer_scope,
                language = :language,
                embedding_version = :embedding_version,
                tags = :tags,
                node_type = :node_type,
                relationship_type = :relationship_type,
                source_node_id = :source_node_id,
                target_node_id = :target_node_id,
                edge_properties = :edge_properties,
                updated_at = :updated_at
            WHERE id = :id"#,
                params! {
                    &entry.title,
                    &entry.knowledge_pack,
                    &entry.vendor,
                    &entry.product,
                    &entry.version,
                    &entry.technology,
                    &entry.author,
                    entry.publication_date.as_deref(),
                    &entry.last_indexed,
                    &entry.security_classification,
                    &entry.customer_scope,
                    &entry.language,
                    &entry.embedding_version,
                    &entry.tags,
                    &entry.node_type,
                    &entry.relationship_type,
                    &entry.source_node_id,
                    &entry.target_node_id,
                    &entry.edge_properties,
                    &updated,
                    &entry.id,
                },
            )
            .context("Failed to update metadata entry")?;

        debug!(id = %entry.id, rows, "Metadata entry updated");
        Ok(rows)
    }

    /// Delete a metadata entry by ID.
    pub fn delete(&self, id: &str) -> Result<usize> {
        let conn = self
            .connection
            .lock()
            .expect("Metadata store lock poisoned");

        let rows = conn
            .execute("DELETE FROM knowledge_metadata WHERE id = ?1", [&id])
            .context("Failed to delete metadata entry")?;

        debug!(id, rows, "Metadata entry deleted");
        Ok(rows)
    }

    /// Get a metadata entry by ID.
    pub fn get_by_id(&self, id: &str) -> Result<Option<MetadataEntry>> {
        let conn = self
            .connection
            .lock()
            .expect("Metadata store lock poisoned");

        conn.query_row(
            "SELECT id, document_id, title, knowledge_pack, vendor, product,
                    version, technology, author, publication_date, last_indexed,
                    security_classification, customer_scope, language,
                    embedding_version, tags, node_type, relationship_type,
                    source_node_id, target_node_id, edge_properties,
                    created_at, updated_at
             FROM knowledge_metadata WHERE id = ?1",
            [id],
            |row| self.row_to_entry(row),
        )
        .optional()
        .context("Failed to query metadata entry by ID")
    }

    /// List all metadata entries, optionally filtering by knowledge pack.
    pub fn list_all(&self) -> Result<Vec<MetadataEntry>> {
        self.query_all(
            "SELECT * FROM knowledge_metadata ORDER BY updated_at DESC",
            &[],
        )
    }

    /// List entries by knowledge pack name.
    pub fn list_by_pack(&self, pack_name: &str) -> Result<Vec<MetadataEntry>> {
        self.query_all(
            "SELECT * FROM knowledge_metadata WHERE knowledge_pack = ?1 ORDER BY updated_at DESC",
            &[&pack_name],
        )
    }

    /// List entries by technology.
    pub fn list_by_technology(&self, technology: &str) -> Result<Vec<MetadataEntry>> {
        self.query_all(
            "SELECT * FROM knowledge_metadata WHERE technology = ?1 ORDER BY updated_at DESC",
            &[&technology],
        )
    }

    /// List entries by vendor.
    pub fn list_by_vendor(&self, vendor: &str) -> Result<Vec<MetadataEntry>> {
        self.query_all(
            "SELECT * FROM knowledge_metadata WHERE vendor = ?1 ORDER BY updated_at DESC",
            &[&vendor],
        )
    }

    /// List entries by node type (graph-ready query).
    pub fn list_by_node_type(&self, node_type: &str) -> Result<Vec<MetadataEntry>> {
        self.query_all(
            "SELECT * FROM knowledge_metadata WHERE node_type = ?1 ORDER BY updated_at DESC",
            &[&node_type],
        )
    }

    /// List entries by relationship type (graph-ready query).
    pub fn list_by_relationship(&self, relationship: &str) -> Result<Vec<MetadataEntry>> {
        self.query_all(
            "SELECT * FROM knowledge_metadata WHERE relationship_type = ?1 ORDER BY updated_at DESC",
            &[&relationship],
        )
    }

    /// Filter entries by tag (tag-based filtering).
    pub fn list_by_tag(&self, tag: &str) -> Result<Vec<MetadataEntry>> {
        self.query_all(
            "SELECT * FROM knowledge_metadata WHERE tags LIKE ?1 ORDER BY updated_at DESC",
            &[&format!("%{}%", tag)],
        )
    }

    /// Filter entries by security classification.
    pub fn list_by_security(&self, classification: &str) -> Result<Vec<MetadataEntry>> {
        self.query_all(
            "SELECT * FROM knowledge_metadata WHERE security_classification = ?1 ORDER BY updated_at DESC",
            &[&classification],
        )
    }

    /// Full-text search on metadata content.
    pub fn fts_search(&self, query: &str) -> Result<Vec<MetadataEntry>> {
        self.query_all(
            r#"SELECT knowledge_metadata.* FROM knowledge_metadata
               INNER JOIN knowledge_metadata_fts
               ON knowledge_metadata.rowid = knowledge_metadata_fts.rowid
               WHERE knowledge_metadata_fts MATCH ?1
               ORDER BY knowledge_metadata_fts.rank"#,
            &[&query],
        )
    }

    /// Full-text search on metadata content with a specific pack filter.
    pub fn fts_search_by_pack(&self, pack_name: &str, query: &str) -> Result<Vec<MetadataEntry>> {
        self.query_all(
            r#"SELECT knowledge_metadata.* FROM knowledge_metadata
               INNER JOIN knowledge_metadata_fts
               ON knowledge_metadata.rowid = knowledge_metadata_fts.rowid
               WHERE knowledge_metadata_fts MATCH ?1
               AND knowledge_metadata.knowledge_pack = ?2
               ORDER BY knowledge_metadata_fts.rank"#,
            &[&query, &pack_name],
        )
    }

    /// Get all distinct technology tags.
    pub fn distinct_technologies(&self) -> Result<Vec<String>> {
        let conn = self
            .connection
            .lock()
            .expect("Metadata store lock poisoned");
        let mut stmt = conn
            .prepare("SELECT DISTINCT technology FROM knowledge_metadata WHERE technology != '' ORDER BY technology")
            .context("Failed to query distinct technologies")?;

        let mut techs = Vec::new();
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        for tech in rows {
            techs.push(tech?);
        }

        Ok(techs)
    }

    /// Get all distinct vendors.
    pub fn distinct_vendors(&self) -> Result<Vec<String>> {
        let conn = self
            .connection
            .lock()
            .expect("Metadata store lock poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT vendor FROM knowledge_metadata WHERE vendor != '' ORDER BY vendor",
            )
            .context("Failed to query distinct vendors")?;

        let mut vendors = Vec::new();
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        for vendor in rows {
            vendors.push(vendor?);
        }

        Ok(vendors)
    }

    /// Get all distinct knowledge packs.
    pub fn distinct_packs(&self) -> Result<Vec<String>> {
        let conn = self
            .connection
            .lock()
            .expect("Metadata store lock poisoned");
        let mut stmt = conn
            .prepare("SELECT DISTINCT knowledge_pack FROM knowledge_metadata WHERE knowledge_pack != '' ORDER BY knowledge_pack")
            .context("Failed to query distinct packs")?;

        let mut packs = Vec::new();
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        for pack in rows {
            packs.push(pack?);
        }

        Ok(packs)
    }

    /// Get metadata entry count.
    pub fn count(&self) -> Result<usize> {
        let conn = self
            .connection
            .lock()
            .expect("Metadata store lock poisoned");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM knowledge_metadata", [], |row| {
                row.get(0)
            })
            .context("Failed to count metadata entries")?;
        Ok(count as usize)
    }

    /// Get document by ID.
    pub fn get_by_document(&self, document_id: &str) -> Result<Vec<MetadataEntry>> {
        self.query_all(
            "SELECT * FROM knowledge_metadata WHERE document_id = ?1 ORDER BY updated_at DESC",
            &[&document_id],
        )
    }

    /// Delete all entries for a given document ID.
    pub fn delete_by_document(&self, document_id: &str) -> Result<usize> {
        let conn = self
            .connection
            .lock()
            .expect("Metadata store lock poisoned");
        let rows = conn
            .execute(
                "DELETE FROM knowledge_metadata WHERE document_id = ?1",
                [document_id],
            )
            .context("Failed to delete metadata by document ID")?;
        debug!(document_id, rows, "Metadata entries deleted for document");
        Ok(rows)
    }

    /// Helper: map a database row to a MetadataEntry.
    fn row_to_entry<'a>(&self, row: &rusqlite::Row<'a>) -> rusqlite::Result<MetadataEntry> {
        Ok(MetadataEntry::from_row(
            row.get(0)?,                                          // id
            row.get(1)?,                                          // document_id
            row.get(2)?,                                          // title
            row.get(3)?,                                          // knowledge_pack
            row.get(4)?,                                          // vendor
            row.get(5)?,                                          // product
            row.get(6)?,                                          // version
            row.get(7)?,                                          // technology
            row.get(8)?,                                          // author
            row.get::<_, Option<String>>(9)?.unwrap_or_default(), // publication_date
            row.get(10)?,                                         // last_indexed
            row.get(11)?,                                         // security_classification
            row.get(12)?,                                         // customer_scope
            row.get(13)?,                                         // language
            row.get(14)?,                                         // embedding_version
            row.get(15)?,                                         // tags
            row.get(16)?,                                         // node_type
            row.get(17)?,                                         // relationship_type
            row.get(18)?,                                         // source_node_id
            row.get(19)?,                                         // target_node_id
            row.get(20)?,                                         // edge_properties
            row.get(21)?,                                         // created_at
            row.get(22)?,                                         // updated_at
        ))
    }

    /// Helper: execute a SELECT query and map rows to MetadataEntry.
    fn query_all(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<Vec<MetadataEntry>> {
        let conn = self
            .connection
            .lock()
            .expect("Metadata store lock poisoned");
        let mut stmt = conn
            .prepare(sql)
            .with_context(|| format!("Failed to prepare query: {}", sql))?;

        let mut entries = Vec::new();
        let rows = stmt.query_map(params, |row| self.row_to_entry(row))?;
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> MetadataStore {
        let conn = Connection::open_in_memory().unwrap();
        MetadataStore::from_conn(conn).unwrap()
    }

    #[test]
    fn test_insert_and_get() {
        let store = test_store();
        let entry = MetadataEntry::new("doc-1", "Test Document", "test-pack");
        store.insert(&entry).unwrap();

        let retrieved = store.get_by_id(&entry.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Test Document");
        assert_eq!(retrieved.knowledge_pack, "test-pack");
        assert_eq!(retrieved.document_id, "doc-1");
    }

    #[test]
    fn test_update() {
        let store = test_store();
        let mut entry = MetadataEntry::new("doc-2", "Original Title", "test-pack");
        store.insert(&entry).unwrap();

        entry.title = "Updated Title".to_string();
        let rows = store.update(&entry).unwrap();
        assert_eq!(rows, 1);

        let retrieved = store.get_by_id(&entry.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Title");
    }

    #[test]
    fn test_delete() {
        let store = test_store();
        let entry = MetadataEntry::new("doc-3", "To Delete", "test-pack");
        store.insert(&entry).unwrap();

        assert!(store.get_by_id(&entry.id).unwrap().is_some());
        store.delete(&entry.id).unwrap();
        assert!(store.get_by_id(&entry.id).unwrap().is_none());
    }

    #[test]
    fn test_list_by_pack() {
        let store = test_store();
        let entry1 = MetadataEntry::new("doc-4a", "Doc A", "pack-a");
        let entry2 = MetadataEntry::new("doc-4b", "Doc B", "pack-b");

        store.insert(&entry1).unwrap();
        store.insert(&entry2).unwrap();

        let from_pack_a = store.list_by_pack("pack-a").unwrap();
        assert_eq!(from_pack_a.len(), 1);
        assert_eq!(from_pack_a[0].title, "Doc A");
    }

    #[test]
    fn test_list_by_tag() {
        let store = test_store();
        let mut entry = MetadataEntry::new("doc-5", "Tagged Doc", "test-pack");
        entry.tags = "openshift,kubernetes,security".to_string();
        store.insert(&entry).unwrap();

        let results = store.list_by_tag("openshift").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tags, "openshift,kubernetes,security");
    }

    #[test]
    fn test_count() {
        let store = test_store();
        assert_eq!(store.count().unwrap(), 0);

        store
            .insert(&MetadataEntry::new("doc-6", "Doc 6", "pack"))
            .unwrap();
        store
            .insert(&MetadataEntry::new("doc-7", "Doc 7", "pack"))
            .unwrap();

        assert_eq!(store.count().unwrap(), 2);
    }

    #[test]
    fn test_graph_node_type() {
        let store = test_store();
        let mut entry = MetadataEntry::new("doc-8", "Tech Entry", "test-pack");
        entry.node_type = "technology".to_string();
        entry.vendor = "Red Hat".to_string();
        entry.technology = "openshift".to_string();
        store.insert(&entry).unwrap();

        let nodes = store.list_by_node_type("technology").unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].vendor, "Red Hat");
    }

    #[test]
    fn test_graph_relationship() {
        let store = test_store();
        let mut entry = MetadataEntry::new("doc-9", "Rel Entry", "test-pack");
        entry.node_type = "technology".to_string();
        entry.relationship_type = "Technology->Workflow".to_string();
        entry.source_node_id = "node-1".to_string();
        entry.target_node_id = "node-2".to_string();
        store.insert(&entry).unwrap();

        let results = store.list_by_relationship("Technology->Workflow").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source_node_id, "node-1");
        assert_eq!(results[0].target_node_id, "node-2");
    }

    #[test]
    fn test_to_node_conversion() {
        let mut entry = MetadataEntry::new("doc-10", "Node Doc", "test-pack");
        entry.id = "550e8400-e29b-41d4-a716-446655440000".to_string();
        entry.node_type = "technology".to_string();
        entry.vendor = "IBM".to_string();
        entry.technology = "ibm-cloud,kubernetes".to_string();

        let node = entry.to_node();
        assert_eq!(node.title, "Node Doc");
        assert_eq!(node.vendor, "IBM");
        assert_eq!(node.technologies.len(), 2);
        assert_eq!(node.technologies[0], "ibm-cloud");
    }
}
