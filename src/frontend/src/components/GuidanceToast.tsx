import { useEffect, useRef, useCallback } from 'react';

interface ActiveRecommendation {
  id: string;
  title: string;
  description: string;
}

function GuidanceToast() {
  const lastShownRef = useRef<string>('');
  const notificationPermissionRef = useRef<string>('default');
  const lastShownContentRef = useRef<string>('');

  const showOsNotification = useCallback((title: string, body: string) => {
    try {
      if (typeof Notification === 'undefined' || !('requestPermission' in Notification)) return;

      const perm = Notification.permission;
      if (perm === 'granted') {
        new Notification(title, { body, tag: 'guidance-toast' });
      } else if (perm === 'default') {
        // Request permission first, then immediately show if granted
        Notification.requestPermission().then((p) => {
          if (p === 'granted') {
            new Notification(title, { body, tag: 'guidance-toast' });
          }
        }).catch(() => {
          // Permission request failed — fall back to trying without permission
          // (some browsers allow notifications even without explicit grant)
          try {
            new Notification(title, { body, tag: 'guidance-toast' });
          } catch {}
        });
      } else {
        // 'denied' — silently skip
      }
    } catch {
      // Notification API not available — silently ignore
    }
  }, []);

  const poll = useCallback(async () => {
    try {
      const res = await fetch('http://127.0.0.1:1420/api/commands/guidance_get_active_recommendations', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: {} }),
      });
      const data = await res.json();
      if (data.success && data.value && data.value.length > 0) {
        const rec = data.value[0] as ActiveRecommendation;
        // Show if either the ID or the content changed (handles rule-based recs with same title)
        if ((rec.id !== lastShownRef.current) || (rec.description !== lastShownContentRef.current)) {
          showOsNotification(rec.title, rec.description);
          lastShownRef.current = rec.id;
          lastShownContentRef.current = rec.description;
        }
      }
    } catch {
      // silently ignore polling errors
    }
  }, [showOsNotification]);

  useEffect(() => {
    // Check existing notification permission on load
    if (typeof Notification !== 'undefined') {
      notificationPermissionRef.current = Notification.permission;
    }
    // Poll more frequently to catch new recommendations faster
    const interval = setInterval(poll, 5000);
    return () => clearInterval(interval);
  }, [poll]);

  return null;
}

export default GuidanceToast;