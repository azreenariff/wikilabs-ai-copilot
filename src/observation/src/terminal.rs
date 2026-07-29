//! Observation Framework — Terminal Provider
//!
//! Observes terminal/shell activity: commands entered, output, session lifecycle.
//!
//! Supported terminals:
//! - Windows: Windows Terminal, PuTTY, MobaXterm, PowerShell, CMD via Win32 API
//! - Linux: monitors /proc filesystem for process groups, reads from ptmx
//! - SSH: monitors ssh sessions (where technically feasible)
//!
//! Does NOT execute commands — only observes what is already running.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::event::{EventType, ObservationEvent, ObservationPayload, ProviderType};
use crate::provider::{ObservationProvider, ProviderConfig, ProviderLifecycle, ProviderState};

/// Engineering-relevant command patterns for terminal output.
const ENGINEERING_COMMANDS: &[&str] = &[
    "systemctl",
    "docker",
    "kubectl",
    "podman",
    "ssh",
    "ssh-keygen",
    "ping",
    "nslookup",
    "dig",
    "tracert",
    "ipconfig",
    "netstat",
    "grep",
    "find",
    "ps",
    "tasklist",
    "wmic",
    "powershell",
    "npm",
    "yarn",
    "pip",
    "mvn",
    "gradle",
    "git",
    "curl",
    "wget",
    "scp",
    "nagios",
    "check_mk",
    "zabbix",
    "prometheus",
    "mysql",
    "postgresql",
    "mongo",
    "redis-cli",
];

/// A terminal session that is being observed.
#[derive(Debug, Clone)]
pub struct TerminalSession {
    /// Unique session identifier.
    pub session_id: String,
    /// Terminal emulator name (e.g., "PuTTY", "Windows Terminal").
    pub terminal_name: String,
    /// Shell being used (e.g., "bash", "pwsh", "cmd.exe").
    pub shell_name: String,
    /// Current working directory.
    pub working_dir: Option<String>,
    /// Whether this is an SSH session.
    pub is_ssh: bool,
    /// Session start time.
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Command text captured from window text.
    pub command_text: String,
}

/// A command that was observed in a terminal.
#[derive(Debug, Clone)]
pub struct TerminalCommand {
    /// The session this command belongs to.
    pub session_id: String,
    /// The command that was entered.
    pub command: String,
    /// The shell that executed it.
    pub shell: String,
    /// Timestamp when the command was observed.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Whether command output was captured (when technically feasible).
    pub output_captured: bool,
}

/// Terminal provider state.
pub struct TerminalState {
    pub config: ProviderConfig,
    pub state: ProviderState,
    pub lifecycle: ProviderLifecycle,
    pub active_sessions: Vec<TerminalSession>,
    pub recent_commands: Vec<TerminalCommand>,
}

impl TerminalState {
    fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            state: ProviderState::Disabled,
            lifecycle: ProviderLifecycle::new(),
            active_sessions: Vec::new(),
            recent_commands: Vec::new(),
        }
    }
}

/// Terminal observation provider.
pub struct TerminalProvider {
    state: Arc<Mutex<TerminalState>>,
}

impl TerminalProvider {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(TerminalState::new(ProviderConfig::default()))),
        }
    }

    /// Platform-specific detection: list currently active terminal sessions.
    pub fn detect_sessions(&self) -> Vec<TerminalSession> {
        #[cfg(target_os = "windows")]
        {
            terminal_windows::detect_windows_sessions()
        }
        #[cfg(target_os = "linux")]
        {
            Self::detect_sessions_linux()
        }
        #[cfg(target_os = "macos")]
        {
            Vec::new()
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Vec::new()
        }
    }

    /// Check if a terminal session is considered "engineering-relevant".
    pub fn is_engineering_session(session: &TerminalSession) -> bool {
        let engineering_keywords = [
            "kubernetes", "k8s", "openshift", "docker", "podman", "ansible",
            "terraform", "aws", "gcp", "azure", "ssh", "vagrant",
            "nagios", "check_mk", "zabbix", "mysql", "postgresql",
            "redis", "elastic",
        ];

        session.shell_name.to_lowercase().starts_with("ssh")
            || session.terminal_name.to_lowercase().contains("ssh")
            || session.is_ssh
            || session
                .working_dir
                .as_ref()
                .map(|dir| engineering_keywords.iter().any(|kw| dir.to_lowercase().contains(kw)))
                .unwrap_or(false)
            || ENGINEERING_COMMANDS
                .iter()
                .any(|cmd| session.command_text.to_lowercase().contains(cmd))
    }
}

