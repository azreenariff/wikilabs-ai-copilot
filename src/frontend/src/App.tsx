import { useState, useEffect } from 'react';
import { BrowserRouter, Routes, Route, Outlet } from 'react-router-dom';
import Sidebar from './components/Sidebar';
import GuidanceToast from './components/GuidanceToast';
import SetupWizard from './pages/SetupWizard';
import ChatAssistant from './pages/ChatAssistant';
import Guidance from './pages/Guidance';
import Skills from './pages/Skills';
import Knowledge from './pages/Knowledge';
import Activity from './pages/Activity';
import Settings from './pages/Settings';
import About from './pages/About';
import PreflightCheck from './pages/PreflightCheck';
import './App.css';

export default function App() {
  console.log('[Wiki Labs] >>> App component rendered');
  const [needsSetup, setNeedsSetup] = useState(true);
  const [preflightDone, setPreflightDone] = useState(false);
  const [preflightChecks, setPreflightChecks] = useState<Record<string, { status: string; label: string; detail: string }>>({});
  const [showingMain, setShowingMain] = useState(false);
  const [loadingPhase, setLoadingPhase] = useState('Initializing...');

  useEffect(() => {
    // Total startup timeout: 30 seconds (was 60 — reduced so the user isn't stuck waiting).
    // If everything hangs (AbortController doesn't work in this WebView2), this ensures
    // the app never stays on the loading screen forever.
    const totalTimeout = setTimeout(() => {
      console.error('[Wiki Labs] >>> Total startup timeout reached (30s) — forcing main UI');
      setLoadingPhase('Startup timeout — showing interface anyway');
      setTimeout(() => setShowingMain(true), 0); // no delay — show immediately
    }, 30000);

    const checkSetup = async () => {
      console.log('[Wiki Labs] >>> checkSetup started');
      try {
        // Step 1: Wait for the API server to be ready.
        // - Polled up to 30 times (30 * 300ms = 9s) with a short 3s fetch timeout.
        // - If we can't reach 127.0.0.1:1420 at all, the fetch will fail quickly
        //   and we fall through gracefully to main UI.
        console.log('[Wiki Labs] >>> Checking API server /ready endpoint');
        setLoadingPhase('Checking API server...');
        let serverReady = false;
        let firstError: any = null;
        for (let r = 0; r < 30; r++) {
          try {
            console.log(`[Wiki Labs] >>> Poll /ready attempt ${r+1}/30`);
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 3000);
            const res = await fetch('http://127.0.0.1:1420/ready', { 
              signal: controller.signal,
              cache: 'no-store',
              mode: 'cors',
            });
            clearTimeout(timeoutId);
            console.log(`[Wiki Labs] >>> /ready response: status=${res.status}`);
            const data = await res.json();
            console.log(`[Wiki Labs] >>> /ready data:`, JSON.stringify(data));
            if (data.ready) {
              serverReady = true;
              console.log('[Wiki Labs] >>> Server ready!');
              break;
            }
          } catch (e: any) {
            if (!firstError) firstError = e;
            // After 10 failed attempts, start logging more aggressively
            if (r >= 10) {
              console.warn(`[Wiki Labs] >>> /ready attempt ${r+1} failed:`, e?.message || e);
            }
          }
          await new Promise(r => setTimeout(r, 300));
        }

        if (!serverReady) {
          console.warn('[Wiki Labs] >>> Server never became ready. First error:', firstError?.message || firstError);
          // Backend never became ready — fall through to main UI
          setLoadingPhase('Interface ready (server unavailable)');
          setPreflightDone(true);
          return;
        }

        // Step 2: Run pre-flight check (health + optional provider test)
        // The preflight check is a best-effort — if it fails, continue anyway.
        console.log('[Wiki Labs] >>> Running preflight check...');
        setLoadingPhase('Running pre-flight checks...');
        try {
          const controller = new AbortController();
          const timeoutId = setTimeout(() => controller.abort(), 5000);
          const pfRes = await fetch('http://127.0.0.1:1420/api/commands/preflight_check', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ test_provider: true }),
            signal: controller.signal,
          });
          clearTimeout(timeoutId);
          console.log('[Wiki Labs] >>> Preflight response:', pfRes.status);
          const pfData = await pfRes.json();
          console.log('[Wiki Labs] >>> Preflight data:', JSON.stringify(pfData));
          if (pfData.success && pfData.value) {
            setPreflightChecks(pfData.value);
            setPreflightDone(true);
          } else {
            // Preflight returned no data — still continue
            console.warn('[Wiki Labs] >>> Preflight returned no value:', JSON.stringify(pfData));
            setPreflightDone(true);
          }
        } catch (pfErr: any) {
          // Preflight check failed — still continue, just mark as incomplete
          console.warn('[App] Preflight check failed, continuing anyway:', pfErr?.message || pfErr);
          setPreflightDone(true);
        }

        // Step 3: Check setup status
        let settingsData: any = null;
        for (let attempt = 0; attempt < 5; attempt++) {
          try {
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 3000);
            const res = await fetch('http://127.0.0.1:1420/api/commands/get_settings', {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ params: {} }),
              signal: controller.signal,
            });
            clearTimeout(timeoutId);
            const data = await res.json();
            if (data.success && data.value) {
              settingsData = data.value;
            }
            console.log('[Wiki Labs] >>> Settings response received');
            break; // success
          } catch {
            if (attempt < 4) {
              await new Promise(r => setTimeout(r, 500)); // retry interval
            } else {
              // All retries failed — fall through to main UI
              settingsData = null;
            }
          }
        }

        if (settingsData && settingsData.ai_provider?.api_key) {
          // API key is configured — hide main window and open floating advice chat
          console.log('[Wiki Labs] >>> AI key found, showing main UI');
          setNeedsSetup(false);
          fetch('http://127.0.0.1:1420/api/commands/hide_main_window', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ params: {} }),
          }).catch(() => {});
          // Also open the floating advice chat window on return visits
          setTimeout(() => {
            fetch('http://127.0.0.1:1420/api/commands/advice_chat_open', {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ params: {} }),
            }).catch(() => {});
          }, 1500); // small delay to let API server be ready
        } else {
          // No API key configured — show wizard
          console.log('[Wiki Labs] >>> No AI key found, showing setup wizard');
          setNeedsSetup(true);
        }
      } catch (e) {
        console.error('[App] Error during setup check:', e);
      } finally {
        clearTimeout(totalTimeout);
      }
    };
    checkSetup();
  }, []);

  // Show pre-flight check screen briefly, then transition to SetupWizard or main UI
  if (preflightDone && !showingMain) {
    return <PreflightCheck checks={preflightChecks} onComplete={() => setShowingMain(true)} />;
  }

  // Show loading while preflight is running
  if (!preflightDone) {
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
          Wiki Labs AI Copilot
        </h2>
        <p style={{ color: '#71717a', fontSize: '13px', marginBottom: '16px' }}>
          {loadingPhase}
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

  // Preflight complete — show SetupWizard or main UI
  if (needsSetup) {
    return <SetupWizard />;
  }

  return (
    <BrowserRouter>
      <Routes>
        {/* Advice chat window route — renders ChatAssistant without sidebar */}
        <Route path="/advice-chat" element={<ChatAssistant />} />
        <Route element={<AppLayout />} >
          <Route path="/" element={<ChatAssistant />} />
          <Route path="/assistant" element={<ChatAssistant />} />
          <Route path="/guidance" element={<Guidance />} />
          <Route path="/skills" element={<Skills />} />
          <Route path="/knowledge" element={<Knowledge />} />
          <Route path="/activity" element={<Activity />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/about" element={<About />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

function AppLayout() {
  const [status, setStatus] = useState<{ version: string; running: boolean }>({
    version: '',
    running: true,
  });

  useEffect(() => {
    fetch('http://127.0.0.1:1420/api/commands/get_status', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ params: {} }),
    })
      .then(r => r.json())
      .then(data => {
        if (data.success && data.value) {
          setStatus({
            version: data.value.version || '',
            running: data.value.status === 'running',
          });
        }
      })
      .catch(() => {
        setStatus({ version: '', running: false });
      });
  }, []);

  return (
    <div style={{ display: 'flex', height: '100vh', background: '#0f0f23', color: '#e4e4e7' }}>
      <Sidebar />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        <header style={{
          padding: '8px 16px',
          borderBottom: '1px solid #27272a',
          background: '#1a1a2e',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          fontSize: '12px',
          color: '#a1a1aa',
        }}>
          <span>Wiki Labs AI Copilot v{status.version}</span>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span style={{
              display: 'inline-block',
              width: '8px',
              height: '8px',
              borderRadius: '50%',
              background: status.running ? '#22c55e' : '#ef4444',
            }} />
            <span>{status.running ? 'Running' : 'Stopped'}</span>
          </div>
        </header>
        <main style={{ flex: 1, overflow: 'auto' }}>
          <Outlet />
        </main>
        <footer style={{
          padding: '4px 16px',
          borderTop: '1px solid #27272a',
          fontSize: '11px',
          color: '#a1a1aa',
          background: '#1a1a2e',
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <span>Phase 4 — MVP Desktop Foundation</span>
            <span>SQLite • Tauri v2 • React 18</span>
          </div>
        </footer>
      </div>
      <GuidanceToast />
    </div>
  );
}