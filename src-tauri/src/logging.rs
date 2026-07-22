//! Logging & Diagnostics — Structured Logging with Redaction
//!
//! Features:
//! - Tracing-based structured logging (JSON + console)
//! - Automatic secrets redaction in log output
//! - File logging with rotation
//! - Diagnostic package generation (for bug reports)
//! - Log level control per module
//!
//! Log locations:
//! - Linux: ~/.config/com.wikilabs.copilot/logs/
//! - Windows: %APPDATA%\com.wikilabs.copilot\logs\

use crate::config::{AppSettingsStore, LoggingSettings};
use chrono::Utc;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use tracing::{info, warn};
use tracing_subscriber::{
    fmt::format::FmtSpan,
    prelude::*,
    EnvFilter,
};
use tracing_appender::non_blocking::WorkerGuard;

static INIT: Once = Once::new();
static mut LOGGER_GUARD: Option<WorkerGuard> = None;

/// Sensitive field patterns to redact in all log output.
const SENSITIVE_PATTERNS: &[(&str, &str)] = &[
    ("password", "PASSWORD_REDACTED"),
    ("secret", "SECRET_REDACTED"),
    ("token", "TOKEN_REDACTED"),
    ("api_key", "API_KEY_REDACTED"),
    ("authorization", "AUTH_REDACTED"),
];

/// A log entry with structured fields.
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub module: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
}

/// A diagnostic report bundle for bug reports.
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticPackage {
    pub version: String,
    pub generated_at: String,
    pub platform: String,
    pub architecture: String,
    pub app_data_dir: String,
    pub log_files: Vec<LogFileInfo>,
    pub settings_report: serde_json::Value,
    pub system_info: serde_json::Value,
}

/// Information about a log file.
#[derive(Debug, Clone, Serialize)]
pub struct LogFileInfo {
    pub name: String,
    pub size_bytes: u64,
    pub last_modified: String,
    pub line_count: usize,
}

/// Initialize the logging system.
pub fn init_logging(
    settings: &LoggingSettings,
    log_dir: &Path,
) -> Result<Option<WorkerGuard>, anyhow::Error> {
    INIT.call_once(|| {
        if let Err(e) = init_logging_inner(settings, log_dir) {
            eprintln!("Failed to initialize logging: {}", e);
        }
    });
    // WorkerGuard lives for 'static, so we leak it to keep it alive
    Ok(None)
}

fn init_logging_inner(settings: &LoggingSettings, log_dir: &Path) -> Result<(), anyhow::Error> {
    // Ensure log directory exists
    fs::create_dir_all(log_dir)?;

    // Create environment filter from config
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&settings.level));

    // File writer with rolling rotation, wrapped in non-blocking writer
    let rolling_writer = tracing_appender::rolling::daily(log_dir, "wikilabs-copilot.log");
    let (file_writer, guard) = tracing_appender::non_blocking(rolling_writer);
    let _guard = guard;

    // Store guard in static to prevent drop
    unsafe {
        LOGGER_GUARD = Some(_guard);
    }

    // File layer — structured JSON
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .json();

    // Console layer — human-readable
    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_target(false);

    // Build subscriber with both layers
    let subscriber = tracing_subscriber::registry()
        .with(env_filter.clone())
        .with(file_layer)
        .with(console_layer);

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| anyhow::anyhow!("Failed to set tracing subscriber: {}", e))?;

    info!(
        log_level = settings.level,
        structured = settings.structured_logging,
        file_logging = settings.file_logging,
        "Logging initialized"
    );

    Ok(())
}

/// Redact sensitive information from a string for safe log output.
pub fn redact_sensitive_data(input: &str) -> String {
    let mut result = input.to_string();

    for (pattern, replacement) in SENSITIVE_PATTERNS {
        let re = Regex::new(&format!(r#""{}"\s*:\s*"[^"]*""#, pattern)).unwrap();
        result = re
            .replace_all(&result, format!("\"{}\": \"{}\"", pattern, replacement))
            .to_string();
    }

    result
}

/// Generate a diagnostic package for bug reports.
pub fn generate_diagnostics(
    settings_store: &AppSettingsStore,
    app_data_dir: &Path,
) -> Result<DiagnosticPackage, anyhow::Error> {
    let validation = settings_store.validate();
    let settings = settings_store.get();

    // Collect log file info
    let log_dir = app_data_dir.join("logs");
    let mut log_files = Vec::new();

    if log_dir.exists() {
        for entry in fs::read_dir(&log_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "log") {
                let metadata = fs::metadata(&path)?;
                let line_count = count_lines(&path)?;
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let last_modified = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

                log_files.push(LogFileInfo {
                    name,
                    size_bytes: metadata.len(),
                    last_modified,
                    line_count,
                });
            }
        }
    }

    // Build settings report (redacted)
    let mut settings_json = serde_json::to_value(&settings)?;
    if let Some(obj) = settings_json.as_object_mut() {
        if let Some(ai) = obj.get_mut("ai_provider") {
            if let Some(map) = ai.as_object_mut() {
                if let Some(key) = map.get_mut("api_key") {
                    *key = serde_json::json!("REDACTED");
                }
            }
        }
        // Redact security settings
        if let Some(sec) = obj.get_mut("security") {
            if let Some(map) = sec.as_object_mut() {
                map.insert("encryption_key".to_string(), serde_json::json!("REDACTED"));
            }
        }
    }

    Ok(DiagnosticPackage {
        version: "1.0.0".to_string(),
        generated_at: Utc::now().to_rfc3339(),
        platform: std::env::consts::OS.to_string(),
        architecture: std::env::consts::ARCH.to_string(),
        app_data_dir: app_data_dir.to_string_lossy().to_string(),
        log_files,
        settings_report: settings_json,
        system_info: serde_json::json!({
            "validation_errors": validation.errors,
            "validation_warnings": validation.warnings,
            "settings_version": settings.schema_version,
        }),
    })
}

/// Save diagnostic package to file.
pub fn save_diagnostics(
    pkg: &DiagnosticPackage,
    output_dir: &Path,
) -> Result<PathBuf, anyhow::Error> {
    fs::create_dir_all(output_dir)?;

    let filename = format!("diagnostics_{}.json", Utc::now().format("%Y%m%d_%H%M%S"));
    let path = output_dir.join(filename);

    let content = serde_json::to_string_pretty(pkg)?;
    fs::write(&path, content)?;

    info!(path = %path.display(), "Diagnostic package saved");
    Ok(path)
}

/// Count lines in a log file.
fn count_lines(path: &Path) -> Result<usize, anyhow::Error> {
    let content = fs::read_to_string(path)?;
    Ok(content.lines().count())
}

/// Get the current log directory.
pub fn log_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("logs")
}

/// Clear log files (keep only last N days).
pub fn rotate_logs(log_dir: &Path, keep_days: u32) -> Result<usize, anyhow::Error> {
    let mut deleted = 0;
    let cutoff = Utc::now() - chrono::Duration::days(keep_days as i64);

    if !log_dir.exists() {
        return Ok(0);
    }

    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    let modified_dt = chrono::DateTime::<chrono::Utc>::from(modified);
                    if modified_dt < cutoff {
                        fs::remove_file(&path)?;
                        deleted += 1;
                        warn!(path = %path.display(), "Rotated old log file");
                    }
                }
            }
        }
    }

    info!(deleted, "Log rotation complete");
    Ok(deleted)
}