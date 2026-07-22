import { useState, useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import Sidebar from './components/Sidebar';
import ChatAssistant from './pages/ChatAssistant';
import Workspaces from './pages/Workspaces';
import Skills from './pages/Skills';
import Knowledge from './pages/Knowledge';
import Activity from './pages/Activity';
import Settings from './pages/Settings';
import About from './pages/About';
import './App.css';

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/*" element={<AppLayout />} />
      </Routes>
    </BrowserRouter>
  );
}

function AppLayout() {
  const [status, setStatus] = useState<{ version: string; running: boolean }>({
    version: '1.1.2',
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
            version: data.value.version || '1.1.1',
            running: data.value.status === 'running',
          });
        }
      })
      .catch(() => {
        setStatus({ version: '1.1.2', running: false });
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
          <Routes>
            <Route path="/" element={<ChatAssistant />} />
            <Route path="/assistant" element={<ChatAssistant />} />
            <Route path="/workspaces" element={<Workspaces />} />
            <Route path="/skills" element={<Skills />} />
            <Route path="/knowledge" element={<Knowledge />} />
            <Route path="/activity" element={<Activity />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/about" element={<About />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
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
    </div>
  );
}

export default App;