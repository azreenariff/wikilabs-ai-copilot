//! Error Handling — Crash Recovery & Graceful Degradation
//!
//! Features:
//! - Global error handler for Tauri events
//! - Crash report collection and persistence
//! - Graceful degradation when features fail
//! - Error categorization and recovery strategies
//! - Session recovery after crash

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{error, info, warn};

/// Error severity levels.
#[derive(Debug, Clone, Serialize)]
pub enum ErrorSeverity {
    /// Non-fatal: app can continue operating.
    Warning,
    /// Degraded: feature failed but app continues with fallback.
    Degraded,
    /// Error: significant error but app survives.
    Error,
    /// Fatal: app must shut down.
    Fatal,
}

/// A structured error event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub id: String,
    pub timestamp: String,
    pub severity: String,
    pub module: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_suggested: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

/// A crash report with full diagnostic context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReport {
    pub timestamp: String,
    pub app_version: String,
    pub platform: String,
    pub architecture: String,
    pub error: ErrorEvent,
    pub previous_errors: Vec<ErrorEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_snapshot: Option<serde_json::Value>,
    pub stack_trace: String,
    pub memory_info: serde_json::Value,
}

/// Error recovery strategy.
#[derive(Debug, Clone, Serialize)]
pub enum RecoveryStrategy {
    /// Retry the operation with exponential backoff.
    Retry {
        max_retries: u32,
        initial_delay_ms: u64,
    },
    /// Fall back to a safe default behavior.
    Fallback,
    /// Show an error dialog and ask user.
    UserPrompt,
    /// Shut down gracefully.
    Shutdown,
}

impl ErrorSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            ErrorSeverity::Warning => "warning",
            ErrorSeverity::Degraded => "degraded",
            ErrorSeverity::Error => "error",
            ErrorSeverity::Fatal => "fatal",
        }
    }

    /// Whether this error can be recovered from.
    pub fn is_recoverable(&self) -> bool {
        match self {
            ErrorSeverity::Warning
            | ErrorSeverity::Degraded
            | ErrorSeverity::Error => true,
            ErrorSeverity::Fatal => false,
        }
    }
}

/// Global error handler with crash report persistence.
pub struct ErrorHandler {
    error_log: Arc<RwLock<Vec<ErrorEvent>>>,
    crash_dir: PathBuf,
    max_errors: usize,
}

impl ErrorHandler {
    pub fn new(crash_dir: PathBuf) -> Self {
        Self {
            error_log: Arc::new(RwLock::new(Vec::new())),
            crash_dir,
            max_errors: 1000,
        }
    }

    /// Handle a non-fatal error.
    pub fn handle_error(
        &self,
        severity: ErrorSeverity,
        module: &str,
        message: &str,
        recovery: Option<&str>,
    ) {
        let event = ErrorEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            severity: severity.as_str().to_string(),
            module: module.to_string(),
            message: message.to_string(),
            stack_trace: None,
            recovery_suggested: recovery.map(|s| s.to_string()),
            context: None,
        };

        // Log to tracing
        match &severity {
            ErrorSeverity::Warning => warn!(module, message, "Error handled (warning)"),
            ErrorSeverity::Degraded => warn!(module, message, "Feature degraded"),
            ErrorSeverity::Error => error!(module, message, "Error handled"),
            ErrorSeverity::Fatal => error!(module, message, "Fatal error — shutdown required"),
        }

        // Store in error log
        let mut log = self.error_log.write().unwrap();
        if log.len() >= self.max_errors {
            log.remove(0);
        }
        log.push(event.clone());

        // Write to error file
        if let Err(e) = self.append_to_error_log(&event) {
            eprintln!("Failed to write error log: {}", e);
        }

