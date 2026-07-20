//! Workspace test fixtures.

pub fn create_test_workspace_id() -> uuid::Uuid {
    uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
}
