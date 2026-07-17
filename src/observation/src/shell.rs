//! Shell integration — bash, zsh, PowerShell command capture.

pub struct ShellObserver;

impl ShellObserver {
    pub fn new() -> Self {
        Self
    }

    pub fn register(&self, _shell: &str) -> anyhow::Result<()> {
        // TODO: Register shell integration hook
        anyhow::bail!("Not yet implemented")
    }
}