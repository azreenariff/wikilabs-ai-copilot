//! Context Window Manager — token budget tracking.
//!
//! Manages token usage within a bounded context window,
//! allocating budgets to system prompts, conversation history,
//! and other context sources.

use serde::{Deserialize, Serialize};

/// A single entry within the context window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    /// Label for this entry (e.g., "system_prompt", "conversation_history").
    pub label: String,
    /// Token count of this entry.
    pub tokens: usize,
}

/// Allocation percentages for context window budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAllocation {
    pub system_prompt_pct: f32,
    pub conversation_history_pct: f32,
    pub knowledge_context_pct: f32,
    pub workspace_context_pct: f32,
    pub padding_pct: f32,
}

impl Default for ContextAllocation {
    fn default() -> Self {
        Self {
            system_prompt_pct: 0.10,
            conversation_history_pct: 0.40,
            knowledge_context_pct: 0.20,
            workspace_context_pct: 0.20,
            padding_pct: 0.10,
        }
    }
}

/// Context window with token budget tracking.
#[derive(Debug)]
pub struct ContextWindow {
    pub total_tokens: usize,
    pub max_tokens: usize,
    allocation: ContextAllocation,
}

impl ContextWindow {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            total_tokens: 0,
            max_tokens,
            allocation: ContextAllocation::default(),
        }
    }

    pub fn new_with_allocation(max_tokens: usize, allocation: ContextAllocation) -> Self {
        Self {
            total_tokens: 0,
            max_tokens,
            allocation,
        }
    }

    /// Current usage as a fraction (0.0 - 1.0+).
    pub fn usage_pct(&self) -> f32 {
        if self.max_tokens == 0 {
            0.0
        } else {
            self.total_tokens as f32 / self.max_tokens as f32
        }
    }

    /// Tokens remaining in the context window.
    pub fn remaining_tokens(&self) -> usize {
        if self.total_tokens >= self.max_tokens {
            0
        } else {
            self.max_tokens - self.total_tokens
        }
    }

    /// Check if adding tokens would fit within the context window.
    pub fn would_fit(&self, additional_tokens: usize) -> bool {
        self.total_tokens + additional_tokens <= self.max_tokens
    }

    /// Allocate budget for a specific context source by percentage.
    pub fn allocate_for(&self, category_pct: f32) -> usize {
        let budget = (self.max_tokens as f32 * category_pct) as usize;
        budget.min(self.remaining_tokens())
    }

    /// Add tokens to the total, clamping at max_tokens.
    pub fn consume(&mut self, tokens: usize) {
        self.total_tokens = (self.total_tokens + tokens).min(self.max_tokens);
    }

    /// Reset total tokens (e.g., after truncation).
    pub fn reset(&mut self) {
        self.total_tokens = 0;
    }

    /// Get the allocation percentages.
    pub fn allocation(&self) -> &ContextAllocation {
        &self.allocation
    }

    /// Check if the context is near full (>85% usage).
    pub fn is_near_full(&self) -> bool {
        self.usage_pct() > 0.85
    }

    /// Truncate conversation history to fit within budget.
    /// Returns the number of messages removed.
    pub fn truncate_for_budget(&mut self, target_tokens: usize) {
        if self.total_tokens > target_tokens {
            self.total_tokens = target_tokens;
        }
    }

    /// Record a context entry for tracking usage.
    pub fn record(&mut self, entry: &ContextEntry) {
        self.consume(entry.tokens);
    }

    /// Build a summary of context entries with their token costs.
    pub fn build_entries(&self) -> Vec<ContextEntry> {
        let alloc = self.allocation();
        vec![
            ContextEntry {
                label: "system_prompt".to_string(),
                tokens: (self.max_tokens as f32 * alloc.system_prompt_pct) as usize,
            },
            ContextEntry {
                label: "conversation_history".to_string(),
                tokens: (self.max_tokens as f32 * alloc.conversation_history_pct) as usize,
            },
            ContextEntry {
                label: "knowledge_context".to_string(),
                tokens: (self.max_tokens as f32 * alloc.knowledge_context_pct) as usize,
            },
            ContextEntry {
                label: "workspace_context".to_string(),
                tokens: (self.max_tokens as f32 * alloc.workspace_context_pct) as usize,
            },
            ContextEntry {
                label: "padding".to_string(),
                tokens: (self.max_tokens as f32 * alloc.padding_pct) as usize,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_context_window() {
        let cw = ContextWindow::new(4096);
        assert_eq!(cw.total_tokens, 0);
        assert_eq!(cw.max_tokens, 4096);
    }

    #[test]
    fn test_usage_pct_zero() {
        let cw = ContextWindow::new(4096);
        assert_eq!(cw.usage_pct(), 0.0);
    }

    #[test]
    fn test_usage_pct_half() {
        let mut cw = ContextWindow::new(1000);
        cw.consume(500);
        assert_eq!(cw.usage_pct(), 0.5);
    }

    #[test]
    fn test_usage_pct_full() {
        let mut cw = ContextWindow::new(1000);
        cw.consume(1000);
        assert_eq!(cw.usage_pct(), 1.0);
    }

    #[test]
    fn test_usage_pct_over() {
        let mut cw = ContextWindow::new(1000);
        cw.consume(2000); // clamped to max
        assert_eq!(cw.usage_pct(), 1.0);
    }

    #[test]
    fn test_remaining_tokens() {
        let mut cw = ContextWindow::new(4096);
        assert_eq!(cw.remaining_tokens(), 4096);
        cw.consume(1024);
        assert_eq!(cw.remaining_tokens(), 3072);
    }

    #[test]
    fn test_remaining_tokens_empty() {
        let mut cw = ContextWindow::new(4096);
        cw.consume(4096);
        assert_eq!(cw.remaining_tokens(), 0);
    }

    #[test]
    fn test_would_fit() {
        let cw = ContextWindow::new(4096);
        assert!(cw.would_fit(100));
        assert!(cw.would_fit(4096));
        assert!(!cw.would_fit(4097));
    }

    #[test]
    fn test_allocate_for() {
        let cw = ContextWindow::new(4096);
        let history_budget = cw.allocate_for(0.40);
        assert_eq!(history_budget, 1638); // 40% of 4096
    }

    #[test]
    fn test_allocate_respects_remaining() {
        let mut cw = ContextWindow::new(1000);
        cw.consume(800);
        // Even though 40% = 400, only 200 remaining
        let history_budget = cw.allocate_for(0.40);
        assert_eq!(history_budget, 200);
    }

    #[test]
    fn test_consume_clamped() {
        let mut cw = ContextWindow::new(1000);
        cw.consume(1500);
        assert_eq!(cw.total_tokens, 1000); // clamped to max
    }

    #[test]
    fn test_reset() {
        let mut cw = ContextWindow::new(4096);
        cw.consume(2048);
        cw.reset();
        assert_eq!(cw.total_tokens, 0);
    }

    #[test]
    fn test_is_near_full() {
        let mut cw = ContextWindow::new(1000);
        assert!(!cw.is_near_full());
        cw.consume(851);
        assert!(cw.is_near_full());
    }

    #[test]
    fn test_default_allocation() {
        let alloc = ContextAllocation::default();
        let total: f32 = [
            alloc.system_prompt_pct,
            alloc.conversation_history_pct,
            alloc.knowledge_context_pct,
            alloc.workspace_context_pct,
            alloc.padding_pct,
        ]
        .iter()
        .sum();
        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_truncate_for_budget() {
        let mut cw = ContextWindow::new(4096);
        cw.consume(3000);
        cw.truncate_for_budget(2000);
        assert_eq!(cw.total_tokens, 2000);
    }

    #[test]
    fn test_truncate_no_op() {
        let mut cw = ContextWindow::new(4096);
        cw.consume(1000);
        cw.truncate_for_budget(2000);
        assert_eq!(cw.total_tokens, 1000);
    }

    #[test]
    fn test_context_entry() {
        let entry = ContextEntry {
            label: "system_prompt".to_string(),
            tokens: 512,
        };
        assert_eq!(entry.label, "system_prompt");
        assert_eq!(entry.tokens, 512);
    }

    #[test]
    fn test_build_entries() {
        let cw = ContextWindow::new(4096);
        let entries = cw.build_entries();
        assert_eq!(entries.len(), 5);
        let total: usize = entries.iter().map(|e| e.tokens).sum();
        // Float truncation: 409.6→409, 1638.4→1638, 819.2→819, 819.2→819, 409.6→409 = 4094
        assert_eq!(total, 4094);
    }

    #[test]
    fn test_record_entry() {
        let mut cw = ContextWindow::new(4096);
        let entry = ContextEntry {
            label: "system".to_string(),
            tokens: 256,
        };
        cw.record(&entry);
        assert_eq!(cw.total_tokens, 256);
    }
}
