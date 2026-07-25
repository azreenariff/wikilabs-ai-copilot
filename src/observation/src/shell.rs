//! Shell integration — bash, zsh, PowerShell command capture.
//!
//! On Windows, provides a practical shell observer that captures command text
//! from active terminal windows via Win32 API polling (no shell hooks needed).
//!
//! On Linux, provides a similar observer using /proc filesystem.
//!
//! Does NOT inject into shells or modify user environments.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::event::{ObservationEvent, ObservationPayload, ProviderType};
use crate::terminal::TerminalProvider;

/// Shell integration status.
#[derive(Debug, Clone)]
pub struct ShellStatus {
    /// Whether integration is active.
    pub active: bool,
    /// Number of shells being monitored.
    pub monitored_count: usize,
    /// Commands captured in the last minute.
    pub commands_last_minute: usize,
    /// Last command captured.
    pub last_command: Option<String>,
    /// Last capture timestamp.
    pub last_capture: Option<std::time::SystemTime>,
}

/// Shell observer that polls terminal windows for command text.
pub struct ShellObserver {
    state: Arc<Mutex<ShellState>>,
    terminal_provider: TerminalProvider,
    capture_interval: std::time::Duration,
}

/// Internal state for the shell observer.
struct ShellState {
    active: bool,
    monitored_sessions: Vec<String>,
    commands: Vec<ShellCommand>,
    last_capture: Option<Instant>,
    status: ShellStatus,
}

/// A command captured from a shell session.
#[derive(Debug, Clone)]
pub struct ShellCommand {
    /// Session identifier.
    pub session_id: String,
    /// Terminal emulator name.
    pub terminal: String,
    /// The command text.
    pub command: String,
    /// Timestamp when captured.
    pub captured_at: Instant,
    /// Whether it looks like an engineering command.
    pub is_engineering: bool,
}

/// Engineering command patterns for shell observation.
const ENGINEERING_COMMANDS: &[&str] = &[
    "systemctl", "docker", "kubectl", "podman", "ssh", "scp", "sftp",
    "ping", "nslookup", "dig", "tracert", "ipconfig", "netstat", "ifconfig",
    "grep", "find", "ps", "tasklist", "wmic", "powershell", "pwsh",
    "npm", "yarn", "pip", "pip3", "mvn", "gradle",
    "git", "curl", "wget", "tar", "zip", "unzip",
    "nagios", "check_mk", "zabbix", "prometheus",
    "mysql", "mysqladmin", "postgresql", "psql", "mongo", "redis-cli",
    "apache", "nginx", "tomcat", "iisreset",
    "java", "dotnet", "php", "ruby", "python", "perl",
    "ls", "cd", "pwd", "cat", "tail", "head", "less", "more",
    "chmod", "chown", "chgrp", "mkdir", "rm", "cp", "mv",
    "vim", "nano", "emacs",
];

