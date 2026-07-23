import { useState, useEffect, useCallback } from 'react';

interface Workspace {
  id: string;
  name: string;
  customer_name: string;
  technology_stack: string[];
  created_at: string;
  updated_at: string;
}

function Workspaces() {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState('');
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newName, setNewName] = useState('');
  const [newCustomer, setNewCustomer] = useState('');
  const [selectedWs, setSelectedWs] = useState<Workspace | null>(null);

  const fetchWorkspaces = useCallback(async () => {
    try {
      const res = await fetch('http://localhost:1420/api/commands/get_workspace_list', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: {} }),
      });
      const data = await res.json();
      if (data.success && data.value) {
        // Backend returns just names, fetch details via get_conversations
        const names = data.value as string[];
        const details = await Promise.all(
          names.map(async (name) => {
            return {
              id: name.toLowerCase().replace(/\s+/g, '-'),
              name,
              customer_name: name,
              technology_stack: [],
              created_at: 'N/A',
              updated_at: 'N/A',
            };
          })
        );
        setWorkspaces(details);
      }
    } catch (e) {
      console.error('Failed to load workspaces:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchWorkspaces();
  }, [fetchWorkspaces]);

  const createWorkspace = async () => {
    if (!newName.trim() || !newCustomer.trim()) {
      setStatus('Name and customer name are required');
      return;
    }
    try {
      const res = await fetch('http://localhost:1420/api/commands/create_workspace', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          params: { name: newName.trim(), customer_name: newCustomer.trim() },
        }),
      });
      const data = await res.json();
      if (data.success && data.value) {
        setStatus(`Workspace "${newName}" created`);
        setNewName('');
        setNewCustomer('');
        setShowCreateForm(false);
        fetchWorkspaces();
      } else {
        setStatus(`Error: ${data.error}`);
      }
    } catch {
      setStatus('Failed to create workspace');
    }
  };

  const formatDate = (date: string) => {
    if (date === 'N/A') return 'N/A';
    try {
      return new Date(date).toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      });
    } catch {
      return date;
    }
  };

  return (
    <div style={{ padding: '32px', maxWidth: '900px', margin: '0 auto' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <h2 style={{ margin: 0, color: 'var(--color-text-primary)' }}>📁 Workspaces</h2>
        <div style={{ display: 'flex', gap: '8px' }}>
          <button
            onClick={() => setShowCreateForm(!showCreateForm)}
            style={{
              padding: '6px 12px',
              borderRadius: '6px',
              border: 'none',
              background: showCreateForm ? 'transparent' : 'var(--color-accent)',
              color: showCreateForm ? 'var(--color-text-primary)' : 'white',
              cursor: 'pointer',
              fontSize: '13px',
              fontWeight: 500,
            }}
          >
            + New Workspace
          </button>
          <button
            onClick={fetchWorkspaces}
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

      {/* Create Form */}
      {showCreateForm && (
        <div style={{
          background: 'var(--color-bg-secondary)',
          border: '1px solid var(--color-accent)',
          borderRadius: '12px',
          padding: '20px',
          marginBottom: '16px',
        }}>
          <h3 style={{ fontSize: '15px', margin: '0 0 12px', color: 'var(--color-text-primary)' }}>
            Create New Workspace
          </h3>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px', marginBottom: '12px' }}>
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--color-text-secondary)', marginBottom: '4px' }}>
                Workspace Name
              </label>
              <input
                type="text"
                placeholder="e.g., Acme Corp Project"
                value={newName}
                onChange={e => setNewName(e.target.value)}
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
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--color-text-secondary)', marginBottom: '4px' }}>
                Customer Name
              </label>
              <input
                type="text"
                placeholder="e.g., Acme Corp"
                value={newCustomer}
                onChange={e => setNewCustomer(e.target.value)}
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
          </div>
          <div style={{ display: 'flex', gap: '8px' }}>
            <button
              onClick={createWorkspace}
              style={{
                padding: '8px 16px',
                borderRadius: '6px',
                border: 'none',
                background: 'var(--color-accent)',
                color: 'white',
                fontSize: '13px',
                fontWeight: 500,
                cursor: 'pointer',
              }}
            >
              Create
            </button>
            <button
              onClick={() => setShowCreateForm(false)}
              style={{
                padding: '8px 16px',
                borderRadius: '6px',
                border: '1px solid var(--color-border)',
                background: 'transparent',
                color: 'var(--color-text-primary)',
                fontSize: '13px',
                cursor: 'pointer',
              }}
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {loading ? (
        <div style={{ textAlign: 'center', padding: '48px', color: 'var(--color-text-secondary)' }}>
          Loading workspaces...
        </div>
      ) : workspaces.length === 0 ? (
        <div style={{
          background: 'var(--color-bg-secondary)',
          border: '1px solid var(--color-border)',
          borderRadius: '12px',
          padding: '48px',
          textAlign: 'center',
          color: 'var(--color-text-secondary)',
        }}>
          <p style={{ fontSize: '16px' }}>No workspaces yet</p>
          <p style={{ fontSize: '13px', marginTop: '8px' }}>
            Create your first workspace to organize AI context per customer or project.
          </p>
        </div>
      ) : (
        <div style={{ display: 'grid', gap: '12px' }}>
          {workspaces.map(ws => (
            <div
              key={ws.id}
              onClick={() => setSelectedWs(selectedWs?.id === ws.id ? null : ws)}
              style={{
                background: 'var(--color-bg-secondary)',
                border: `1px solid ${selectedWs?.id === ws.id ? 'var(--color-accent)' : 'var(--color-border)'}`,
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
                      {ws.name}
                    </h3>
                    <span style={{
                      fontSize: '11px',
                      padding: '2px 6px',
                      borderRadius: '4px',
                      background: 'rgba(99, 102, 241, 0.15)',
                      color: 'var(--color-accent)',
                    }}>
                      Workspace
                    </span>
                  </div>
                  <p style={{ fontSize: '13px', color: 'var(--color-text-secondary)', margin: '4px 0' }}>
                    Customer: {ws.customer_name}
                  </p>
                  <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>
                    Created: {formatDate(ws.created_at)} • Updated: {formatDate(ws.updated_at)}
                  </div>
                </div>
                <div style={{ color: 'var(--color-text-secondary)', fontSize: '14px' }}>
                  {selectedWs?.id === ws.id ? '▲' : '▼'}
                </div>
              </div>

              {/* Expanded details */}
              {selectedWs?.id === ws.id && (
                <div style={{
                  marginTop: '12px',
                  paddingTop: '12px',
                  borderTop: '1px solid var(--color-border)',
                  fontSize: '13px',
                }}>
                  <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px', marginBottom: '12px' }}>
                    <div style={{ color: 'var(--color-text-secondary)' }}>
                      <strong>ID:</strong> {ws.id}
                    </div>
                    <div style={{ color: 'var(--color-text-secondary)' }}>
                      <strong>Technology:</strong> {ws.technology_stack.length > 0 ? ws.technology_stack.join(', ') : 'Not specified'}
                    </div>
                  </div>
                  <div style={{ display: 'flex', gap: '8px' }}>
                    <button
                      onClick={e => { e.stopPropagation(); }}
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
                      Open in Chat
                    </button>
                    <button
                      onClick={e => { e.stopPropagation(); }}
                      style={{
                        padding: '4px 12px',
                        borderRadius: '4px',
                        border: '1px solid var(--color-border)',
                        background: 'transparent',
                        color: 'var(--color-error)',
                        fontSize: '12px',
                        cursor: 'pointer',
                      }}
                    >
                      Delete
                    </button>
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Info */}
      <div style={{
        marginTop: '24px',
        padding: '16px',
        background: 'var(--color-bg-secondary)',
        border: '1px solid var(--color-border)',
        borderRadius: '12px',
        fontSize: '13px',
        color: 'var(--color-text-secondary)',
      }}>
        <h4 style={{ fontSize: '14px', color: 'var(--color-text-primary)', margin: '0 0 8px' }}>
          About Workspaces
        </h4>
        <p style={{ margin: 0, lineHeight: 1.5 }}>
          Workspaces isolate AI context per customer or project. Each workspace maintains its own chat history,
          knowledge base, and AI configuration. Use workspaces to manage multiple clients or projects
          independently.
        </p>
      </div>
    </div>
  );
}

export default Workspaces;