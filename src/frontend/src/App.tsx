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
import './App.css';

function App() {
  const [needsSetup, setNeedsSetup] = useState(true);
  const [checking, setChecking] = useState(true);

  useEffect(() => {
    const checkSetup = async () => {
      // Retry up to 5 times with 300ms delay in case API server hasn't started yet.
      // Each fetch has a 3s timeout so a dead API server doesn't freeze the UI indefinitely.
      for (let attempt = 0; attempt < 5; attempt++) {
        try {
          // AbortSignal.timeout may not exist in very old browsers; fall back to manual AbortController
          const makeSignal = () => {
            if (typeof AbortSignal.timeout === 'function') {
              return AbortSignal.timeout(3000);
            }
            const ac = new AbortController();
            setTimeout(() => ac.abort(), 3000);
            return ac.signal;
          };
          const res = await fetch('http://localhost:1420/api/commands/get_settings', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ params: {} }),
            signal: makeSignal(),
          });
          const data = await res.json();
          // DEBUG: log raw response to understand first-run behavior
          console.log('[startup] get_settings response:', JSON.stringify(data, null, 2));
          if (data.success && data.value) {
            const apiKey = data.value.ai_provider?.api_key || '';
            console.log('[startup] apiKey length:', apiKey.length, 'needsSetup:', !apiKey);
            // Show wizard if no API key configured.
            // This handles both fresh installs and upgrades where the user never
            // actually completed the wizard but the settings file existed.
            setNeedsSetup(!apiKey);
            // If API key is configured, hide main window and open floating advice chat
            if (apiKey) {
              fetch('http://localhost:1420/api/commands/hide_main_window', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ params: {} }),
              }).catch(() => {});
              // Also open the floating advice chat window on return visits
              setTimeout(() => {
                fetch('http://localhost:1420/api/commands/advice_chat_open', {
                  method: 'POST',
                  headers: { 'Content-Type': 'application/json' },
                  body: JSON.stringify({ params: {} }),
                }).catch(() => {});
              }, 1500); // small delay to let API server be ready
            }
          }
          break; // success
        } catch {
          if (attempt < 4) {
            await new Promise(r => setTimeout(r, 300)); // faster retry interval
          } else {
            // All retries failed — fall through to main UI
            setNeedsSetup(false);
          }
        }
      }
      setChecking(false);
    };
    checkSetup();
  }, []);

  if (checking) {
    return (
      <div style={{
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        height: '100vh', background: 'var(--color-bg-primary)',
        color: 'var(--color-text-secondary)', fontSize: '14px',
      }}>
        <div style={{ textAlign: 'center' }}>
          <img src="/logo.png" alt="Logo" style={{ width: '48px', height: '48px', borderRadius: '10px', marginBottom: '12px' }} />
          <div>Loading...</div>
        </div>
      </div>
    );
  }

  if (needsSetup) {
    return <SetupWizard />;
  }

  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppLayout />}>
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
    fetch('http://localhost:1420/api/commands/get_status', {
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
    <div style={{ display: 'flex', height: '100vh', background: 'var(--color-bg-primary)', color: 'var(--color-text-primary)' }}>
      <Sidebar />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        <header style={{
          padding: '8px 16px',
          borderBottom: '1px solid var(--color-border)',
          background: 'var(--color-bg-secondary)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          fontSize: '12px',
          color: 'var(--color-text-secondary)',
        }}>
          <span>Wiki Labs AI Copilot v{status.version}</span>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span style={{
              display: 'inline-block',
              width: '8px',
              height: '8px',
              borderRadius: '50%',
              background: status.running ? 'var(--color-success)' : 'var(--color-error)',
            }} />
            <span>{status.running ? 'Running' : 'Stopped'}</span>
          </div>
        </header>
        <main style={{ flex: 1, overflow: 'auto' }}>
          <Outlet />
        </main>
        <footer style={{
          padding: '4px 16px',
          borderTop: '1px solid var(--color-border)',
          fontSize: '11px',
          color: 'var(--color-text-secondary)',
          background: 'var(--color-bg-secondary)',
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

export default App;