//! OS keychain — Windows Credential Manager, macOS Keychain, Linux Secret Service.

pub struct Keychain;

pub struct Secret {
    pub service: String,
    pub username: String,
    pub password: String,
}

impl Keychain {
    pub fn new() -> Self {
        Self
    }

    pub async fn store(&self, _secret: Secret) -> anyhow::Result<()> {
        // TODO: Store in OS keychain
        anyhow::bail!("Not yet implemented")
    }

    pub async fn retrieve(&self, _service: &str, _username: &str) -> anyhow::Result<Secret> {
        // TODO: Retrieve from OS keychain
        anyhow::bail!("Not yet implemented")
    }
}