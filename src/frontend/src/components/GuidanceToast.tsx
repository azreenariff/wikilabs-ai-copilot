import { useState, useEffect } from 'react';

interface ToastNotification {
  id: string;
  title: string;
  body: string;
  timestamp: number;
}

function GuidanceToast() {
  const [toasts, setToasts] = useState<ToastNotification[]>([]);
  const [lastCount, setLastCount] = useState(0);

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const res = await fetch('http://localhost:1420/api/commands/guidance_get_active_recommendations', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ params: {} }),
        });
        const data = await res.json();
        if (data.success && data.value && data.value.length > 0) {
          const count = data.value.length;
          if (count > lastCount) {
            setLastCount(count);
            const newToast: ToastNotification = {
              id: Date.now().toString(),
              title: '🧭 New Guidance Available',
              body: data.value[0].title || `${count} recommendation${count > 1 ? 's' : ''} ready`,
              timestamp: Date.now(),
            };
            setToasts(prev => [...prev, newToast]);
            // Play a subtle notification sound
            try {
              const audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
              const osc = audioCtx.createOscillator();
              const gain = audioCtx.createGain();
              osc.connect(gain);
              gain.connect(audioCtx.destination);
              osc.frequency.setValueAtTime(880, audioCtx.currentTime);
              osc.frequency.setValueAtTime(660, audioCtx.currentTime + 0.1);
              gain.gain.setValueAtTime(0.15, audioCtx.currentTime);
              gain.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + 0.3);
              osc.start(audioCtx.currentTime);
              osc.stop(audioCtx.currentTime + 0.3);
            } catch {}
          }
        }
      } catch {}
    }, 10000);

    return () => clearInterval(interval);
  }, [lastCount]);

  // Auto-dismiss toasts after 6 seconds
  useEffect(() => {
    if (toasts.length === 0) return;
    const timer = setTimeout(() => {
      setToasts(prev => prev.slice(1));
    }, 6000);
    return () => clearTimeout(timer);
  }, [toasts]);

  if (toasts.length === 0) return null;

  return (
    <div style={{
      position: 'fixed',
      bottom: '20px',
      right: '20px',
      zIndex: 9999,
      display: 'flex',
      flexDirection: 'column',
      gap: '8px',
      maxWidth: '360px',
    }}>
      {toasts.map(toast => (
        <div
          key={toast.id}
          onClick={() => {
            window.location.href = '/guidance';
            setToasts([]);
          }}
          style={{
            background: 'var(--color-bg-secondary)',
            border: '1px solid var(--color-accent)',
            borderRadius: '12px',
            padding: '14px 16px',
            cursor: 'pointer',
            boxShadow: '0 8px 32px rgba(0,0,0,0.3)',
            animation: 'slideInRight 0.3s ease-out',
            display: 'flex',
            alignItems: 'flex-start',
            gap: '10px',
          }}
        >
          <span style={{ fontSize: '18px' }}>🧭</span>
          <div style={{ flex: 1 }}>
            <div style={{ fontSize: '13px', fontWeight: 600, color: 'var(--color-text-primary)' }}>
              {toast.title}
            </div>
            <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)', marginTop: '2px' }}>
              {toast.body}
            </div>
            <div style={{ fontSize: '11px', color: 'var(--color-accent)', marginTop: '4px' }}>
              Click to view →
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

export default GuidanceToast;