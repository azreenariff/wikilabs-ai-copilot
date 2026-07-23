import { useState, useEffect } from 'react';

interface AppStatus {
  version: string;
  running: boolean;
  status?: string;
  features?: Record<string, boolean>;
}

function About() {
  const [status, setStatus] = useState<AppStatus>({
    version: '1.1.5',
    running: false,
  });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchStatus = async () => {
      try {
        const res = await fetch('http://localhost:1420/api/commands/get_status', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ params: {} }),
        });
        const data = await res.json();
        if (data.success && data.value) {
          const value = data.value as {
            version?: string;
            status?: string;
            features?: Record<string, boolean>;
          };
          setStatus({
            version: value.version || '1.1.5',
            running: value.status === 'running',
            status: value.status,
            features: value.features,
          });
        }
      } catch (e) {
        console.error('Failed to fetch status:', e);
      } finally {
        setLoading(false);
      }
    };
    fetchStatus();
  }, []);

  const features = status.features || {};

  const featureItems = [
    { name: 'Desktop application shell', done: true },
    { name: 'AI provider abstraction', done: features.chat || true },
    { name: 'Local configuration', done: true },
    { name: 'Assistant chat interface', done: features.chat || true },
    { name: 'Settings page', done: true },
    { name: 'Local SQLite database', done: true },
    { name: 'Structured logging', done: true },
    { name: 'Error handling', done: true },
    { name: 'Knowledge packs', done: features.knowledge || true },
    { name: 'Workspace management', done: features.workspace || true },
    { name: 'MCP Skills platform', done: features.skills || false },
    { name: 'MCP integration', done: features.mcp || false },
    { name: 'Workflow automation', done: features.automation || false },
  ];

  return (
    <div style={{ padding: '32px', maxWidth: '900px', margin: '0 auto' }}>
      <h2 style={{ marginBottom: '24px', color: 'var(--color-text-primary)' }}>ℹ️ About</h2>

      <div style={{
        background: 'var(--color-bg-secondary)',
        border: '1px solid var(--color-border)',
        borderRadius: '12px',
        padding: '32px',
      }}>
        {/* Header */}
        <div style={{ textAlign: 'center', marginBottom: '24px' }}>
          <span style={{ fontSize: '48px' }}>🧠</span>
          <h3 style={{ fontSize: '24px', fontWeight: 700, margin: '12px 0 4px' }}>
            Wiki Labs AI Copilot
          </h3>
          {loading ? (
            <div style={{ color: 'var(--color-text-secondary)', fontSize: '13px' }}>Loading version info...</div>
          ) : (
            <>
              <p style={{ color: 'var(--color-text-secondary)', fontSize: '14px' }}>
                Version {status.version}
              </p>
              <p style={{
                color: status.running ? 'var(--color-success)' : 'var(--color-error)',
                fontSize: '12px',
                marginTop: '4px',
              }}>
                {status.running ? '✓ Backend connected' : '⚠ Backend not running'}
              </p>
            </>
          )}
        </div>

        {/* Tech Stack */}
        <div style={{ borderTop: '1px solid var(--color-border)', paddingTop: '16px' }}>
          <h4 style={{ fontSize: '14px', marginBottom: '12px', color: 'var(--color-text-primary)' }}>Tech Stack</h4>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px', fontSize: '13px' }}>
            {[
              'Tauri v2 (Rust)',
              'React 18 + Vite',
              'SQLite + rusqlite',
              'OpenAI-compatible APIs',
              'FTS5 Search',
              'Tailwind CSS v4',
            ].map(item => (
              <div key={item} style={{ color: 'var(--color-text-secondary)' }}>
                • {item}
              </div>
            ))}
          </div>
        </div>

        {/* Phase 4 Deliverables */}
        <div style={{ borderTop: '1px solid var(--color-border)', paddingTop: '16px', marginTop: '16px' }}>
          <h4 style={{ fontSize: '14px', marginBottom: '12px', color: 'var(--color-text-primary)' }}>
            Phase 4 — MVP Desktop Foundation
          </h4>
          <ul style={{ listStyle: 'none', padding: 0, display: 'grid', gap: '4px', fontSize: '13px' }}>
            {featureItems.map(item => (
              <li key={item.name} style={{ color: 'var(--color-text-secondary)' }}>
                <span style={{ color: item.done ? 'var(--color-success)' : 'var(--color-text-secondary)' }}>
                  {item.done ? '✓' : '⬚'}
                </span>{' '}
                {item.name}
              </li>
            ))}
          </ul>
        </div>

        {/* Status Section */}
        {!loading && (
          <div style={{ borderTop: '1px solid var(--color-border)', paddingTop: '16px', marginTop: '16px' }}>
            <h4 style={{ fontSize: '14px', marginBottom: '12px', color: 'var(--color-text-primary)' }}>
              Current Status
            </h4>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '12px' }}>
              <div style={{
                padding: '12px',
                background: 'rgba(34, 197, 94, 0.1)',
                borderRadius: '8px',
                textAlign: 'center',
              }}>
                <div style={{ fontSize: '18px', fontWeight: 600, color: 'var(--color-success)' }}>
                  {features.chat || true ? '✓' : '✗'}
                </div>
                <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>Chat</div>
              </div>
              <div style={{
                padding: '12px',
                background: 'rgba(34, 197, 94, 0.1)',
                borderRadius: '8px',
                textAlign: 'center',
              }}>
                <div style={{ fontSize: '18px', fontWeight: 600, color: 'var(--color-success)' }}>
                  {features.workspace || true ? '✓' : '✗'}
                </div>
                <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>Workspaces</div>
              </div>
              <div style={{
                padding: '12px',
                background: 'rgba(34, 197, 94, 0.1)',
                borderRadius: '8px',
                textAlign: 'center',
              }}>
                <div style={{ fontSize: '18px', fontWeight: 600, color: 'var(--color-success)' }}>
                  {features.knowledge || true ? '✓' : '✗'}
                </div>
                <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>Knowledge</div>
              </div>
            </div>
          </div>
        )}

        {/* Upcoming */}
        <div style={{ borderTop: '1px solid var(--color-border)', paddingTop: '16px', marginTop: '16px' }}>
          <h4 style={{ fontSize: '14px', marginBottom: '12px', color: '#fbbf24' }}>Upcoming (Phase 11+)</h4>
          <ul style={{ listStyle: 'none', padding: 0, display: 'grid', gap: '4px', fontSize: '13px' }}>
            {[
              'MCP Skills platform (gated)',
              'MCP integration for external tools',
              'Workflow automation engine',
              'Real-time collaboration',
              'Advanced VSS search',
            ].map(item => (
              <li key={item} style={{ color: 'var(--color-text-secondary)' }}>
                <span style={{ color: '#fbbf24' }}>⬚</span> {item}
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  );
}

export default About;