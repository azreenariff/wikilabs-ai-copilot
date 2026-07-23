import { useState, useEffect, useCallback } from 'react';

interface ChatMessage {
  id: string;
  role: string;
  content: string;
  created_at: string;
}

interface ActivityLog {
  timestamp: string;
  action: string;
  actor: string;
  hash: string;
  type: 'chat' | 'workspace' | 'knowledge' | 'skill' | 'settings' | 'system';
}

function Activity() {
  const [logs, setLogs] = useState<string[]>([]);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [auditLogs, setAuditLogs] = useState<ActivityLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<'chat' | 'system' | 'audit'>('chat');
  const [status] = useState('');
  const [filterType, setFilterType] = useState<string>('all');
  const [selectedMessage, setSelectedMessage] = useState<ChatMessage | null>(null);

  const fetchActivity = useCallback(async () => {
    try {
      // Fetch system logs
      const logsRes = await fetch('http://localhost:1420/api/commands/get_logs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: {} }),
      });
      const logsData = await logsRes.json();
      if (logsData.success && logsData.value) {
        setLogs(logsData.value as string[]);
      }

      // Fetch chat history (default workspace)
      const historyRes = await fetch('http://localhost:1420/api/commands/get_history', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: { workspace_id: 'default', limit: 50 } }),
      });
      const historyData = await historyRes.json();
      if (historyData.success && historyData.value) {
        setMessages(historyData.value as ChatMessage[]);
      }
    } catch (e) {
      console.error('Failed to load activity:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchActivity();
  }, [fetchActivity]);

  // Generate synthetic audit log from messages
  useEffect(() => {
    const audits: ActivityLog[] = messages.map((msg, i) => ({
      timestamp: msg.created_at,
      action: `${msg.role === 'user' ? 'User' : 'Assistant'} message sent`,
      actor: msg.role === 'user' ? 'User' : 'AI System',
      hash: `sha256:${i.toString().padStart(64, '0')}`,
      type: 'chat' as const,
    }));
    setAuditLogs(audits);
  }, [messages]);

  const formatDate = (date: string) => {
    try {
      return new Date(date).toLocaleString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      });
    } catch {
      return date;
    }
  };

  const getMessageIcon = (type: string) => {
    switch (type) {
      case 'chat': return '💬';
      case 'workspace': return '📁';
      case 'knowledge': return '📚';
      case 'skill': return '🔧';
      case 'settings': return '⚙️';
      case 'system': return '⚡';
      default: return '📋';
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'chat': return 'var(--color-accent)';
      case 'workspace': return '#f472b6';
      case 'knowledge': return '#34d399';
      case 'skill': return '#fbbf24';
      case 'settings': return '#a78bfa';
      case 'system': return 'var(--color-text-secondary)';
      default: return 'var(--color-text-secondary)';
    }
  };

  const filteredLogs = filterType === 'all'
    ? logs
    : logs.filter(log => log.toLowerCase().includes(filterType.toLowerCase()));

  const filteredMessages = messages;
  const filteredAudits = filterType === 'all'
    ? auditLogs
    : auditLogs.filter(a => a.type === filterType);

  const stats = {
    totalMessages: messages.length,
    userMessages: messages.filter(m => m.role === 'user').length,
    assistantMessages: messages.filter(m => m.role === 'assistant').length,
    totalAudits: auditLogs.length,
  };

  if (loading) {
    return (
      <div style={{ padding: '32px', maxWidth: '900px', margin: '0 auto', textAlign: 'center', color: 'var(--color-text-secondary)' }}>
        Loading activity...
      </div>
    );
  }

  return (
    <div style={{ padding: '32px', maxWidth: '900px', margin: '0 auto' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <h2 style={{ margin: 0, color: 'var(--color-text-primary)' }}>📊 Activity</h2>
        <button
          onClick={fetchActivity}
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

      {/* Stats */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '12px', marginBottom: '24px' }}>
        <div style={{
          background: 'var(--color-bg-secondary)',
          border: '1px solid var(--color-border)',
          borderRadius: '8px',
          padding: '16px',
          textAlign: 'center',
        }}>
          <div style={{ fontSize: '24px', fontWeight: 700, color: 'var(--color-accent)' }}>{stats.totalMessages}</div>
          <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>Total Messages</div>
        </div>
        <div style={{
          background: 'var(--color-bg-secondary)',
          border: '1px solid var(--color-border)',
          borderRadius: '8px',
          padding: '16px',
          textAlign: 'center',
        }}>
          <div style={{ fontSize: '24px', fontWeight: 700, color: 'var(--color-success)' }}>{stats.userMessages}</div>
          <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>User Messages</div>
        </div>
        <div style={{
          background: 'var(--color-bg-secondary)',
          border: '1px solid var(--color-border)',
          borderRadius: '8px',
          padding: '16px',
          textAlign: 'center',
        }}>
          <div style={{ fontSize: '24px', fontWeight: 700, color: '#a78bfa' }}>{stats.assistantMessages}</div>
          <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>AI Responses</div>
        </div>
        <div style={{
          background: 'var(--color-bg-secondary)',
          border: '1px solid var(--color-border)',
          borderRadius: '8px',
          padding: '16px',
          textAlign: 'center',
        }}>
          <div style={{ fontSize: '24px', fontWeight: 700, color: '#fbbf24' }}>{stats.totalAudits}</div>
          <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>Audit Entries</div>
        </div>
      </div>

      {/* Tabs */}
      <div style={{ display: 'flex', gap: '4px', marginBottom: '16px' }}>
        {(['chat', 'system', 'audit'] as const).map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            style={{
              padding: '6px 12px',
              borderRadius: '6px',
              border: 'none',
              background: activeTab === tab ? 'var(--color-accent)' : 'transparent',
              color: activeTab === tab ? 'white' : 'var(--color-text-secondary)',
              cursor: 'pointer',
              fontSize: '13px',
              fontWeight: activeTab === tab ? 600 : 400,
            }}
          >
            {tab === 'chat' ? '💬 Chat History' : tab === 'system' ? '⚡ System Logs' : '📋 Audit Log'}
          </button>
        ))}
      </div>

      {/* Filter */}
      <select
        value={filterType}
        onChange={e => setFilterType(e.target.value)}
        style={{
          width: '100%',
          padding: '8px 12px',
          borderRadius: '6px',
          border: '1px solid var(--color-border)',
          background: 'var(--color-bg-primary)',
          color: 'var(--color-text-primary)',
          fontSize: '13px',
          outline: 'none',
          marginBottom: '16px',
        }}
      >
        <option value="all">All Types</option>
        <option value="chat">Chat</option>
        <option value="workspace">Workspace</option>
        <option value="knowledge">Knowledge</option>
        <option value="skill">Skill</option>
        <option value="settings">Settings</option>
        <option value="system">System</option>
      </select>

      {/* Chat History Tab */}
      {activeTab === 'chat' && (
        <div style={{ display: 'grid', gap: '8px' }}>
          {filteredMessages.length === 0 ? (
            <div style={{
              textAlign: 'center',
              padding: '48px',
              color: 'var(--color-text-secondary)',
              background: 'var(--color-bg-secondary)',
              borderRadius: '12px',
              border: '1px solid var(--color-border)',
            }}>
              <p style={{ fontSize: '16px' }}>No chat messages yet</p>
              <p style={{ fontSize: '13px', marginTop: '8px' }}>
                Start a conversation in the Assistant tab to see messages here.
              </p>
            </div>
          ) : (
            filteredMessages.map(msg => (
              <div
                key={msg.id}
                onClick={() => setSelectedMessage(selectedMessage?.id === msg.id ? null : msg)}
                style={{
                  background: 'var(--color-bg-secondary)',
                  border: `1px solid ${selectedMessage?.id === msg.id ? 'var(--color-accent)' : 'var(--color-border)'}`,
                  borderRadius: '8px',
                  padding: '12px',
                  cursor: 'pointer',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
                  <span style={{
                    fontSize: '11px',
                    padding: '2px 6px',
                    borderRadius: '4px',
                    background: msg.role === 'user' ? 'rgba(99, 102, 241, 0.15)' : 'rgba(34, 197, 94, 0.15)',
                    color: msg.role === 'user' ? 'var(--color-accent)' : 'var(--color-success)',
                    fontWeight: 600,
                  }}>
                    {msg.role === 'user' ? '👤 User' : '🤖 Assistant'}
                  </span>
                  <span style={{ fontSize: '11px', color: 'var(--color-text-secondary)' }}>
                    {formatDate(msg.created_at)}
                  </span>
                </div>
                <p style={{ fontSize: '13px', color: 'var(--color-text-primary)', margin: 0, lineHeight: 1.5 }}>
                  {msg.content.length > 100 ? msg.content.substring(0, 100) + '...' : msg.content}
                </p>
                {selectedMessage?.id === msg.id && (
                  <div style={{ marginTop: '8px', paddingTop: '8px', borderTop: '1px solid var(--color-border)', fontSize: '12px', color: 'var(--color-text-secondary)' }}>
                    <strong>ID:</strong> {msg.id}<br />
                    <strong>Role:</strong> {msg.role}<br />
                    <strong>Full Content:</strong><br />
                    <pre style={{ whiteSpace: 'pre-wrap', fontSize: '12px', marginTop: '4px' }}>{msg.content}</pre>
                  </div>
                )}
              </div>
            ))
          )}
        </div>
      )}

      {/* System Logs Tab */}
      {activeTab === 'system' && (
        <div style={{
          background: 'var(--color-bg-secondary)',
          border: '1px solid var(--color-border)',
          borderRadius: '12px',
          padding: '16px',
          fontFamily: 'monospace',
          fontSize: '12px',
          lineHeight: 1.6,
          maxHeight: '400px',
          overflow: 'auto',
        }}>
          {filteredLogs.length === 0 ? (
            <div style={{ color: 'var(--color-text-secondary)', textAlign: 'center', padding: '24px' }}>
              No system logs available
            </div>
          ) : (
            filteredLogs.map((log, i) => (
              <div key={i} style={{ padding: '2px 0', borderBottom: '1px solid rgba(255,255,255,0.05)' }}>
                <span style={{ color: 'var(--color-accent)' }}>{i + 1}.</span> {log}
              </div>
            ))
          )}
        </div>
      )}

      {/* Audit Log Tab */}
      {activeTab === 'audit' && (
        <div style={{ display: 'grid', gap: '8px' }}>
          {filteredAudits.length === 0 ? (
            <div style={{
              textAlign: 'center',
              padding: '48px',
              color: 'var(--color-text-secondary)',
              background: 'var(--color-bg-secondary)',
              borderRadius: '12px',
              border: '1px solid var(--color-border)',
            }}>
              <p style={{ fontSize: '16px' }}>No audit entries yet</p>
              <p style={{ fontSize: '13px', marginTop: '8px' }}>
                Audit entries are generated from activity.
              </p>
            </div>
          ) : (
            filteredAudits.map((audit, i) => (
              <div
                key={i}
                style={{
                  background: 'var(--color-bg-secondary)',
                  border: '1px solid var(--color-border)',
                  borderRadius: '8px',
                  padding: '12px',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <span style={{ fontSize: '16px' }}>{getMessageIcon(audit.type)}</span>
                    <span style={{ fontSize: '13px', fontWeight: 500, color: getTypeColor(audit.type) }}>
                      {audit.action}
                    </span>
                  </div>
                  <span style={{ fontSize: '11px', color: 'var(--color-text-secondary)' }}>
                    {formatDate(audit.timestamp)}
                  </span>
                </div>
                <div style={{ fontSize: '11px', color: 'var(--color-text-secondary)', marginTop: '4px' }}>
                  Actor: {audit.actor}
                </div>
                <div style={{
                  fontSize: '10px',
                  color: 'var(--color-text-secondary)',
                  marginTop: '4px',
                  fontFamily: 'monospace',
                  opacity: 0.5,
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                }}>
                  {audit.hash}
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}

export default Activity;