        // If fatal, create crash report
        if let ErrorSeverity::Fatal = severity {
            if let Err(e) = self.create_crash_report(&event, &log) {
                eprintln!("Failed to create crash report: {}", e);
            }
        }
    }

    /// Handle a fatal error with full context.
    pub fn handle_fatal(
        &self,
        module: &str,
        message: &str,
        stack_trace: &str,
        context: Option<serde_json::Value>,
    ) {
        let event = ErrorEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            severity: ErrorSeverity::Fatal.as_str().to_string(),
            module: module.to_string(),
            message: message.to_string(),
            stack_trace: Some(stack_trace.to_string()),
            recovery_suggested: Some(
                "Application will shut down. Crash report saved.".to_string(),
            ),
            context,
        };

        error!(
            module,
            message, stack_trace, "FATAL ERROR — initiating shutdown"
        );

        // Write to error log
        let mut log = self.error_log.write().unwrap();
        if log.len() >= self.max_errors {
            log.remove(0);
        }
        log.push(event.clone());

        // Create crash report
        if let Err(e) = self.create_crash_report(&event, &log) {
            eprintln!("Failed to create crash report: {}", e);
        }

        // Write to stderr for crashpad integration
        eprintln!("FATAL ERROR [{}]: {}\n{}", module, message, stack_trace);
    }

    /// Get recovery strategy for an error type.
    pub fn get_recovery_strategy(error_type: &str) -> RecoveryStrategy {
        match error_type {
            "network_timeout" => RecoveryStrategy::Retry {
                max_retries: 3,
                initial_delay_ms: 1000,
            },
            "auth_error" => RecoveryStrategy::UserPrompt,
            "disk_full" => RecoveryStrategy::Shutdown,
            "memory_error" => RecoveryStrategy::Shutdown,
            "feature_unavailable" => RecoveryStrategy::Fallback,
            "config_parse" => RecoveryStrategy::Fallback,
            _ => RecoveryStrategy::Retry {
                max_retries: 1,
                initial_delay_ms: 500,
            },
        }
    }

    /// Check for previous crash and report to user.
    pub fn has_previous_crash(&self) -> bool {
        fs::exists(self.crash_dir.join("last_crash.json")).unwrap_or(false)
    }

    /// Get previous crash report for display.
    pub fn get_previous_crash(&self) -> Option<CrashReport> {
        let path = self.crash_dir.join("last_crash.json");
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Clear crash reports (after user has seen them).
    pub fn clear_crash_reports(&self) -> Result<(), anyhow::Error> {
        fs::remove_file(self.crash_dir.join("last_crash.json"))?;
        info!("Crash reports cleared");
        Ok(())
    }

    /// Append error event to persistent error log.
    fn append_to_error_log(
        &self,
        event: &ErrorEvent,
    ) -> Result<(), anyhow::Error> {
        let path = self.crash_dir.join("error_log.jsonl");
        let line = serde_json::to_string(event)?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        use std::io::Write;
        file.write_all(format!("{}\n", line).as_bytes())?;
        Ok(())
    }

    /// Create a crash report file.
    fn create_crash_report(
        &self,
        error: &ErrorEvent,
        all_errors: &[ErrorEvent],
    ) -> Result<(), anyhow::Error> {
        fs::create_dir_all(&self.crash_dir)?;

        let crash_report = CrashReport {
            timestamp: Utc::now().to_rfc3339(),
            app_version: "1.0.0".to_string(),
            platform: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            error: error.clone(),
            previous_errors: all_errors[..all_errors.len().min(20)].to_vec(),
            settings_snapshot: None,
            stack_trace: error.stack_trace.clone().unwrap_or_default(),
            memory_info: serde_json::json!({
                "available": "unknown",
                "usage": "unknown",
            }),
        };

        // Save current crash and previous
        let content = serde_json::to_string_pretty(&crash_report)?;
        fs::write(self.crash_dir.join("last_crash.json"), &content)?;
        fs::write(
            self.crash_dir.join("crash_history.jsonl"),
            content.as_bytes(),
        )?;

        info!(crash_id = error.id, "Crash report saved");
        Ok(())
    }

    /// Get recent error events.
    pub fn recent_errors(&self, limit: usize) -> Vec<ErrorEvent> {
        let log = self.error_log.read().unwrap();
        log.iter().rev().take(limit).cloned().collect()
    }

    /// Get error statistics.
    pub fn error_stats(&self) -> serde_json::Value {
        let log = self.error_log.read().unwrap();
        let mut counts = serde_json::Map::new();
        for event in log.iter() {
            let entry = counts
                .entry(event.severity.clone())
                .or_insert(serde_json::json!(0));
            if let Some(n) = entry.as_u64() {
                *entry = serde_json::json!(n + 1);
            }
        }
        serde_json::json!({
            "total": log.len(),
            "by_severity": counts,
            "by_module": {
                "main": log.iter().filter(|e| e.module == "main").count(),
            },
        })
    }
}

/// Graceful shutdown handler.
pub struct GracefulShutdown {
    handlers: Arc<RwLock<Vec<Box<dyn Fn() + Send + Sync>>>>,
}

impl GracefulShutdown {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a shutdown handler.
    pub fn register<F>(&self, handler: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.write().unwrap();
        handlers.push(Box::new(handler));
        info!("Shutdown handler registered");
    }

    /// Execute all shutdown handlers.
    pub fn execute(&self) {
        let handlers = self.handlers.read().unwrap();
        for handler in handlers.iter() {
            handler();
        }
        info!("All shutdown handlers executed");
    }
}

/// Panic hook for crash reporting.
pub fn setup_panic_hook(error_handler: Arc<ErrorHandler>) {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());

        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let stack_trace = std::backtrace::Backtrace::force_capture().to_string();

        error_handler.handle_fatal(
            "panic",
            &format!("Panic at {}: {}", location, message),
            &stack_trace,
            None,
        );

        // Call original hook for default behavior
        original_hook(info);
    }));
}

/// Get error count by module.
pub fn get_error_count_by_module(errors: &[ErrorEvent]) -> serde_json::Value {
    let mut module_counts: HashMap<String, usize> = HashMap::new();
    for event in errors {
        *module_counts.entry(event.module.clone()).or_insert(0) += 1;
    }
    serde_json::to_value(&module_counts).unwrap_or(serde_json::json!({}))
}

/// Get most recent error for a given module.
pub fn recent_error_for_module<'a>(errors: &'a [ErrorEvent], module: &str) -> Option<&'a ErrorEvent> {
    errors.iter().rev().find(|e| e.module == module)
}

/// Check if error count exceeds threshold (used for degraded mode).
pub fn is_error_rate_high(errors: &[ErrorEvent], threshold: usize, window: usize) -> bool {
    if errors.len() < window {
        return false;
    }
    let recent = &errors[errors.len() - window..];
    recent
        .iter()
        .filter(|e| e.severity == "error" || e.severity == "fatal")
        .count()
        > threshold
}