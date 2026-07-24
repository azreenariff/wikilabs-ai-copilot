import { useState, useEffect, useRef } from 'react';

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  created_at: string;
}

function ChatAssistant() {
  const [messages, setMessages] = useState<Message[]>([
    {
      id: '1',
      role: 'assistant',
      content: "I'm your Wiki Labs AI Copilot assistant. I'm ready to help!\n\nAvailable capabilities in this version:\n- Answer questions about your project\n- Help troubleshoot issues\n- Access workspace knowledge base\n- Execute guided workflows\n\nNote: Full AI responses require the app backend running.",
      created_at: new Date().toISOString(),
    },
  ]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadHistory();
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  async function loadHistory() {
    try {
      const res = await fetch('http://localhost:1420/api/commands/get_history', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: { workspace_id: 'default', limit: 50 } }),
      });
      if (res.ok) {
        const data = await res.json();
        if (data.success) {
          setMessages(data.value as Message[]);
        }
      }
    } catch (e) {
      console.error('Failed to load history:', e);
    }
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!input.trim() || loading) return;

    const userMsg: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: input.trim(),
      created_at: new Date().toISOString(),
    };

    setMessages(prev => [...prev, userMsg]);
    setInput('');
    setLoading(true);

    try {
      const res = await fetch('http://localhost:1420/api/commands/send_message', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          params: { message: userMsg.content, workspace_id: 'default' },
        }),
      });

      if (res.ok) {
        const data = await res.json();
        if (data.success) {
          setMessages(prev => [...prev, {
            ...data.value,
            role: 'assistant',
            created_at: data.value.created_at || new Date().toISOString(),
          } as Message]);
        }
      } else {
        throw new Error('API error');
      }
    } catch (err) {
      const fallbackMsg: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: `You asked: "${userMsg.content}"\n\nNote: AI response requires the app backend running.\n\nTo test the full flow:\n1. Run the Tauri app with \`npm run dev\`\n2. The assistant will connect to the AI provider`,
        created_at: new Date().toISOString(),
      };
      setMessages(prev => [...prev, fallbackMsg]);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      height: '100%',
      maxWidth: '900px',
      margin: '0 auto',
      width: '100%',
    }}>
      <div style={{
        flex: 1,
        overflow: 'auto',
        padding: '20px',
      }}>
        {messages.map(msg => (
          <div key={msg.id} style={{
            display: 'flex',
            justifyContent: msg.role === 'user' ? 'flex-end' : 'flex-start',
            marginBottom: '16px',
          }}>
            <div style={{
              maxWidth: '75%',
              padding: '12px 16px',
              borderRadius: '12px',
              background: msg.role === 'user' ? 'var(--color-user-msg)' : 'var(--color-assistant-msg)',
              color: msg.role === 'user' ? 'var(--color-user-text)' : 'var(--color-text-primary)',
              border: '1px solid var(--color-border)',
              fontSize: '14px',
              lineHeight: '1.5',
              whiteSpace: 'pre-wrap',
            }}>
              <div style={{
                fontSize: '11px',
                fontWeight: 600,
                marginBottom: '4px',
                opacity: 0.7,
              }}>
                {msg.role === 'user' ? '👤 You' : '🤖 Assistant'}
              </div>
              {msg.content}
              <div style={{
                fontSize: '10px',
                marginTop: '6px',
                opacity: 0.5,
                textAlign: 'right',
              }}>
                {new Date(msg.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
              </div>
            </div>
          </div>
        ))}
        {loading && (
          <div style={{
            display: 'flex',
            justifyContent: 'flex-start',
            marginBottom: '16px',
          }}>
            <div style={{
              padding: '12px 16px',
              borderRadius: '12px',
              background: 'var(--color-assistant-msg)',
              border: '1px solid var(--color-border)',
            }}>
              <div className="typing-indicator">
                <span></span>
                <span></span>
                <span></span>
              </div>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>
      <div style={{
        padding: '16px 20px',
        borderTop: '1px solid var(--color-border)',
      }}>
        <form onSubmit={handleSubmit} style={{
          display: 'flex',
          gap: '8px',
        }}>
          <input
            value={input}
            onChange={e => setInput(e.target.value)}
            placeholder="Ask a question..."
            style={{
              flex: 1,
              padding: '10px 14px',
              borderRadius: '8px',
              border: '1px solid var(--color-border)',
              background: 'var(--color-bg-secondary)',
              color: 'var(--color-text-primary)',
              fontSize: '14px',
              outline: 'none',
            }}
            onFocus={e => e.target.style.borderColor = 'var(--color-accent)'}
            onBlur={e => e.target.style.borderColor = 'var(--color-border)'}
          />
          <button
            type="submit"
            disabled={!input.trim() || loading}
            style={{
              padding: '10px 20px',
              borderRadius: '8px',
              border: 'none',
              background: input.trim() && !loading ? 'var(--color-accent)' : 'var(--color-bg-tertiary)',
              color: input.trim() && !loading ? 'white' : 'var(--color-text-secondary)',
              fontSize: '14px',
              fontWeight: 600,
              cursor: input.trim() && !loading ? 'pointer' : 'not-allowed',
              transition: 'all 0.15s',
            }}
          >
            {loading ? '...' : 'Send'}
          </button>
        </form>
      </div>
    </div>
  );
}

export default ChatAssistant;