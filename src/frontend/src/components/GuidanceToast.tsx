import { useEffect, useState, useCallback } from 'react';

interface ActiveRecommendation {
  id: string;
  title: string;
  description: string;
}

function GuidanceToast() {
  const [visible, setVisible] = useState(false);
  const [rec, setRec] = useState<ActiveRecommendation | null>(null);
  const [lastId, setLastId] = useState('');

  const poll = useCallback(async () => {
    try {
      const res = await fetch('http://localhost:1420/api/commands/guidance_get_active_recommendations', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: {} }),
      });
      const data = await res.json();
      if (data.success && data.value && data.value.length > 0) {
        const recommendation = data.value[0] as ActiveRecommendation;
        if (recommendation.id !== lastId) {
          setLastId(recommendation.id);
          setRec(recommendation);
          setVisible(true);
        }
      }
    } catch {
      // silently ignore polling errors
    }
  }, [lastId]);

  useEffect(() => {
    const interval = setInterval(poll, 10000);
    return () => clearInterval(interval);
  }, [poll]);

  // Auto-dismiss after 15 seconds
  useEffect(() => {
    if (!visible) return;
    const timer = setTimeout(() => setVisible(false), 15000);
    return () => clearTimeout(timer);
  }, [visible]);

  const handleClick = useCallback(() => {
    window.location.href = '/guidance';
  }, []);

  if (!visible || !rec) return null;

  return (
    <div
      onClick={handleClick}
      style={{
        position: 'fixed',
        top: '16px',
        right: '16px',
        zIndex: 9999,
        maxWidth: '420px',
        background: '#1e1e2e',
        border: '1px solid #45475a',
        borderRadius: '12px',
        padding: '14px 16px',
        boxShadow: '0 8px 32px rgba(0,0,0,0.4)',
        cursor: 'pointer',
        animation: 'slideIn 0.3s ease-out',
        color: '#cdd6f4',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '6px' }}>
        <span style={{ fontSize: '13px', fontWeight: 600, color: '#a6e3a1' }}>
          💡 AI Copilot
        </span>
        <button
          onClick={(e) => { e.stopPropagation(); e.currentTarget.parentElement?.parentElement?.remove(); }}
          style={{
            background: 'none',
            border: 'none',
            color: '#6c7086',
            cursor: 'pointer',
            fontSize: '16px',
            padding: '0 4px',
            lineHeight: 1,
          }}
          aria-label="Dismiss"
        >
          ×
        </button>
      </div>
      <p style={{ margin: 0, fontSize: '13px', lineHeight: 1.5, color: '#cdd6f4', whiteSpace: 'pre-wrap' }}>
        {rec.description}
      </p>
      <style>{`
        @keyframes slideIn {
          from { transform: translateX(100%); opacity: 0; }
          to { transform: translateX(0); opacity: 1; }
        }
      `}</style>
    </div>
  );
}

export default GuidanceToast;