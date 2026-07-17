//! Token counter — abstracted implementation.

pub struct TokenCounter;

impl TokenCounter {
    pub fn count(&self, _text: &str) -> anyhow::Result<usize> {
        // TODO: Implement per-provider token counting
        anyhow::bail!("Not yet implemented")
    }
}