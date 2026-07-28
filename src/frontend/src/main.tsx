import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { ErrorBoundary } from './components/ErrorBoundary';

console.log('[Wiki Labs] >>> Frontend loading...');
ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
);
console.log('[Wiki Labs] >>> React root rendered');
// Signal that React has rendered
window.__react_ready__ = true;