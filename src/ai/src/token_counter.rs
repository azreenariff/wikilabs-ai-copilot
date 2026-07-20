//! Token counter — approximated via character ratio.
//!
//! Uses a simple chars-per-token heuristic (~4 chars/token)
//! suitable for MVP without an actual tokenizer library.

/// Count tokens in text using a simple heuristic.
/// ~4 characters per token is a rough approximation
/// that works reasonably well for English text.
pub fn count_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    (text.chars().count() / 4).max(1)
}

/// Count tokens in a list of messages, summing individual counts.
pub fn count_tokens_messages(messages: &[&str]) -> usize {
    messages.iter().map(|m| count_tokens(m)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens_empty() {
        assert_eq!(count_tokens(""), 0);
    }

    #[test]
    fn test_count_tokens_single_word() {
        assert_eq!(count_tokens("hello"), 1);
    }

    #[test]
    fn test_count_tokens_approximation() {
        // ~40 chars ≈ 10 tokens
        let text = "a".repeat(40);
        let tokens = count_tokens(&text);
        assert!(
            tokens >= 8 && tokens <= 12,
            "Expected ~10 tokens for 40 chars, got {}",
            tokens
        );
    }

    #[test]
    fn test_count_tokens_minimum_one() {
        // Any non-empty text should return at least 1
        let result = count_tokens("x");
        assert!(result >= 1);
    }

    #[test]
    fn test_count_tokens_messages() {
        let msgs = vec!["hello", "world", "foo"];
        let total = count_tokens_messages(&msgs);
        assert_eq!(total, 3); // Each is 1 token
    }

    #[test]
    fn test_count_tokens_messages_empty() {
        let msgs: Vec<&str> = vec![];
        let total = count_tokens_messages(&msgs);
        assert_eq!(total, 0);
    }

    #[test]
    fn test_count_tokens_unicode() {
        // Unicode characters each count as one char
        let text = "こんにちは"; // 5 chars
        let tokens = count_tokens(text);
        assert!(tokens >= 1);
    }
}
