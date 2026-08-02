import { useState, useEffect, useRef } from 'react';
import { marked } from 'marked';

// Extend Window interface for drag state
declare global {
  interface Window {
    __TAURI_DRAG_START?: boolean;
    __TAURI_DRAG_START_X?: number;
    __TAURI_DRAG_START_Y?: number;
  }
}

// Configure marked: disable global highlight to avoid XSS, use DOMPurify if available
marked.setOptions({
  breaks: true,
  gfm: true,
});

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  created_at: string;
  attachments?: string[]; // file paths/names
}

async function hideWindow() {
  try {
    await fetch('http://127.0.0.1:1420/api/commands/hide_advice_window', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ params: {} }),
    });
  } catch {}
}

async function showWindow() {
  try {
    await fetch('http://127.0.0.1:1420/api/commands/advice_chat_open', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ params: {} }),
    });
  } catch {}
}

// Drag-to-move handlers for the custom title bar
const dragState = { interval: null as ReturnType<typeof setTimeout> | null };
let dragDx = 0;
let dragDy = 0;

function handleDragMove(e: MouseEvent) {
  if (!(window as any).__TAURI_DRAG_START) return;
  dragDx += e.movementX || 0;
  dragDy += e.movementY || 0;
}

function handleDragEnd() {
  (window as any).__TAURI_DRAG_START = false;
  window.removeEventListener('mousemove', handleDragMove);
  window.removeEventListener('mouseup', handleDragEnd);
  // Flush remaining offset via HTTP request
  if (dragDx !== 0 || dragDy !== 0) {
    const adx = dragDx;
    const ady = dragDy;
    dragDx = 0;
    dragDy = 0;
    fetch('http://127.0.0.1:1420/api/commands/drag_advice_window', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ params: { dx: adx, dy: ady } }),
    }).catch(() => {});
  }
}

// Periodically flush accumulated drag offset to the backend
function flushDragOffset() {
  if (dragDx !== 0 || dragDy !== 0) {
    const adx = dragDx;
    const ady = dragDy;
    dragDx = 0;
    dragDy = 0;
    fetch('http://127.0.0.1:1420/api/commands/drag_advice_window', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ params: { dx: adx, dy: ady } }),
    }).catch(() => {});
  }
  if ((window as any).__TAURI_DRAG_START) {
    dragState.interval = setTimeout(flushDragOffset, 50);
  }
}

