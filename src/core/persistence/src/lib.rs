//! Database persistence layer — SQLite connection management.
//!
//! - Single-file SQLite database at `~/.local/share/wikilabs/wikilabs.db`
//! - Schema migrations
//! - Repository implementations for all entities
//! - Connection pool for concurrent access

pub mod db;
pub mod migrations;
pub mod repositories;
pub mod schema;

pub use db::Database;
pub use repositories::{
    AuditLogRepository, ChatMessageRepository, ChatMessageRow, KnowledgeChunkRepository,
    KnowledgeDocumentRepository, KnowledgeDocumentRow, RepositoryFactory, SettingsRepository,
    WorkspaceRepository, WorkspaceRow,
};
pub use schema::INIT_SQL;