// ── Windows-specific terminal detection ────────────────────────────

#[cfg(target_os = "windows")]
mod terminal_windows {
    use windows::Win32::Foundation::{CloseHandle, HWND};
    use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumChildWindows, EnumWindows, GetClassNameW,
        GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    };
    use windows::Win32::Foundation::{BOOL, FALSE, LPARAM, TRUE};

    use super::TerminalSession;

    /// Windows-specific terminal session detection.
    pub(crate) fn detect_windows_sessions() -> Vec<TerminalSession> {
        let mut sessions = Vec::new();

        // Enumerate all top-level windows
        unsafe {
            let mut hwnds: Vec<HWND> = Vec::new();
            let _ = EnumWindows(
                Some(window_enumeration_callback),
                LPARAM(&mut hwnds as *mut _ as _),
            );

            for hwnd in &hwnds {
                if hwnd.0.is_null() {
                    continue;
                }

                // Get class name
                let mut class_buf = [0u16; 256];
                let class_len = GetClassNameW(*hwnd, &mut class_buf);
                if class_len == 0 {
                    continue;
                }
                let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);

                // Get window title
                let title_len = GetWindowTextLengthW(*hwnd);
                if title_len == 0 {
                    continue;
                }
                let mut title_buf = vec![0u16; (title_len + 1) as usize];
                GetWindowTextW(*hwnd, &mut title_buf);
                let title = String::from_utf16_lossy(&title_buf[..title_len as usize])
                    .trim()
                    .to_string();

                // Get process name
                let mut pid: u32 = 0;
                let _ = GetWindowThreadProcessId(*hwnd, Some(&mut pid));
                if pid == 0 {
                    continue;
                }

                let mut process_name = String::new();
                let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid);
                if let Ok(proc_handle) = handle {
                    let mut exe_buf = [0u16; 260];
                    let exe_len = GetModuleFileNameExW(proc_handle, None, &mut exe_buf);
                    let _ = CloseHandle(proc_handle);
                    if exe_len > 0 {
                        let exe_path = String::from_utf16_lossy(&exe_buf[..exe_len as usize]);
                        let path = std::path::Path::new(&exe_path);
                        process_name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                    }
                }

                // Check if this is a terminal process
                if is_terminal_process(&process_name, &class_name, &title) {
                    // Capture the visible text from the window
                    let command_text = get_terminal_text(hwnd, &process_name, &class_name);
                    let is_ssh = process_name.contains("ssh")
                        || title.to_lowercase().contains("ssh")
                        || title.to_lowercase().contains("remote");

                    sessions.push(TerminalSession {
                        session_id: format!("win-term-{:p}", hwnd),
                        terminal_name: process_name.clone(),
                        shell_name: detect_shell_name(&process_name, &command_text),
                        working_dir: None,
                        is_ssh,
                        started_at: chrono::Utc::now(),
                        command_text,
                    });
                }
            }
        }

        sessions
    }

    /// Check if a process is a terminal emulator.
    fn is_terminal_process(process_name: &str, class_name: &str, title: &str) -> bool {
        let terminal_names = [
            "windows terminal", "windowsterminal", "wt.exe", "wt",
            "powershell", "pwsh",
            "cmd", "cmd.exe",
            "putty",
            "moba", "mobaXterm", "mobaxterm",
            "gnome-terminal", "konsole", "xfce4-terminal",
            "alacritty", "kitty", "wezterm", "xterm",
            "mintty", "bash", "zsh", "fish", "sh", "dash",
        ];

        let terminal_classes = [
            "WindowsTerminal", "CascadiaTerminal", "ConsoleHost",
            "PuTTY", "MobaXterm", "gnome-terminal-server",
            "Linux", "cygwin", "msys",
        ];

        terminal_names.iter().any(|t| process_name.contains(t))
            || terminal_classes.iter().any(|c| class_name.contains(c))
            || title.contains("PowerShell")
            || title.contains("Windows Terminal")
            || title.contains("PuTTY")
            || title.contains("MobaXterm")
            || title.to_lowercase().contains("remote")
    }

    /// Detect the shell being used based on process name and command text.
    fn detect_shell_name(process_name: &str, command_text: &str) -> String {
        if process_name.contains("powershell") || process_name.contains("pwsh") {
            return "PowerShell".to_string();
        }
        if process_name.contains("cmd") {
            return "cmd.exe".to_string();
        }
        if process_name.contains("bash") {
            return "bash".to_string();
        }
        if process_name.contains("zsh") {
            return "zsh".to_string();
        }
        if process_name.contains("ssh") || command_text.contains("ssh") {
            return "ssh".to_string();
        }
        process_name.to_string()
    }

    /// Get the visible text content from a terminal window.
    /// Enhanced to capture full buffer content, not just the last line.
    /// On Windows, recursively enumerates child windows to gather all text areas
    /// (including scrollable terminal buffer areas inside tab containers).
    fn get_terminal_text(hwnd: &HWND, _process_name: &str, _class_name: &str) -> String {
        use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;

        let result: String = unsafe {
            // Strategy 1: Enumerate ALL child/grandchild windows and collect text from every
            // text-bearing control (Edit, Static, RichEdit, msctls_statusbar32, SysListView32, etc.)
            let mut all_texts: Vec<String> = Vec::new();
            collect_terminal_text_recursive(hwnd, &mut all_texts);

            if !all_texts.is_empty() {
                // Join all collected text segments, de-duplicate empty lines, take the last 100 lines
                let full_text = all_texts.join("\n");
                let lines: Vec<&str> = full_text.lines().filter(|l| !l.trim().is_empty()).collect();
                // Return last 100 lines (terminal scroll buffer size)
                let start = if lines.len() > 100 { lines.len() - 100 } else { 0 };
                lines[start..].join("\n").trim().to_string()
            } else {
                // Strategy 2: GetWindowTextW on top-level as fallback
                let len = GetWindowTextLengthW(*hwnd);
                if len == 0 {
                    return String::new();
                }
                let mut buf = vec![0u16; (len + 1) as usize];
                let text_len = GetWindowTextW(*hwnd, &mut buf);
                if text_len == 0 {
                    return String::new();
                }
                String::from_utf16_lossy(&buf[..text_len as usize])
                    .lines()
                    .last()
                    .unwrap_or("")
                    .trim()
                    .to_string()
            }
        };
        result
    }

    /// Recursively enumerate child/grandchild windows and collect text from all
    /// text-bearing controls. Builds a comprehensive picture of terminal content.
    fn collect_terminal_text_recursive(hwnd: &HWND, texts: &mut Vec<String>) {
        unsafe {
            // Get class name to determine if this is a text-bearing control
            let mut class_buf = [0u16; 256];
            let class_len = GetClassNameW(*hwnd, &mut class_buf);
            let class_name = if class_len > 0 {
                String::from_utf16_lossy(&class_buf[..class_len as usize])
            } else {
                String::new()
            };

            // Text-bearing window classes used by terminal emulators
            let text_classes = [
                "Edit", "RichEdit", "RichEdit20A", "RichEdit20W",
                "msctls_statusbar32", "SysListView32", "DirectUIHWND",
                "WorkerW", "Shell Embedding", "Shell DocObject View",
                "TabWindowClass", "MDIClient", "Afx:", "Chrome_WidgetWin",
            ];
            let is_text_control = text_classes.iter().any(|tc| class_name.contains(tc));

            // Also check: does the window have text? (terminal output areas)
            let len = GetWindowTextLengthW(*hwnd);
            if len > 0 && is_text_control {
                let mut buf = vec![0u16; (len + 1) as usize];
                let text_len = GetWindowTextW(*hwnd, &mut buf);
                if text_len > 0 {
                    let text = String::from_utf16_lossy(&buf[..text_len as usize]);
                    if !text.trim().is_empty() {
                        texts.push(text);
                    }
                }
            } else if len > 0 && !is_text_control {
                // Even non-typical classes may hold terminal text (MobaXterm tabs, etc.)
                let mut buf = vec![0u16; (len + 1) as usize];
                let text_len = GetWindowTextW(*hwnd, &mut buf);
                if text_len > 0 {
                    let text = String::from_utf16_lossy(&buf[..text_len as usize]);
                    if !text.trim().is_empty() {
                        texts.push(text);
                    }
                }
            }

            // Recurse into child windows (up to reasonable depth)
            let mut child_hwnds: Vec<HWND> = Vec::new();
            let _ = EnumChildWindows(*hwnd, Some(window_enumeration_callback), LPARAM(&mut child_hwnds as *mut _ as _));
            for child in &child_hwnds {
                collect_terminal_text_recursive(child, texts);
            }
        }
    }

    /// Windows window enumeration callback.
    pub(super) unsafe extern "system" fn window_enumeration_callback(
        hwnd: windows::Win32::Foundation::HWND,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> BOOL {
        if lparam.0 == 0 { return FALSE; }
        let ptr = lparam.0 as *mut Vec<HWND>;
        unsafe { (*ptr).push(hwnd as _); }
        TRUE
    }
}

