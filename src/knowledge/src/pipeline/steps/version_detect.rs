//! Version detection step — check for document version history.

use crate::processing::{Document, VersionEntry, VersionInfo};
use regex::Regex;
use tracing::debug;

/// The version detection pipeline step.
pub struct VersionDetectStep;

impl VersionDetectStep {
    pub fn new() -> Self {
        Self
    }

    /// Run the version detection step on a parsed document.
    pub fn run(&self, doc: &mut Document) {
        let mut has_version_header = false;
        let mut version = String::new();
        let mut detected_history = Vec::new();

        // Check for version headers in metadata (YAML frontmatter patterns)
        let version_re = Regex::new(r"(?i)version[:\s]+([0-9]+\.?[0-9]*\.?[0-9]*)").unwrap();
        if let Some(cap) = version_re.captures(&doc.full_text) {
            has_version_header = true;
            version = cap[1].to_string();
        }

        // Check for changelog/version history patterns
        let changelog_re =
            Regex::new(r"(?im)^v?\s*([0-9]+\.?[0-9]*\.?[0-9]*)\s*[-–]\s*(.*?)$").unwrap();
        for cap in changelog_re.captures_iter(&doc.full_text) {
            let ver = cap[1].trim().to_string();
            let summary = cap[2].trim().to_string();
            detected_history.push(VersionEntry {
                version: ver,
                date: String::new(),
                summary,
            });
        }

        // Extract dates from version history if date patterns exist
        let date_re = Regex::new(r"(\d{4}[-/]\d{1,2}[-/]\d{1,2})").unwrap();
        for line in doc.full_text.split('\n') {
            if let Some(date_cap) = date_re.captures(line) {
                let date_str = date_cap[1].to_string();
                // Try to associate with the last detected version
                if let Some(last) = detected_history.last_mut() {
                    if last.date.is_empty() {
                        last.date = date_str;
                    }
                }
            }
        }

        debug!(
            has_version = has_version_header,
            version = %version,
            history_count = detected_history.len(),
            "Version detection complete"
        );

        // Store version info in document metadata
        let version_info = VersionInfo {
            has_version_header,
            version,
            detected_history,
        };
        doc.version_info = version_info.clone();
    }
}

impl Default for VersionDetectStep {
    fn default() -> Self {
        Self::new()
    }
}
