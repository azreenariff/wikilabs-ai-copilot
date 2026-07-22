//! Credential storage — API keys, SSH keys, database credentials.

pub struct CredentialStore;

pub struct Credential {
    pub id: uuid::Uuid,
    pub name: String,
    pub encrypted_value: Vec<u8>,
    pub workspace_id: uuid::Uuid,
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore {
    pub fn new() -> Self {
        Self
    }

    pub async fn store(&self, _credential: Credential) -> anyhow::Result<()> {
        anyhow::bail!("Not yet implemented")
    }

    pub async fn list(&self, _workspace_id: uuid::Uuid) -> anyhow::Result<Vec<Credential>> {
        anyhow::bail!("Not yet implemented")
    }
}
