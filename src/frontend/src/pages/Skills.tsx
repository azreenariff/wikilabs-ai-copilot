import { useState, useEffect, useCallback } from 'react';

interface SkillCard {
  id: string;
  name: string;
  version: string;
  technology: string;
  category: string;
  enabled: boolean;
  active: boolean;
  confidence: number;
  dependencies: string[];
  knowledge_count: number;
  workflow_count: number;
  detection_count: number;
  guidance_count: number;
  command_count: number;
  validated: boolean;
  path: string;
  vendor: string;
  environments: string[];
  docs_link: string;
  updated_at: string | null;
}

function Skills() {
  const [skills, setSkills] = useState<SkillCard[]>([]);
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState('');
  const [selectedSkill, setSelectedSkill] = useState<SkillCard | null>(null);
  const [validationIssues, setValidationIssues] = useState<string[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterCategory, setFilterCategory] = useState<string>('all');
  const [filterStatus, setFilterStatus] = useState<string>('all');

  const fetchSkills = useCallback(async () => {
    try {
      const res = await fetch('http://localhost:1420/api/commands/skill_list', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: {} }),
      });
      const data = await res.json();
      if (data.success && data.value) {
        setSkills(data.value);
      }
    } catch (e) {
      console.error('Failed to load skills:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchSkills();
  }, [fetchSkills]);

  const toggleSkill = async (name: string) => {
    const skill = skills.find(s => s.id === name || s.name === name);
    if (!skill) return;
    try {
      const cmd = skill.enabled ? 'skill_disable' : 'skill_enable';
      const res = await fetch(`http://localhost:1420/api/commands/${cmd}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: { name } }),
      });
      const data = await res.json();
      if (data.success) {
        setStatus(`${skill.enabled ? 'Disabled' : 'Enabled'} ${name}`);
        fetchSkills();
      } else {
        setStatus(`Error: ${data.error}`);
      }
    } catch {
      setStatus('Failed to toggle skill');
    }
  };

  const validateSkill = async (name: string) => {
    try {
      const res = await fetch('http://localhost:1420/api/commands/skill_validate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: { name } }),
      });
      const data = await res.json();
      if (data.success && data.value) {
        setValidationIssues(data.value);
      } else {
        setValidationIssues(['No issues found']);
      }
    } catch {
      setValidationIssues(['Failed to validate']);
    }
  };

  const categories = Array.from(new Set(skills.map(s => s.category)));
  const filteredSkills = skills.filter(skill => {
    const matchesSearch = searchQuery === '' ||
      skill.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      skill.technology.toLowerCase().includes(searchQuery.toLowerCase()) ||
      skill.id.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesCategory = filterCategory === 'all' || skill.category === filterCategory;
    const matchesStatus = filterStatus === 'all' ||
      (filterStatus === 'enabled' && skill.enabled) ||
      (filterStatus === 'disabled' && !skill.enabled) ||
      (filterStatus === 'active' && skill.active) ||
      (filterStatus === 'inactive' && !skill.active);
    return matchesSearch && matchesCategory && matchesStatus;
  });

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return 'var(--color-success)';
    if (confidence >= 0.5) return '#fbbf24';
    return 'var(--color-error)';
  };

  return (
    <div style={{ padding: '32px', maxWidth: '900px', margin: '0 auto' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <h2 style={{ margin: 0, color: 'var(--color-text-primary)' }}>🔧 Skills</h2>
        <button
          onClick={fetchSkills}
          style={{
            padding: '6px 12px',
            borderRadius: '6px',
            border: '1px solid var(--color-border)',
            background: 'transparent',
            color: 'var(--color-text-primary)',
            cursor: 'pointer',
            fontSize: '13px',
          }}
        >
          ↻ Refresh
        </button>
      </div>

      {status && (
        <div style={{
          padding: '8px 12px',
          borderRadius: '6px',
          fontSize: '13px',
          marginBottom: '12px',
          background: 'rgba(99, 102, 241, 0.1)',
          color: 'var(--color-accent)',
        }}>
          {status}
        </div>
      )}

      {/* Filters */}
      <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr 1fr', gap: '12px', marginBottom: '16px' }}>
        <input
          type="text"
          placeholder="Search skills..."
          value={searchQuery}
          onChange={e => setSearchQuery(e.target.value)}
          style={{
            padding: '8px 12px',
            borderRadius: '6px',
            border: '1px solid var(--color-border)',
            background: 'var(--color-bg-primary)',
            color: 'var(--color-text-primary)',
            fontSize: '13px',
            outline: 'none',
          }}
        />
        <select
          value={filterCategory}
          onChange={e => setFilterCategory(e.target.value)}
          style={{
            padding: '8px 12px',
            borderRadius: '6px',
            border: '1px solid var(--color-border)',
            background: 'var(--color-bg-primary)',
            color: 'var(--color-text-primary)',
            fontSize: '13px',
            outline: 'none',
          }}
        >
          <option value="all">All Categories</option>
          {categories.map(cat => <option key={cat} value={cat}>{cat}</option>)}
        </select>
        <select
          value={filterStatus}
          onChange={e => setFilterStatus(e.target.value)}
          style={{
            padding: '8px 12px',
            borderRadius: '6px',
            border: '1px solid var(--color-border)',
            background: 'var(--color-bg-primary)',
            color: 'var(--color-text-primary)',
            fontSize: '13px',
            outline: 'none',
          }}
        >
          <option value="all">All Status</option>
          <option value="enabled">Enabled</option>
          <option value="disabled">Disabled</option>
          <option value="active">Active</option>
          <option value="inactive">Inactive</option>
        </select>
      </div>

      {loading ? (
        <div style={{ textAlign: 'center', padding: '48px', color: 'var(--color-text-secondary)' }}>
          Loading skills...
        </div>
      ) : skills.length === 0 ? (
        <div style={{
          background: 'var(--color-bg-secondary)',
          border: '1px solid var(--color-border)',
          borderRadius: '12px',
          padding: '48px',
          textAlign: 'center',
          color: 'var(--color-text-secondary)',
        }}>
          <p style={{ fontSize: '16px' }}>No skills installed</p>
          <p style={{ fontSize: '13px', marginTop: '8px' }}>Skills are loaded from your skills directory.</p>
        </div>
      ) : (
        <div style={{ display: 'grid', gap: '12px' }}>
          {filteredSkills.map(skill => (
            <div
              key={skill.id}
              onClick={() => setSelectedSkill(selectedSkill?.id === skill.id ? null : skill)}
              style={{
                background: 'var(--color-bg-secondary)',
                border: `1px solid ${selectedSkill?.id === skill.id ? 'var(--color-accent)' : 'var(--color-border)'}`,
                borderRadius: '12px',
                padding: '16px',
                cursor: 'pointer',
                transition: 'all 0.15s',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                <div style={{ flex: 1 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
                    <h3 style={{ fontSize: '15px', fontWeight: 600, margin: 0, color: 'var(--color-text-primary)' }}>
                      {skill.name}
                    </h3>
                    <span style={{
                      fontSize: '11px',
                      padding: '2px 6px',
                      borderRadius: '4px',
                      background: skill.enabled ? 'rgba(34, 197, 94, 0.15)' : 'rgba(107, 114, 128, 0.15)',
                      color: skill.enabled ? 'var(--color-success)' : 'var(--color-text-secondary)',
                    }}>
                      {skill.enabled ? '✓ Enabled' : 'Disabled'}
                    </span>
                    <span style={{
                      fontSize: '11px',
                      padding: '2px 6px',
                      borderRadius: '4px',
                      background: skill.active ? 'rgba(99, 102, 241, 0.15)' : 'rgba(107, 114, 128, 0.15)',
                      color: skill.active ? 'var(--color-accent)' : 'var(--color-text-secondary)',
                    }}>
                      {skill.active ? 'Active' : 'Inactive'}
                    </span>
                    <span style={{
                      fontSize: '11px',
                      padding: '2px 6px',
                      borderRadius: '4px',
                      background: skill.validated ? 'rgba(34, 197, 94, 0.15)' : 'rgba(107, 114, 128, 0.15)',
                      color: skill.validated ? 'var(--color-success)' : 'var(--color-text-secondary)',
                    }}>
                      {skill.validated ? '✓ Validated' : 'Not Validated'}
                    </span>
                  </div>
                  <p style={{ fontSize: '12px', color: 'var(--color-text-secondary)', margin: '4px 0' }}>
                    {skill.technology} • {skill.category} • {skill.vendor} • v{skill.version}
                  </p>
                  {skill.dependencies.length > 0 && (
                    <div style={{ fontSize: '11px', color: 'var(--color-text-secondary)', marginTop: '4px' }}>
                      Deps: {skill.dependencies.join(', ')}
                    </div>
                  )}
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '4px' }}>
                  <div style={{ fontSize: '14px', fontWeight: 600, color: getConfidenceColor(skill.confidence) }}>
                    {Math.round(skill.confidence * 100)}%
                  </div>
                  <div style={{ fontSize: '10px', color: 'var(--color-text-secondary)' }}>confidence</div>
                </div>
              </div>

              {/* Expanded details */}
              {selectedSkill?.id === skill.id && (
                <div style={{
                  marginTop: '12px',
                  paddingTop: '12px',
                  borderTop: '1px solid var(--color-border)',
                  display: 'grid',
                  gridTemplateColumns: '1fr 1fr 1fr',
                  gap: '8px',
                  fontSize: '12px',
                }}>
                  <div style={{ color: 'var(--color-text-secondary)' }}>
                    <strong>Knowledge:</strong> {skill.knowledge_count}
                  </div>
                  <div style={{ color: 'var(--color-text-secondary)' }}>
                    <strong>Workflows:</strong> {skill.workflow_count}
                  </div>
                  <div style={{ color: 'var(--color-text-secondary)' }}>
                    <strong>Detections:</strong> {skill.detection_count}
                  </div>
                  <div style={{ color: 'var(--color-text-secondary)' }}>
                    <strong>Guidance Rules:</strong> {skill.guidance_count}
                  </div>
                  <div style={{ color: 'var(--color-text-secondary)' }}>
                    <strong>Commands:</strong> {skill.command_count}
                  </div>
                  <div style={{ color: 'var(--color-text-secondary)' }}>
                    <strong>Env:</strong> {skill.environments.join(', ') || 'All'}
                  </div>
                  <div style={{ gridColumn: '1 / -1', fontSize: '11px', color: 'var(--color-text-secondary)', marginTop: '4px' }}>
                    <strong>Path:</strong> {skill.path}
                  </div>
                  <div style={{ display: 'flex', gap: '8px', gridColumn: '1 / -1', marginTop: '4px' }}>
                    <button
                      onClick={e => { e.stopPropagation(); toggleSkill(skill.id); }}
                      style={{
                        padding: '4px 12px',
                        borderRadius: '4px',
                        border: 'none',
                        background: skill.enabled ? 'var(--color-error)' : 'var(--color-success)',
                        color: 'white',
                        fontSize: '12px',
                        cursor: 'pointer',
                      }}
                    >
                      {skill.enabled ? 'Disable' : 'Enable'}
                    </button>
                    <button
                      onClick={e => { e.stopPropagation(); validateSkill(skill.id); }}
                      style={{
                        padding: '4px 12px',
                        borderRadius: '4px',
                        border: '1px solid var(--color-border)',
                        background: 'transparent',
                        color: 'var(--color-text-primary)',
                        fontSize: '12px',
                        cursor: 'pointer',
                      }}
                    >
                      Validate
                    </button>
                  </div>
                </div>
              )}
            </div>
          ))}

          {filteredSkills.length === 0 && (
            <div style={{ textAlign: 'center', padding: '24px', color: 'var(--color-text-secondary)', fontSize: '13px' }}>
              No skills match your filters
            </div>
          )}
        </div>
      )}

      {/* Validation Report */}
      {validationIssues.length > 0 && selectedSkill && (
        <div style={{
          marginTop: '16px',
          background: 'var(--color-bg-secondary)',
          border: '1px solid var(--color-border)',
          borderRadius: '12px',
          padding: '16px',
        }}>
          <h4 style={{ fontSize: '14px', marginBottom: '8px' }}>
            Validation: {selectedSkill.name}
          </h4>
          <ul style={{ margin: '4px 0', padding: '0 0 0 20px', fontSize: '12px' }}>
            {validationIssues.map((issue, i) => (
              <li key={i} style={{ color: issue.startsWith('Error') ? 'var(--color-error)' : '#fbbf24' }}>
                {issue}
              </li>
            ))}
          </ul>
          <button
            onClick={() => setValidationIssues([])}
            style={{
              marginTop: '8px',
              padding: '4px 12px',
              borderRadius: '4px',
              border: '1px solid var(--color-border)',
              background: 'transparent',
              color: 'var(--color-text-primary)',
              fontSize: '12px',
              cursor: 'pointer',
            }}
          >
            Close
          </button>
        </div>
      )}
    </div>
  );
}

export default Skills;