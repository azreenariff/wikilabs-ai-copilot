//! Security layer.
//!
//! - OS keychain master key (Windows Credential Manager, macOS Keychain, Linux Secret Service)
//! - HKDF-SHA256 sub-key derivation
//! - AES-256-GCM data encryption
//! - Credential storage & retrieval
//! - Data classification types (Public, Internal, Confidential, Restricted)
//! - Audit log integrity (hash chain / Ed25519-signed)
//! - Prompt injection defense layer
//! - Secret detection in source code

pub mod audit;
pub mod classification;
pub mod credentials;
pub mod encryption;
pub mod injection_defense;
pub mod key_derivation;
pub mod keychain;

#[cfg(test)]
mod tests;
