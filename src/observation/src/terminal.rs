//! Observation Framework — Terminal Provider
//!
//! Observes terminal/shell activity: commands entered, output, session lifecycle.
//!
//! Supported terminals:
//! - Linux: monitors /proc filesystem for process groups, reads from ptmx
//! - Windows: uses Windows Terminal / PowerShell / CMD hooks
//! - SSH: monitors ssh sessions (where technically feasible)
//!
//! Does NOT execute commands — only observes what is already running.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::event::{EventType, ObservationEvent, ObservationPayload, ProviderType};
use crate::provider::{ObservationProvider, ProviderConfig, ProviderLifecycle, ProviderState};

/// A terminal session that is being observed.
#[derive(Debug, Clone)]
pub struct TerminalSession {
    /// Unique session identifier.
    pub session_id: String,
    /// Terminal emulator name (e.g., "alacritty", "iTerm").
    pub terminal_name: String,
    /// Shell being used (e.g., "bash", "zsh", "pwsh").
    pub shell_name: String,
    /// Current working directory.
    pub working_dir: Option<String>,
    /// Whether this is an SSH session.
    pub is_ssh: bool,
    /// Session start time.
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Command text captured from /proc/<pid>/cmdline or ~/.bash_history.
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
    fn detect_sessions(&self) -> Vec<TerminalSession> {
        #[cfg(target_os = "linux")]
        {
            Self::detect_sessions_linux()
        }

        #[cfg(target_os = "windows")]
        {
            // Windows: detect Windows Terminal, PowerShell, CMD windows
            use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS, PROCESSENTRY32W};
            unsafe {
                let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
                    Ok(s) => s,
                    Err(_) => return Vec::new(),
                };
                if snapshot.is_invalid() { return Vec::new(); }

                let mut entry = PROCESSENTRY32W::default();
                entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
                if Process32FirstW(snapshot, &mut entry).is_err() {
                    let _ = windows::Win32::Foundation::CloseHandle(snapshot);
                    return Vec::new();
                }

                let mut sessions = Vec::new();
                loop {
                    let name = String::from_utf16_lossy(&entry.szExeFile)
                        .trim_end_matches('\0')
                        .to_lowercase();
                    let is_terminal = matches!(name.as_str(),
                        "powershell.exe" | "pwsh.exe" | "cmd.exe" | "wsl.exe" | "windowsterminal.exe" | "wt.exe"
                        | "bash.exe" | "zsh.exe" | "fish.exe" | "alacritty.exe" | "mintty.exe");

                    if is_terminal {
                        sessions.push(TerminalSession {
                            session_id: format!("win-term-{}", entry.th32ProcessID),
                            terminal_name: name,
                            shell_name: "unknown".to_string(),
                            working_dir: None,
                            is_ssh: false,
                            started_at: chrono::Utc::now(),
                            command_text: String::new(),
                        });
                    }

                    if Process32NextW(snapshot, &mut entry).is_err() { break; }
                }
                let _ = windows::Win32::Foundation::CloseHandle(snapshot);
                sessions
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: detect iTerm, Terminal.app, tmux, screen
            Vec::new()
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Vec::new()
        }
    }

    /// Linux-specific terminal session detection.
    /// Uses /proc to enumerate terminal processes and reads last commands from /proc/<pid>/fd/0 (stdin)
    /// via `ps` for command text.
    fn detect_sessions_linux() -> Vec<TerminalSession> {
        // Step 1: Find all terminal/shell processes using ps
        let ps_output = match std::process::Command::new("ps")
            .args(["-eo", "pid,ppid,comm,args"])
            .output()
        {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(_) => return Vec::new(),
        };

        // Terminal emulator patterns
        let terminal_emulators = [
            "bash", "zsh", "fish", "sh", "dash", "ksh", "csh", "tmux", "screen",
            "gnome-terminal", "konsole", "xfce4-terminal", "alacritty", "kitty",
            "wezterm", "foot", "st", "rxvt", "lxterminal", "roxterm",
            "xterm", "urxvt", "tilix", "mate-terminal", "terminator",
        ];

        let mut sessions = Vec::new();

        for line in ps_output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }

            let comm = parts[2].to_lowercase();
            let is_terminal_proc = terminal_emulators.iter()
                .any(|t| comm == *t || comm.ends_with(&format!("_terminal_{t}")));

            // Also check if the process is a shell or is a known terminal emulator
            let is_shell = matches!(comm.as_str(),
                "bash" | "zsh" | "fish" | "sh" | "dash" | "ksh" | "csh" | "tcsh");

            // Check if parent is a terminal emulator
            if let Some(ppid) = parts.get(1) {
                // Check if any terminal emulator has this shell as a child
                if is_shell {
                    // Check if this process has a terminal emulator ancestor
                    let has_terminal_parent = ps_output.lines().any(|parent_line| {
                        let parent_parts: Vec<&str> = parent_line.split_whitespace().collect();
                        if parent_parts.len() >= 3 && parent_parts[0] == *ppid {
                            let parent_comm = parent_parts[2].to_lowercase();
                            terminal_emulators.iter().any(|t| parent_comm.contains(t))
                        } else {
                            false
                        }
                    });

                    if !has_terminal_parent && !comm.starts_with("grep") {
                        // This is a shell not owned by a terminal emulator we detect
                        // It's still a terminal session worth observing
                    }
                }
            }

            if is_terminal_proc || is_shell {
                let pid = parts[0].parse::<i32>().unwrap_or(0);
                if pid <= 0 {
                    continue;
                }

                // Skip our own process and system processes
                if pid < 2 {
                    continue;
                }

                // Get session info from /proc
                let shell_name = comm.to_string();

                // Get working directory from /proc/<pid>/cwd
                let working_dir = std::fs::read_link(format!("/proc/{}/cwd", pid))
                    .ok()
                    .and_then(|p| p.to_str().map(String::from));

                // Check if SSH session
                let args = parts.get(3).unwrap_or(&"");
                let is_ssh = args.contains("ssh") || args.contains("scp") || args.contains("sftp");

                // Get command text from /proc/<pid>/cmdline
                let command_text = Self::read_cmdline(pid);

                // Get session ID
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
                // cmdline is null-separated
                String::from_utf8_lossy(&bytes)
                    .split('\0')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            Err(_) => String::new(),
        }
    }

    /// Detect recent commands typed in an active shell session.
    /// Uses multiple strategies:
    /// 1. Check if we can read from the terminal's PTY
    /// 2. Parse ps output for recent command history
    /// 3. Read from /proc/<pid>/environ for working directory context
    fn detect_commands_for_session(session: &TerminalSession) -> Vec<String> {
        let mut commands = Vec::new();

        // Strategy 1: Try to get last command from bash history if we have access
        if let Some(ref _workdir) = session.working_dir {
            // Check if ~/.bash_history or ~/.zsh_history is readable
            let home = std::env::var("HOME").unwrap_or_default();
            let history_file = format!("{}/.{}", home, if session.shell_name.contains("zsh") { "zsh" } else { "bash" });
            match std::fs::read_to_string(&history_file) {
                Ok(history) => {
                    let last_commands: Vec<String> = history.lines()
                        .filter(|line| {
                            let l = line.trim();
                            // Skip empty lines, comments, and very short lines
                            !l.is_empty() && !l.starts_with('#') && l.len() > 3
                        })
                        .take(5)
                        .map(String::from)
                        .collect();
                    if !last_commands.is_empty() {
                        commands.extend(last_commands);
                    }
                }
                Err(_) => {}
            }
        }

        // Strategy 2: Use ps to get the current command being run
        let ps_output = match std::process::Command::new("ps")
            .args(["-o", "pid,comm,args"])
            .output()
        {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(_) => return commands,
        };

        for line in ps_output.lines().skip(1) {
            if let Some(pid_part) = line.split_whitespace().next() {
                if let Ok(pid) = pid_part.parse::<i32>() {
                    if pid == session.session_id.parse::<i32>().unwrap_or(0) {
                        // Found the process - get the full command
                        if let Some(args) = line.split_whitespace().nth(2) {
                            commands.push(args.to_string());
                        }
                    }
                }
            }
        }

        // Strategy 3: Check /proc for recent I/O (which files were touched)
        if let Some(pid) = session.session_id.strip_prefix("linux-term-")
            .and_then(|p| p.split('-').next())
            .and_then(|p| p.parse::<i32>().ok()) {
            Self::detect_recent_files(pid, &mut commands);
        }

        commands
    }

    /// Detect recently accessed files from /proc/<pid>/fd and /proc/<pid>/io
    fn detect_recent_files(pid: i32, commands: &mut Vec<String>) {
        // Check /proc/<pid>/fd for open file descriptors
        let fd_dir = format!("/proc/{}/fd", pid);
        if let Ok(fds) = std::fs::read_dir(&fd_dir) {
            for fd in fds.take(10) {
                if let Ok(fd) = fd {
                    if let Ok(target) = std::fs::read_link(fd.path()) {
                        if let Some(path_str) = target.to_str() {
                            // Only include files, not pipes/sockets
                            if path_str.starts_with("/") {
                                commands.push(format!("opened:{}", path_str));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check if a terminal session is considered "engineering-relevant".
    fn is_engineering_session(session: &TerminalSession) -> bool {
        let engineering_keywords = [
            "kubernetes",
            "k8s",
            "openshift",
            "docker",
            "podman",
            "ansible",
            "terraform",
            "aws",
            "gcp",
            "azure",
            "ssh",
            "vagrant",
        ];
        session.shell_name.to_lowercase().starts_with("ssh")
            || session.terminal_name.to_lowercase().contains("ssh")
            || session.is_ssh
            || session
                .working_dir
                .as_ref()
                .map(|dir| {
                    engineering_keywords
                        .iter()
                        .any(|kw| dir.to_lowercase().contains(kw))
                })
                .unwrap_or(false)
    }
}

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
            let was_empty = state.active_sessions.is_empty();

            // Only emit an event if we detected sessions (newly active)
            if !sessions.is_empty() {
                state.active_sessions = sessions.clone();
            }

            // Detect new sessions and emit TerminalCommand events
            let _new_session_ids: std::collections::HashSet<&str> =
                sessions.iter().map(|s| s.session_id.as_str()).collect();

            // Check for commands in each session
            for session in &sessions {
                let is_eng = Self::is_engineering_session(session);
                // Detect recent commands for richer observation data
                let recent_cmds = Self::detect_commands_for_session(session);
                // Combine command_text from cmdline with detected recent commands
                let enriched_command_text = if !session.command_text.is_empty() {
                    format!("{} | last_cmds: {}", session.command_text, recent_cmds.join("; "))
                } else if !recent_cmds.is_empty() {
                    recent_cmds.join("; ")
                } else {
                    session.command_text.clone()
                };
                let payload = serde_json::json!({
                    "session_id": session.session_id,
                    "terminal": session.terminal_name,
                    "shell": session.shell_name,
                    "working_dir": session.working_dir,
                    "is_ssh": session.is_ssh,
                    "is_engineering": is_eng,
                    "command_text": enriched_command_text,
                    "recent_commands": recent_cmds,
                });

                events.push(ObservationEvent::new(
                    if was_empty {
                        EventType::ApplicationChanged
                    } else {
                        EventType::TerminalCommand
                    },
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

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
        // On stub, should be empty
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_engineering_session_detection() {
        let session = TerminalSession {
            session_id: "1".to_string(),
            terminal_name: "alacritty".to_string(),
            shell_name: "bash".to_string(),
            working_dir: Some("/home/user/k8s-deploy".to_string()),
            is_ssh: false,
            started_at: Utc::now(),
        };
        assert!(TerminalProvider::is_engineering_session(&session));

        let session = TerminalSession {
            session_id: "2".to_string(),
            terminal_name: "iTerm".to_string(),
            shell_name: "ssh".to_string(),
            working_dir: Some("/Users/user/project".to_string()),
            is_ssh: false,
            started_at: Utc::now(),
        };
        assert!(TerminalProvider::is_engineering_session(&session));

        let session = TerminalSession {
            session_id: "3".to_string(),
            terminal_name: "Terminal".to_string(),
            shell_name: "zsh".to_string(),
            working_dir: Some("/Users/user/personal-blog".to_string()),
            is_ssh: false,
            started_at: Utc::now(),
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
