//! AI Response types and streaming helpers.

/// A streaming chunk from the AI provider.
#[derive(Debug, Clone)]
pub struct StreamItem {
    pub content: String,
    pub is_complete: bool,
}

/// A tool call from the AI.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

impl ToolCall {
    pub fn new(name: &str, arguments: serde_json::Value) -> Self {
        Self {
            name: name.to_string(),
            arguments,
        }
    }
}

/// Builder for constructing StreamItems incrementally.
#[derive(Debug, Default)]
pub struct StreamBuilder {
    items: Vec<StreamItem>,
}

impl StreamBuilder {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_chunk(mut self, content: &str, complete: bool) -> Self {
        self.items.push(StreamItem {
            content: content.to_string(),
            is_complete: complete,
        });
        self
    }

    pub fn build(self) -> Vec<StreamItem> {
        self.items
    }

    /// Convenience: build a single complete item.
    pub fn complete(content: &str) -> Vec<StreamItem> {
        vec![StreamItem {
            content: content.to_string(),
            is_complete: true,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_new() {
        let tc = ToolCall::new("test_tool", serde_json::json!({"key": "value"}));
        assert_eq!(tc.name, "test_tool");
        assert_eq!(tc.arguments["key"], "value");
    }

    #[test]
    fn test_stream_builder_complete() {
        let items = StreamBuilder::complete("Hello, world!");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].content, "Hello, world!");
        assert!(items[0].is_complete);
    }

    #[test]
    fn test_stream_builder_chunks() {
        let items = StreamBuilder::new()
            .add_chunk("Hello", false)
            .add_chunk(", ", false)
            .add_chunk("world!", true)
            .build();

        assert_eq!(items.len(), 3);
        assert!(!items[0].is_complete);
        assert!(!items[1].is_complete);
        assert!(items[2].is_complete);

        // Concatenated content should be "Hello, world!"
        let combined: String = items.iter().map(|i| i.content.as_str()).collect();
        assert_eq!(combined, "Hello, world!");
    }

    #[test]
    fn test_stream_builder_empty() {
        let items = StreamBuilder::new().build();
        assert!(items.is_empty());
    }
}
