function Skills() {
  return (
    <div style={{ padding: '32px', maxWidth: '800px', margin: '0 auto' }}>
      <h2 style={{ marginBottom: '24px', color: 'var(--color-text-primary)' }}>🔧 Skills</h2>
      <div style={{
        background: 'var(--color-bg-secondary)',
        border: '1px solid var(--color-border)',
        borderRadius: '12px',
        padding: '48px',
        textAlign: 'center',
      }}>
        <p style={{ fontSize: '16px', color: 'var(--color-text-secondary)' }}>
          Skills are MCP-based modules that extend the AI's capabilities.
        </p>
        <p style={{ fontSize: '14px', color: 'var(--color-text-secondary)', marginTop: '12px' }}>
          MCP skills are gated out for Phase 4 MVP — coming in a future milestone.
        </p>
      </div>
    </div>
  );
}

export default Skills;