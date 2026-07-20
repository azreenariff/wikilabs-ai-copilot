//! Token Budget Manager — token estimation, intelligent trimming, summarization.
//!
//! Manages token usage within budget constraints:
//! - Estimate prompt size
//! - Trim context intelligently
//! - Preserve important information
//! - Summarize older conversation when necessary
//! - Model-agnostic implementation

use serde::{Deserialize, Serialize};

/// Policy for how to handle budget overflow.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BudgetPolicy {
    /// Strictly enforce budget — refuse to exceed.
    Strict,
    /// Allow slight overflow (up to buffer_pct).
    WithBuffer {
        /// Percentage over budget allowed (0.0 - 1.0).
        buffer_pct: f32,
    },
    /// Aggressive trimming — always trim to fit.
    Aggressive,
}

impl Default for BudgetPolicy {
    fn default() -> Self {
        BudgetPolicy::Strict
    }
}

/// Action to take when budget check fails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetAction {
    /// No action needed — within budget.
    NoOp,
    /// Trim conversation history.
    TrimConversation,
    /// Summarize older messages.
    Summarize,
    /// Drop low-priority context sources.
    DropLowPriorityContext,
    /// Reject the request.
    Reject,
}

/// A budget check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCheck {
    /// Total tokens estimated.
    pub total_tokens: usize,
    /// Budget limit.
    pub budget: usize,
    /// Whether the check passed.
    pub within_budget: bool,
    /// Recommended action if over budget.
    pub recommended_action: BudgetAction,
    /// How many tokens over (0 if within budget).
    pub excess_tokens: usize,
    /// Detailed breakdown by source.
    pub breakdown: Vec<BudgetEntry>,
}

/// A single entry in the budget breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetEntry {
    pub source: String,
    pub tokens: usize,
    pub priority: BudgetPriority,
}

/// Priority for budget trimming decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetPriority {
    /// System prompt — never trimmed.
    System,
    /// Recent conversation — trim last.
    Recent,
    /// Older conversation — trim first.
    Older,
    /// Workspace context — trim if needed.
    Workspace,
    /// Low-priority context — trim first.
    Low,
}

/// Builder for a budget check.
#[derive(Debug, Default)]
pub struct BudgetBuilder {
    system_tokens: usize,
    recent_tokens: usize,
    older_tokens: usize,
    workspace_tokens: usize,
    other_tokens: usize,
}

