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

export default function App() {
  const [needsSetup, setNeedsSetup] = useState(true);
  const [checking, setChecking] = useState(true);

  useEffect(() => {
    const checkSetup = async () => {
      try {
        // Step 1: Wait for the API server to be ready (polled up to 40 times, 500ms apart = 20s max).
        let serverReady = false;
        for (let r = 0; r < 40; r++) {
          try {
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 3000);
            const res = await fetch('http://localhost:1420/ready', { signal: controller.signal });
            clearTimeout(timeoutId);
            const data = await res.json();
            if (data.ready) {
              serverReady = true;
              break;
            }
          } catch {
            // Server not ready yet, keep polling
          }
          await new Promise(r => setTimeout(r, 500));
        }

        if (!serverReady) {
          // Backend never became ready — fall through to main UI
          setChecking(false);
          return;
        }

        // Step 2: Server is ready. Now check setup status.
        let settingsData: any = null;
        for (let attempt = 0; attempt < 5; attempt++) {
          try {
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 5000);
            const res = await fetch('http://localhost:1420/api/commands/get_settings', {
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
          setNeedsSetup(false);
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
        } else {
          // No API key configured — show wizard
          setNeedsSetup(true);
        }
      } catch (e) {
        console.error('[App] Error during setup check:', e);
      } finally {
        setChecking(false);
      }
    };
    checkSetup();
  }, []);

  if (checking) {
    return (
      <div style={{
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        height: '100vh', background: '#0f0f23',
        color: '#a1a1aa', fontSize: '14px',
      }}>
        <div style={{ textAlign: 'center' }}>
          <div style={{ width: '48px', height: '48px', borderRadius: '10px', marginBottom: '12px', background: '#1a1a2e' }} />
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