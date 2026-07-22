function Workspaces() {
  return (
    <div style={{ padding: '32px', maxWidth: '800px', margin: '0 auto' }}>
      <h2 style={{ marginBottom: '24px', color: 'var(--color-text-primary)' }}>📁 Workspaces</h2>
      <div style={{
        background: 'var(--color-bg-secondary)',
        border: '1px solid var(--color-border)',
        borderRadius: '12px',
        padding: '48px',
        textAlign: 'center',
      }}>
        <p style={{ fontSize: '16px', color: 'var(--color-text-secondary)' }}>
          Workspaces let you organize your AI context per customer or project.
        </p>
        <p style={{ fontSize: '14px', color: 'var(--color-text-secondary)', marginTop: '12px' }}>
          Feature coming in next milestone.
        </p>
      </div>
      <div style={{ marginTop: '24px' }}>
        <h3 style={{ fontSize: '14px', color: 'var(--color-text-secondary)', marginBottom: '12px' }}>
          Upcoming Features
        </h3>
        <ul style={{ listStyle: 'none', padding: 0, display: 'grid', gap: '8px' }}>
          {['Create new workspaces', 'Switch between workspaces', 'Workspace-specific AI context', 'Workspace settings'].map(feat => (
            <li key={feat} style={{
              padding: '8px 12px',
              background: 'var(--color-bg-tertiary)',
              borderRadius: '6px',
              fontSize: '13px',
              color: 'var(--color-text-secondary)',
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
            }}>
              <span style={{ opacity: 0.5 }}>⬚</span> {feat}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

export default Workspaces;