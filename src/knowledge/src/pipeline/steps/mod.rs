//! Pipeline step trait and common types.

pub mod chunk;
pub mod clean;
pub mod dedup;
pub mod discover;
pub mod incremental;
pub mod index_prepare;
pub mod metadata_extract;
pub mod normalize;
pub mod parse;
pub mod validate;
pub mod version_detect;

use std::path::Path;

use regex::Regex;
use sha2::{Digest, Sha256};

/// Supported languages for detection.
#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    English,
    Unknown,
}

impl Language {
    /// Basic language detection based on character patterns.
    pub fn detect(text: &str) -> Self {
        if text.is_empty() {
            return Self::Unknown;
        }

        // Check for non-ASCII characters (non-English scripts)
        let non_ascii_ratio =
            text.chars().filter(|c| *c > '\x7f').count() as f64 / text.chars().count() as f64;

        if non_ascii_ratio > 0.1 {
            return Self::Unknown;
        }

        Self::English
    }
}

/// Compute SHA-256 hash of content.
pub fn compute_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Detect file extension in lowercase.
pub fn file_extension(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}

/// Extract filename from path.
pub fn filename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}

/// Extract title from filename (remove extension, convert to readable form).
pub fn filename_to_title(filename: &str) -> String {
    let re = Regex::new(r"[\._-]+").unwrap();
    re.replace_all(filename, " ")
        .to_string()
        .trim()
        .to_string()
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect()
}