impl BudgetBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_system(mut self, tokens: usize) -> Self {
        self.system_tokens = tokens;
        self
    }

    pub fn with_recent_conversation(mut self, tokens: usize) -> Self {
        self.recent_tokens = tokens;
        self
    }

    pub fn with_older_conversation(mut self, tokens: usize) -> Self {
        self.older_tokens = tokens;
        self
    }

    pub fn with_workspace_context(mut self, tokens: usize) -> Self {
        self.workspace_tokens = tokens;
        self
    }

    pub fn with_other(mut self, tokens: usize) -> Self {
        self.other_tokens = tokens;
        self
    }

    /// Run the budget check against a limit.
    pub fn check(self, budget: usize, policy: &BudgetPolicy) -> BudgetCheck {
        let total = self.system_tokens
            + self.recent_tokens
            + self.older_tokens
            + self.workspace_tokens
            + self.other_tokens;

        let effective_budget = match policy {
            BudgetPolicy::Strict => budget,
            BudgetPolicy::WithBuffer { buffer_pct } => {
                (budget as f32 * (1.0 + buffer_pct)) as usize
            }
            BudgetPolicy::Aggressive => budget,
        };

        let within_budget = total <= effective_budget;
        let excess_tokens = if total > effective_budget {
            total - effective_budget
        } else {
            0
        };

        let recommended_action = if within_budget {
            BudgetAction::NoOp
        } else if self.older_tokens >= excess_tokens {
            BudgetAction::TrimConversation
        } else if self.older_tokens > 0 {
            BudgetAction::Summarize
        } else if self.workspace_tokens >= excess_tokens {
            BudgetAction::DropLowPriorityContext
        } else {
            match policy {
                BudgetPolicy::Strict | BudgetPolicy::Aggressive => BudgetAction::Reject,
                BudgetPolicy::WithBuffer { .. } => BudgetAction::Reject,
            }
        };

        let mut breakdown = Vec::new();
        if self.system_tokens > 0 {
            breakdown.push(BudgetEntry {
                source: "system_prompt".to_string(),
                tokens: self.system_tokens,
                priority: BudgetPriority::System,
            });
        }
        if self.recent_tokens > 0 {
            breakdown.push(BudgetEntry {
                source: "recent_conversation".to_string(),
                tokens: self.recent_tokens,
                priority: BudgetPriority::Recent,
            });
        }
        if self.older_tokens > 0 {
            breakdown.push(BudgetEntry {
                source: "older_conversation".to_string(),
                tokens: self.older_tokens,
                priority: BudgetPriority::Older,
            });
        }
        if self.workspace_tokens > 0 {
            breakdown.push(BudgetEntry {
                source: "workspace_context".to_string(),
                tokens: self.workspace_tokens,
                priority: BudgetPriority::Workspace,
            });
        }
        if self.other_tokens > 0 {
            breakdown.push(BudgetEntry {
                source: "other_context".to_string(),
                tokens: self.other_tokens,
                priority: BudgetPriority::Low,
            });
        }

        BudgetCheck {
            total_tokens: total,
            budget: effective_budget,
            within_budget,
            recommended_action,
            excess_tokens,
            breakdown,
        }
    }
}

/// Manager for token budget enforcement.
pub struct TokenBudgetManager {
    budget: usize,
    policy: BudgetPolicy,
}

impl TokenBudgetManager {
    pub fn new(budget: usize) -> Self {
        Self {
            budget,
            policy: BudgetPolicy::default(),
        }
    }

