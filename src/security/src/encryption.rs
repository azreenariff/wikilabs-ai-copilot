//! Encryption — AES-256-GCM for data at rest.

pub struct EncryptionEngine;

impl EncryptionEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn encrypt(&self, _key: &[u8], _plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
        // TODO: AES-256-GCM encrypt
        anyhow::bail!("Not yet implemented")
    }

    pub fn decrypt(&self, _key: &[u8], _ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> {
        // TODO: AES-256-GCM decrypt
        anyhow::bail!("Not yet implemented")
    }
}
