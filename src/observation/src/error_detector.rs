//! Error Detector — Proactive Error Detection Engine
//!
//! Detects and classifies errors in real-time from multiple sources:
//! - Terminal output (systemctl, journalctl, application logs)
//! - Browser content (HTTP errors, page titles, visible text)
//! - Application windows (error dialogs, status indicators)
//!
//! This is the "eyes" that spots problems before the user even notices.

use std::sync::{Arc, Mutex};

use chrono::Utc;

// Uses AnalysisResult from semantic_analyzer via session_tracker

/// Severity level of a detected error.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ErrorSeverity {
    /// Low severity — informational, may not need attention.
    Low,
    /// Medium severity — something is wrong but not critical.
    Medium,
    /// High severity — critical issue requiring immediate attention.
    High,
    /// Error is preventing a service from functioning.
    Critical,
}

/// Source where the error was detected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorSource {
    /// Detected from terminal output.
    Terminal,
    /// Detected from browser page content.
    Browser,
    /// Detected from application window.
    Application,
    /// Detected from system monitoring (CPU, memory, disk).
    System,
}

/// An error or warning detected by the analyzer.
#[derive(Debug, Clone)]
pub struct DetectedError {
    /// Unique error ID.
    pub id: u64,
    /// When the error was detected.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Where the error was found.
    pub source: ErrorSource,
    /// How severe the error is.
    pub severity: ErrorSeverity,
    /// Short error title.
    pub title: String,
    /// Detailed description of the error.
    pub description: String,
    /// The raw content that triggered detection (truncated).
    pub raw_content: String,
    /// Suggested next action.
    pub suggested_action: Option<String>,
    /// Related service/system name (if identifiable).
    pub related_service: Option<String>,
}

/// Pattern that can be detected in text.
#[derive(Debug, Clone)]
pub struct ErrorPattern {
    /// Text pattern to match (case-insensitive).
    pub pattern: String,
    /// Severity when matched.
    pub severity: ErrorSeverity,
    /// Title for this error type.
    pub title: String,
    /// Human-readable description.
    pub description: String,
    /// Suggested action to resolve.
    pub suggested_action: Option<String>,
    /// Related service name.
    pub related_service: Option<String>,
}

/// Error detector state.
#[derive(Debug, Clone)]
pub struct DetectorState {
    /// All detected errors (up to 100).
    pub errors: Vec<DetectedError>,
    /// Total count of errors detected.
    pub total_detected: u64,
    /// Count by severity.
    pub errors_by_severity: BumpCount,
    /// Most recent error source.
    pub last_source: Option<ErrorSource>,
}

/// Simple bump counter.
#[derive(Debug, Clone, Default)]
pub struct BumpCount {
    pub low: u32,
    pub medium: u32,
    pub high: u32,
    pub critical: u32,
}

/// Error detection engine — the "eyes" that spot problems.
pub struct ErrorDetector {
    state: Arc<Mutex<DetectorState>>,
    error_counter: Arc<Mutex<u64>>,
}

