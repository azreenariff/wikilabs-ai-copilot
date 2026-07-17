//! Tests for AI module: context window, token counting, and response types.

use crate::context::{ContextWindow, ContextAllocation};
use crate::response::{StreamItem, ToolCall};

mod context_tests {
    use super::*;

    #[test]
    fn test_context_window_new() {
        let ctx = ContextWindow::new(1000);
        assert_eq!(ctx.total_tokens, 0);
        assert_eq!(ctx.max_tokens, 1000);
    }

    #[test]
    fn test_context_window_usage_zero() {
        let ctx = ContextWindow::new(1000);
        assert_eq!(ctx.usage_pct(), 0.0);
    }

    #[test]
    fn test_context_window_usage_partial() {
        let ctx = ContextWindow::new(1000);
        // total_tokens is 0, so usage is 0%
        assert_eq!(ctx.usage_pct(), 0.0);
    }

    #[test]
    fn test_context_window_usage_zero_max() {
        let ctx = ContextWindow::new(0);
        assert_eq!(ctx.usage_pct(), 0.0);
    }

    #[test]
    fn test_context_allocation_defaults() {
        let alloc = ContextAllocation {
            system_prompt_pct: 0.1,
            observation_context_pct: 0.1,
            knowledge_context_pct: 0.2,
            conversation_history_pct: 0.5,
            tool_results_pct: 0.05,
            padding_pct: 0.05,
        };
        let total: f32 = alloc.system_prompt_pct + alloc.observation_context_pct
            + alloc.knowledge_context_pct + alloc.conversation_history_pct
            + alloc.tool_results_pct + alloc.padding_pct;
        assert_eq!(total, 1.0);
    }
}

mod token_counter_tests {
    use super::*;

    #[test]
    fn test_count_tokens_free_fn() {
        // "test text" = 9 chars / 4 = 2.25 -> 2
        assert_eq!(crate::token_counter::count_tokens("test text"), 2);
        assert_eq!(crate::token_counter::count_tokens(""), 0);
    }
}

mod response_tests {
    use super::*;

    #[test]
    fn test_stream_item_complete() {
        let item = StreamItem {
            content: "hello".to_string(),
            is_complete: true,
        };
        assert_eq!(item.content, "hello");
        assert!(item.is_complete);
    }

    #[test]
    fn test_stream_item_partial() {
        let item = StreamItem {
            content: "world".to_string(),
            is_complete: false,
        };
        assert_eq!(item.content, "world");
        assert!(!item.is_complete);
    }

    #[test]
    fn test_tool_call_json_args() {
        let tc = ToolCall {
            name: "read_file".to_string(),
            arguments: serde_json::json!({"path": "/test.txt"}),
        };
        assert_eq!(tc.name, "read_file");
        assert_eq!(tc.arguments["path"], "/test.txt");
    }

    #[test]
    fn test_tool_call_empty_args() {
        let tc = ToolCall {
            name: "list_files".to_string(),
            arguments: serde_json::Value::Object(serde_json::Map::new()),
        };
        assert_eq!(tc.name, "list_files");
    }
}