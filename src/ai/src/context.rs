//! Context Window Manager — token budget tracking.

pub struct ContextWindow {
    pub total_tokens: usize,
    pub max_tokens: usize,
}

pub struct ContextAllocation {
    pub system_prompt_pct: f32,
    pub observation_context_pct: f32,
    pub knowledge_context_pct: f32,
    pub conversation_history_pct: f32,
    pub tool_results_pct: f32,
    pub padding_pct: f32,
}

impl ContextWindow {
    pub fn new(max_tokens: usize) -> Self {
        Self { total_tokens: 0, max_tokens }
    }

    pub fn usage_pct(&self) -> f32 {
        if self.max_tokens == 0 {
            0.0
        } else {
            self.total_tokens as f32 / self.max_tokens as f32
        }
    }
}