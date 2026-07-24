import { useState, useEffect } from 'react';

const PROVIDERS = [
  { name: 'OpenAI', endpoint: 'https://api.openai.com/v1', models: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo'] },
  { name: 'OpenRouter', endpoint: 'https://openrouter.ai/api/v1', models: ['anthropic/claude-sonnet-4', 'google/gemini-2.0-flash-001', 'deepseek/deepseek-chat'] },
  { name: 'Custom Endpoint', endpoint: 'http://localhost:8000/v1', models: ['Custom'] },
  { name: 'Ollama', endpoint: 'http://localhost:11434/v1', models: ['llama3', 'mistral', 'codellama'] },
];

function SetupWizard() {
  const [step, setStep] = useState(0);
  const [selectedProvider, setSelectedProvider] = useState(PROVIDERS[0]);
  const [apiKey, setApiKey] = useState('');
  const [model, setModel] = useState(PROVIDERS[0].models[0]);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<'idle' | 'testing' | 'success' | 'fail'>('idle');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  const isSetupComplete = step === 4;

  const handleSelectProvider = (p: typeof PROVIDERS[0]) => {
    setSelectedProvider(p);
    setModel(p.models[0]);
    setApiKey('');
    setTestResult('idle');
  };

  const handleTestConnection = async () => {
    setTesting(true);
    setTestResult('testing');
    setError('');
    try {
      const res = await fetch('http://localhost:1420/api/commands/test_connection', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          params: {
            name: selectedProvider.name.toLowerCase().replace(/\s+/g, '_'),
            endpoint: selectedProvider.endpoint,
            api_key: apiKey,
            model,
            max_tokens: 4096,
            context_window: 128000,
          },
        }),
      });
      const data = await res.json();
      if (data.success && data.value) {
        setTestResult('success');
      } else {
        setTestResult('fail');
        setError(data.error || 'Connection failed');
      }
    } catch {
      setTestResult('fail');
      setError('Cannot reach backend');
    }
    setTesting(false);
  };

  const handleSave = async () => {
    setSaving(true);
    setError('');
    try {
      const res = await fetch('http://localhost:1420/api/commands/update_settings', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          params: {
            ai_provider: {
              name: selectedProvider.name.toLowerCase().replace(/\s+/g, '_'),
              endpoint: selectedProvider.endpoint,
              api_key: apiKey,
              model,
              max_tokens: 4096,
              context_window: 128000,
            },
          },
        }),
      });
      const data = await res.json();
      if (data.success) {
        setStep(4);
      } else {
        setError(data.error || 'Failed to save');
      }
    } catch {
      setError('Cannot reach backend');
    }
    setSaving(false);
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
            <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>Choose your AI Provider</h2>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {PROVIDERS.map(p => (
                <div
                  key={p.name}
                  onClick={() => handleSelectProvider(p)}
                  style={{
                    padding: '14px 16px', borderRadius: '10px', cursor: 'pointer',
                    border: `2px solid ${selectedProvider.name === p.name ? 'var(--color-accent)' : 'var(--color-border)'}`,
                    background: selectedProvider.name === p.name ? 'rgba(99,102,241,0.1)' : 'var(--color-bg-secondary)',
                    display: 'flex', alignItems: 'center', gap: '12px',
                  }}
                >
                  <span style={{ fontSize: '24px' }}>
                    {p.name === 'OpenAI' ? '⚡' : p.name === 'OpenRouter' ? '🌐' : p.name === 'Custom Endpoint' ? '🔧' : '🦙'}
                  </span>
                  <div>
                    <div style={{ fontWeight: 600, fontSize: '14px' }}>{p.name}</div>
                    <div style={{ fontSize: '12px', color: 'var(--color-text-secondary)', marginTop: '2px' }}>{p.endpoint}</div>
                  </div>
                </div>
              ))}
            </div>
            <div style={{ display: 'flex', gap: '8px', marginTop: '24px', justifyContent: 'flex-end' }}>
              <button onClick={() => setStep(0)} style={{
                padding: '8px 20px', borderRadius: '6px', border: '1px solid var(--color-border)',
                background: 'transparent', color: 'var(--color-text-primary)', cursor: 'pointer', fontSize: '13px',
              }}>Back</button>
              <button onClick={() => setStep(2)} style={{
                padding: '8px 20px', borderRadius: '6px', border: 'none',
                background: 'var(--color-accent)', color: 'white', cursor: 'pointer', fontSize: '13px',
              }}>Next →</button>
            </div>
          </div>
        );

      case 2:
        return (
          <div>
            <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '8px' }}>Enter API Key</h2>
            <p style={{ fontSize: '13px', color: 'var(--color-text-secondary)', marginBottom: '20px' }}>
              {selectedProvider.name === 'Ollama'
                ? 'Ollama runs locally — no API key needed. You can leave this blank.'
                : `Enter your ${selectedProvider.name} API key.`}
            </p>
            <div style={{ marginBottom: '16px' }}>
              <label style={{ fontSize: '12px', color: 'var(--color-text-secondary)', marginBottom: '4px', display: 'block' }}>Model</label>
              <select
                value={model}
                onChange={e => setModel(e.target.value)}
                style={{
                  width: '100%', padding: '10px 12px', borderRadius: '6px',
                  border: '1px solid var(--color-border)', background: 'var(--color-bg-primary)',
                  color: 'var(--color-text-primary)', fontSize: '13px', outline: 'none',
                }}
              >
                {selectedProvider.models.map(m => <option key={m} value={m}>{m}</option>)}
              </select>
            </div>
            {selectedProvider.name !== 'Ollama' && (
              <div style={{ marginBottom: '16px' }}>
                <label style={{ fontSize: '12px', color: 'var(--color-text-secondary)', marginBottom: '4px', display: 'block' }}>API Key</label>
                <input
                  type="password"
                  value={apiKey}
                  onChange={e => setApiKey(e.target.value)}
                  placeholder="sk-..."
                  style={{
                    width: '100%', padding: '10px 12px', borderRadius: '6px',
                    border: '1px solid var(--color-border)', background: 'var(--color-bg-primary)',
                    color: 'var(--color-text-primary)', fontSize: '13px', outline: 'none',
                  }}
                />
              </div>
            )}
            <div style={{ display: 'flex', gap: '8px', marginTop: '24px', justifyContent: 'flex-end' }}>
              <button onClick={() => setStep(1)} style={{
                padding: '8px 20px', borderRadius: '6px', border: '1px solid var(--color-border)',
                background: 'transparent', color: 'var(--color-text-primary)', cursor: 'pointer', fontSize: '13px',
              }}>Back</button>
              <button onClick={() => setStep(3)} style={{
                padding: '8px 20px', borderRadius: '6px', border: 'none',
                background: 'var(--color-accent)', color: 'white', cursor: 'pointer', fontSize: '13px',
              }}>Test Connection →</button>
            </div>
          </div>
        );

      case 3:
        return (
          <div>
            <h2 style={{ fontSize: '18px', fontWeight: 600, marginBottom: '16px' }}>Test Connection</h2>
            <div style={{ marginBottom: '20px' }}>
              <div style={{ fontSize: '13px', color: 'var(--color-text-secondary)', marginBottom: '4px' }}>Provider</div>
              <div style={{ fontSize: '14px', fontWeight: 600 }}>{selectedProvider.name}</div>
              <div style={{ fontSize: '13px', color: 'var(--color-text-secondary)', marginTop: '12px', marginBottom: '4px' }}>Endpoint</div>
              <div style={{ fontSize: '14px' }}>{selectedProvider.endpoint}</div>
              <div style={{ fontSize: '13px', color: 'var(--color-text-secondary)', marginTop: '12px', marginBottom: '4px' }}>Model</div>
              <div style={{ fontSize: '14px' }}>{model}</div>
            </div>

            {testResult === 'idle' && (
              <button onClick={handleTestConnection} style={{
                padding: '10px 24px', borderRadius: '6px', border: 'none',
                background: 'var(--color-accent)', color: 'white', cursor: 'pointer', fontSize: '14px',
              }}>🔌 Test Connection</button>
            )}
            {testResult === 'testing' && (
              <div style={{ fontSize: '14px', color: 'var(--color-text-secondary)' }}>Testing connection...</div>
            )}
            {testResult === 'success' && (
              <div style={{ fontSize: '14px', color: 'var(--color-success)', display: 'flex', alignItems: 'center', gap: '8px' }}>
                ✅ Connection successful!
              </div>
            )}
            {testResult === 'fail' && (
              <div style={{ fontSize: '14px', color: 'var(--color-error)' }}>
                ❌ Connection failed: {error || 'Unknown error'}
                <button onClick={handleTestConnection} style={{
                  display: 'block', marginTop: '8px', padding: '6px 16px', borderRadius: '4px',
                  border: '1px solid var(--color-border)', background: 'transparent',
                  color: 'var(--color-text-primary)', cursor: 'pointer', fontSize: '12px',
                }}>Retry</button>
              </div>
            )}

            <div style={{ display: 'flex', gap: '8px', marginTop: '24px', justifyContent: 'flex-end' }}>
              <button onClick={() => setStep(2)} style={{
                padding: '8px 20px', borderRadius: '6px', border: '1px solid var(--color-border)',
                background: 'transparent', color: 'var(--color-text-primary)', cursor: 'pointer', fontSize: '13px',
              }}>Back</button>
              <button onClick={handleSave} disabled={saving} style={{
                padding: '8px 20px', borderRadius: '6px', border: 'none',
                background: 'var(--color-success)', color: 'white', cursor: 'pointer', fontSize: '13px',
                opacity: saving ? 0.6 : 1,
              }}>{saving ? 'Saving...' : 'Save & Finish ✓'}</button>
            </div>
            {error && step === 3 && (
              <div style={{ fontSize: '12px', color: 'var(--color-error)', marginTop: '8px' }}>{error}</div>
            )}
          </div>
        );

      case 4:
        return (
          <div style={{ textAlign: 'center' }}>
            <div style={{ fontSize: '64px', marginBottom: '16px' }}>🎉</div>
            <h2 style={{ fontSize: '22px', fontWeight: 700, margin: '0 0 8px' }}>You're all set!</h2>
            <p style={{ color: 'var(--color-text-secondary)', fontSize: '14px', marginBottom: '24px' }}>
              {selectedProvider.name} is configured and ready. Start using the copilot now.
            </p>
            <button onClick={() => window.location.href = '/assistant'} style={{
              padding: '12px 32px', borderRadius: '8px', border: 'none',
              background: 'var(--color-accent)', color: 'white', fontSize: '15px',
              fontWeight: 600, cursor: 'pointer',
            }}>Start Using Copilot →</button>
          </div>
        );
    }
  };

  return (
    <div style={{
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      height: '100vh', background: 'var(--color-bg-primary)',
    }}>
      <div style={{
        width: '440px', maxWidth: '90vw',
        background: 'var(--color-bg-secondary)',
        border: '1px solid var(--color-border)',
        borderRadius: '16px', padding: '40px',
      }}>
        {/* Step indicator */}
        <div style={{ display: 'flex', gap: '6px', marginBottom: '32px', justifyContent: 'center' }}>
          {[0, 1, 2, 3].map(s => (
            <div key={s} style={{
              width: '8px', height: '8px', borderRadius: '50%',
              background: step >= s ? 'var(--color-accent)' : 'var(--color-border)',
              transition: 'background 0.2s',
            }} />
          ))}
        </div>
        {renderStep()}
      </div>
    </div>
  );
}

export default SetupWizard;