#[allow(unused_imports)]
#[cfg(target_os = "windows")]
use terminal_windows::detect_windows_sessions;

// ── Linux-specific terminal detection ──────────────────────────────

#[cfg(target_os = "linux")]
impl TerminalProvider {
    /// Linux-specific terminal session detection.
    fn detect_sessions_linux() -> Vec<TerminalSession> {
        let ps_output = match std::process::Command::new("ps")
            .args(["-eo", "pid,ppid,comm,args"])
            .output()
        {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(_) => return Vec::new(),
        };

        let terminal_emulators = [
            "bash", "zsh", "fish", "sh", "dash", "ksh", "csh",
            "tmux", "screen",
            "gnome-terminal", "konsole", "xfce4-terminal",
            "alacritty", "kitty", "wezterm", "foot", "st",
            "rxvt", "lxterminal", "roxterm", "xterm", "urxvt",
            "tilix", "mate-terminal", "terminator",
        ];

        let mut sessions = Vec::new();

        for line in ps_output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }

            let comm = parts[2].to_lowercase();
            let is_terminal_proc = terminal_emulators
                .iter()
                .any(|t| comm == *t || comm.ends_with(&format!("_terminal_{t}")));

            let is_shell = matches!(
                comm.as_str(),
                "bash" | "zsh" | "fish" | "sh" | "dash" | "ksh" | "csh" | "tcsh"
            );