impl ErrorDetector {
    /// Create a new error detector.
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(DetectorState {
                errors: Vec::new(),
                total_detected: 0,
                errors_by_severity: BumpCount::default(),
                last_source: None,
            })),
            error_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Analyze terminal output and detect errors.
    pub fn analyze_terminal_output(&self, output: &str, source_name: Option<&str>) -> Vec<DetectedError> {
        let mut errors = Vec::new();
        let raw = output.to_string();
        let output_lower = output.to_lowercase();

        // Check for systemd service failures
        let service_patterns = [
            ("active (failed)", ErrorSeverity::Critical, "Service failed", "Systemd service is in failed state"),
            ("inactive (dead)", ErrorSeverity::High, "Service inactive", "Systemd service is not running"),
            ("active (exited)", ErrorSeverity::Medium, "Service exited cleanly", "Service exited but may need to be running"),
            ("dead", ErrorSeverity::High, "Service dead", "Systemd service is stopped"),
            ("failed", ErrorSeverity::High, "Operation failed", "An operation has failed"),
        ];

        for (pattern, severity, title, desc) in &service_patterns {
            if output_lower.contains(pattern) {
                errors.push(self.create_error(
                    *severity,
                    title.to_string(),
                    desc.to_string(),
                    raw.clone(),
                    None,
                    source_name.map(String::from),
                    ErrorSource::Terminal,
                ));
            }
        }

        // Check for common error messages
        let common_errors = [
            ("permission denied", ErrorSeverity::High, "Permission denied", "Insufficient permissions to perform this operation"),
            ("connection refused", ErrorSeverity::High, "Connection refused", "Cannot connect to the target — service may be down or port blocked"),
            ("no such file or directory", ErrorSeverity::Medium, "File not found", "The specified file or directory does not exist"),
            ("timeout", ErrorSeverity::Medium, "Timeout detected", "Operation timed out — target may be unresponsive"),
            ("segmentation fault", ErrorSeverity::Critical, "Segmentation fault", "Program crashed with a segmentation fault"),
            ("out of memory", ErrorSeverity::Critical, "Out of memory", "System or process has run out of memory"),
            ("oom-killer", ErrorSeverity::Critical, "OOM killer activated", "Kernel OOM killer has terminated a process"),
            ("cannot resolve", ErrorSeverity::High, "DNS resolution failed", "Cannot resolve hostname — check DNS configuration"),
            ("host unreachable", ErrorSeverity::High, "Host unreachable", "Target host is not reachable"),
            ("network unreachable", ErrorSeverity::High, "Network unreachable", "Network path to target is unavailable"),
            ("address already in use", ErrorSeverity::Medium, "Port already in use", "Another process is using this port"),
            ("broken pipe", ErrorSeverity::Medium, "Broken pipe", "Connection was unexpectedly broken"),
            ("connection reset", ErrorSeverity::High, "Connection reset", "Remote host reset the connection"),
            ("tls handshake failed", ErrorSeverity::High, "TLS handshake failed", "SSL/TLS negotiation failed — certificate or version mismatch"),
            ("certificate expired", ErrorSeverity::High, "Certificate expired", "SSL certificate has expired"),
        ];

        for (pattern, severity, title, desc) in &common_errors {
            if output_lower.contains(pattern) {
                errors.push(self.create_error(
                    *severity,
                    title.to_string(),
                    desc.to_string(),
                    raw.clone(),
                    None,
                    source_name.map(String::from),
                    ErrorSource::Terminal,
                ));
            }
        }

        // Check for HTTP error status codes
        let http_errors = [
            ("500", ErrorSeverity::High, "HTTP 500 Internal Server Error", "Server encountered an internal error"),
            ("502 bad gateway", ErrorSeverity::High, "HTTP 502 Bad Gateway", "Server received an invalid response from upstream"),
            ("503", ErrorSeverity::Critical, "HTTP 503 Service Unavailable", "Server is temporarily unable to handle the request"),
            ("504", ErrorSeverity::High, "HTTP 504 Gateway Timeout", "Gateway did not receive a timely response"),
        ];

        for (pattern, severity, title, desc) in &http_errors {
            if output_lower.contains(pattern) {
                errors.push(self.create_error(
                    *severity,
                    title.to_string(),
                    desc.to_string(),
                    raw.clone(),
                    None,
                    source_name.map(String::from),
                    ErrorSource::Terminal,
                ));
            }
        }

        // Check for container/pod errors
        let container_errors = [
            ("imagepullbackoff", ErrorSeverity::High, "Container image pull failed", "Kubernetes cannot pull the container image"),
            ("errimagepull", ErrorSeverity::High, "Container image pull error", "Error pulling the container image"),
            ("oomkilled", ErrorSeverity::Critical, "Container OOM killed", "Container was terminated due to out-of-memory"),
            ("crashloopbackoff", ErrorSeverity::High, "Container crash loop", "Container keeps crashing and restarting"),
            ("pending", ErrorSeverity::Medium, "Container pending", "Container is waiting for resources"),
            ("evicted", ErrorSeverity::High, "Container evicted", "Container was evicted by the node"),
        ];

        for (pattern, severity, title, desc) in &container_errors {
            if output_lower.contains(pattern) {
                errors.push(self.create_error(
                    *severity,
                    title.to_string(),
                    desc.to_string(),
                    raw.clone(),
                    None,
                    source_name.map(String::from),
                    ErrorSource::Terminal,
                ));
            }
        }

        // Check for disk issues
        let disk_errors = [
            ("no space left on device", ErrorSeverity::Critical, "Disk full", "No space left on the filesystem"),
            ("disk full", ErrorSeverity::Critical, "Disk full", "The disk has no remaining space"),
            ("i/o error", ErrorSeverity::Critical, "I/O error", "Disk read/write error detected"),
        ];

        for (pattern, severity, title, desc) in &disk_errors {
            if output_lower.contains(pattern) {
                errors.push(self.create_error(
                    *severity,
                    title.to_string(),
                    desc.to_string(),
                    raw.clone(),
                    None,
                    source_name.map(String::from),
                    ErrorSource::Terminal,
                ));
            }
        }

        errors
    }

    /// Analyze browser content and detect errors.
    pub fn analyze_browser_content(&self, content: &str, url: Option<&str>) -> Vec<DetectedError> {
        let mut errors = Vec::new();
        let raw = content.to_string();
        let content_lower = content.to_lowercase();
        let url_str = url.unwrap_or("");
        let _url_lower = url_str.to_lowercase();

        // Check for common error page patterns
        let browser_errors = [
            ("500 internal server error", ErrorSeverity::Critical, "HTTP 500 Error", "Server returned a 500 Internal Server Error"),
            ("502 bad gateway", ErrorSeverity::High, "HTTP 502 Error", "Server returned a 502 Bad Gateway"),
            ("503 service unavailable", ErrorSeverity::Critical, "HTTP 503 Error", "Server is temporarily unavailable"),
            ("403 forbidden", ErrorSeverity::Medium, "HTTP 403 Forbidden", "Access is forbidden — check authentication/permissions"),
            ("404 not found", ErrorSeverity::Medium, "HTTP 404 Not Found", "The requested page or resource was not found"),
            ("connection refused", ErrorSeverity::High, "Connection refused", "Browser cannot connect to the server"),
            ("unable to connect", ErrorSeverity::High, "Unable to connect", "Browser is unable to establish a connection"),
            ("dns_probe_finished_nxdomain", ErrorSeverity::High, "DNS error", "Domain name could not be resolved"),
            ("ssl", ErrorSeverity::Medium, "SSL/TLS Error", "SSL/TLS certificate or connection issue detected"),
            ("certificate", ErrorSeverity::Medium, "Certificate Error", "SSL certificate issue detected"),
            ("maintenance", ErrorSeverity::Low, "Maintenance Mode", "The site may be under maintenance"),
            ("down for maintenance", ErrorSeverity::Low, "Site Under Maintenance", "The site is currently down for maintenance"),
        ];

        for (pattern, severity, title, desc) in &browser_errors {
            if content_lower.contains(pattern) {
                errors.push(self.create_error(
                    *severity,
                    title.to_string(),
                    desc.to_string(),
                    raw.clone(),
                    Some(url_str.to_string()),
                    self.extract_service_from_url(url_str),
                    ErrorSource::Browser,
                ));
            }
        }

        errors
    }

    /// Analyze application window content for errors.
    pub fn analyze_window_content(&self, title: &str, content: &str) -> Vec<DetectedError> {
        let mut errors = Vec::new();
        let title_lower = title.to_lowercase();
        let content_lower = content.to_lowercase();

        let window_errors = [
            ("error", ErrorSeverity::High, "Application Error", "An error was detected in the application window"),
            ("warning", ErrorSeverity::Medium, "Application Warning", "A warning was detected in the application window"),
            ("alert", ErrorSeverity::Medium, "Application Alert", "An alert was shown in the application"),
            ("exception", ErrorSeverity::High, "Application Exception", "An exception occurred in the application"),
            ("crash", ErrorSeverity::Critical, "Application Crash", "The application has crashed"),
        ];

        for (pattern, severity, title_text, desc) in &window_errors {
            if title_lower.contains(pattern) || content_lower.contains(pattern) {
                errors.push(self.create_error(
                    *severity,
                    title_text.to_string(),
                    desc.to_string(),
                    content.to_string(),
                    Some(title.to_string()),
                    None,
                    ErrorSource::Application,
                ));
            }
        }

        errors
    }

    /// Run a comprehensive analysis tick across all sources.
    pub fn analyze_tick(
        &self,
        browser_url: Option<&str>,
        browser_content: Option<&str>,
        terminal_cmd: Option<&str>,
        terminal_output: Option<&str>,
        window_title: Option<&str>,
        window_content: Option<&str>,
    ) -> Vec<DetectedError> {
        let mut all_errors = Vec::new();

        if let Some(content) = browser_content {
            all_errors.extend(self.analyze_browser_content(content, browser_url));
        }
        if let Some(output) = terminal_output {
            all_errors.extend(self.analyze_terminal_output(output, terminal_cmd));
        }
        if let Some(title) = window_title {
            all_errors.extend(self.analyze_window_content(title, window_content.unwrap_or("")));
        }

        // Store in state
        {
            let mut state = self.state.lock().unwrap();
            for err in &all_errors {
                state.errors.push(err.clone());
                if state.errors.len() > 100 {
                    let end = state.errors.len() - 100;
                    state.errors.drain(0..end);
                }
                state.total_detected += 1;
                match err.severity {
                    ErrorSeverity::Low => state.errors_by_severity.low += 1,
                    ErrorSeverity::Medium => state.errors_by_severity.medium += 1,
                    ErrorSeverity::High => state.errors_by_severity.high += 1,
                    ErrorSeverity::Critical => state.errors_by_severity.critical += 1,
                }
                state.last_source = Some(err.source.clone());
            }
        }

        all_errors
    }

    /// Get all detected errors.
    pub fn get_errors(&self) -> Vec<DetectedError> {
        self.state.lock().unwrap().errors.clone()
    }

    /// Get summary of detected errors.
    pub fn get_summary(&self) -> String {
        let state = self.state.lock().unwrap();
        format!(
            "Detected {} errors: {} low, {} medium, {} high, {} critical",
            state.errors_by_severity.low
                + state.errors_by_severity.medium
                + state.errors_by_severity.high
                + state.errors_by_severity.critical,
            state.errors_by_severity.low,
            state.errors_by_severity.medium,
            state.errors_by_severity.high,
            state.errors_by_severity.critical,
        )
    }

    /// Clear all detected errors.
    pub fn clear(&self) {
        let mut state = self.state.lock().unwrap();
        state.errors.clear();
        state.total_detected = 0;
        state.errors_by_severity = BumpCount::default();
        state.last_source = None;
    }

    /// Create a new detected error.
    #[allow(clippy::too_many_arguments)]
    fn create_error(
        &self,
        severity: ErrorSeverity,
        title: String,
        description: String,
        raw_content: String,
        _url: Option<String>,
        service: Option<String>,
        source: ErrorSource,
    ) -> DetectedError {
        let id = {
            let mut counter = self.error_counter.lock().unwrap();
            *counter += 1;
            *counter
        };

        DetectedError {
            id,
            timestamp: Utc::now(),
            source,
            severity,
            title: title.clone(),
            description,
            raw_content: if raw_content.len() > 500 {
                format!("{}...", &raw_content[..500])
            } else {
                raw_content
            },
            suggested_action: self.generate_suggestion(&title, &service),
            related_service: service,
        }
    }

    /// Generate a suggested action based on error type.
    fn generate_suggestion(&self, title: &str, service: &Option<String>) -> Option<String> {
        let title_lower = title.to_lowercase();

        if title_lower.contains("service failed") || title_lower.contains("service inactive") {
            if let Some(ref svc) = service {
                return Some(format!("Try restarting the service with `systemctl restart {}`", svc));
            }
            return Some("Try restarting the affected service and check its logs with `journalctl -xe`".to_string());
        }
        if title_lower.contains("connection refused") || title_lower.contains("unable to connect") {
            return Some("Check if the target service is running: `systemctl status <service>`".to_string());
        }
        if title_lower.contains("disk full") || title_lower.contains("no space left") {
            return Some("Check disk usage with `df -h` and `du -sh /var/log/*` to find what's consuming space".to_string());
        }
        if title_lower.contains("out of memory") {
            return Some("Check memory usage with `free -h` and identify memory-hungry processes with `top`".to_string());
        }
        if title_lower.contains("permission denied") {
            return Some("Check file permissions with `ls -la` and consider using `sudo` if needed".to_string());
        }
        if title_lower.contains("timeout") {
            return Some("Check network connectivity with `ping` and verify the service is listening with `ss -tlnp`".to_string());
        }
        if title_lower.contains("dns") || title_lower.contains("cannot resolve") {
            return Some("Check DNS resolution with `nslookup <hostname>` and verify `/etc/hosts` if applicable".to_string());
        }
        if title_lower.contains("http 500") {
            return Some("Check application logs for the root cause: `journalctl -u <service> --since '1 hour ago'`".to_string());
        }
        if title_lower.contains("http 503") {
            return Some("The service is overloaded or down. Check if dependent services are running and consider scaling.".to_string());
        }
        if title_lower.contains("certificate") {
            return Some("Check certificate validity: `openssl x509 -in /path/to/cert -noout -dates`".to_string());
        }
        if title_lower.contains("container") {
            return Some("Check container logs: `kubectl logs <pod-name>` or `docker logs <container-id>`".to_string());
        }

        // Generic fallback
        None
    }

    /// Extract service name from URL.
    fn extract_service_from_url(&self, url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        if url_lower.contains("nagios") { return Some("nagios".to_string()); }
        if url_lower.contains("grafana") { return Some("grafana".to_string()); }
        if url_lower.contains("kubernetes") || url_lower.contains("openshift") { return Some("kubernetes".to_string()); }
        if url_lower.contains("jenkins") { return Some("jenkins".to_string()); }
        if url_lower.contains("prometheus") { return Some("prometheus".to_string()); }
        if url_lower.contains("gitlab") { return Some("gitlab".to_string()); }
        if url_lower.contains("vcenter") || url_lower.contains("vmware") { return Some("vmware".to_string()); }
        if url_lower.contains("elastic") || url_lower.contains("kibana") { return Some("elasticsearch".to_string()); }
        if url_lower.contains("zabbix") { return Some("zabbix".to_string()); }
        if url_lower.contains("elastic") { return Some("elasticsearch".to_string()); }
        None
    }
}

impl Default for ErrorDetector {
    fn default() -> Self {
        Self::new()
    }
}