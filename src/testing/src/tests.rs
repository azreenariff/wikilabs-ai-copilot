//! Tests for testing utilities: mocks, fixtures.

use crate::mocks::openai_mock::OpenAIMock;
use crate::fixtures::workspace_fixtures::create_test_workspace_id;
use crate::fixtures::knowledge_fixtures::sample_document_content;

mod mock_tests {
    use super::*;

    #[test]
    fn test_openai_mock_new() {
        let mock = OpenAIMock::new();
        // just verify it constructs
    }
}

mod fixture_tests {
    use super::*;

    #[test]
    fn test_create_test_workspace_id() {
        let id = create_test_workspace_id();
        assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000001");
    }

    #[test]
    fn test_sample_document_content() {
        let content = sample_document_content();
        assert!(content.contains("knowledge content"));
    }
}