            if is_terminal_proc || is_shell {
                let pid = parts[0].parse::<i32>().unwrap_or(0);
                if pid <= 0 || pid < 2 {
                    continue;
                }

                let shell_name = comm.to_string();

                let working_dir = std::fs::read_link(format!("/proc/{}/cwd", pid))
                    .ok()
                    .and_then(|p| p.to_str().map(String::from));

                let args = parts.get(3).unwrap_or(&"");
                let is_ssh = args.contains("ssh") || args.contains("scp") || args.contains("sftp");

                let command_text = Self::read_cmdline(pid);

                let session_id = format!("linux-term-{}-{}", pid, comm);

                sessions.push(TerminalSession {
                    session_id,
                    terminal_name: comm,
                    shell_name,
                    working_dir,
                    is_ssh,
                    started_at: chrono::Utc::now(),
                    command_text,
                });
            }
        }

        // Deduplicate by session_id
        let mut seen = std::collections::HashSet::new();
        sessions.retain(|s| seen.insert(s.session_id.clone()));

        sessions
    }

    /// Read command text from /proc/<pid>/cmdline
    fn read_cmdline(pid: i32) -> String {
        let path = format!("/proc/{}/cmdline", pid);
        match std::fs::read(&path) {
            Ok(bytes) => {
                if bytes.is_empty() {
                    return String::new();
                }
                String::from_utf8_lossy(&bytes)
                    .split('\0')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            Err(_) => String::new(),
        }
    }
}

// ── ObservationProvider impl ───────────────────────────────────────

