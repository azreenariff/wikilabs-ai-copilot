import { useState, useEffect, useCallback } from 'react';

interface PackInfo {
  name: string;
  version: string;
  description: string;
  author: string;
  license: string;
  document_count: number;
  embedding_model: string;
  embedding_dimensions: number;
  enabled: boolean;
  indexed: boolean;
  last_indexed: string | null;
  validation_status: string;
  validation_errors: number;
  validation_warnings: number;
  path: string;
  tags: string[];
  categories: string[];
  sdk_created: boolean;
}

interface ValidationReport {
  pack_name: string;
  validated: boolean;
  errors: string[];
  warnings: string[];
  checks_performed: number;
  checks_passed: number;
}

function Knowledge() {
  const [packs, setPacks] = useState<PackInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedPack, setSelectedPack] = useState<PackInfo | null>(null);
  const [validation, setValidation] = useState<ValidationReport | null>(null);

  const fetchPacks = useCallback(async () => {
    try {
      const res = await fetch('http://localhost:1420/api/commands/knowledge_list_packs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: {} }),
      });
      if (res.ok) {
        const data = await res.json();
        if (data.success && data.value) {
          setPacks(data.value);
        }
      }
    } catch (e) {
      console.error('Failed to load knowledge packs:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchPacks();
  }, [fetchPacks]);

  const togglePack = async (name: string) => {
    const pack = packs.find(p => p.name === name);
    if (!pack) return;
    try {
      const cmd = pack.enabled ? 'knowledge_disable_pack' : 'knowledge_enable_pack';
      const res = await fetch(`http://localhost:1420/api/commands/${cmd}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: { name } }),
      });
      const data = await res.json();
      if (data.success) {
        setStatus(pack.enabled ? `Disabled ${name}` : `Enabled ${name}`);
        fetchPacks();
        if (selectedPack?.name === name) {
          setSelectedPack(prev => prev ? { ...prev, enabled: !prev.enabled } : null);
        }
      } else {
        setStatus(`Error: ${data.error}`);
      }
    } catch (e) {
      setStatus('Failed to toggle pack');
    }
  };

  const reindexPack = async (name: string) => {
    try {
      const res = await fetch('http://localhost:1420/api/commands/knowledge_reindex_pack', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: { name } }),
      });
      const data = await res.json();
      if (data.success) {
        setStatus(`Reindexed ${name}`);
        fetchPacks();
      } else {
        setStatus(`Error: ${data.error}`);
      }
    } catch {
      setStatus('Failed to reindex');
    }
  };

  const checkValidation = async (name: string) => {
    try {
      const res = await fetch('http://localhost:1420/api/commands/knowledge_get_validation_report', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: { name } }),
      });
      const data = await res.json();
      if (data.success && data.value) {
        setValidation(data.value);
      }
    } catch {
      setStatus('Failed to get validation report');
    }
  };

  const filteredPacks = searchQuery
    ? packs.filter(p =>
        p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        p.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        p.tags.some(t => t.toLowerCase().includes(searchQuery.toLowerCase()))
      )
    : packs;

  const getValidationColor = (status: string) => {
    switch (status) {
      case 'VALID': return 'var(--color-success)';
      case 'ERROR': return 'var(--color-error)';
      case 'WARNING': return '#fbbf24';
      default: return 'var(--color-text-secondary)';
    }
  };

  return (
    <div style={{ padding: '32px', maxWidth: '900px', margin: '0 auto' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <h2 style={{ margin: 0, color: 'var(--color-text-primary)' }}>📚 Knowledge Packs</h2>
        <button
          onClick={fetchPacks}
          style={{
            padding: '6px 12px',
            borderRadius: '6px',
            border: '1px solid var(--color-border)',
            background: 'transparent',
            color: 'var(--color-text-primary)',
            cursor: 'pointer',
            fontSize: '13px',
          }}
        >
          ↻ Refresh
        </button>
      </div>

      {status && (
        <div style={{
          padding: '8px 12px',
          borderRadius: '6px',
          fontSize: '13px',
          marginBottom: '12px',
          background: 'rgba(99, 102, 241, 0.1)',
          color: 'var(--color-accent)',
        }}>
          {status}
        </div>
      )}

      {/* Search */}
      <div style={{ marginBottom: '16px' }}>
        <input
          type="text"
          placeholder="Search knowledge packs..."
          value={searchQuery}
          onChange={e => setSearchQuery(e.target.value)}
          style={{
            width: '100%',
            padding: '8px 12px',
            borderRadius: '6px',
            border: '1px solid var(--color-border)',
            background: 'var(--color-bg-primary)',
            color: 'var(--color-text-primary)',
            fontSize: '13px',
            outline: 'none',
            boxSizing: 'border-box',
          }}
        />
      </div>

      {loading ? (
        <div style={{ textAlign: 'center', padding: '48px', color: 'var(--color-text-secondary)' }}>
          Loading knowledge packs...
        </div>
      ) : packs.length === 0 ? (
        <div style={{
          background: 'var(--color-bg-secondary)',
          border: '1px solid var(--color-border)',
          borderRadius: '12px',
          padding: '48px',
          textAlign: 'center',
          color: 'var(--color-text-secondary)',
        }}>
          <p style={{ fontSize: '16px' }}>No knowledge packs loaded</p>
          <p style={{ fontSize: '13px', marginTop: '8px' }}>Knowledge packs are stored in your knowledge directory.</p>
        </div>
      ) : (
        <div style={{ display: 'grid', gap: '12px' }}>
          {filteredPacks.map(pack => (
            <div
              key={pack.name}
              onClick={() => setSelectedPack(selectedPack?.name === pack.name ? null : pack)}
              style={{
                background: 'var(--color-bg-secondary)',
                border: `1px solid ${selectedPack?.name === pack.name ? 'var(--color-accent)' : 'var(--color-border)'}`,
                borderRadius: '12px',
                padding: '16px',
                cursor: 'pointer',
                transition: 'all 0.15s',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                <div style={{ flex: 1 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
                    <h3 style={{ fontSize: '15px', fontWeight: 600, margin: 0, color: 'var(--color-text-primary)' }}>
                      {pack.name}
                    </h3>
                    <span style={{
                      fontSize: '11px',
                      padding: '2px 6px',
                      borderRadius: '4px',
                      background: pack.enabled ? 'rgba(34, 197, 94, 0.15)' : 'rgba(107, 114, 128, 0.15)',
                      color: pack.enabled ? 'var(--color-success)' : 'var(--color-text-secondary)',
                    }}>
                      {pack.enabled ? '✓ Enabled' : 'Disabled'}
                    </span>
                    <span style={{
                      fontSize: '11px',
                      padding: '2px 6px',
                      borderRadius: '4px',
                      background: pack.indexed ? 'rgba(99, 102, 241, 0.15)' : 'rgba(107, 114, 128, 0.15)',
                      color: pack.indexed ? 'var(--color-accent)' : 'var(--color-text-secondary)',
                    }}>
                      {pack.indexed ? 'Indexed' : 'Not Indexed'}
                    </span>
                    <span style={{
                      fontSize: '11px',
                      padding: '2px 6px',
                      borderRadius: '4px',
                      background: `${getValidationColor(pack.validation_status)}20`,
                      color: getValidationColor(pack.validation_status),
                    }}>
                      {pack.validation_status}
                    </span>
                  </div>
                  <p style={{ fontSize: '13px', color: 'var(--color-text-secondary)', margin: '4px 0 8px' }}>
                    {pack.description}
                  </p>
                  <div style={{ display: 'flex', gap: '12px', fontSize: '12px', color: 'var(--color-text-secondary)' }}>
                    <span>📄 {pack.document_count} docs</span>
                    <span>🏷️ {pack.author}</span>
                    <span>📦 v{pack.version}</span>
                    <span>🤖 {pack.embedding_model}</span>
                  </div>
                  {pack.tags.length > 0 && (
                    <div style={{ display: 'flex', gap: '4px', marginTop: '8px', flexWrap: 'wrap' }}>
                      {pack.tags.map(tag => (
                        <span key={tag} style={{
                          fontSize: '11px',
                          padding: '2px 6px',
                          borderRadius: '4px',
                          background: 'var(--color-bg-tertiary)',
                          color: 'var(--color-text-secondary)',
                        }}>
                          {tag}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              </div>

              {/* Expanded details */}
              {selectedPack?.name === pack.name && (
                <div style={{
                  marginTop: '12px',
                  paddingTop: '12px',
                  borderTop: '1px solid var(--color-border)',
                  display: 'grid',
                  gridTemplateColumns: '1fr 1fr',
                  gap: '8px',
                }}>
                  <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>
                    <strong>Embedding:</strong> {pack.embedding_model} ({pack.embedding_dimensions}d)
                  </div>
                  <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>
                    <strong>License:</strong> {pack.license}
                  </div>
                  <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>
                    <strong>Path:</strong> {pack.path}
                  </div>
                  <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>
                    <strong>Last Indexed:</strong> {pack.last_indexed || 'Never'}
                  </div>
                  {pack.validation_errors > 0 && (
                    <div style={{ fontSize: '12px', color: 'var(--color-error)' }}>
                      ⚠ {pack.validation_errors} errors, {pack.validation_warnings} warnings
                    </div>
                  )}
                  <div style={{ display: 'flex', gap: '8px', gridColumn: '1 / -1', marginTop: '4px' }}>
                    <button
                      onClick={e => { e.stopPropagation(); togglePack(pack.name); }}
                      style={{
                        padding: '4px 12px',
                        borderRadius: '4px',
                        border: 'none',
                        background: pack.enabled ? 'var(--color-error)' : 'var(--color-success)',
                        color: 'white',
                        fontSize: '12px',
                        cursor: 'pointer',
                      }}
                    >
                      {pack.enabled ? 'Disable' : 'Enable'}
                    </button>
                    <button
                      onClick={e => { e.stopPropagation(); reindexPack(pack.name); }}
                      style={{
                        padding: '4px 12px',
                        borderRadius: '4px',
                        border: '1px solid var(--color-border)',
                        background: 'transparent',
                        color: 'var(--color-text-primary)',
                        fontSize: '12px',
                        cursor: 'pointer',
                      }}
                    >
                      Reindex
                    </button>
                    <button
                      onClick={e => { e.stopPropagation(); checkValidation(pack.name); }}
                      style={{
                        padding: '4px 12px',
                        borderRadius: '4px',
                        border: '1px solid var(--color-border)',
                        background: 'transparent',
                        color: 'var(--color-text-primary)',
                        fontSize: '12px',
                        cursor: 'pointer',
                      }}
                    >
                      Validate
                    </button>
                  </div>
                </div>
              )}
            </div>
          ))}

          {filteredPacks.length === 0 && (
            <div style={{
              textAlign: 'center',
              padding: '24px',
              color: 'var(--color-text-secondary)',
              fontSize: '13px',
            }}>
              No packs match "{searchQuery}"
            </div>
          )}
        </div>
      )}

      {/* Validation Report */}
      {validation && (
        <div style={{
          marginTop: '16px',
          background: 'var(--color-bg-secondary)',
          border: '1px solid var(--color-border)',
          borderRadius: '12px',
          padding: '16px',
        }}>
          <h4 style={{ fontSize: '14px', marginBottom: '8px' }}>Validation Report: {validation.pack_name}</h4>
          <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)', marginBottom: '8px' }}>
            {validation.checks_passed}/{validation.checks_performed} checks passed ({validation.validated ? '✓ Valid' : '✗ Invalid'})
          </div>
          {validation.errors.length > 0 && (
            <div style={{ marginBottom: '4px' }}>
              <strong style={{ color: 'var(--color-error)', fontSize: '12px' }}>Errors:</strong>
              <ul style={{ margin: '4px 0', padding: '0 0 0 20px', fontSize: '12px', color: 'var(--color-error)' }}>
                {validation.errors.map((e, i) => <li key={i}>{e}</li>)}
              </ul>
            </div>
          )}
          {validation.warnings.length > 0 && (
            <div>
              <strong style={{ color: '#fbbf24', fontSize: '12px' }}>Warnings:</strong>
              <ul style={{ margin: '4px 0', padding: '0 0 0 20px', fontSize: '12px', color: '#fbbf24' }}>
                {validation.warnings.map((w, i) => <li key={i}>{w}</li>)}
              </ul>
            </div>
          )}
          <button
            onClick={() => setValidation(null)}
            style={{
              marginTop: '8px',
              padding: '4px 12px',
              borderRadius: '4px',
              border: '1px solid var(--color-border)',
              background: 'transparent',
              color: 'var(--color-text-primary)',
              fontSize: '12px',
              cursor: 'pointer',
            }}
          >
            Close
          </button>
        </div>
      )}
    </div>
  );
}

export default Knowledge;