function ChatAssistant() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [dragOver, setDragOver] = useState(false);
  const [pendingAttachments, setPendingAttachments] = useState<string[]>([]);
  const [windowHidden, setWindowHidden] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    loadHistory();

    // Poll for new messages every 3 seconds for near real-time advice
    const interval = setInterval(loadHistory, 3000);
    return () => clearInterval(interval);
  }, []);

  // When window is hidden, poll for new advice every 1s to auto-restore
  useEffect(() => {
    if (!windowHidden) return;
    const iv = setInterval(async () => {
      try {
        const res = await fetch('http://127.0.0.1:1420/api/commands/guidance_get_active_recommendations', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ params: {} }),
        });
        const data = await res.json();
        if (data.success && data.value && data.value.length > 0) {
          // New advice available — restore the window
          await showWindow();
          setWindowHidden(false);
        }
      } catch {}
    }, 1000);
    return () => clearInterval(iv);
  }, [windowHidden]);

  // Track whether user has manually scrolled up so we don't auto-scroll
  const [isUserAtBottom, setIsUserAtBottom] = useState(true);

  // Detect user scrolling: track if user is at the bottom of the messages container
  useEffect(() => {
    const container = messagesEndRef.current?.parentElement;
    if (!container) return;

    const checkAtBottom = () => {
      const { scrollTop, scrollHeight, clientHeight } = container;
      // If within 50px of bottom, consider the user "at bottom"
      setIsUserAtBottom(scrollHeight - scrollTop - clientHeight < 50);
    };

    container.addEventListener('scroll', checkAtBottom);
    return () => container.removeEventListener('scroll', checkAtBottom);
  }, []);

  // Only auto-scroll to bottom when new messages arrive if user was already at bottom
  useEffect(() => {
    if (isUserAtBottom && messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages, pendingAttachments, isUserAtBottom]);

  // Auto-resize textarea
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = Math.min(textareaRef.current.scrollHeight, 160) + 'px';
    }
  }, [input, pendingAttachments]);

  async function loadHistory() {
    try {
      const res = await fetch('http://127.0.0.1:1420/api/commands/get_history', {
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

  // Handle file drops and file input
  function handleFiles(files: FileList | null) {
    if (!files || files.length === 0) return;
    const newAttachments: string[] = [];
    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      // Read as data URL for display, or store filename
      const reader = new FileReader();
      if (file.type.startsWith('image/')) {
        reader.onload = (e) => {
          if (e.target?.result) {
            newAttachments.push(`image:${e.target.result}`);
            setPendingAttachments(prev => [...prev, ...newAttachments]);
          }
        };
        reader.readAsDataURL(file);
      } else {
        // For non-image files, store as blob URL
        const url = URL.createObjectURL(file);
        newAttachments.push(`file:${file.name}:${url}`);
        setPendingAttachments(prev => [...prev, ...newAttachments]);
      }
    }
    // Wait for FileReader to complete (sync fallback)
    if (files.length > 0 && !files[0].type.startsWith('image/')) {
      setPendingAttachments(prev => [...prev, ...newAttachments]);
    }
  }

  function removeAttachment(idx: number) {
    setPendingAttachments(prev => {
      const url = prev[idx];
      if (url.startsWith('file:')) {
        try { URL.revokeObjectURL(url.split(':').slice(2).join(':')); } catch {}
      }
      return prev.filter((_, i) => i !== idx);
    });
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if ((!input.trim() && pendingAttachments.length === 0) || loading) return;

    // Build user message content — prepend file/image context
    let content = input.trim();
    const attachmentPreviews: string[] = [];
    const fileDescriptions: string[] = [];

    for (const att of pendingAttachments) {
      if (att.startsWith('image:')) {
        attachmentPreviews.push(att.substring(6)); // data URL for display
      } else if (att.startsWith('file:')) {
        const parts = att.split(':');
        if (parts.length >= 3) {
          const fileName = parts[2];
          fileDescriptions.push(`[file:${fileName}]`);
        }
      }
    }

    const userMsg: Message = {
      id: Date.now().toString(),
      role: 'user',
      content,
      created_at: new Date().toISOString(),
      attachments: attachmentPreviews.length > 0 ? attachmentPreviews : undefined,
    };

    setMessages(prev => [...prev, userMsg]);
    setInput('');
    setPendingAttachments([]);
    setLoading(true);

    try {
      // Build the full message for AI: include file descriptions and image context
      let aiMessage = content;
      if (fileDescriptions.length > 0) {
        aiMessage = `[Attached: ${fileDescriptions.join(', ')}]\n\n${content}`;
      }
      if (attachmentPreviews.length > 0) {
        aiMessage = `[${attachmentPreviews.length} image(s) attached - see preview above]\n\n${aiMessage}`;
      }

      const res = await fetch('http://127.0.0.1:1420/api/commands/send_message', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          params: { message: aiMessage, workspace_id: 'default' },
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

  function handleKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    // Enter without Shift = send message
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e as unknown as React.FormEvent);
      return;
    }
    // Shift+Enter = insert newline
    if (e.key === 'Enter' && e.shiftKey) {
      e.preventDefault();
      const start = e.currentTarget.selectionStart;
      const end = e.currentTarget.selectionEnd;
      const val = e.currentTarget.value;
      const newVal = val.substring(0, start) + '\n' + val.substring(end);
      setInput(newVal);
      // Move cursor after the inserted newline
      requestAnimationFrame(() => {
        if (e.currentTarget.selectionStart !== undefined) {
          e.currentTarget.selectionStart = e.currentTarget.selectionEnd = start + 1;
        }
      });
      return;
    }
    // Tab in textarea = insert 2 spaces
    if (e.key === 'Tab') {
      e.preventDefault();
      const start = e.currentTarget.selectionStart;
      const end = e.currentTarget.selectionEnd;
      const val = e.currentTarget.value;
      setInput(val.substring(0, start) + '  ' + val.substring(end));
      requestAnimationFrame(() => {
        if (e.currentTarget.selectionStart !== undefined) {
          e.currentTarget.selectionStart = e.currentTarget.selectionEnd = start + 2;
        }
      });
    }
  }

  return (
      <>
        {/* ── Desktop roll-up pill when OS window is hidden ── */}
        {windowHidden && (
          <div
            onClick={async () => {
              await showWindow();
              setWindowHidden(false);
            }}
            style={{
              position: 'fixed',
              bottom: '16px',
              right: '16px',
              background: 'var(--color-bg-secondary)',
              border: '1px solid var(--color-accent)',
              borderRadius: '12px',
              padding: '8px 14px',
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              cursor: 'pointer',
              boxShadow: '0 4px 16px rgba(0,0,0,0.4)',
              zIndex: 10000,
              transition: 'all 0.2s',
              fontSize: '13px',
              color: 'var(--color-text-primary)',
            }}
            title="Click to open advice chat"
          >
            <span style={{ fontSize: '16px' }}>💬</span>
            <span>AI Copilot Advice</span>
          </div>
        )}

        {/* ── Main window content (only shown when window is NOT hidden) ── */}
        {!windowHidden && (
          <div style={{
            display: 'flex',
            flexDirection: 'column',
            height: '100%',
            maxWidth: '900px',
            margin: '0 auto',
            width: '100%',
          }}>
            {/* ── Custom title bar (no native decorations) ── */}
            <div
              onMouseDown={(e) => {
                // Only start drag on left side of title bar (not buttons)
                if ((e.target as HTMLElement).closest('.tb-no-drag')) return;
                // Start drag
                (window as any).__TAURI_DRAG_START = true;
                window.addEventListener('mousemove', handleDragMove);
                window.addEventListener('mouseup', handleDragEnd);
                // Start periodic flush
                dragState.interval = setTimeout(flushDragOffset, 50);
              }}
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                padding: '6px 10px',
                background: 'var(--color-bg-primary)',
                borderBottom: '1px solid var(--color-border)',
                cursor: 'move',
                userSelect: 'none',
                WebkitUserSelect: 'none',
                MozUserSelect: 'none',
                msUserSelect: 'none',
                flexShrink: 0,
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <span style={{ fontSize: '14px' }}>🤖</span>
                <span style={{ fontSize: '13px', fontWeight: 600, color: 'var(--color-text-primary)' }}>
                  AI Copilot — Live Advice
                </span>
              </div>
              <div style={{ display: 'flex', gap: '4px' }} className="tb-no-drag">
                {/* Minimize button — rolls up to desktop pill */}
                <button
                  onClick={() => {
                    setWindowHidden(true);
                    hideWindow();
                  }}
                  style={{
                    background: 'transparent',
                    border: '1px solid var(--color-border)',
                    color: 'var(--color-text-secondary)',
                    borderRadius: '6px',
                    padding: '2px 8px',
                    cursor: 'pointer',
                    fontSize: '12px',
                    lineHeight: 1,
                  }}
                  title="Minimize to roll-up pill"
                >
                  ─
                </button>
              </div>
            </div>
          {/* ── Messages area ── */}
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
                  maxWidth: '80%',
                  padding: '12px 16px',
                  borderRadius: '16px',
                  background: msg.role === 'user' ? 'var(--color-user-msg)' : 'var(--color-assistant-msg)',
                  color: msg.role === 'user' ? 'var(--color-user-text)' : 'var(--color-text-primary)',
                  border: 'none',
                  fontSize: '14px',
                  lineHeight: '1.6',
                  wordBreak: 'break-word',
                  borderBottomLeftRadius: msg.role === 'assistant' ? '4px' : undefined,
                  borderBottomRightRadius: msg.role === 'user' ? '4px' : undefined,
                }}>
                  {msg.role === 'assistant'
                    ? <div dangerouslySetInnerHTML={{ __html: marked.parse(msg.content, { async: false }) as string }} />
                    : <div dangerouslySetInnerHTML={{ __html: marked.parse(msg.content, { async: false }) as string }} />
                  }

                  {/* Render attached images for user messages */}
                  {msg.attachments && msg.role === 'user' && msg.attachments.map((att, idx) => {
                    if (att.startsWith('http') || att.startsWith('data:image')) {
                      return (
                        <img
                          key={idx}
                          src={att}
                          alt="Attached image"
                          style={{
                            maxWidth: '240px',
                            maxHeight: '180px',
                            borderRadius: '8px',
                            marginTop: '8px',
                            display: 'block',
                          }}
                        />
                      );
                    }
                    return null;
                  })}

                  <div style={{
                    fontSize: '10px',
                    marginTop: '6px',
                    opacity: 0.4,
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

          {/* Input area */}
          <div style={{
            padding: '12px 20px',
            borderTop: '1px solid var(--color-border)',
            background: 'var(--color-bg-secondary)',
          }}>
            {/* Pending attachment preview */}
            {pendingAttachments.length > 0 && (
              <div style={{
                display: 'flex',
                gap: '8px',
                marginBottom: '8px',
                flexWrap: 'wrap',
              }}>
                {pendingAttachments.map((att, idx) => (
                  <div
                    key={idx}
                    style={{
                      position: 'relative',
                      display: 'inline-flex',
                      alignItems: 'center',
                      gap: '6px',
                      background: 'rgba(99,102,241,0.1)',
                      border: '1px solid var(--color-border)',
                      borderRadius: '8px',
                      padding: '4px 8px',
                      fontSize: '12px',
                    }}
                  >
                    {att.startsWith('image:') ? (
                      <img
                        src={att.substring(6)}
                        alt="Preview"
                        style={{
                          width: '32px',
                          height: '32px',
                          objectFit: 'cover',
                          borderRadius: '4px',
                        }}
                      />
                    ) : att.startsWith('file:') ? (
                      <span style={{ color: 'var(--color-text-secondary)' }}>📄</span>
                    ) : null}
                    <span style={{ color: 'var(--color-text-primary)' }}>
                      {att.startsWith('file:') ? att.split(':').slice(2, 3).join(':') : 'Image'}
                    </span>
                    <button
                      onClick={() => removeAttachment(idx)}
                      style={{
                        background: 'transparent',
                        border: 'none',
                        color: 'var(--color-text-secondary)',
                        cursor: 'pointer',
                        fontSize: '14px',
                        padding: '0 4px',
                        lineHeight: 1,
                      }}
                    >
                      ✕
                    </button>
                  </div>
                ))}
              </div>
            )}

            {/* Drag-over indicator */}
            {dragOver && (
              <div style={{
                padding: '12px',
                margin: '0 -20px 8px',
                textAlign: 'center',
                background: 'rgba(99,102,241,0.15)',
                border: '2px dashed var(--color-accent)',
                borderRadius: '8px',
                fontSize: '13px',
                color: 'var(--color-accent)',
              }}>
                Drop files here to attach
              </div>
            )}

            <form onSubmit={handleSubmit} style={{
              display: 'flex',
              gap: '8px',
              alignItems: 'flex-end',
            }}>
              {/* File attachment button */}
              <button
                type="button"
                onClick={() => fileInputRef.current?.click()}
                style={{
                  padding: '10px 12px',
                  borderRadius: '8px',
                  border: '1px solid var(--color-border)',
                  background: dragOver ? 'rgba(99,102,241,0.2)' : 'var(--color-bg-tertiary)',
                  color: 'var(--color-text-primary)',
                  fontSize: '18px',
                  cursor: 'pointer',
                  transition: 'all 0.15s',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  minWidth: '40px',
                }}
                title="Attach image or file"
                onMouseEnter={e => {
                  if (!dragOver) e.currentTarget.style.borderColor = 'var(--color-accent)';
                }}
                onMouseLeave={e => {
                  if (!dragOver) e.currentTarget.style.borderColor = 'var(--color-border)';
                }}
              >
                📎
              </button>
              <input
                ref={fileInputRef}
                type="file"
                accept="image/*,.txt,.md,.json,.csv,.log,.xml,.html,.css,.js,.ts,.py,.sh"
                multiple
                onChange={e => handleFiles(e.target.files)}
                style={{ display: 'none' }}
              />

              {/* Textarea for message input */}
              <textarea
                ref={textareaRef}
                value={input}
                onChange={e => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Ask a question... (Shift+Enter for new line)"
                rows={1}
                style={{
                  flex: 1,
                  padding: '10px 14px',
                  borderRadius: '8px',
                  border: '1px solid var(--color-border)',
                  background: 'var(--color-bg-secondary)',
                  color: 'var(--color-text-primary)',
                  fontSize: '14px',
                  outline: 'none',
                  resize: 'none',
                  fontFamily: 'inherit',
                  lineHeight: '1.5',
                }}
                onFocus={e => e.target.style.borderColor = 'var(--color-accent)'}
                onBlur={e => e.target.style.borderColor = 'var(--color-border)'}
                onDragOver={e => { e.preventDefault(); setDragOver(true); }}
                onDragLeave={() => setDragOver(false)}
                onDrop={e => {
                  e.preventDefault();
                  setDragOver(false);
                  handleFiles(e.dataTransfer.files);
                }}
              />

              <button
                type="submit"
                disabled={!input.trim() && pendingAttachments.length === 0 || loading}
                style={{
                  padding: '10px 20px',
                  borderRadius: '8px',
                  border: 'none',
                  background: (input.trim() || pendingAttachments.length > 0) && !loading ? 'var(--color-accent)' : 'var(--color-bg-tertiary)',
                  color: (input.trim() || pendingAttachments.length > 0) && !loading ? 'white' : 'var(--color-text-secondary)',
                  fontSize: '14px',
                  fontWeight: 600,
                  cursor: (input.trim() || pendingAttachments.length > 0) && !loading ? 'pointer' : 'not-allowed',
                  transition: 'all 0.15s',
                  minHeight: '40px',
                }}
              >
                {loading ? '...' : 'Send'}
              </button>
            </form>
          </div>
        </div>
      )}
    </>
  );
}

export default ChatAssistant;