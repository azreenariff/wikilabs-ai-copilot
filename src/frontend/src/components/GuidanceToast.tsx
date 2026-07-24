import { useEffect, useState } from 'react';

function GuidanceToast() {
  const [lastId, setLastId] = useState('');

  useEffect(() => {
    // Request notification permission on mount
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission();
    }

    const interval = setInterval(async () => {
      try {
        const res = await fetch('http://localhost:1420/api/commands/guidance_get_active_recommendations', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ params: {} }),
        });
        const data = await res.json();
        if (data.success && data.value && data.value.length > 0) {
          const rec = data.value[0];
          if (rec.id !== lastId) {
            setLastId(rec.id);

            const notification = new Notification(`🧭 Guidance: ${rec.title}`, {
              body: rec.description || `You have ${data.value.length} active recommendation(s) in the Guidance page.`,
              tag: rec.id,
            });

            notification.onclick = () => {
              window.focus();
              window.location.href = '/guidance';
            };

            setTimeout(() => notification.close(), 15000);
          }
        }
      } catch {
        // silently ignore polling errors
      }
    }, 10000);

    return () => clearInterval(interval);
  }, [lastId]);

  return null;
}

export default GuidanceToast;