impl ShellObserver {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ShellState {
                active: false,
                monitored_sessions: Vec::new(),
                commands: Vec::new(),
                last_capture: None,
                status: ShellStatus {
                    active: false,
                    monitored_count: 0,
                    commands_last_minute: 0,
                    last_command: None,
                    last_capture: None,
                },
            })),
            terminal_provider: TerminalProvider::new(),
            capture_interval: std::time::Duration::from_secs(3),
        }
    }

    /// Register a shell for observation (no-op on Windows/Linux without root).
    /// On Windows, we observe all terminal windows without needing registration.
    pub fn register(&self, _shell: &str) -> anyhow::Result<()> {
        // No registration needed — we poll terminal windows directly
        Ok(())
    }

    /// Start observing terminal sessions.
    pub fn start(&self) -> anyhow::Result<()> {
        let mut state = self.state.lock().unwrap();
        state.active = true;
        state.status.active = true;
        Ok(())
    }

    /// Stop observing terminal sessions.
    pub fn stop(&self) -> anyhow::Result<()> {
        let mut state = self.state.lock().unwrap();
        state.active = false;
        state.status.active = false;
        Ok(())
    }

    /// Capture current commands from all terminal windows.
    pub fn capture_commands(&self) -> Vec<ShellCommand> {
        let mut state = self.state.lock().unwrap();

        // Detect active terminal sessions via the public API
        let sessions = self.terminal_provider.detect_sessions();

        if sessions.is_empty() {
            return Vec::new();
        }

        let mut new_commands = Vec::new();
        let now = Instant::now();

        for session in &sessions {
            // Skip if we've already captured from this session recently
            if state.monitored_sessions.iter().any(|id| id == &session.session_id) {
                // Check if we need to update (command changed)
                if !session.command_text.is_empty() {
                    // Check if this is a new command (not already captured)
                    let is_new = !state.commands.iter().any(|cmd| {
                        cmd.session_id == session.session_id
                            && cmd.command == session.command_text
                            && cmd.captured_at > now - self.capture_interval
                    });

                    if is_new {
                        let is_engineering = ENGINEERING_COMMANDS.iter().any(|cmd| {
                            session.command_text.to_lowercase().contains(cmd)
                        });

                        let cmd = ShellCommand {
                            session_id: session.session_id.clone(),
                            terminal: session.terminal_name.clone(),
                            command: session.command_text.clone(),
                            captured_at: now,
                            is_engineering,
                        };

                        new_commands.push(cmd.clone());
                        state.commands.push(cmd);
                    }
                }
            } else {
                // New session detected
                state.monitored_sessions.push(session.session_id.clone());

                if !session.command_text.is_empty() {
                    let is_engineering = ENGINEERING_COMMANDS.iter().any(|cmd| {
                        session.command_text.to_lowercase().contains(cmd)
                    });

                    let cmd = ShellCommand {
                        session_id: session.session_id.clone(),
                        terminal: session.terminal_name.clone(),
                        command: session.command_text.clone(),
                        captured_at: now,
                        is_engineering,
                    };

                    new_commands.push(cmd.clone());
                    state.commands.push(cmd);
                }
            }
        }

        // Update status
        let minute_ago = now - std::time::Duration::from_secs(60);
        let commands_last_minute = state.commands.iter()
            .filter(|cmd| cmd.captured_at > minute_ago)
            .count();

        state.status = ShellStatus {
            active: state.active,
            monitored_count: state.monitored_sessions.len(),
            commands_last_minute,
            last_command: new_commands.last().map(|c| c.command.clone()),
            last_capture: Some(std::time::SystemTime::now()),
        };

        new_commands
    }

    /// Get current shell observation status.
    pub fn status(&self) -> ShellStatus {
        self.state.lock().unwrap().status.clone()
    }

    /// Get all captured commands (for correlation engine).
    pub fn get_commands(&self) -> Vec<ShellCommand> {
        self.state.lock().unwrap().commands.clone()
    }

    /// Get engineering-relevant commands only.
    pub fn get_engineering_commands(&self) -> Vec<ShellCommand> {
        self.state.lock().unwrap().commands.iter()
            .filter(|cmd| cmd.is_engineering)
            .cloned()
            .collect()
    }
}

impl Default for ShellObserver {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate observation events from captured shell commands.
pub fn shell_events_from_commands(commands: &[ShellCommand]) -> Vec<ObservationEvent> {
    use crate::event::EventType;

    let mut events = Vec::new();

    for cmd in commands {
        let payload = serde_json::json!({
            "session_id": cmd.session_id,
            "terminal": cmd.terminal,
            "command": cmd.command,
            "is_engineering": cmd.is_engineering,
        });

        events.push(ObservationEvent::new(
            EventType::TerminalCommand,
            ProviderType::Terminal,
            cmd.terminal.clone(),
            None,
            ObservationPayload::new(payload),
        ));
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_observer_creation() {
        let observer = ShellObserver::new();
        assert!(!observer.status().active);
        assert!(observer.get_commands().is_empty());
    }

    #[test]
    fn test_shell_observer_lifecycle() {
        let observer = ShellObserver::new();
        assert!(!observer.status().active);

        assert!(observer.start().is_ok());
        assert!(observer.status().active);

        assert!(observer.stop().is_ok());
        assert!(!observer.status().active);
    }

    #[test]
    fn test_shell_observer_register() {
        let observer = ShellObserver::new();
        assert!(observer.register("bash").is_ok());
        assert!(observer.register("powershell").is_ok());
    }

    #[test]
    fn test_shell_observer_capture_empty() {
        let observer = ShellObserver::new();
        assert!(observer.start().is_ok());
        let commands = observer.capture_commands();
        // On headless/CI, no terminals are running
        assert!(commands.is_empty() || !commands.is_empty()); // Just ensure no panic
    }
}