import { Link, useLocation } from 'react-router-dom';
import { useState, type MouseEvent } from 'react';

const navItems = [
  { path: '/assistant', label: 'Assistant', icon: '💬' },
  { path: '/workspaces', label: 'Workspaces', icon: '📁' },
  { path: '/skills', label: 'Skills', icon: '🔧' },
  { path: '/knowledge', label: 'Knowledge', icon: '📚' },
  { path: '/activity', label: 'Activity', icon: '📊' },
  { path: '/settings', label: 'Settings', icon: '⚙️' },
  { path: '/about', label: 'About', icon: 'ℹ️' },
];

function Sidebar() {
  const [collapsed, setCollapsed] = useState(false);
  const location = useLocation();

  const NavLinkStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    textDecoration: 'none',
    fontSize: '13px',
    cursor: 'pointer',
  };

  return (
    <aside style={{
      width: collapsed ? '56px' : '220px',
      background: 'var(--color-bg-secondary)',
      borderRight: '1px solid var(--color-border)',
      display: 'flex',
      flexDirection: 'column',
      transition: 'width 0.2s',
      flexShrink: 0,
    }}>
      <div style={{
        padding: collapsed ? '12px 8px' : '16px',
        borderBottom: '1px solid var(--color-border)',
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        minWidth: '56px',
        justifyContent: collapsed ? 'center' : 'flex-start',
      }}>
        <span style={{ fontSize: '20px' }}>🧠</span>
        {!collapsed && <span style={{ fontWeight: 700, fontSize: '14px', whiteSpace: 'nowrap' }}>Wiki Labs AI Copilot</span>}
      </div>
      <nav style={{ flex: 1, padding: '8px 4px' }}>
        {navItems.map(item => {
          const isActive = location.pathname === item.path;
          return (
            <Link
              key={item.path}
              to={item.path}
              style={{
                ...NavLinkStyle,
                padding: collapsed ? '10px 0' : '10px 12px',
                margin: '2px 0',
                borderRadius: '6px',
                color: isActive ? 'var(--color-accent)' : 'var(--color-text-secondary)',
                background: isActive ? 'rgba(99, 102, 241, 0.1)' : 'transparent',
                fontWeight: isActive ? 600 : 400,
                justifyContent: collapsed ? 'center' : 'flex-start',
              }}
              title={collapsed ? item.label : undefined}
            >
              <span>{item.icon}</span>
              {!collapsed && <span>{item.label}</span>}
            </Link>
          );
        })}
      </nav>
      <div style={{ padding: '8px', borderTop: '1px solid var(--color-border)' }}>
        <button
          onClick={() => setCollapsed(!collapsed)}
          style={{
            width: '100%',
            padding: '6px',
            background: 'transparent',
            border: 'none',
            color: 'var(--color-text-secondary)',
            cursor: 'pointer',
            borderRadius: '4px',
            fontSize: '12px',
          }}
          onMouseOver={(e: MouseEvent<HTMLButtonElement>) => (e.target as HTMLButtonElement).style.background = 'var(--color-bg-tertiary)'}
          onMouseOut={(e: MouseEvent<HTMLButtonElement>) => (e.target as HTMLButtonElement).style.background = 'transparent'}
        >
          {collapsed ? '→' : '← Collapse'}
        </button>
      </div>
    </aside>
  );
}

export default Sidebar;