import { useState, useEffect, useCallback } from 'react';

interface RecommendationCard {
  id: string;
  title: string;
  description: string;
  risk_level: string;
  status: string;
  created_at: string;
}

interface EvidenceStatus {
  collected_count: number;
  missing_count: number;
  collected: any[];
  missing: any[];
  confidence: number;
  is_sufficient: boolean;
}

interface ObservationStatus {
  observation_enabled: boolean;
  status: string;
  providers: string[];
}

interface AdviceMessage {
  id: string;
  text: string;
  timestamp: string;
  type: 'suggestion' | 'error' | 'info';
}

function Guidance() {
  const [recommendations, setRecommendations] = useState<RecommendationCard[]>([]);
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState('');
  const [selectedRec, setSelectedRec] = useState<RecommendationCard | null>(null);
  const [evidence, setEvidence] = useState<EvidenceStatus | null>(null);
  const [obsStatus, setObsStatus] = useState<ObservationStatus | null>(null);
  const [obsLoading, setObsLoading] = useState(true);
  const [adviceMessages, setAdviceMessages] = useState<AdviceMessage[]>([]);
  const [activeTab, setActiveTab] = useState<'chat' | 'cards'>('chat');

  const fetchRecommendations = useCallback(async () => {
    try {
      const res = await fetch('http://127.0.0.1:1420/api/commands/guidance_get_active_recommendations', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: {} }),
      });
      const data = await res.json();
      if (data.success && data.value) {
        setRecommendations(data.value);
      }
    } catch (e) {
      console.error('Failed to load recommendations:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchEvidence = useCallback(async () => {
    try {
      const res = await fetch('http://127.0.0.1:1420/api/commands/guidance_get_evidence_status', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: {} }),
      });
      const data = await res.json();
      if (data.success && data.value) {
        setEvidence(data.value);
      }
    } catch {}
  }, []);

  const fetchObsStatus = useCallback(async () => {
    try {
      const res = await fetch('http://127.0.0.1:1420/api/commands/observation_get_status', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: {} }),
      });
      const data = await res.json();
      if (data.success && data.value) {
        setObsStatus(data.value);
      }
    } catch {
      setObsStatus(null);
    } finally {
      setObsLoading(false);
    }
  }, []);

  // Poll for new advice messages from the observation loop
  useEffect(() => {
    const poll = () => {
      fetchRecommendations();
      fetchEvidence();
      fetchObsStatus();
    };

    poll();

    const interval = setInterval(poll, 10000);

    return () => clearInterval(interval);
  }, [fetchRecommendations, fetchEvidence, fetchObsStatus]);

  const dismissRec = async (recId: string) => {
    try {
      const res = await fetch('http://127.0.0.1:1420/api/commands/guidance_dismiss_recommendation', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: { rec_id: recId } }),
      });
      const data = await res.json();
      if (data.success) {
        setStatus('Recommendation dismissed');
        fetchRecommendations();
      }
    } catch {
      setStatus('Failed to dismiss');
    }
  };

  const clearAdvice = () => {
    setAdviceMessages([]);
  };

  const getRiskColor = (level: string) => {
    switch (level) {
      case 'Critical': return 'var(--color-error)';
      case 'High': return '#f97316';
      case 'Medium': return '#fbbf24';
      case 'Low': return '#4ade80';
      default: return 'var(--color-text-secondary)';
    }
  };

  const getStatusBadge = (rec: RecommendationCard) => {
    const color = rec.status === 'Active' ? 'var(--color-accent)' :
      rec.status === 'Accepted' ? 'var(--color-success)' :
      rec.status === 'Rejected' ? 'var(--color-error)' : 'var(--color-text-secondary)';
    return <span style={{ fontSize: '11px', padding: '2px 6px', borderRadius: '4px', background: `${color}20`, color }}>{rec.status}</span>;
  };

  const formatTime = (iso: string) => {
    try {
      const d = new Date(iso);
      return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } catch {
      return '';
    }
  };

  const getMessageIcon = (type: string) => {
    switch (type) {
      case 'error': return '🔴';
      case 'suggestion': return '💡';
      case 'info': return 'ℹ️';
      default: return '💡';
    }
  };

  return (
    <div style={{ padding: '32px', maxWidth: '900px', margin: '0 auto' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <h2 style={{ margin: 0, color: 'var(--color-text-primary)' }}>🧭 Guidance Engine</h2>
        <button onClick={fetchRecommendations} style={{ padding: '6px 12px', borderRadius: '6px', border: '1px solid var(--color-border)', background: 'transparent', color: 'var(--color-text-primary)', cursor: 'pointer', fontSize: '13px' }}>↻ Refresh</button>
      </div>

      {status && (
        <div style={{ padding: '8px 12px', borderRadius: '6px', fontSize: '13px', marginBottom: '12px', background: 'rgba(99, 102, 241, 0.1)', color: 'var(--color-accent)' }}>
          {status}
        </div>
      )}

      {/* Observation Status */}
      <div style={{ background: 'var(--color-bg-secondary)', border: '1px solid var(--color-border)', borderRadius: '12px', padding: '16px', marginBottom: '16px' }}>
        <h3 style={{ fontSize: '14px', margin: '0 0 8px', color: 'var(--color-text-primary)' }}>🔍 Observation Status</h3>
        {obsLoading ? (
          <div style={{ fontSize: '13px', color: 'var(--color-text-secondary)' }}>Loading observation status...</div>
        ) : obsStatus ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', fontSize: '13px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span style={{ display: 'inline-block', width: '8px', height: '8px', borderRadius: '50%', background: obsStatus.status === 'active' ? 'var(--color-success)' : 'var(--color-error)' }} />
              <span style={{ color: 'var(--color-text-primary)', fontWeight: 600 }}>{obsStatus.status === 'active' ? 'Active' : 'Inactive'}</span>
              {obsStatus.observation_enabled && <span style={{ color: 'var(--color-text-secondary)', fontSize: '12px' }}>(Engine running)</span>}
            </div>
            <div>
              <span style={{ color: 'var(--color-text-secondary)' }}>Providers: </span>
              <span style={{ color: 'var(--color-text-primary)' }}>
                {obsStatus.providers.map(p => (
                  <span key={p} style={{ display: 'inline-block', padding: '2px 8px', margin: '2px 4px', borderRadius: '4px', background: 'rgba(99, 102, 241, 0.1)', color: 'var(--color-accent)', fontSize: '12px' }}>
                    {p}
                  </span>
                ))}
              </span>
            </div>
          </div>
        ) : (
          <div style={{ fontSize: '13px', color: 'var(--color-text-secondary)' }}>Observation engine not reachable</div>
        )}
      </div>

      {/* View Toggle */}
      <div style={{ display: 'flex', gap: '4px', marginBottom: '16px' }}>
        <button onClick={() => setActiveTab('chat')} style={{
          padding: '8px 16px', borderRadius: '8px', border: 'none', fontSize: '13px', cursor: 'pointer',
          background: activeTab === 'chat' ? 'var(--color-accent)' : 'var(--color-bg-tertiary)',
          color: activeTab === 'chat' ? 'white' : 'var(--color-text-primary)',
        }}>
          💬 Advice Chat {adviceMessages.length > 0 && `(${adviceMessages.length})`}
        </button>
        <button onClick={() => setActiveTab('cards')} style={{
          padding: '8px 16px', borderRadius: '8px', border: 'none', fontSize: '13px', cursor: 'pointer',
          background: activeTab === 'cards' ? 'var(--color-accent)' : 'var(--color-bg-tertiary)',
          color: activeTab === 'cards' ? 'white' : 'var(--color-text-primary)',
        }}>
          📋 Recommendation Cards {recommendations.length > 0 && `(${recommendations.length})`}
        </button>
      </div>

      {/* Evidence Status */}
      {evidence && (
        <div style={{ background: 'var(--color-bg-secondary)', border: '1px solid var(--color-border)', borderRadius: '12px', padding: '16px', marginBottom: '16px' }}>
          <h3 style={{ fontSize: '14px', margin: '0 0 8px', color: 'var(--color-text-primary)' }}>Evidence Status</h3>
          <div style={{ display: 'flex', gap: '16px', fontSize: '13px' }}>
            <span style={{ color: 'var(--color-success)' }}>✅ Collected: {evidence.collected_count}</span>
            <span style={{ color: 'var(--color-error)' }}>❌ Missing: {evidence.missing_count}</span>
            <span style={{ color: 'var(--color-text-secondary)' }}>📊 Confidence: {Math.round(evidence.confidence * 100)}%</span>
          </div>
        </div>
      )}

      {/* Advice Chat View */}
      {activeTab === 'chat' && (
        <div style={{ background: 'var(--color-bg-secondary)', border: '1px solid var(--color-border)', borderRadius: '12px', padding: '16px', marginBottom: '16px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
            <h3 style={{ fontSize: '14px', margin: 0, color: 'var(--color-text-primary)' }}>💬 Live Advice</h3>
            {adviceMessages.length > 0 && (
              <button onClick={clearAdvice} style={{
                padding: '4px 12px', borderRadius: '4px', border: '1px solid var(--color-border)',
                background: 'transparent', color: 'var(--color-text-secondary)', fontSize: '12px', cursor: 'pointer',
              }}>
                Clear
              </button>
            )}
          </div>
          {adviceMessages.length === 0 ? (
            <div style={{ textAlign: 'center', padding: '32px', color: 'var(--color-text-secondary)', fontSize: '13px' }}>
              <p>Waiting for observations...</p>
              <p style={{ fontSize: '12px', marginTop: '4px' }}>The guidance engine watches your activity and provides suggestions in real-time.</p>
            </div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', maxHeight: '400px', overflowY: 'auto' }}>
              {adviceMessages.map(msg => (
                <div key={msg.id} style={{
                  background: msg.type === 'error' ? 'rgba(239, 68, 68, 0.1)' : 'rgba(99, 102, 241, 0.1)',
                  border: `1px solid ${msg.type === 'error' ? 'rgba(239, 68, 68, 0.2)' : 'rgba(99, 102, 241, 0.2)'}`,
                  borderRadius: '8px', padding: '12px',
                }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
                    <span>{getMessageIcon(msg.type)}</span>
                    <span style={{ fontSize: '11px', color: 'var(--color-text-secondary)' }}>{formatTime(msg.timestamp)}</span>
                  </div>
                  <p style={{ fontSize: '13px', margin: 0, color: 'var(--color-text-primary)', lineHeight: 1.4 }}>{msg.text}</p>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Recommendation Cards View */}
      {activeTab === 'cards' && (
        loading ? (
          <div style={{ textAlign: 'center', padding: '48px', color: 'var(--color-text-secondary)' }}>Loading recommendations...</div>
        ) : recommendations.length === 0 ? (
          <div style={{ background: 'var(--color-bg-secondary)', border: '1px solid var(--color-border)', borderRadius: '12px', padding: '48px', textAlign: 'center', color: 'var(--color-text-secondary)' }}>
            <p style={{ fontSize: '16px' }}>No active recommendations</p>
            <p style={{ fontSize: '13px', marginTop: '8px' }}>The guidance engine will provide recommendations as it observes your activity.</p>
          </div>
        ) : (
          <div style={{ display: 'grid', gap: '12px' }}>
            {recommendations.map(rec => (
              <div key={rec.id} onClick={() => setSelectedRec(selectedRec?.id === rec.id ? null : rec)} style={{
                background: 'var(--color-bg-secondary)', border: `1px solid ${selectedRec?.id === rec.id ? 'var(--color-accent)' : 'var(--color-border)'}`,
                borderRadius: '12px', padding: '16px', cursor: 'pointer', transition: 'all 0.15s',
              }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                  <div style={{ flex: 1 }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
                      <h3 style={{ fontSize: '15px', fontWeight: 600, margin: 0, color: 'var(--color-text-primary)' }}>{rec.title}</h3>
                      {getStatusBadge(rec)}
                      <span style={{ fontSize: '11px', padding: '2px 6px', borderRadius: '4px', background: `${getRiskColor(rec.risk_level)}20`, color: getRiskColor(rec.risk_level) }}>{rec.risk_level}</span>
                    </div>
                    <p style={{ fontSize: '13px', color: 'var(--color-text-secondary)', margin: '4px 0' }}>{rec.description}</p>
                  </div>
                </div>
                {selectedRec?.id === rec.id && (
                  <div style={{ marginTop: '12px', paddingTop: '12px', borderTop: '1px solid var(--color-border)', display: 'flex', gap: '8px' }}>
                    <button onClick={e => { e.stopPropagation(); dismissRec(rec.id); }} style={{
                      padding: '4px 12px', borderRadius: '4px', border: 'none', background: 'var(--color-error)', color: 'white', fontSize: '12px', cursor: 'pointer',
                    }}>Dismiss</button>
                  </div>
                )}
              </div>
            ))}
          </div>
        )
      )}
    </div>
  );
}

export default Guidance;