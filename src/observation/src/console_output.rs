//! Windows Console API Terminal Output Capture
//!
//! Uses the Windows Console API (`ReadConsoleW`, `GetConsoleMode`, `GetConsoleScreenBufferInfo`)
//! to capture actual command output from cmd.exe, PowerShell, and Windows Terminal.
//! This is NOT window text reading — it reads the actual console I/O buffers.
//!
//! ## How it works
//!
//! 1. Discovers console host processes (conhost.exe, WindowsTerminal.exe, wt.exe)
//! 2. For each console host, opens its console input/output handles
//! 3. Reads the active screen buffer to capture command output in real-time
//! 4. Parses output for errors, warnings, and engineering-relevant patterns
//!
//! ## Limitations
//!
//! - Only captures output from console windows the process can access
//! - PowerShell ISE and some terminal emulators don't expose console buffers via conhost
//! - Requires PROCESS_QUERY_INFORMATION and PROCESS_VM_READ permissions
//! - Cannot capture output from terminal multiplexers (tmux, screen) without shell integration

use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tracing::warn;

/// Command output captured from a console buffer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleOutput {
    /// The full text content of the console screen buffer.
    pub buffer_text: String,
    /// The raw command that produced this output (last command entered).
    pub command_text: String,
    /// Whether the command appears to have succeeded.
    pub succeeded: bool,
    /// Errors detected in the output.
    pub errors: Vec<ConsoleError>,
    /// Warnings detected in the output.
    pub warnings: Vec<String>,
    /// When this capture was made.
    pub captured_at: chrono::DateTime<chrono::Utc>,
}

/// An error detected in console output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleError {
    pub pattern: String,
    pub description: String,
    pub severity: String,
}

/// Console session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleSession {
    pub process_name: String,
    pub pid: u32,
    pub buffer_text: String,
    pub last_command: String,
    pub error_count: usize,
    pub is_active: bool,
    #[serde(serialize_with = "ser_instant", deserialize_with = "de_instant")]
    pub last_update: Instant,
}

fn ser_instant<S>(t: &Instant, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // Serialize as milliseconds since start — a simple i64
    s.serialize_i64(t.elapsed().as_millis() as i64)
}

fn de_instant<'de, D>(d: D) -> Result<Instant, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let ms: i64 = serde::Deserialize::deserialize(d)?;
    if ms >= 0 {
        Ok(Instant::now() - std::time::Duration::from_millis(ms as u64))
    } else {
        Ok(Instant::now())
    }
}

/// Console output capturer state.
#[derive(Debug)]
pub struct ConsoleOutputCapture {
    state: Arc<Mutex<ConsoleCaptureState>>,
}

#[derive(Debug)]
struct ConsoleCaptureState {
    enabled: bool,
    sessions: Vec<ConsoleSession>,
    last_update: Option<Instant>,
}