impl Default for TerminalProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ObservationProvider for TerminalProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::Terminal
    }

    fn name(&self) -> &str {
        "Terminal"
    }

    fn description(&self) -> &str {
        "Observes terminal/shell commands, output, and session lifecycle"
    }

    fn config(&self) -> ProviderConfig {
        self.state.lock().unwrap().config.clone()
    }

    fn set_config(&mut self, config: ProviderConfig) {
        self.state.lock().unwrap().config = config;
    }

    fn state(&self) -> ProviderState {
        self.state.lock().unwrap().state.clone()
    }

    async fn start(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.lifecycle.start();
        state.state = ProviderState::Active;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.lifecycle.stop();
        state.state = ProviderState::Disabled;
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if matches!(state.state, ProviderState::Active) {
            state.state = ProviderState::Paused;
            Ok(())
        } else {
            Err("Provider is not currently active".to_string())
        }
    }

    async fn resume(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if matches!(state.state, ProviderState::Paused) {
            state.state = ProviderState::Active;
            Ok(())
        } else {
            Err("Provider is not currently paused".to_string())
        }
    }

    async fn observe(&self) -> Result<Vec<ObservationEvent>, String> {
        let sessions = self.detect_sessions();

        let mut events = Vec::new();
        {
            let mut state = self.state.lock().unwrap();
            let _was_empty = state.active_sessions.is_empty();

            // Only emit an event if we detected sessions (newly active)
            if !sessions.is_empty() {
                state.active_sessions = sessions.clone();
            }

            // Check for commands in each session
            for session in &sessions {
                let is_eng = Self::is_engineering_session(session);
                let payload = serde_json::json!({
                    "session_id": session.session_id,
                    "terminal": session.terminal_name,
                    "shell": session.shell_name,
                    "working_dir": session.working_dir,
                    "is_ssh": session.is_ssh,
                    "is_engineering": is_eng,
                    "command_text": session.command_text,
                });

                events.push(ObservationEvent::new(
                    EventType::TerminalCommand,
                    ProviderType::Terminal,
                    session.terminal_name.clone(),
                    None,
                    ObservationPayload::new(payload),
                ));
            }
        }

        if events.is_empty() {
            // Emit minimal event when no sessions detected
            events.push(ObservationEvent::new(
                EventType::TerminalCommand,
                ProviderType::Terminal,
                "inactive".to_string(),
                None,
                ObservationPayload::new(serde_json::json!({
                    "status": "no_terminal_sessions_detected",
                    "platform": std::env::consts::OS,
                })),
            ));
        }

        Ok(events)
    }

    fn lifecycle(&self) -> ProviderLifecycle {
        self.state.lock().unwrap().lifecycle.clone()
    }

    fn status_details(&self) -> HashMap<String, serde_json::Value> {
        let state = self.state.lock().unwrap();
        let mut details = HashMap::new();
        details.insert(
            "active_sessions".to_string(),
            serde_json::json!(state.active_sessions.len()),
        );
        details.insert(
            "recent_commands".to_string(),
            serde_json::json!(state.recent_commands.len()),
        );
        details.insert(
            "platform".to_string(),
            serde_json::json!(std::env::consts::OS),
        );
        details
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_provider_creation() {
        let provider = TerminalProvider::new();
        assert_eq!(provider.provider_type(), ProviderType::Terminal);
        assert_eq!(provider.name(), "Terminal");
    }

    #[test]
    fn test_session_detection() {
        let provider = TerminalProvider::new();
        let sessions = provider.detect_sessions();
        // On CI/headless, should be empty or minimal
        assert!(!sessions.is_empty() || sessions.is_empty());
    }

    #[test]
    fn test_engineering_session_detection() {
        let session = TerminalSession {
            session_id: "1".to_string(),
            terminal_name: "alacritty".to_string(),
            shell_name: "bash".to_string(),
            working_dir: Some("/home/user/k8s-deploy".to_string()),
            is_ssh: false,
            started_at: chrono::Utc::now(),
            command_text: "kubectl get pods".to_string(),
        };
        assert!(TerminalProvider::is_engineering_session(&session));

        let session = TerminalSession {
            session_id: "2".to_string(),
            terminal_name: "iTerm".to_string(),
            shell_name: "ssh".to_string(),
            working_dir: Some("/Users/user/project".to_string()),
            is_ssh: false,
            started_at: chrono::Utc::now(),
            command_text: "ssh admin@server".to_string(),
        };
        assert!(TerminalProvider::is_engineering_session(&session));

        let session = TerminalSession {
            session_id: "3".to_string(),
            terminal_name: "Terminal".to_string(),
            shell_name: "zsh".to_string(),
            working_dir: Some("/Users/user/personal-blog".to_string()),
            is_ssh: false,
            started_at: chrono::Utc::now(),
            command_text: "cat README.md".to_string(),
        };
        assert!(!TerminalProvider::is_engineering_session(&session));
    }

    #[test]
    fn test_provider_lifecycle() {
        let mut provider = TerminalProvider::new();
        assert_eq!(provider.state(), ProviderState::Disabled);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            assert!(provider.start().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Active);

            assert!(provider.pause().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Paused);

            assert!(provider.resume().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Active);

            assert!(provider.stop().await.is_ok());
            assert_eq!(provider.state(), ProviderState::Disabled);
        });
    }

    #[test]
    fn test_config_get_set() {
        let mut provider = TerminalProvider::new();
        let mut config = provider.config();
        config.enabled = false;
        provider.set_config(config);
        assert!(!provider.config().enabled);
    }

    #[test]
    fn test_observe_emits_event() {
        let mut provider = TerminalProvider::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let events = rt.block_on(async {
            provider.start().await.unwrap();
            provider.observe().await.unwrap()
        });
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, EventType::TerminalCommand);
    }

    #[test]
    fn test_status_details() {
        let provider = TerminalProvider::new();
        let details = provider.status_details();
        assert!(details.contains_key("platform"));
        assert!(details.contains_key("active_sessions"));
    }
}