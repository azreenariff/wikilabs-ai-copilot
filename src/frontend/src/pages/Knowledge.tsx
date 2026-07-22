function Knowledge() {
  return (
    <div style={{ padding: '32px', maxWidth: '800px', margin: '0 auto' }}>
      <h2 style={{ marginBottom: '24px', color: 'var(--color-text-primary)' }}>📚 Knowledge</h2>
      <div style={{
        background: 'var(--color-bg-secondary)',
        border: '1px solid var(--color-border)',
        borderRadius: '12px',
        padding: '48px',
        textAlign: 'center',
      }}>
        <p style={{ fontSize: '16px', color: 'var(--color-text-secondary)' }}>
          Knowledge base for storing and searching project documentation.
        </p>
        <p style={{ fontSize: '14px', color: 'var(--color-text-secondary)', marginTop: '12px' }}>
          Basic import and FTS5 search are built — UI integration coming soon.
        </p>
      </div>
    </div>
  );
}

export default Knowledge;