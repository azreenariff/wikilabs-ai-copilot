import { useEffect, useRef, useCallback } from 'react';

interface ActiveRecommendation {
  id: string;
  title: string;
  description: string;
}

function GuidanceToast() {
  const lastShownRef = useRef<string>('');
  const notificationPermissionRef = useRef<string>('default');

  const showOsNotification = useCallback((title: string, body: string) => {
    // Use the browser's native Notification API — this shows an OS-level toast
    // at the top-right of the screen, visible even when the app window is minimized.
    // On Windows WebView2, this delegates to the Windows Toast API.
    try {
      if (typeof Notification !== 'undefined' && 'requestPermission' in Notification) {
        if (notificationPermissionRef.current === 'granted') {
          new Notification(title, { body, tag: 'guidance-toast' });
        } else if (notificationPermissionRef.current === 'default') {
          Notification.requestPermission().then((perm) => {
            notificationPermissionRef.current = perm;
            if (perm === 'granted') {
              new Notification(title, { body, tag: 'guidance-toast' });
            }
          });
        }
        // If 'denied', silently skip
      }
    } catch {
      // Notification API not available — silently ignore
    }
  }, []);

  const poll = useCallback(async () => {
    try {
      const res = await fetch('http://localhost:1420/api/commands/guidance_get_active_recommendations', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: {} }),
      });
      const data = await res.json();
      if (data.success && data.value && data.value.length > 0) {
        const rec = data.value[0] as ActiveRecommendation;
        if (rec.id !== lastShownRef.current) {
          showOsNotification(rec.title, rec.description);
          lastShownRef.current = rec.id;
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
    const interval = setInterval(poll, 10000);
    return () => clearInterval(interval);
  }, [poll]);

  // No in-app DOM — notification appears at OS level
  return null;
}

export default GuidanceToast;