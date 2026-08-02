import { useState } from 'react';

const API = 'http://127.0.0.1:1420/api/commands';

const PROVIDERS = [
  { name: 'OpenAI', defaultEndpoint: 'https://api.openai.com/v1', needsKey: true },
  { name: 'OpenRouter', defaultEndpoint: 'https://openrouter.ai/api/v1', needsKey: true },
  { name: 'Custom Endpoint', defaultEndpoint: 'http://localhost:8000/v1', needsKey: true },
  { name: 'Ollama', defaultEndpoint: 'http://localhost:11434/v1', needsKey: false },
];

function SetupWizard() {
  const [step, setStep] = useState(0);
  const [selectedProvider, setSelectedProvider] = useState(PROVIDERS[0]);
  const [endpoint, setEndpoint] = useState(PROVIDERS[0].defaultEndpoint);
  const [apiKey, setApiKey] = useState('');
  const [model, setModel] = useState('');
  const [contextWindow, setContextWindow] = useState('128000');
  const [maxTokens, setMaxTokens] = useState('4096');
  const [fetchedModels, setFetchedModels] = useState<string[]>([]);
  const [testResult, setTestResult] = useState<'idle' | 'testing' | 'success' | 'fail'>('idle');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  const handleSelectProvider = (p: typeof PROVIDERS[0]) => {
    setSelectedProvider(p);
    setEndpoint(p.defaultEndpoint);
    setApiKey('');
    setModel('');
    setFetchedModels([]);
    setTestResult('idle');
    setError('');
  };

  async function httpPost(action: string, params: any) {
    const res = await fetch(`${API}/${action}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ params }),
    });
    return res.json();
  }

  const handleTestConnection = async () => {
    setTestResult('testing');
    setError('');
    setFetchedModels([]);
    try {
      const result: any = await httpPost('test_connection', {
        name: selectedProvider.name.toLowerCase().replace(/\s+/g, '_'),
        endpoint,
        api_key: apiKey,
        model: model || 'gpt-4o',
        max_tokens: parseInt(maxTokens) || 4096,
        context_window: parseInt(contextWindow) || 128000,
      });
      if (result.success || result.value === true) {
        setTestResult('success');
        // On success, fetch model list via HTTP
        try {
          const modelResp: any = await httpPost('list_models', {
            endpoint,
            api_key: apiKey,
          });
          const modelList: string[] = modelResp.success ? (modelResp.value || []) : [];
          if (modelList.length > 0) {
            setFetchedModels(modelList);
            setModel(modelList[0]);
          }
        } catch {
          // Model listing is optional — user can type model manually
        }
      } else {
        setTestResult('fail');
        setError('Connection failed — check URL and key');
      }
    } catch (e: any) {
      setTestResult('fail');
      setError(e.message || 'Connection failed — check URL and key');
    }
  };

  const handleSave = async () => {
    setSaving(true);
    setError('');
    try {
      await httpPost('update_settings', {
        ai_provider: {
          name: selectedProvider.name.toLowerCase().replace(/\s+/g, '_'),
          endpoint,
          api_key: apiKey,
          model,
          max_tokens: parseInt(maxTokens) || 4096,
          context_window: parseInt(contextWindow) || 128000,
        },
      });
      setStep(5);
    } catch (e: any) {
      setError(e.message || 'Cannot reach backend');
      setSaving(false);
    }
  };

  const renderStep = () => {
    switch (step) {
      case 0:
        return (
          <div style={{ textAlign: 'center' }}>
            <img src="/logo.png" alt="Logo" style={{ width: '80px', height: '80px', borderRadius: '16px', marginBottom: '16px' }} />
            <h1 style={{ fontSize: '24px', fontWeight: 700, margin: '0 0 8px' }}>Wiki Labs AI Copilot</h1>
            <p style={{ color: 'var(--color-text-secondary)', fontSize: '14px', marginBottom: '32px' }}>
              Welcome! Let's set up your AI provider so you can start using the copilot.
            </p>
            <button onClick={() => setStep(1)} style={{
              padding: '12px 32px', borderRadius: '8px', border: 'none',
              background: 'var(--color-accent)', color: 'white', fontSize: '15px',
              fontWeight: 600, cursor: 'pointer',
            }}>Get Started →</button>
          </div>
        );

      case 1:
        return (
          <div>
            <style>{`
              @keyframes btn-pulse {
                0%, 100% { opacity: 1; }
                50% { opacity: 0.6; }
              }
              @keyframes spin {
                0% { transform: rotate(0deg); }
                100% { transform: rotate(360deg); }
              }
            `}</style>
            <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>Choose your AI Provider</h2>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '24px' }}>
              {PROVIDERS.map(p => (
                <div
                  key={p.name}
                  onClick={() => handleSelectProvider(p)}
                  style={{
                    padding: '12px 16px', borderRadius: '8px', border: selectedProvider.name === p.name ? '2px solid var(--color-accent)' : '2px solid var(--color-border)',
                    cursor: 'pointer', background: selectedProvider.name === p.name ? 'rgba(66,153,225,0.1)' : 'var(--color-bg-secondary)',
                    transition: 'border 0.2s',
                  }}
                >
                  <div style={{ fontWeight: 600 }}>{p.name}</div>
                  <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)' }}>{p.defaultEndpoint}</div>
                </div>
              ))}
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
              <input
                type="text"
                placeholder="API Endpoint URL"
                value={endpoint}
                onChange={e => setEndpoint(e.target.value)}
                style={{ padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--color-border)', background: 'var(--color-bg-secondary)', color: 'var(--color-text-primary)', fontSize: '14px' }}
              />
              {selectedProvider.needsKey && (
                <input
                  type="password"
                  placeholder="API Key"
                  value={apiKey}
                  onChange={e => setApiKey(e.target.value)}
                  style={{ padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--color-border)', background: 'var(--color-bg-secondary)', color: 'var(--color-text-primary)', fontSize: '14px' }}
                />
              )}
              <div style={{ display: 'flex', gap: '8px' }}>
                {fetchedModels.length > 0 ? (
                  <select
                    value={model}
                    onChange={e => setModel(e.target.value)}
                    style={{ flex: 1, padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--color-border)', background: 'var(--color-bg-secondary)', color: 'var(--color-text-primary)', fontSize: '14px' }}
                  >
                    {fetchedModels.map(m => (
                      <option key={m} value={m}>{m}</option>
                    ))}
                  </select>
                ) : (
                  <input
                    type="text"
                    placeholder="Model name (e.g., gpt-4o)"
                    value={model}
                    onChange={e => setModel(e.target.value)}
                    style={{ flex: 1, padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--color-border)', background: 'var(--color-bg-secondary)', color: 'var(--color-text-primary)', fontSize: '14px' }}
                  />
                )}
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <input
                  type="number"
                  placeholder="Context Window"
                  value={contextWindow}
                  onChange={e => setContextWindow(e.target.value)}
                  style={{ flex: 1, padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--color-border)', background: 'var(--color-bg-secondary)', color: 'var(--color-text-primary)', fontSize: '14px' }}
                />
                <input
                  type="number"
                  placeholder="Max Tokens"
                  value={maxTokens}
                  onChange={e => setMaxTokens(e.target.value)}
                  style={{ flex: 1, padding: '10px 14px', borderRadius: '6px', border: '1px solid var(--color-border)', background: 'var(--color-bg-secondary)', color: 'var(--color-text-primary)', fontSize: '14px' }}
                />
              </div>
              <button
                onClick={handleTestConnection}
                disabled={testResult === 'testing'}
                style={{
                  padding: '10px 20px', borderRadius: '6px', border: 'none',
                  background: testResult === 'testing' ? 'var(--color-text-secondary)' : 'var(--color-accent)',
                  color: 'white', fontSize: '14px', fontWeight: 600, cursor: testResult === 'testing' ? 'not-allowed' : 'pointer',
                  display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '8px',
                }}
              >
                {testResult === 'testing' ? (
                  <>
                    <span style={{
                      width: '16px', height: '16px',
                      border: '2px solid rgba(255,255,255,0.3)',
                      borderTopColor: 'white',
                      borderRadius: '50%',
                      animation: 'spin 0.8s linear infinite',
                      display: 'inline-block',
                    }} />
                    Testing...
                  </>
                ) : (
                  'Test Connection'
                )}
              </button>
              {testResult && (
                <div style={{
                  padding: '8px 12px', borderRadius: '6px', fontSize: '13px',
                  background: testResult === 'success' ? 'rgba(49,130,206,0.1)' : 'rgba(245,101,101,0.1)',
                  color: testResult === 'success' ? 'var(--color-accent)' : '#f56565',
                }}>
                  {testResult === 'success' ? '✅ Connection successful!' : `❌ ${error}`}
                </div>
              )}
              {fetchedModels.length > 0 && (
                <div style={{ fontSize: '13px', color: 'var(--color-text-secondary)' }}>
                  Available models: {fetchedModels.join(', ')}
                </div>
              )}
            </div>
          </div>
        );

      case 2:
        return (
          <div>
            <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>Privacy & Security</h2>
            <div style={{ fontSize: '14px', lineHeight: 1.6 }}>
              <p>Your data is processed locally or via your configured AI provider. No telemetry is sent externally.</p>
              <ul style={{ paddingLeft: '20px', marginTop: '8px' }}>
                <li>AI responses use your configured provider</li>
                <li>Chat history stored locally in the app</li>
                <li>Skills and knowledge packs loaded from your workspace</li>
              </ul>
            </div>
          </div>
        );

      case 3:
        return (
          <div>
            <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>Workspace</h2>
            <div style={{ fontSize: '14px', lineHeight: 1.6 }}>
              <p>The AI copilot will monitor your activity to provide context-aware suggestions. This includes:</p>
              <ul style={{ paddingLeft: '20px', marginTop: '8px' }}>
                <li>Active window tracking</li>
                <li>File open events</li>
              </ul>
              <p style={{ marginTop: '12px' }}>You can always disable monitoring from Settings.</p>
            </div>
          </div>
        );
    }
  };

  const nextStep = () => {
    if (step < 3) setStep(step + 1);
    else handleSave();
  };

  const prevStep = () => {
    if (step > 0) setStep(step - 1);
  };

  return (
    <div style={{
      minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center',
      background: 'var(--color-bg-primary)', fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    }}>
      <div style={{
        width: '100%', maxWidth: '520px', padding: '32px',
        background: 'var(--color-bg-secondary)', borderRadius: '12px',
        boxShadow: '0 4px 24px rgba(0,0,0,0.3)',
      }}>
        {step < 5 && (
          <div style={{ display: 'flex', gap: '6px', marginBottom: '32px', justifyContent: 'center' }}>
            {[0, 1, 2].map(s => (
              <div key={s} style={{
                width: '8px', height: '8px', borderRadius: '50%',
                background: step >= s ? 'var(--color-accent)' : 'var(--color-border)',
                transition: 'background 0.2s',
              }} />
            ))}
          </div>
        )}
        {renderStep()}

        {step >= 1 && step < 5 && (
          <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '24px' }}>
            <button onClick={prevStep} style={{
              padding: '10px 20px', borderRadius: '6px', border: '1px solid var(--color-border)',
              background: 'transparent', color: 'var(--color-text-secondary)', fontSize: '14px', cursor: 'pointer',
            }}>← Back</button>
            <button onClick={nextStep} disabled={saving} style={{
              padding: '10px 20px', borderRadius: '6px', border: 'none',
              background: saving ? 'var(--color-text-secondary)' : 'var(--color-accent)',
              color: 'white', fontSize: '14px', fontWeight: 600, cursor: saving ? 'not-allowed' : 'pointer',
            }}>
              {step === 3 ? 'Save & Minimize' : 'Next →'}
            </button>
          </div>
        )}

        {error && step !== 5 && (
          <div style={{
            marginTop: '16px', padding: '10px 14px', borderRadius: '6px',
            background: 'rgba(245,101,101,0.1)', color: '#f56565', fontSize: '13px',
          }}>{error}</div>
        )}

        {step === 5 && (
          <div style={{ textAlign: 'center', marginTop: '16px' }}>
            <div style={{ fontSize: '48px', marginBottom: '16px' }}>✅</div>
            <h2 style={{ fontSize: '20px', fontWeight: 600, margin: '0 0 8px' }}>Setup Complete!</h2>
            <p style={{ color: 'var(--color-text-secondary)', fontSize: '14px', marginBottom: '24px' }}>
              The copilot is now running in the background. You can access it from the system tray.
            </p>
            <button onClick={() => {
              // Navigate within the already-loaded React app — same approach as main.rs
              window.history.pushState({}, '', '/advice-chat');
              window.dispatchEvent(new PopStateEvent('popstate'));
            }} style={{
              padding: '10px 20px', borderRadius: '6px', border: '1px solid var(--color-accent)',
              background: 'transparent', color: 'var(--color-accent)', fontSize: '14px', cursor: 'pointer',
            }}>Open Copilot</button>
          </div>
        )}
      </div>
    </div>
  );
}

export default SetupWizard;