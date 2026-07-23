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
  collected: number;
  missing: number;
  total: number;
}

function Guidance() {
  const [recommendations, setRecommendations] = useState<RecommendationCard[]>([]);
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState('');
  const [selectedRec, setSelectedRec] = useState<RecommendationCard | null>(null);
  const [evidence, setEvidence] = useState<EvidenceStatus | null>(null);
  const [copilotMode, setCopilotMode] = useState('balanced');

  const fetchRecommendations = useCallback(async () => {
    try {
      const res = await fetch('http://localhost:1420/api/commands/guidance_get_active_recommendations', {
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
      const res = await fetch('http://localhost:1420/api/commands/guidance_get_evidence_status', {
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

  const fetchMode = useCallback(async () => {
    try {
      const res = await fetch('http://localhost:1420/api/commands/guidance_get_mode', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: {} }),
      });
      const data = await res.json();
      if (data.success && data.value) {
        setCopilotMode(data.value.mode || data.value || 'balanced');
      }
    } catch {}
  }, []);

  useEffect(() => {
    fetchRecommendations();
    fetchEvidence();
    fetchMode();
  }, [fetchRecommendations, fetchEvidence, fetchMode]);

  const dismissRec = async (recId: string) => {
    try {
      const res = await fetch('http://localhost:1420/api/commands/guidance_dismiss_recommendation', {
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

  const setMode = async (mode: string) => {
    try {
      const res = await fetch('http://localhost:1420/api/commands/guidance_set_mode', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: { mode } }),
      });
      const data = await res.json();
      if (data.success) {
        setCopilotMode(mode);
        setStatus(`Mode set to ${mode}`);
      }
    } catch {
      setStatus('Failed to set mode');
    }
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

      {/* Copilot Mode Selector */}
      <div style={{ background: 'var(--color-bg-secondary)', border: '1px solid var(--color-border)', borderRadius: '12px', padding: '16px', marginBottom: '16px' }}>
        <h3 style={{ fontSize: '14px', margin: '0 0 8px', color: 'var(--color-text-primary)' }}>Copilot Mode</h3>
        <div style={{ display: 'flex', gap: '8px' }}>
          {['Teaching', 'Balanced', 'Expert', 'Silent'].map(mode => (
            <button key={mode} onClick={() => setMode(mode.toLowerCase())} style={{
              padding: '6px 14px', borderRadius: '6px', border: 'none', fontSize: '12px', cursor: 'pointer',
              background: copilotMode === mode.toLowerCase() ? 'var(--color-accent)' : 'var(--color-bg-tertiary)',
              color: copilotMode === mode.toLowerCase() ? 'white' : 'var(--color-text-primary)',
            }}>{mode}</button>
          ))}
        </div>
      </div>

      {/* Evidence Status */}
      {evidence && (
        <div style={{ background: 'var(--color-bg-secondary)', border: '1px solid var(--color-border)', borderRadius: '12px', padding: '16px', marginBottom: '16px' }}>
          <h3 style={{ fontSize: '14px', margin: '0 0 8px', color: 'var(--color-text-primary)' }}>Evidence Status</h3>
          <div style={{ display: 'flex', gap: '16px', fontSize: '13px' }}>
            <span style={{ color: 'var(--color-success)' }}>✅ Collected: {evidence.collected}</span>
            <span style={{ color: 'var(--color-error)' }}>❌ Missing: {evidence.missing}</span>
            <span style={{ color: 'var(--color-text-secondary)' }}>📊 Total: {evidence.total}</span>
          </div>
        </div>
      )}

      {/* Recommendations */}
      {loading ? (
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
      )}
    </div>
  );
}

export default Guidance;