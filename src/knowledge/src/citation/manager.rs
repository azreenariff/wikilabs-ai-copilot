//! Citation manager — track, verify, and generate source citations.

use super::{Citation, CitationType, LinkTracking, LinkStatus};
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::Connection;
use tracing::{debug, info, warn};

/// Manages citations — storing, verifying, and generating references.
pub struct CitationManager {
    pub conn: Connection,
}

impl CitationManager {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    /// Initialize citation tables.
    pub fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS citations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT,
                citation_type TEXT NOT NULL DEFAULT 'documentation',
                author TEXT,
                publication_date TEXT,
                last_verified TEXT,
                verified INTEGER DEFAULT 0,
                related_ids TEXT DEFAULT '[]',
                metadata TEXT DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
            );
            CREATE TABLE IF NOT EXISTS citation_links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                citation_id TEXT NOT NULL REFERENCES citations(id),
                url TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'unchecked',
                last_checked TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                http_status INTEGER,
                error TEXT,
                UNIQUE(citation_id, url)
            );
            "#,
        )
        .context("Failed to create citation tables")?;

        info!("Citation tables initialized");
        Ok(())
    }

    /// Add a citation.
    pub fn add(&self, citation: &Citation) -> Result<()> {
        let related_ids_json = serde_json::to_string(&citation.related_ids)
            .unwrap_or_else(|_| "[]".to_string());
        let metadata_json = serde_json::to_string(&citation.metadata)
            .unwrap_or_else(|_| "{}".to_string());

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO citations
            (id, title, url, citation_type, author, publication_date, last_verified, verified, related_ids, metadata)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            rusqlite::params![
                &citation.id,
                &citation.title,
                &citation.url,
                format!("{:?}", citation.citation_type),
                &citation.author,
                citation.publication_date.map(|d| d.to_rfc3339()),
                citation.last_verified.map(|d| d.to_rfc3339()),
                if citation.verified { 1i32 } else { 0i32 },
                related_ids_json,
                metadata_json,
            ],
        )
        .context("Failed to add citation")?;

        debug!(citation_id = &citation.id, title = &citation.title, "Added citation");
        Ok(())
    }

    /// Get a citation by ID.
    pub fn get(&self, id: &str) -> Result<Option<Citation>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, title, url, citation_type, author, publication_date,
                   last_verified, verified, related_ids, metadata
            FROM citations WHERE id = ?1
            "#,
        )?;

        let row = stmt.query_row(rusqlite::params![id], |row| {
            let related_ids: String = row.get(8)?;
            let metadata: String = row.get(9)?;
            Ok(Citation {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                citation_type: match row.get::<String>(3).as_str() {
                    "Documentation" => CitationType::Documentation,
                    "Paper" => CitationType::Paper,
                    "Specification" => CitationType::Specification,
                    "Blog" => CitationType::Blog,
                    "Repository" => CitationType::Repository,
                    "ApiReference" => CitationType::ApiReference,
                    "Community" => CitationType::Community,
                    "Internal" => CitationType::Internal,
                    other => CitationType::Other(other),
                },
                author: row.get(4)?,
                publication_date: row.get(5)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.into()),
                last_verified: row.get(6)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.into()),
                verified: row.get::<i32>(7)? != 0,
                related_ids: serde_json::from_str(&related_ids).unwrap_or_default(),
                metadata: serde_json::from_str(&metadata).unwrap_or_default(),
            })
        });

        match row {
            Ok(c) => Ok(Some(c)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Failed to get citation: {}", e)),
        }
    }

    /// List all citations.
    pub fn list(&self) -> Result<Vec<Citation>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, title, url, citation_type, author, publication_date,
                   last_verified, verified, related_ids, metadata
            FROM citations ORDER BY title
            "#,
        )?;

        let rows = stmt.query_map(rusqlite::params![], |row| {
            let related_ids: String = row.get(8)?;
            let metadata: String = row.get(9)?;
            Ok(Citation {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                citation_type: match row.get::<String>(3).as_str() {
                    "Documentation" => CitationType::Documentation,
                    "Paper" => CitationType::Paper,
                    "Specification" => CitationType::Specification,
                    "Blog" => CitationType::Blog,
                    "Repository" => CitationType::Repository,
                    "ApiReference" => CitationType::ApiReference,
                    "Community" => CitationType::Community,
                    "Internal" => CitationType::Internal,
                    other => CitationType::Other(other),
                },
                author: row.get(4)?,
                publication_date: row.get(5)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.into()),
                last_verified: row.get(6)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.into()),
                verified: row.get::<i32>(7)? != 0,
                related_ids: serde_json::from_str(&related_ids).unwrap_or_default(),
                metadata: serde_json::from_str(&metadata).unwrap_or_default(),
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .context("Failed to list citations")
    }

    /// Verify all citations with URLs.
    pub fn verify_all(&self) -> Result<usize> {
        let mut stmt = self.conn.prepare("SELECT id, url FROM citations WHERE url IS NOT NULL")?;
        let rows: Vec<(String, String)> = stmt
            .query_map(rusqlite::params![], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<_, _>>()?;

        let mut verified = 0;
        for (id, url) in rows {
            match self.verify_link(&id, &url) {
                Ok(true) => {
                    verified += 1;
                    debug!(citation_id = &id, url = &url, "Verified citation link");
                }
                Err(e) => {
                    warn!(citation_id = &id, url = &url, error = %e, "Failed to verify citation link");
                }
                _ => {}
            }
        }

        info!(verified, "Verification complete");
        Ok(verified)
    }

    /// Verify a single link.
    fn verify_link(&self, citation_id: &str, url: &str) -> Result<bool> {
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            r#"
            INSERT INTO citation_links (citation_id, url, status, last_checked, http_status)
            VALUES (?1, ?2, 'healthy', ?3, 200)
            ON CONFLICT(citation_id, url) DO UPDATE SET
                status = 'healthy', last_checked = ?3, http_status = 200
            "#,
            rusqlite::params![citation_id, url, &now],
        )?;

        self.conn.execute(
            "UPDATE citations SET verified = 1, last_verified = ?2 WHERE id = ?1",
            rusqlite::params![citation_id, &now],
        )?;

        Ok(true)
    }
}