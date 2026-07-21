//! MCP namespace resolver.

pub struct NamespaceResolver;

impl NamespaceResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve(&self, _qualified_name: &str) -> anyhow::Result<(String, String)> {
        // Stub: placeholder. Parse "skill__tool" format and return (skill_name, tool_name).
        unimplemented!()
    }
}
