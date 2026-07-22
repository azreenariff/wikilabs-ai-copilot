function Activity() {
  return (
    <div style={{ padding: '32px', maxWidth: '800px', margin: '0 auto' }}>
      <h2 style={{ marginBottom: '24px', color: 'var(--color-text-primary)' }}>📊 Activity</h2>
      <div style={{
        background: 'var(--color-bg-secondary)',
        border: '1px solid var(--color-border)',
        borderRadius: '12px',
        padding: '48px',
        textAlign: 'center',
      }}>
        <p style={{ fontSize: '16px', color: 'var(--color-text-secondary)' }}>
          Activity log tracks system events, AI interactions, and changes.
        </p>
        <p style={{ fontSize: '14px', color: 'var(--color-text-secondary)', marginTop: '12px' }}>
          Audit log infrastructure is ready — UI to follow.
        </p>
      </div>
    </div>
  );
}

export default Activity;