//! MCP namespace resolver.

pub struct NamespaceResolver;

impl NamespaceResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve(&self, _qualified_name: &str) -> anyhow::Result<(String, String)> {
        // TODO: Parse "skill__tool" format
        // Returns (skill_name, tool_name)
        anyhow::bail!("Not yet implemented")
    }
}
