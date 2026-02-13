// ================================================
// Folio Web UI - Application
// ================================================

const API_BASE = window.location.origin + '/api';

// --- State ---
const state = {
  activities: [],
  stats: null,
  currentView: 'dashboard',
};

// --- API Client ---
const api = {
  async get(path) {
    const res = await fetch(`${API_BASE}${path}`);
    if (!res.ok) throw new Error(`API error: ${res.status}`);
    return res.json();
  },

  async post(path, body) {
    const res = await fetch(`${API_BASE}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
    if (!res.ok) throw new Error(`API error: ${res.status}`);
    return res.json();
  },

  async put(path, body) {
    const res = await fetch(`${API_BASE}${path}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
    if (!res.ok) throw new Error(`API error: ${res.status}`);
    return res.json();
  },

  async del(path) {
    const res = await fetch(`${API_BASE}${path}`, { method: 'DELETE' });
    if (!res.ok) throw new Error(`API error: ${res.status}`);
    return res;
  },

  async health() {
    return this.get('/health');
  },

  async listActivities(params = {}) {
    const qs = new URLSearchParams();
    if (params.limit) qs.set('limit', params.limit);
    if (params.importance) qs.set('importance', params.importance);
    if (params.project) qs.set('project', params.project);
    const query = qs.toString();
    return this.get(`/activities${query ? '?' + query : ''}`);
  },

  async getActivity(id) {
    return this.get(`/activities/${id}`);
  },

  async createActivity(data) {
    return this.post('/activities', data);
  },

  async updateActivity(id, data) {
    return this.put(`/activities/${id}`, data);
  },

  async deleteActivity(id) {
    return this.del(`/activities/${id}`);
  },

  async searchActivities(q, limit = 50) {
    return this.get(`/activities/search?q=${encodeURIComponent(q)}&limit=${limit}`);
  },

  async getStats() {
    return this.get('/stats');
  },
};

// --- Helpers ---
function $(sel, parent = document) {
  return parent.querySelector(sel);
}

function $$(sel, parent = document) {
  return [...parent.querySelectorAll(sel)];
}

function timeAgo(dateStr) {
  const date = new Date(dateStr);
  const now = new Date();
  const diff = Math.floor((now - date) / 1000);

  if (diff < 60) return 'just now';
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

function formatDate(dateStr) {
  return new Date(dateStr).toLocaleDateString('en-US', {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });
}

function sourceIcon(source) {
  const map = {
    git: 'G',
    github: 'GH',
    linear: 'LN',
    jira: 'JR',
    manual: 'M',
    screen_capture: 'SC',
    active_window: 'AW',
    calendar: 'CA',
    transcript: 'TR',
    voice_note: 'VN',
    meeting: 'MT',
    browser: 'BR',
  };
  return map[source] || source.charAt(0).toUpperCase();
}

function sourceLabel(source) {
  return source.replace(/_/g, ' ');
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

// --- Render Functions ---
function renderActivityItem(activity, compact = false) {
  const source = activity.source || 'manual';
  const importance = activity.importance || 'medium';
  const project = activity.project ? `<span>${escapeHtml(activity.project)}</span>` : '';

  return `
    <div class="activity-item" data-id="${escapeHtml(activity.id)}">
      <div class="activity-source-icon ${source}">${sourceIcon(source)}</div>
      <div class="activity-info">
        <div class="activity-title">${escapeHtml(activity.title)}</div>
        <div class="activity-meta">
          <span>${sourceLabel(source)}</span>
          <span>${timeAgo(activity.timestamp)}</span>
          ${project}
        </div>
      </div>
      <span class="importance-badge ${importance}">${importance}</span>
    </div>
  `;
}

function renderSourcesChart(bySource) {
  if (!bySource || Object.keys(bySource).length === 0) {
    return '<p style="color: var(--text-muted); font-size: 13px;">No data yet</p>';
  }

  const max = Math.max(...Object.values(bySource));
  const sorted = Object.entries(bySource).sort((a, b) => b[1] - a[1]);

  return sorted.map(([source, count]) => {
    const pct = max > 0 ? (count / max) * 100 : 0;
    return `
      <div class="chart-bar">
        <span class="chart-bar-label">${sourceLabel(source)}</span>
        <div class="chart-bar-track">
          <div class="chart-bar-fill" style="width: ${pct}%"></div>
        </div>
        <span class="chart-bar-value">${count}</span>
      </div>
    `;
  }).join('');
}

function renderActivityDetail(activity) {
  const fields = [];

  if (activity.description) {
    fields.push(`
      <div class="detail-field">
        <div class="detail-field-label">Description</div>
        <div class="detail-field-value">${escapeHtml(activity.description)}</div>
      </div>
    `);
  }

  const impact = activity.metadata && activity.metadata.impact;
  if (impact) {
    fields.push(`
      <div class="detail-field">
        <div class="detail-field-label">Impact</div>
        <div class="detail-field-value">${escapeHtml(impact)}</div>
      </div>
    `);
  }

  if (activity.employer) {
    fields.push(`
      <div class="detail-field">
        <div class="detail-field-label">Employer</div>
        <div class="detail-field-value">${escapeHtml(activity.employer)}</div>
      </div>
    `);
  }

  return `
    <div class="detail-header">
      <div class="detail-meta" style="margin-bottom: 10px;">
        <span class="importance-badge ${activity.importance}">${activity.importance}</span>
        <span class="detail-tag">${sourceLabel(activity.source)}</span>
        ${activity.project ? `<span class="detail-tag">${escapeHtml(activity.project)}</span>` : ''}
      </div>
      <h3>${escapeHtml(activity.title)}</h3>
      <div style="font-size: 12px; color: var(--text-muted); margin-top: 4px;">
        ${formatDate(activity.timestamp)}
      </div>
    </div>
    ${fields.join('')}
    <div class="detail-field">
      <div class="detail-field-label">Activity ID</div>
      <div class="detail-field-value" style="font-family: monospace; font-size: 12px; color: var(--text-muted);">${escapeHtml(activity.id)}</div>
    </div>
    <div class="detail-actions">
      <button class="btn btn-primary" onclick="showEditForm('${escapeHtml(activity.id)}')">Edit</button>
      <button class="btn btn-secondary" onclick="closeModal()">Close</button>
      <button class="btn btn-secondary" style="color: var(--high);" onclick="deleteActivity('${escapeHtml(activity.id)}')">Delete</button>
    </div>
  `;
}

function renderEditForm(activity) {
  const impact = (activity.metadata && activity.metadata.impact) || '';

  return `
    <div class="detail-header">
      <h3>Edit Activity</h3>
    </div>
    <form id="edit-form" class="edit-form" onsubmit="handleEditSubmit(event, '${escapeHtml(activity.id)}')">
      <div class="form-group">
        <label for="edit-title">Title</label>
        <input type="text" id="edit-title" value="${escapeHtml(activity.title)}" required>
      </div>
      <div class="form-group">
        <label for="edit-impact">Impact <span class="optional">(optional)</span></label>
        <input type="text" id="edit-impact" value="${escapeHtml(impact)}">
      </div>
      <div class="form-row">
        <div class="form-group">
          <label for="edit-project">Project <span class="optional">(optional)</span></label>
          <input type="text" id="edit-project" value="${escapeHtml(activity.project || '')}">
        </div>
        <div class="form-group">
          <label for="edit-importance">Importance</label>
          <select id="edit-importance" class="select-input">
            <option value="high" ${activity.importance === 'high' ? 'selected' : ''}>High</option>
            <option value="medium" ${activity.importance === 'medium' ? 'selected' : ''}>Medium</option>
            <option value="low" ${activity.importance === 'low' ? 'selected' : ''}>Low</option>
          </select>
        </div>
      </div>
      <div class="form-group">
        <label for="edit-employer">Employer <span class="optional">(optional)</span></label>
        <input type="text" id="edit-employer" value="${escapeHtml(activity.employer || '')}">
      </div>
      <div class="form-actions">
        <button type="submit" class="btn btn-primary">Save Changes</button>
        <button type="button" class="btn btn-secondary" onclick="showActivityDetail('${escapeHtml(activity.id)}')">Cancel</button>
      </div>
      <div id="edit-error" class="error-message hidden">
        Failed to save changes. Please try again.
      </div>
    </form>
  `;
}

// --- Views ---
async function loadDashboard() {
  try {
    const [stats, activities] = await Promise.all([
      api.getStats(),
      api.listActivities({ limit: 8 }),
    ]);

    state.stats = stats;

    $('#stat-total').textContent = stats.total_activities;
    $('#stat-high').textContent = stats.by_importance.high;
    $('#stat-medium').textContent = stats.by_importance.medium;
    $('#stat-low').textContent = stats.by_importance.low;

    $('#sources-chart').innerHTML = renderSourcesChart(stats.by_source);

    $('#recent-activities').innerHTML = activities.length > 0
      ? activities.map(a => renderActivityItem(a, true)).join('')
      : '<p style="color: var(--text-muted); font-size: 13px; padding: 12px 0;">No activities yet. Capture your first one!</p>';

  } catch (err) {
    console.error('Failed to load dashboard:', err);
  }
}

async function loadActivities() {
  try {
    const importance = $('#filter-importance').value;
    const project = $('#filter-project').value;

    const activities = await api.listActivities({
      limit: 100,
      importance: importance || undefined,
      project: project || undefined,
    });

    state.activities = activities;

    if (activities.length === 0) {
      $('#activities-list').innerHTML = '';
      $('#activities-empty').classList.remove('hidden');
    } else {
      $('#activities-empty').classList.add('hidden');
      $('#activities-list').innerHTML = activities.map(a => renderActivityItem(a)).join('');
    }

    // Populate project filter
    const projects = [...new Set(activities.map(a => a.project).filter(Boolean))];
    const filterProject = $('#filter-project');
    const currentVal = filterProject.value;
    filterProject.innerHTML = '<option value="">All Projects</option>' +
      projects.map(p => `<option value="${escapeHtml(p)}" ${p === currentVal ? 'selected' : ''}>${escapeHtml(p)}</option>`).join('');

  } catch (err) {
    console.error('Failed to load activities:', err);
  }
}

let searchTimeout;
async function handleSearch() {
  const query = $('#search-input').value.trim();

  if (!query) {
    $('#search-results').innerHTML = '';
    $('#search-empty').classList.remove('hidden');
    return;
  }

  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(async () => {
    try {
      const results = await api.searchActivities(query);

      if (results.length === 0) {
        $('#search-results').innerHTML = '';
        $('#search-empty').innerHTML = `
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.4"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
          <p>No results for "${escapeHtml(query)}"</p>
          <p class="muted">Try a different search term</p>
        `;
        $('#search-empty').classList.remove('hidden');
      } else {
        $('#search-empty').classList.add('hidden');
        $('#search-results').innerHTML = results.map(a => renderActivityItem(a)).join('');
      }
    } catch (err) {
      console.error('Search failed:', err);
    }
  }, 300);
}

// --- Modal ---
async function showActivityDetail(id) {
  try {
    const activity = await api.getActivity(id);
    $('#modal-body').innerHTML = renderActivityDetail(activity);
    $('#activity-modal').classList.remove('hidden');
  } catch (err) {
    console.error('Failed to load activity:', err);
  }
}

function closeModal() {
  $('#activity-modal').classList.add('hidden');
}

async function showEditForm(id) {
  try {
    const activity = await api.getActivity(id);
    $('#modal-body').innerHTML = renderEditForm(activity);
  } catch (err) {
    console.error('Failed to load activity for editing:', err);
  }
}

async function handleEditSubmit(e, id) {
  e.preventDefault();

  const data = {
    title: $('#edit-title').value.trim() || undefined,
    impact: $('#edit-impact').value.trim() || undefined,
    project: $('#edit-project').value.trim() || undefined,
    employer: $('#edit-employer').value.trim() || undefined,
    importance: $('#edit-importance').value,
  };

  try {
    await api.updateActivity(id, data);
    // Show the updated detail view
    await showActivityDetail(id);
    // Refresh the active list view in the background
    if (state.currentView === 'dashboard') loadDashboard();
    if (state.currentView === 'activities') loadActivities();
  } catch (err) {
    console.error('Failed to update activity:', err);
    const errorEl = $('#edit-error');
    if (errorEl) errorEl.classList.remove('hidden');
  }
}

async function deleteActivity(id) {
  if (!confirm('Delete this activity? This cannot be undone.')) return;

  try {
    await api.deleteActivity(id);
    closeModal();
    // Refresh current view
    if (state.currentView === 'dashboard') loadDashboard();
    if (state.currentView === 'activities') loadActivities();
  } catch (err) {
    console.error('Failed to delete activity:', err);
  }
}

// --- Capture Form ---
async function handleCapture(e) {
  e.preventDefault();

  const title = $('#capture-title').value.trim();
  if (!title) return;

  const data = {
    title,
    description: $('#capture-description').value.trim() || undefined,
    impact: $('#capture-impact').value.trim() || undefined,
    importance: $('#capture-importance').value,
    project: $('#capture-project').value.trim() || undefined,
    employer: $('#capture-employer').value.trim() || undefined,
  };

  try {
    await api.createActivity(data);
    $('#capture-form').reset();
    const msg = $('#capture-success');
    msg.classList.remove('hidden');
    setTimeout(() => msg.classList.add('hidden'), 3000);
  } catch (err) {
    console.error('Failed to capture activity:', err);
    alert('Failed to capture activity. Is the API server running?');
  }
}

// --- Navigation ---
function switchView(viewName) {
  state.currentView = viewName;

  // Update nav
  $$('.nav-link').forEach(link => {
    link.classList.toggle('active', link.dataset.view === viewName);
  });

  // Update views
  $$('.view').forEach(view => {
    view.classList.toggle('active', view.id === `view-${viewName}`);
  });

  // Load data for view
  switch (viewName) {
    case 'dashboard':
      loadDashboard();
      break;
    case 'activities':
      loadActivities();
      break;
    case 'search':
      setTimeout(() => $('#search-input').focus(), 100);
      break;
  }
}

// --- Health Check ---
async function checkHealth() {
  try {
    await api.health();
    $('#health-status').className = 'status-dot connected';
    $('#health-text').textContent = 'API Connected';
    return true;
  } catch {
    $('#health-status').className = 'status-dot disconnected';
    $('#health-text').textContent = 'Disconnected';
    return false;
  }
}

// --- Initialize ---
function init() {
  // Navigation
  $$('.nav-link').forEach(link => {
    link.addEventListener('click', (e) => {
      e.preventDefault();
      switchView(link.dataset.view);
    });
  });

  // Activity click handlers (delegated)
  document.addEventListener('click', (e) => {
    const item = e.target.closest('.activity-item');
    if (item && item.dataset.id) {
      showActivityDetail(item.dataset.id);
    }
  });

  // Modal close
  $('.modal-backdrop').addEventListener('click', closeModal);
  $('.modal-close').addEventListener('click', closeModal);
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') closeModal();
  });

  // Search
  $('#search-input').addEventListener('input', handleSearch);

  // Capture form
  $('#capture-form').addEventListener('submit', handleCapture);

  // Filters
  $('#filter-importance').addEventListener('change', loadActivities);
  $('#filter-project').addEventListener('change', loadActivities);
  $('#btn-refresh').addEventListener('click', loadActivities);

  // Initial load
  checkHealth().then(ok => {
    if (ok) loadDashboard();
  });

  // Periodic health check
  setInterval(checkHealth, 30000);
}

document.addEventListener('DOMContentLoaded', init);
