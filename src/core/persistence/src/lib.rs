//! Database persistence layer — SQLite connection management.
//!
//! - Single-file SQLite database at `~/.local/share/wikilabs/wikilabs.db`
//! - Schema migrations
//! - Repository implementations for all entities
//! - Connection pool for concurrent access

pub mod db;
pub mod schema;
pub mod migrations;
pub mod repositories;

pub use db::Database;
pub use schema::INIT_SQL;
pub use repositories::{RepositoryFactory, WorkspaceRepository, ChatMessageRepository,
    KnowledgeDocumentRepository, KnowledgeChunkRepository, SettingsRepository,
    AuditLogRepository, WorkspaceRow, ChatMessageRow, KnowledgeDocumentRow};