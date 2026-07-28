import { useEffect, useState } from 'react';

interface CheckItem {
  status: 'pending' | 'running' | 'pass' | 'fail' | 'skip';
  label: string;
  detail: string;
}

interface PreflightCheckProps {
  checks?: Record<string, { status: string; label: string; detail: string }>;
  onComplete?: () => void;
}

export default function PreflightCheck({ checks, onComplete }: PreflightCheckProps) {
  const [checkItems, setCheckItems] = useState<Record<string, CheckItem>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [allDone, setAllDone] = useState(false);
  const [transitioning, setTransitioning] = useState(false);

  useEffect(() => {
    let cancelled = false;

    // If checks are already provided as props, use them
    if (checks && Object.keys(checks).length > 0) {
      if (!cancelled) {
        const mapped: Record<string, CheckItem> = {};
        for (const [key, item] of Object.entries(checks)) {
          mapped[key] = {
            status: (item.status as CheckItem['status']) || 'skip',
            label: item.label || key,
            detail: item.detail || '',
          };
        }
        setCheckItems(mapped);
        setAllDone(true);
        setLoading(false);

        // Auto-transition to main UI after showing results
        if (onComplete) {
          const timer = setTimeout(() => {
            setTransitioning(true);
            setTimeout(() => onComplete(), 600);
          }, 2500);
          return () => clearTimeout(timer);
        }
      }
      return;
    }

    // Otherwise, fetch from backend
    const runPreflight = async () => {
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 10000);

        const res = await fetch('http://localhost:1420/api/preflight_check', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ test_provider: false }),
          signal: controller.signal,
        });
        clearTimeout(timeoutId);

        const data = await res.json();

        if (!cancelled) {
          if (data.value) {
            const mapped: Record<string, CheckItem> = {};
            for (const [key, item] of Object.entries(data.value)) {
              mapped[key] = {
                status: (item as any).status || 'skip',
                label: (item as any).label || key,
                detail: (item as any).detail || '',
              };
            }
            setCheckItems(mapped);
            setAllDone(true);
          }
          if (data.error) {
            setError(data.error);
          }
          setLoading(false);
        }
      } catch (e: any) {
        if (!cancelled) {
          setError(e.message || 'Cannot reach backend');
          setLoading(false);
        }
      }
    };

    runPreflight();
    return () => { cancelled = true; };
  }, [checks]);

  if (loading) {
    return (
      <div style={{
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        height: '100vh', background: '#0f0f23',
      }}>
        <style>{`
          @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
          }
          @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.4; }
          }
          @keyframes dot-bounce {
            0%, 80%, 100% { transform: scale(0.6); opacity: 0.4; }
            40% { transform: scale(1); opacity: 1; }
          }
          @keyframes checkmark {
            0% { transform: scale(0); opacity: 0; }
            50% { transform: scale(1.2); }
            100% { transform: scale(1); opacity: 1; }
          }
          @keyframes slide-up {
            from { transform: translateY(10px); opacity: 0; }
            to { transform: translateY(0); opacity: 1; }
          }
        `}</style>
        <div style={{ textAlign: 'center' }}>
          <div style={{
            width: '56px', height: '56px', borderRadius: '14px', marginBottom: '20px',
            background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
            animation: 'pulse 2s ease-in-out infinite',
            position: 'relative', display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
          }}>
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="2"
              style={{ animation: 'spin 1.5s linear infinite' }}>
              <path d="M21 12a9 9 0 11-6.219-8.56" />
            </svg>
          </div>
          <h2 style={{ color: '#e4e4e7', fontSize: '18px', fontWeight: 600, marginBottom: '8px' }}>
            Starting up...
          </h2>
          <p style={{ color: '#71717a', fontSize: '13px', marginBottom: '16px' }}>
            Checking server health and configuration
          </p>
          <div style={{ display: 'flex', gap: '6px', justifyContent: 'center' }}>
            <span style={{ animation: 'dot-bounce 1.4s ease-in-out infinite', animationDelay: '0ms' }}>.</span>
            <span style={{ animation: 'dot-bounce 1.4s ease-in-out infinite', animationDelay: '0.2s' }}>.</span>
            <span style={{ animation: 'dot-bounce 1.4s ease-in-out infinite', animationDelay: '0.4s' }}>.</span>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div style={{
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        height: '100vh', background: '#0f0f23',
      }}>
        <style>{`
          @keyframes slide-up {
            from { transform: translateY(10px); opacity: 0; }
            to { transform: translateY(0); opacity: 1; }
          }
        `}</style>
        <div style={{ textAlign: 'center', maxWidth: '400px', padding: '24px', animation: 'slide-up 0.3s ease-out' }}>
          <div style={{
            width: '56px', height: '56px', borderRadius: '14px', margin: '0 auto 20px',
            background: 'linear-gradient(135deg, #ef4444 0%, #dc2626 100%)',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
          }}>
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </div>
          <h2 style={{ color: '#e4e4e7', fontSize: '18px', fontWeight: 600, marginBottom: '8px' }}>
            Startup Check Failed
          </h2>
          <p style={{ color: '#a1a1aa', fontSize: '13px', marginBottom: '20px' }}>
            {error}
          </p>
          <button
            onClick={() => window.location.reload()}
            style={{
              padding: '10px 24px', borderRadius: '8px', border: 'none',
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              color: 'white', fontSize: '14px', fontWeight: 500, cursor: 'pointer',
            }}
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div style={{
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      height: '100vh', background: '#0f0f23',
      opacity: transitioning ? 0 : 1,
      transition: 'opacity 0.6s ease-out',
    }}>
      <style>{`
        @keyframes checkmark {
          0% { transform: scale(0); opacity: 0; }
          50% { transform: scale(1.2); }
          100% { transform: scale(1); opacity: 1; }
        }
        @keyframes slide-up {
          from { transform: translateY(10px); opacity: 0; }
          to { transform: translateY(0); opacity: 1; }
        }
        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }
      `}</style>
      <div style={{ maxWidth: '420px', width: '100%', padding: '24px' }}>
        <div style={{ textAlign: 'center', marginBottom: '24px' }}>
          <h2 style={{ color: '#e4e4e7', fontSize: '18px', fontWeight: 600, marginBottom: '4px' }}>
            Wiki Labs AI Copilot
          </h2>
          <p style={{ color: '#71717a', fontSize: '13px' }}>
            Verifying system health
          </p>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          {Object.entries(checkItems).map(([key, item], index) => (
            <div key={key} style={{
              display: 'flex', alignItems: 'center', gap: '12px',
              padding: '12px 16px', borderRadius: '10px',
              background: item.status === 'pass' ? 'rgba(34,197,94,0.08)' :
                          item.status === 'fail' ? 'rgba(239,68,68,0.08)' :
                          item.status === 'skip' ? 'rgba(113,113,122,0.08)' :
                          'rgba(100,100,113,0.08)',
              border: '1px solid ' + (item.status === 'pass' ? 'rgba(34,197,94,0.2)' :
                        item.status === 'fail' ? 'rgba(239,68,68,0.2)' :
                        item.status === 'skip' ? 'rgba(113,113,122,0.15)' :
                        'rgba(100,100,113,0.15)'),
              animation: `slide-up 0.3s ease-out ${index * 0.1}s both`,
            }}>
              {/* Status icon */}
              <div style={{
                width: '28px', height: '28px', borderRadius: '8px',
                display: 'flex', alignItems: 'center', justifyContent: 'center',
                flexShrink: 0,
                background: item.status === 'pass' ? 'rgba(34,197,94,0.15)' :
                            item.status === 'fail' ? 'rgba(239,68,68,0.15)' :
                            item.status === 'skip' ? 'rgba(113,113,122,0.1)' :
                            'rgba(100,100,113,0.1)',
              }}>
                {item.status === 'pass' && (
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#22c55e" strokeWidth="3"
                    style={{ animation: 'checkmark 0.3s ease-out' }}>
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                )}
                {item.status === 'fail' && (
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#ef4444" strokeWidth="3">
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                  </svg>
                )}
                {item.status === 'skip' && (
                  <span style={{ fontSize: '14px', color: '#71717a' }}>—</span>
                )}
                {item.status === 'pending' || item.status === 'running' ? (
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="#a1a1aa" strokeWidth="2"
                    style={{ animation: 'spin 1s linear infinite' }}>
                    <path d="M21 12a9 9 0 11-6.219-8.56" />
                  </svg>
                ) : null}
              </div>

              {/* Text */}
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ color: '#e4e4e7', fontSize: '14px', fontWeight: 500 }}>{item.label}</div>
                <div style={{
                  color: item.status === 'pass' ? '#22c55e' :
                         item.status === 'fail' ? '#ef4444' : '#71717a',
                  fontSize: '12px', marginTop: '1px',
                }}>
                  {item.detail}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Transition notice */}
        {allDone && (
          <p style={{
            textAlign: 'center', color: '#52525b', fontSize: '12px', marginTop: '20px',
            animation: 'slide-up 0.3s ease-out',
          }}>
            Continuing...
          </p>
        )}
      </div>
    </div>
  );
}