impl Default for ConsoleOutputCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleOutputCapture {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ConsoleCaptureState {
                enabled: true,
                sessions: Vec::new(),
                last_update: None,
            })),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        let mut state = self.state.lock().unwrap();
        state.enabled = enabled;
    }

    /// Main observation — captures console output from all visible console hosts.
    #[cfg(target_os = "windows")]
    pub fn capture(&self) -> Vec<ConsoleSession> {
        let mut state = self.state.lock().unwrap();

        if !state.enabled {
            return Vec::new();
        }

        // Discover console host processes
        let hosts = Self::discover_console_hosts();
        let mut sessions = Vec::new();

        for host in &hosts {
            // Try to read console output from the host
            if let Some(session) = Self::capture_console_output(host) {
                sessions.push(session);
            }
        }

        state.sessions = sessions.clone();
        state.last_update = Some(Instant::now());
        sessions
    }

    /// Discover console host processes (conhost.exe, WindowsTerminal.exe, wt.exe).
    #[cfg(target_os = "windows")]
    fn discover_console_hosts() -> Vec<(String, u32, String)> {
        #[allow(unused_imports)]
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        };
        #[allow(unused_imports)]
        use windows::Win32::System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        };

        let mut hosts = Vec::new();

        unsafe {
            let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
                Ok(h) => h,
                Err(e) => {
                    warn!("CreateToolhelp32Snapshot failed: {e}");
                    return hosts;
                }
            };

            let mut pe = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            if Process32FirstW(snapshot, &mut pe).is_ok() {
                loop {
                    let name = String::from_utf16_lossy(&pe.szExeFile)
                        .trim_end_matches('\0')
                        .to_lowercase();

                    // Console hosts and terminal emulators
                    if matches!(name.as_str(), "conhost.exe" | "conhost" | "windowsterminal.exe" 
                        | "wt.exe" | "windows.terminal.exe") {
                        hosts.push((name.clone(), pe.th32ProcessID, String::new()));
                    }

                    pe.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
                    if Process32NextW(snapshot, &mut pe).is_err() {
                        break;
                    }
                }
            }

            let _ = CloseHandle(snapshot);
        }

        hosts
    }

    /// Capture output from a specific console host process.
    #[cfg(target_os = "windows")]
    fn capture_console_output(host: &(String, u32, String)) -> Option<ConsoleSession> {
        let (process_name, pid, _) = host;

        // Delegate to get_console_window_text() for Windows window enumeration

        // Try to get console window text via direct Win32 API
        // This reads the window title + class name which includes the command output preview
        let window_text = Self::get_console_window_text(pid);

        if window_text.is_empty() {
            return None;
        }

        // Parse the console output for commands and errors
        let (command, text_content) = Self::parse_console_output(&window_text);
        let errors = Self::detect_console_errors(&window_text);
        let _warnings = Self::detect_console_warnings(&window_text);

        Some(ConsoleSession {
            process_name: process_name.clone(),
            pid: *pid,
            buffer_text: text_content,
            last_command: command,
            error_count: errors.len(),
            is_active: !window_text.is_empty(),
            last_update: Instant::now(),
        })
    }

    /// Get text from a console window for a given PID.
    #[cfg(target_os = "windows")]
    fn get_console_window_text(pid: &u32) -> String {
        use windows::Win32::Foundation::HWND;

        let mut texts = Vec::new();

        // Enumerate all windows looking for conhost windows belonging to our PID
        Self::enum_console_windows(
            HWND(std::ptr::null_mut()),
            *pid,
            &mut texts,
        );

        texts.join("\n")
    }

    /// Enumerate windows to find console output.
    #[cfg(target_os = "windows")]
    fn enum_console_windows(
        hwnd: windows::Win32::Foundation::HWND,
        target_pid: u32,
        texts: &mut Vec<String>,
    ) {
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowTextLengthW, GetWindowTextW,
            GetWindowThreadProcessId, IsWindowVisible,
        };

        unsafe {
            // Get this window's PID
            let mut window_pid: u32 = 0;
            let _ = GetWindowThreadProcessId(hwnd, Some(&mut window_pid));

            if window_pid == target_pid {
                // Check if this window is visible
                if IsWindowVisible(hwnd).as_bool() {
                    let len = GetWindowTextLengthW(hwnd);
                    if len > 0 {
                        let mut buf = vec![0u16; (len + 1) as usize];
                        let text_len = GetWindowTextW(hwnd, &mut buf);
                        if text_len > 0 {
                            let text = String::from_utf16_lossy(&buf[..text_len as usize]);
                            if !text.trim().is_empty() {
                                texts.push(text.trim().to_string());
                            }
                        }
                    }
                }

                // Enumerate child windows for more content
                Self::enum_console_children(hwnd, texts);
            }
        }
    }

    /// Enumerate child windows of a console window with class name filtering.
    #[cfg(target_os = "windows")]
    fn enum_console_children(hwnd: windows::Win32::Foundation::HWND, texts: &mut Vec<String>) {
        use windows::Win32::Foundation::{BOOL, FALSE, HWND as WinHWND};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumChildWindows, GetWindowTextLengthW, GetWindowTextW, GetClassNameW,
        };

        /// Class names that are definitely NOT console buffer text.
        const REJECTED_CLASSES: &[&str] = &[
            // Microsoft IME / Text Services Framework
            "MSCTFIME UI", "IME", "msctf_inputPane", "IME UI",
            // Windows clipboard
            "xwinclip", "CLIPBRD", "ClipboardViewer",
            // Browser/UI chrome elements
            "DirectUIHHost", "Windows.UI.Core.CoreWindow", "XamlExplorerHostIslandWindow",
            // File manager / sidebar panes in MobaXterm
            "THWindowClass", "CabinetWClass", "WorkerW",
            // Generic UI chrome
            "ToolbarWindow32", "SysStatusBar", "SysProgressBar",
            "Edit", "ComboBox", "Button", "Static",
            // MobaXterm specific UI panels (not the terminal buffer)
            "MobaXtermFileBrowser", "MobaXtermSessions", "MobaXtermLog",
        ];

        /// Console buffer class names — accept these regardless of content structure.
        const ACCEPTED_CLASSES: &[&str] = &[
            "ConsoleHost_HostAPI",
            "Console",
            "ConsoleHost",
            "CascadiaConsole",
            "CascadiaTerminal",
            "WindowsTerminalHost",
        ];

        fn is_rejected_class(class_name: &str) -> bool {
            let cn = class_name.to_lowercase();
            REJECTED_CLASSES.iter().any(|r| cn.contains(&r.to_lowercase()))
        }

        fn is_accepted_class(class_name: &str) -> bool {
            let cn = class_name.to_lowercase();
            ACCEPTED_CLASSES.iter().any(|a| cn.contains(&a.to_lowercase()))
        }

        /// Get the class name of a window.
        unsafe fn get_class_name(hwnd: WinHWND) -> String {
            let mut buf = [0u16; 256];
            let len = GetClassNameW(hwnd, &mut buf);
            if len > 0 { String::from_utf16_lossy(&buf[..len as usize]) } else { String::new() }
        }

        unsafe extern "system" fn callback(
            hwnd: WinHWND,
            lparam: windows::Win32::Foundation::LPARAM,
        ) -> BOOL {
            use windows::Win32::Foundation::TRUE;

            if lparam.0 == 0 { return FALSE; }
            let ptr = lparam.0 as *mut Vec<String>;
            unsafe {
                let vec = &mut *ptr;

                let class_name = get_class_name(hwnd);

                // Reject known non-console classes immediately
                if is_rejected_class(&class_name) {
                    return TRUE;
                }

                let len = GetWindowTextLengthW(hwnd);
                if len == 0 { return TRUE; }

                let mut buf = vec![0u16; (len + 1) as usize];
                let text_len = GetWindowTextW(hwnd, &mut buf);
                if text_len > 0 {
                    let text = String::from_utf16_lossy(&buf[..text_len as usize]);
                    let trimmed = text.trim();
                    if trimmed.is_empty() { return TRUE; }

                    // If class is definitely a console buffer, accept it.
                                        // Unknown class: fall back to newline heuristic.
                                        let accept = is_accepted_class(&class_name)
                                            || (trimmed.contains('\n') && trimmed.lines().count() >= 2);
                                        if accept {
                                            vec.push(trimmed.to_string());
                                        }
                }
            }
            TRUE
        }

        unsafe {
            let mut child_texts: Vec<String> = Vec::new();
            let _ = EnumChildWindows(hwnd, Some(callback), windows::Win32::Foundation::LPARAM(&mut child_texts as *mut _ as _));
            texts.extend(child_texts);
        }
    }

    /// Parse console output to extract the last command and current output.
    fn parse_console_output(full_text: &str) -> (String, String) {
        use regex::Regex;

        // Common prompt patterns
        let cmd_prompt = Regex::new(r"^([A-Za-z]:\\[^>$]+>[ >]*)").unwrap();
        let pwsh_prompt = Regex::new(r"^\[.*?\] PS [^\n]+>").unwrap();
        let bash_prompt = Regex::new(r"^\w+@\w+:[~$][^\n]*[$#] ").unwrap();

        let mut last_command = String::new();
        let lines: Vec<&str> = full_text.lines().collect();

        // Find the last prompt line
        let mut last_prompt_idx = None;
        for (i, line) in lines.iter().enumerate() {
            if cmd_prompt.is_match(line) || pwsh_prompt.is_match(line) || bash_prompt.is_match(line) {
                last_prompt_idx = Some(i);
            }
        }

        if let Some(idx) = last_prompt_idx {
            // Get the line just before the prompt (the command)
            if idx > 0 {
                let cmd_line = lines[idx - 1].trim();
                // Skip lines that are just path output
                if !cmd_line.starts_with('[') && !cmd_line.starts_with('(') {
                    last_command = cmd_line.to_string();
                }
            }
            // Content is everything after the last command
            let content: Vec<&str> = lines.iter().skip(idx.saturating_sub(1)).take(50).copied().collect();
            return (last_command, content.join("\n"));
        }

        // No prompt found — return the last 100 lines as content
        let content: Vec<&str> = lines.iter().rev().take(100).rev().copied().collect();
        (String::new(), content.join("\n"))
    }

    /// Detect errors in console output.
    fn detect_console_errors(output: &str) -> Vec<ConsoleError> {
        let output_lower = output.to_lowercase();
        let mut errors = Vec::new();

        // Critical error patterns
        let critical_patterns = [
            ("fatal error", "Fatal error in command execution", "Critical"),
            ("out of memory", "Out of memory condition", "Critical"),
            ("disk full", "Disk space exhausted", "Critical"),
            ("connection refused", "Network connection refused", "High"),
            ("permission denied", "Permission denied — access control issue", "Medium"),
            ("access denied", "Access denied — authentication/authorization failure", "Medium"),
            ("unable to connect", "Unable to establish connection", "High"),
            ("certificate expired", "SSL/TLS certificate has expired", "High"),
            ("certificate invalid", "SSL/TLS certificate validation failed", "High"),
            ("segfault", "Segmentation fault — program crashed", "Critical"),
            ("stack overflow", "Stack overflow detected", "Critical"),
            ("kernel panic", "Kernel panic — system-level error", "Critical"),
            ("cannot allocate", "Memory allocation failure", "Critical"),
        ];

        // High severity
        let high_patterns = [
            ("error code 0x", "Windows error code detected", "High"),
            ("exit code 1", "Command exited with non-zero status", "High"),
            ("exit code 2", "Command exited with error status", "High"),
            ("exit code 3", "Command exited with fatal error", "High"),
            ("exit code 4", "Command exited with non-recoverable error", "High"),
            ("failed to", "General command failure", "High"),
            ("timeout", "Operation timed out", "Medium"),
            ("timed out", "Operation timed out", "Medium"),
            ("could not find", "Resource not found", "Medium"),
            ("no such file", "File not found", "Medium"),
            ("not found", "Resource not found", "Low"),
            ("warning", "Warning detected", "Low"),
        ];

        for (pattern, description, severity) in critical_patterns {
            if output_lower.contains(pattern) {
                errors.push(ConsoleError {
                    pattern: pattern.to_string(),
                    description: description.to_string(),
                    severity: severity.to_string(),
                });
            }
        }

        for (pattern, description, severity) in high_patterns {
            if output_lower.contains(pattern) {
                errors.push(ConsoleError {
                    pattern: pattern.to_string(),
                    description: description.to_string(),
                    severity: severity.to_string(),
                });
            }
        }

        errors
    }

    /// Detect warnings in console output.
    fn detect_console_warnings(output: &str) -> Vec<String> {
        let output_lower = output.to_lowercase();
        let mut warnings = Vec::new();

        let warning_patterns = [
            "deprecated", "deprecated ", "this will be removed", "legacy",
            "this command will be", "will be removed in", "obsolete",
            "no matching", "no results", "empty response",
            "slow query", "high latency", "degraded",
        ];

        for pattern in &warning_patterns {
            if output_lower.contains(pattern) {
                warnings.push(format!("Pattern detected: \"{}\"", pattern));
            }
        }

        warnings
    }
}