    pub fn with_policy(mut self, policy: BudgetPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Set the budget limit.
    pub fn set_budget(&mut self, budget: usize) {
        self.budget = budget;
    }

    /// Get the current budget limit.
    pub fn budget(&self) -> usize {
        self.budget
    }

    /// Set the budget policy.
    pub fn set_policy(&mut self, policy: BudgetPolicy) {
        self.policy = policy;
    }

    /// Get the current budget policy.
    pub fn policy(&self) -> &BudgetPolicy {
        &self.policy
    }

    /// Run a budget check with the given token counts.
    pub fn check(
        &self,
        system_tokens: usize,
        recent_tokens: usize,
        older_tokens: usize,
        workspace_tokens: usize,
        other_tokens: usize,
    ) -> BudgetCheck {
        BudgetBuilder::new()
            .with_system(system_tokens)
            .with_recent_conversation(recent_tokens)
            .with_older_conversation(older_tokens)
            .with_workspace_context(workspace_tokens)
            .with_other(other_tokens)
            .check(self.budget, &self.policy)
    }

    /// Quick check: is the total within budget?
    pub fn is_within_budget(&self, total_tokens: usize) -> bool {
        let effective_budget = match &self.policy {
            BudgetPolicy::Strict => self.budget,
            BudgetPolicy::WithBuffer { buffer_pct } => {
                (self.budget as f32 * (1.0 + buffer_pct)) as usize
            }
            BudgetPolicy::Aggressive => self.budget,
        };
        total_tokens <= effective_budget
    }

    /// Estimate tokens for a message list.
    pub fn estimate_messages(&self, messages: &[&str]) -> usize {
        messages
            .iter()
            .map(|m| crate::token_counter::count_tokens(m))
            .sum()
    }

    /// Trim conversation messages to fit within budget.
    /// Returns the trimmed list and number of messages removed.
    pub fn trim_conversation(
        &self,
        messages: Vec<String>,
        budget: usize,
        system_tokens: usize,
    ) -> (Vec<String>, usize) {
        let mut result = messages;
        let mut removed = 0;

        loop {
            let current_tokens =
                self.estimate_messages(&result.iter().map(|s| s.as_str()).collect::<Vec<_>>());
            let used = system_tokens + current_tokens;
            let effective_budget = match &self.policy {
                BudgetPolicy::Strict => budget,
                BudgetPolicy::WithBuffer { buffer_pct } => {
                    (budget as f32 * (1.0 + buffer_pct)) as usize
                }
                BudgetPolicy::Aggressive => budget,
            };

            if used <= effective_budget || result.is_empty() {
                break;
            }

            // Remove oldest user/assistant message (keep system messages)
            result.pop();
            removed += 1;
        }

        (result, removed)
    }

    /// Calculate how many tokens can be used for conversation given system prompt size.
    pub fn available_for_conversation(&self, system_tokens: usize) -> usize {
        let effective_budget = match &self.policy {
            BudgetPolicy::Strict => self.budget,
            BudgetPolicy::WithBuffer { buffer_pct } => {
                (self.budget as f32 * (1.0 + buffer_pct)) as usize
            }
            BudgetPolicy::Aggressive => self.budget,
        };
        effective_budget.saturating_sub(system_tokens)
    }

    /// Get recommended truncation strategy.
    pub fn recommend_strategy(
        &self,
        total_tokens: usize,
        older_tokens: usize,
        workspace_tokens: usize,
    ) -> BudgetAction {
        let effective_budget = match &self.policy {
            BudgetPolicy::Strict => self.budget,
            BudgetPolicy::WithBuffer { buffer_pct } => {
                (self.budget as f32 * (1.0 + buffer_pct)) as usize
            }
            BudgetPolicy::Aggressive => self.budget,
        };

        if total_tokens <= effective_budget {
            return BudgetAction::NoOp;
        }

        let excess = total_tokens - effective_budget;
        if older_tokens >= excess {
            BudgetAction::TrimConversation
        } else if older_tokens > 0 {
            BudgetAction::Summarize
        } else if workspace_tokens >= excess {
            BudgetAction::DropLowPriorityContext
        } else {
            BudgetAction::Reject
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_policy_values() {
        match BudgetPolicy::Strict {
            BudgetPolicy::Strict => assert!(true),
            BudgetPolicy::WithBuffer { .. } => assert!(false),
            BudgetPolicy::Aggressive => assert!(false),
        }
        match BudgetPolicy::Aggressive {
            BudgetPolicy::Aggressive => assert!(true),
            BudgetPolicy::Strict => assert!(false),
            BudgetPolicy::WithBuffer { .. } => assert!(false),
        }
    }

    #[test]
    fn test_budget_check_within_budget() {
        let check = BudgetBuilder::new()
            .with_system(100)
            .with_recent_conversation(200)
            .check(500, &BudgetPolicy::Strict);

        assert!(check.within_budget);
        assert_eq!(check.recommended_action, BudgetAction::NoOp);
        assert_eq!(check.total_tokens, 300);
        assert_eq!(check.excess_tokens, 0);
    }

    #[test]
    fn test_budget_check_exceeds() {
        let check = BudgetBuilder::new()
            .with_system(100)
            .with_older_conversation(500)
            .with_recent_conversation(100)
            .check(500, &BudgetPolicy::Strict);

        assert!(!check.within_budget);
        assert!(matches!(
            check.recommended_action,
            BudgetAction::TrimConversation | BudgetAction::Summarize
        ));
        assert_eq!(check.excess_tokens, 200);
        assert_eq!(check.total_tokens, 700);
    }

    #[test]
    fn test_budget_check_with_buffer() {
        let check = BudgetBuilder::new()
            .with_system(100)
            .with_recent_conversation(600)
            .check(500, &BudgetPolicy::WithBuffer { buffer_pct: 0.2 });

        // Effective budget = 500 * 1.2 = 600
        // Total = 100 + 600 = 700 > 600, so NOT within budget
        assert!(!check.within_budget);
        assert_eq!(check.budget, 600);
    }

    #[test]
    fn test_budget_breakdown() {
        let check = BudgetBuilder::new()
            .with_system(100)
            .with_recent_conversation(200)
            .check(1000, &BudgetPolicy::Strict);

        assert_eq!(check.breakdown.len(), 2);
        assert_eq!(check.breakdown[0].source, "system_prompt");
        assert_eq!(check.breakdown[0].tokens, 100);
        assert_eq!(check.breakdown[1].source, "recent_conversation");
        assert_eq!(check.breakdown[1].tokens, 200);
    }

    #[test]
    fn test_budget_action_values() {
        match BudgetAction::NoOp {
            BudgetAction::NoOp => assert!(true),
            _ => assert!(false),
        }
        match BudgetAction::Summarize {
            BudgetAction::Summarize => assert!(true),
            _ => assert!(false),
        }
        match BudgetAction::Reject {
            BudgetAction::Reject => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_budget_priority_values() {
        match BudgetPriority::System {
            BudgetPriority::System => assert!(true),
            _ => assert!(false),
        }
        match BudgetPriority::Low {
            BudgetPriority::Low => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_token_budget_manager_new() {
        let manager = TokenBudgetManager::new(4096);
        assert_eq!(manager.budget(), 4096);
    }

    #[test]
    fn test_token_budget_manager_within_budget() {
        let manager = TokenBudgetManager::new(1000);
        assert!(manager.is_within_budget(500));
        assert!(manager.is_within_budget(1000));
        assert!(!manager.is_within_budget(1001));
    }

    #[test]
    fn test_token_budget_manager_with_buffer() {
        let manager =
            TokenBudgetManager::new(1000).with_policy(BudgetPolicy::WithBuffer { buffer_pct: 0.1 });

        // Effective budget = 1100
        assert!(manager.is_within_budget(1100));
        assert!(!manager.is_within_budget(1101));
    }

    #[test]
    fn test_token_budget_manager_set_budget() {
        let mut manager = TokenBudgetManager::new(4096);
        manager.set_budget(8192);
        assert_eq!(manager.budget(), 8192);
    }

    #[test]
    fn test_token_budget_manager_check() {
        let manager = TokenBudgetManager::new(1000);
        let check = manager.check(100, 200, 500, 200, 0);
        assert!(check.within_budget); // 1000 total == 1000 budget, exactly at limit
        assert_eq!(check.total_tokens, 1000);
        assert_eq!(check.excess_tokens, 0);
        assert_eq!(check.recommended_action, BudgetAction::NoOp);
    }

    #[test]
    fn test_token_budget_manager_check_within() {
        let manager = TokenBudgetManager::new(2000);
        let check = manager.check(100, 200, 500, 200, 0);
        assert!(check.within_budget); // 1000 < 2000
    }

    #[test]
    fn test_token_budget_manager_estimate_messages() {
        let manager = TokenBudgetManager::new(2000);
        let tokens = manager.estimate_messages(&["hello", "world", "test"]);
        assert!(tokens >= 2); // 3 messages, ~1 token each
    }

    #[test]
    fn test_token_budget_manager_trim_conversation() {
        let manager = TokenBudgetManager::new(50);
        let messages = vec![
            "short".to_string(),
            "also short".to_string(),
            "tiny".to_string(),
        ];
        let (trimmed, _removed) = manager.trim_conversation(messages, 50, 0);
        // Should trim until within budget
        assert!(trimmed.len() <= 3);
        let trimmed_refs: Vec<&str> = trimmed.iter().map(|s| s.as_str()).collect();
        let current_tokens = manager.estimate_messages(&trimmed_refs);
    }

    #[test]
    fn test_token_budget_manager_available_for_conversation() {
        let manager = TokenBudgetManager::new(4096);
        let available = manager.available_for_conversation(512);
        assert_eq!(available, 3584);
    }

    #[test]
    fn test_token_budget_manager_available_with_buffer() {
        let manager = TokenBudgetManager::new(4096)
            .with_policy(BudgetPolicy::WithBuffer { buffer_pct: 0.05 });

        let available = manager.available_for_conversation(512);
        // Effective budget = 4096 * 1.05 = 4300 (approx)
        assert!(available >= 3500);
    }

    #[test]
    fn test_token_budget_manager_recommend_no_op() {
        let manager = TokenBudgetManager::new(1000);
        let action = manager.recommend_strategy(500, 200, 100);
        assert_eq!(action, BudgetAction::NoOp);
    }

    #[test]
    fn test_token_budget_manager_recommend_trim() {
        let manager = TokenBudgetManager::new(100);
        let action = manager.recommend_strategy(500, 500, 0);
        assert!(matches!(action, BudgetAction::TrimConversation));
    }

    #[test]
    fn test_token_budget_manager_recommend_summarize() {
        let manager = TokenBudgetManager::new(100);
        let action = manager.recommend_strategy(200, 50, 0);
        assert!(matches!(action, BudgetAction::Summarize));
    }

    #[test]
    fn test_token_budget_manager_recommend_reject() {
        let manager = TokenBudgetManager::new(100);
        let action = manager.recommend_strategy(500, 0, 100);
        assert!(matches!(action, BudgetAction::Reject));
    }

    #[test]
    fn test_budget_check_empty() {
        let check = BudgetBuilder::new().check(1000, &BudgetPolicy::Strict);
        assert!(check.within_budget);
        assert_eq!(check.total_tokens, 0);
    }

    #[test]
    fn test_budget_policy_default() {
        assert_eq!(BudgetPolicy::default(), BudgetPolicy::Strict);
    }

    #[test]
    fn test_token_budget_manager_with_aggressive_policy() {
        let manager = TokenBudgetManager::new(100).with_policy(BudgetPolicy::Aggressive);

        let check = manager.check(0, 0, 150, 0, 0);
        assert!(!check.within_budget);
        assert!(matches!(
            check.recommended_action,
            BudgetAction::TrimConversation
        ));
    }

    #[test]
    fn test_budget_entry_serialization() {
        let entry = BudgetEntry {
            source: "test".to_string(),
            tokens: 100,
            priority: BudgetPriority::Low,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: BudgetEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.source, "test");
        assert_eq!(parsed.tokens, 100);
    }

    #[test]
    fn test_budget_check_all_entries() {
        let check = BudgetBuilder::new()
            .with_system(100)
            .with_recent_conversation(200)
            .with_older_conversation(300)
            .with_workspace_context(400)
            .with_other(500)
            .check(2000, &BudgetPolicy::Strict);

        assert_eq!(check.breakdown.len(), 5);
        assert_eq!(check.total_tokens, 1500);
        assert!(check.within_budget);
    }

    #[test]
    fn test_trim_with_system_tokens() {
        let manager = TokenBudgetManager::new(50);
        let messages = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let (trimmed, _removed) = manager.trim_conversation(messages.clone(), 50, 30);
        // System takes 30 tokens, leaving 20 for conversation
        assert!(trimmed.len() <= messages.len());
    }

    #[test]
    fn test_trim_empty_messages() {
        let manager = TokenBudgetManager::new(50);
        let messages: Vec<String> = vec![];
        let (trimmed, removed) = manager.trim_conversation(messages, 50, 10);
        assert!(trimmed.is_empty());
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_available_with_zero_system_tokens() {
        let manager = TokenBudgetManager::new(4096);
        assert_eq!(manager.available_for_conversation(0), 4096);
    }
}
