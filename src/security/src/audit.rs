//! Audit log integrity — hash chain or Ed25519-signed entries.

pub struct AuditLog;

pub struct AuditEntry {
    pub id: uuid::Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: String,
    pub actor: String,
    pub hash: String,      // SHA-256 of previous entry
    pub signature: String, // Ed25519 signature (if applicable)
}

impl AuditLog {
    pub fn new() -> Self {
        Self
    }

    pub async fn append(&self, _entry: AuditEntry) -> anyhow::Result<()> {
        // TODO: Append with hash chain integrity
        anyhow::bail!("Not yet implemented")
    }
}
