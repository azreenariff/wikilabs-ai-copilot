import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { ErrorBoundary } from './components/ErrorBoundary';

console.log('[UI] Frontend loading...');
console.log('[UI] Tauri available:', typeof window !== 'undefined' && '__TAURI__' in window);
const rootEl = document.getElementById('root');
console.log('[UI] Root element found:', !!rootEl);
ReactDOM.createRoot(rootEl!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
);
console.log('[UI] React root rendered');
// Signal that React has mounted — used by the self-diagnostic in index.html
window.__react_ready__ = true;