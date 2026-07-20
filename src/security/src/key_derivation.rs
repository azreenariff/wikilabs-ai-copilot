//! Key derivation — HKDF-SHA256 from OS keychain master key.

pub struct KeyDerivation;

impl KeyDerivation {
    pub fn new() -> Self {
        Self
    }

    pub fn derive_data_enc_key(&self, _master_key: &[u8]) -> [u8; 32] {
        // TODO: HKDF-Expand(master_key, "data-enc")
        [0u8; 32]
    }

    pub fn derive_memory_auth_key(&self, _master_key: &[u8]) -> [u8; 32] {
        // TODO: HKDF-Expand(master_key, "memory-auth")
        [0u8; 32]
    }

    pub fn derive_session_key(&self, _master_key: &[u8], _session_id: &str) -> [u8; 32] {
        // TODO: HKDF-Expand(master_key, "session:")
        [0u8; 32]
    }
}
