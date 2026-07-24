import { useState, useEffect, useCallback } from 'react';

interface AiProvider {
  name: string;
  url: string;
  api_version: string;
}

interface AiProviderConfig {
  name: string;
  endpoint: string;
  api_key: string;
  model: string;
  max_tokens: number;
  context_window: number;
}

// Common models list for dropdown — fallback when endpoint can't be reached
const commonModels: Record<string, string[]> = {
  OpenAI: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-4', 'o1', 'o1-mini', 'o3-mini'],
  OpenRouter: ['anthropic/claude-sonnet-4', 'anthropic/claude-3.5-haiku', 'google/gemini-2.0-flash-001', 'deepseek/deepseek-chat', 'meta-llama/llama-3-70b-instruct', 'mistralai/mixtral-8x22b-instruct'],
  'Custom Endpoint': ['meta-llama/Llama-3-70b-chat-hf', 'meta-llama/Llama-3-8b-chat-hf', 'mistralai/Mixtral-8x7B-Instruct-v0.1', 'gpt-4o', 'gpt-4o-mini'],
  Ollama: ['llama3', 'mistral', 'codellama', 'llama2', 'vicuna', 'phi3', 'gemma'],
};

function Settings() {
  const [settings, setSettings] = useState({
    ai_provider: {
      name: 'openai',
      endpoint: 'https://api.openai.com/v1',
      api_key: '',
      model: 'gpt-4o',
      max_tokens: 4096,
      context_window: 128000,
    } as AiProviderConfig,
    theme: 'dark',
    log_level: 'info',
    privacy_mode: false,
  });

  const [providers, setProviders] = useState<AiProvider[]>([]);
  const [status, setStatus] = useState('');
  const [saving, setSaving] = useState(false);
  const [fetchedModels, setFetchedModels] = useState<string[]>([]);
  const [loadingModels, setLoadingModels] = useState(false);

  // Fetch models from the endpoint's /v1/models API
  const fetchModels = useCallback(async (endpoint: string, apiKey: string) => {
    if (!endpoint) {
      setFetchedModels([]);
      return;
    }
    setLoadingModels(true);
    try {
      const res = await fetch('http://localhost:1420/api/commands/list_models', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: { endpoint, api_key: apiKey } }),
      });
      const data = await res.json();
      if (data.success && Array.isArray(data.value) && data.value.length > 0) {
        setFetchedModels(data.value);
        return;
      }
      // Couldn't fetch — fall back to hardcoded
      setFetchedModels([]);
    } catch {
      setFetchedModels([]);
    } finally {
      setLoadingModels(false);
    }
  }, []);

  // Fetch models when endpoint changes
  useEffect(() => {
    if (settings.ai_provider.endpoint) {
      fetchModels(settings.ai_provider.endpoint, settings.ai_provider.api_key);
    }
  }, [settings.ai_provider.endpoint, settings.ai_provider.api_key, fetchModels]);

  useEffect(() => {
    // Fetch saved settings
    fetch('http://localhost:1420/api/commands/get_settings', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ params: {} }),
    })
      .then(r => r.json())
      .then(data => {
        if (data.success && data.value) {
          setSettings(prev => ({
            ...prev,
            ...data.value,
          }));
        }
      })
      .catch(() => {});

    // Fetch available providers
    fetch('http://localhost:1420/api/commands/list_providers', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ params: {} }),
    })
      .then(r => r.json())
      .then(data => {
        if (data.success) {
          setProviders(data.value);
        } else {
          // Fallback providers
          setProviders([
            { name: 'OpenAI', url: 'https://api.openai.com/v1', api_version: 'v1' },
            { name: 'OpenRouter', url: 'https://openrouter.ai/api/v1', api_version: 'v1' },
            { name: 'Custom Endpoint', url: 'http://localhost:8000/v1', api_version: 'v1' },
            { name: 'Ollama', url: 'http://localhost:11434/v1', api_version: 'v1' },
          ]);
        }
      })
      .catch(() => {
        // Default providers on error
        setProviders([
          { name: 'OpenAI', url: 'https://api.openai.com/v1', api_version: 'v1' },
          { name: 'OpenRouter', url: 'https://openrouter.ai/api/v1', api_version: 'v1' },
          { name: 'Custom Endpoint', url: 'http://localhost:8000/v1', api_version: 'v1' },
          { name: 'Ollama', url: 'http://localhost:11434/v1', api_version: 'v1' },
        ]);
      });
  }, []);

  async function handleSave() {
    setSaving(true);
    setStatus('');
    try {
      await fetch('http://localhost:1420/api/commands/update_settings', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: settings }),
      });
      setStatus('✓ Settings saved');
    } catch (err) {
      setStatus('✗ Failed to save settings');
    }
    setSaving(false);
  }

  async function handleTestConnection() {
    setStatus('Testing...');
    try {
      const res = await fetch('http://localhost:1420/api/commands/test_connection', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: settings.ai_provider }),
      });
      const data = await res.json();
      if (data.success) {
        setStatus(data.value ? '✓ Connection successful' : '✗ Connection failed');
      } else {
        setStatus(`✗ ${data.error || 'Connection failed'}`);
      }
    } catch {
      setStatus('⚠ Cannot reach backend');
    }
  }

  async function handleRefreshModels() {
    await fetchModels(settings.ai_provider.endpoint, settings.ai_provider.api_key);
    if (fetchedModels.length > 0) {
      setStatus(`✓ Loaded ${fetchedModels.length} models from endpoint`);
    } else {
      setStatus('✗ Could not fetch models from endpoint');
    }
  }

  const inputStyle: React.CSSProperties = {
    width: '100%',
    padding: '8px 12px',
    borderRadius: '6px',
    border: '1px solid var(--color-border)',
    background: 'var(--color-bg-primary)',
    color: 'var(--color-text-primary)',
    fontSize: '13px',
    outline: 'none',
    boxSizing: 'border-box',
  };

  // Build the model list: fetched models first, then fallback hardcoded, then current model
  const hardcodedList = commonModels[settings.ai_provider.name] || [];
  const modelList = fetchedModels.length > 0 ? fetchedModels : hardcodedList;
  // Always include the current model value in the options
  const allModelOptions = modelList.includes(settings.ai_provider.model)
    ? modelList
    : [settings.ai_provider.model, ...modelList];

  return (
    <div style={{ padding: '32px', maxWidth: '700px', margin: '0 auto' }}>
      <h2 style={{ marginBottom: '24px', color: 'var(--color-text-primary)' }}>⚙️ Settings</h2>

      {/* AI Provider Section */}
      <div style={{
        background: 'var(--color-bg-secondary)',
        border: '1px solid var(--color-border)',
        borderRadius: '12px',
        padding: '24px',
        marginBottom: '16px',
      }}>
        <h3 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '16px' }}>AI Provider</h3>
        <div style={{ display: 'grid', gap: '12px' }}>
          {/* Provider dropdown */}
          <div>
            <label style={{ fontSize: '12px', color: 'var(--color-text-secondary)', display: 'block', marginBottom: '4px' }}>
              Provider
            </label>
            <select
              value={settings.ai_provider.name}
              onChange={e => {
                const providerName = e.target.value;
                const matched = providers.find(p => p.name === providerName);
                setSettings(prev => ({
                  ...prev,
                  ai_provider: {
                    ...prev.ai_provider,
                    name: providerName,
                    endpoint: matched ? matched.url : prev.ai_provider.endpoint,
                  },
                }));
              }}
              style={inputStyle}
            >
              {providers.map(p => (
                <option key={p.name} value={p.name}>{p.name}</option>
              ))}
            </select>
          </div>

          {/* Endpoint */}
          <div>
            <label style={{ fontSize: '12px', color: 'var(--color-text-secondary)', display: 'block', marginBottom: '4px' }}>
              Endpoint URL
            </label>
            <input
              value={settings.ai_provider.endpoint}
              onChange={e => setSettings(prev => ({
                ...prev,
                ai_provider: { ...prev.ai_provider, endpoint: e.target.value },
              }))}
              style={inputStyle}
            />
          </div>

          {/* API Key */}
          <div>
            <label style={{ fontSize: '12px', color: 'var(--color-text-secondary)', display: 'block', marginBottom: '4px' }}>
              API Key
            </label>
            <input
              type="password"
              value={settings.ai_provider.api_key}
              onChange={e => setSettings(prev => ({
                ...prev,
                ai_provider: { ...prev.ai_provider, api_key: e.target.value },
              }))}
              placeholder="sk-..."
              style={inputStyle}
            />
          </div>

          {/* Model — dynamically fetched from endpoint */}
          <div>
            <label style={{ fontSize: '12px', color: 'var(--color-text-secondary)', display: 'block', marginBottom: '4px' }}>
              Model
            </label>
            <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
              <select
                value={settings.ai_provider.model}
                onChange={e => setSettings(prev => ({
                  ...prev,
                  ai_provider: { ...prev.ai_provider, model: e.target.value },
                }))}
                style={{ ...inputStyle, flex: 1 }}
              >
                {loadingModels && (
                  <option value="" disabled>Loading models...</option>
                )}
                {!loadingModels && allModelOptions.map(m => (
                  <option key={m} value={m}>{m}</option>
                ))}
                {!loadingModels && allModelOptions.length === 0 && (
                  <option value="" disabled>No models available</option>
                )}
              </select>
              <button
                onClick={handleRefreshModels}
                disabled={loadingModels || !settings.ai_provider.endpoint}
                style={{
                  padding: '8px 12px',
                  borderRadius: '6px',
                  border: '1px solid var(--color-border)',
                  background: 'transparent',
                  color: 'var(--color-text-primary)',
                  cursor: loadingModels ? 'not-allowed' : 'pointer',
                  fontSize: '13px',
                  opacity: loadingModels ? 0.5 : 1,
                  whiteSpace: 'nowrap',
                }}
                title="Refresh model list from endpoint"
              >
                {loadingModels ? '⟳ Loading...' : '↻ Refresh'}
              </button>
            </div>
          </div>

          {/* Tokens & Context */}
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
            <div>
              <label style={{ fontSize: '12px', color: 'var(--color-text-secondary)', display: 'block', marginBottom: '4px' }}>
                Max Tokens
              </label>
              <input
                type="number"
                value={settings.ai_provider.max_tokens}
                onChange={e => setSettings(prev => ({
                  ...prev,
                  ai_provider: { ...prev.ai_provider, max_tokens: parseInt(e.target.value) || 4096 },
                }))}
                style={inputStyle}
              />
            </div>
            <div>
              <label style={{ fontSize: '12px', color: 'var(--color-text-secondary)', display: 'block', marginBottom: '4px' }}>
                Context Window
              </label>
              <input
                type="number"
                value={settings.ai_provider.context_window}
                onChange={e => setSettings(prev => ({
                  ...prev,
                  ai_provider: { ...prev.ai_provider, context_window: parseInt(e.target.value) || 128000 },
                }))}
                style={inputStyle}
              />
            </div>
          </div>

          {/* Test Connection Button */}
          <button
            onClick={handleTestConnection}
            style={{
              padding: '8px 16px',
              borderRadius: '6px',
              border: '1px solid var(--color-border)',
              background: 'transparent',
              color: 'var(--color-text-primary)',
              cursor: 'pointer',
              fontSize: '13px',
              alignSelf: 'flex-start',
            }}
          >
            Test Connection
          </button>

          {/* Status message */}
          {status && (
            <div style={{
              padding: '8px 12px',
              borderRadius: '6px',
              fontSize: '13px',
              background: status.includes('✓') ? 'rgba(34, 197, 94, 0.1)' :
                          status.includes('✗') ? 'rgba(239, 68, 68, 0.1)' :
                          status.includes('⚠') ? 'rgba(251, 191, 36, 0.1)' :
                          'rgba(99, 102, 241, 0.1)',
              color: status.includes('✓') ? 'var(--color-success)' :
                     status.includes('✗') ? 'var(--color-error)' :
                     status.includes('⚠') ? 'fbbf24' :
                     'var(--color-accent)',
            }}>
              {status}
            </div>
          )}
        </div>
      </div>

      {/* Application Settings */}
      <div style={{
        background: 'var(--color-bg-secondary)',
        border: '1px solid var(--color-border)',
        borderRadius: '12px',
        padding: '24px',
        marginBottom: '16px',
      }}>
        <h3 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '16px' }}>Application</h3>
        <div style={{ display: 'grid', gap: '12px' }}>
          {/* Theme */}
          <div>
            <label style={{ fontSize: '12px', color: 'var(--color-text-secondary)', display: 'block', marginBottom: '4px' }}>
              Theme
            </label>
            <select
              value={settings.theme}
              onChange={e => setSettings(prev => ({ ...prev, theme: e.target.value }))}
              style={inputStyle}
            >
              <option value="dark">Dark</option>
              <option value="light">Light</option>
            </select>
          </div>

          {/* Log Level */}
          <div>
            <label style={{ fontSize: '12px', color: 'var(--color-text-secondary)', display: 'block', marginBottom: '4px' }}>
              Logging Level
            </label>
            <select
              value={settings.log_level}
              onChange={e => setSettings(prev => ({ ...prev, log_level: e.target.value }))}
              style={inputStyle}
            >
              <option value="debug">Debug</option>
              <option value="info">Info</option>
              <option value="warn">Warn</option>
              <option value="error">Error</option>
            </select>
          </div>

          {/* Privacy Mode */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <input
              type="checkbox"
              checked={settings.privacy_mode}
              onChange={e => setSettings(prev => ({ ...prev, privacy_mode: e.target.checked }))}
              id="privacy_mode"
            />
            <label htmlFor="privacy_mode" style={{ fontSize: '13px' }}>
              Privacy Mode (minimize logged data)
            </label>
          </div>
        </div>
      </div>

      {/* Save Settings Button */}
      <button
        onClick={handleSave}
        disabled={saving}
        style={{
          width: '100%',
          padding: '10px',
          borderRadius: '8px',
          border: 'none',
          background: 'var(--color-accent)',
          color: 'white',
          fontSize: '14px',
          fontWeight: 600,
          cursor: saving ? 'not-allowed' : 'pointer',
          opacity: saving ? 0.6 : 1,
        }}
      >
        {saving ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
  );
}

export default Settings;