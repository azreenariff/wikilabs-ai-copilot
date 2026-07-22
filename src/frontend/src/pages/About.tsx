import { useState, useEffect } from 'react';

function About() {
  const [status, setStatus] = useState<{ version: string; running: boolean }>({
    version: '1.1.5',
    running: true,
  });

  useEffect(() => {
    // Fetch real version from backend
    fetch('http://localhost:1420/api/commands/get_status', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ params: {} }),
    })
      .then(r => r.json())
      .then(data => {
        if (data.success && data.value) {
          setStatus({
            version: data.value.version || '1.1.5',
            running: data.value.status === 'running',
          });
        }
      })
      .catch(() => {
        // Backend not running — show the real Cargo.toml version
        setStatus({ version: '1.1.5', running: false });
      });
  }, []);

  return (
    <div style={{ padding: '32px', maxWidth: '800px', margin: '0 auto' }}>
      <h2 style={{ marginBottom: '24px', color: 'var(--color-text-primary)' }}>ℹ️ About</h2>
      <div style={{
        background: 'var(--color-bg-secondary)',
        border: '1px solid var(--color-border)',
        borderRadius: '12px',
        padding: '32px',
      }}>
        <div style={{ textAlign: 'center', marginBottom: '24px' }}>
          <span style={{ fontSize: '48px' }}>🧠</span>
          <h3 style={{ fontSize: '24px', fontWeight: 700, margin: '12px 0 4px' }}>
            Wiki Labs AI Copilot
          </h3>
          <p style={{ color: 'var(--color-text-secondary)', fontSize: '14px' }}>
            Version {status.version} (Phase 4 — MVP Desktop Foundation)
          </p>
          <p style={{ color: status.running ? 'var(--color-success)' : 'var(--color-error)', fontSize: '12px', marginTop: '4px' }}>
            {status.running ? '✓ Backend connected' : '⚠ Backend not running (showing Cargo.toml version)'}
          </p>
        </div>

        <div style={{ borderTop: '1px solid var(--color-border)', paddingTop: '16px' }}>
          <h4 style={{ fontSize: '14px', marginBottom: '12px' }}>Tech Stack</h4>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px', fontSize: '13px' }}>
            <div style={{ color: 'var(--color-text-secondary)' }}>• Tauri v2 (Rust)</div>
            <div style={{ color: 'var(--color-text-secondary)' }}>• React 18 + Vite</div>
            <div style={{ color: 'var(--color-text-secondary)' }}>• SQLite + rusqlite</div>
            <div style={{ color: 'var(--color-text-secondary)' }}>• OpenAI-compatible APIs</div>
            <div style={{ color: 'var(--color-text-secondary)' }}>• FTS5 Search</div>
            <div style={{ color: 'var(--color-text-secondary)' }}>• Tailwind CSS v4</div>
          </div>
        </div>

        <div style={{ borderTop: '1px solid var(--color-border)', paddingTop: '16px', marginTop: '16px' }}>
          <h4 style={{ fontSize: '14px', marginBottom: '12px' }}>Phase 4 Deliverables</h4>
          <ul style={{ listStyle: 'none', padding: 0, display: 'grid', gap: '4px', fontSize: '13px' }}>
            {['Desktop application shell', 'AI provider abstraction', 'Local configuration', 'Assistant chat interface', 'Settings page', 'Local SQLite database', 'Structured logging', 'Error handling'].map(item => (
              <li key={item} style={{ color: 'var(--color-text-secondary)' }}>
                <span style={{ color: 'var(--color-success)' }}>✓</span> {item}
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  );
}

export default About;