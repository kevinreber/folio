// ================================================
// Folio Web UI - Application
// ================================================

const API_BASE = window.location.origin + '/api';

// --- State ---
const state = {
  activities: [],
  stats: null,
  currentView: 'dashboard',
  heatmapData: null,
  selectedDate: null,
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

  async getHeatmap(months = 6) {
    return this.get(`/heatmap?months=${months}`);
  },

  async getTimeline(days = 7, date = null) {
    const params = date ? `date=${date}&days=1` : `days=${days}`;
    return this.get(`/timeline?${params}`);
  },

  async listAccomplishments() {
    return this.get('/accomplishments');
  },

  async createAccomplishment(data) {
    return this.post('/accomplishments', data);
  },

  async getConfig() {
    return this.get('/config');
  },

  async updateConfig(data) {
    return this.post('/config', data);
  },

  async discoverConfig() {
    return this.get('/config/discover');
  },

  async getInsights(date = null, days = 7) {
    const params = date ? `date=${date}` : `days=${days}`;
    return this.get(`/insights?${params}`);
  },

  async getClaudeProjects(days = 30) {
    return this.get(`/claude/projects?days=${days}`);
  },

  async getClaudeHeatmap(days = 90) {
    return this.get(`/claude/heatmap?days=${days}`);
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
    claude_code: 'CC',
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
    <div class="detail-actions-section">
      <div class="promote-hint">
        <button class="btn btn-primary" onclick="showPromoteForm('${escapeHtml(activity.id)}')">Promote to Accomplishment</button>
        <p class="promote-hint-text">Turn this activity into a polished STAR story for resumes, interviews, and performance reviews</p>
      </div>
      <div class="detail-actions">
        <button class="btn btn-secondary" onclick="showEditForm('${escapeHtml(activity.id)}')">Edit</button>
        <button class="btn btn-secondary" onclick="closeModal()">Close</button>
        <button class="btn btn-secondary" style="color: var(--high);" onclick="deleteActivity('${escapeHtml(activity.id)}')">Delete</button>
      </div>
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
    const sourceFilter = $('#filter-source').value;

    const activities = await api.listActivities({
      limit: 200,
      importance: importance || undefined,
      project: project || undefined,
    });

    // Client-side source filter (API doesn't have source param yet)
    const filtered = sourceFilter
      ? activities.filter(a => a.source === sourceFilter)
      : activities;

    state.activities = activities;

    if (filtered.length === 0) {
      $('#activities-list').innerHTML = '';
      $('#activities-empty').classList.remove('hidden');
    } else {
      $('#activities-empty').classList.add('hidden');
      $('#activities-list').innerHTML = filtered.map(a => renderActivityItem(a)).join('');
    }

    // Populate source filter
    const sources = [...new Set(activities.map(a => a.source).filter(Boolean))].sort();
    const filterSource = $('#filter-source');
    const currentSource = filterSource.value;
    filterSource.innerHTML = '<option value="">All Sources</option>' +
      sources.map(s => `<option value="${escapeHtml(s)}" ${s === currentSource ? 'selected' : ''}>${sourceLabel(s)}</option>`).join('');

    // Populate project filter
    const projects = [...new Set(activities.map(a => a.project).filter(Boolean))].sort();
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

// --- What's New (Build Logs) ---
const CHANGELOG = [
  {
    date: '2026-02-14',
    title: 'Weekly and Monthly Digests',
    category: 'New Feature',
    description: 'You can now generate weekly or monthly summaries of your career activities. Great for preparing for 1-on-1s or performance reviews.',
  },
  {
    date: '2026-02-10',
    title: 'Search Your Activities',
    category: 'New Feature',
    description: 'Quickly find any past activity using the new search page. Search by title, description, or project name to locate exactly what you need.',
  },
  {
    date: '2026-02-05',
    title: 'Edit and Delete Activities',
    category: 'Improvement',
    description: 'You can now edit activity details or remove entries you no longer need, directly from the activity detail view.',
  },
  {
    date: '2026-01-28',
    title: 'Activity Importance Levels',
    category: 'New Feature',
    description: 'Tag your activities as High, Medium, or Low importance so you can focus on the accomplishments that matter most.',
  },
  {
    date: '2026-01-20',
    title: 'Dashboard Overview',
    category: 'New Feature',
    description: 'A new dashboard gives you a snapshot of your total activities, broken down by importance and source, plus your most recent entries.',
  },
  {
    date: '2026-01-15',
    title: 'Filter Activities by Project',
    category: 'Improvement',
    description: 'The activities page now lets you filter by project and importance level, making it easier to review work for a specific team or initiative.',
  },
  {
    date: '2026-01-08',
    title: 'GitHub Integration',
    category: 'New Feature',
    description: 'Folio can now automatically pull in your contributions from GitHub, including pull requests and code reviews, so you never lose track of your work.',
  },
  {
    date: '2025-12-18',
    title: 'Web Interface Launched',
    category: 'New Feature',
    description: 'Access Folio from your browser with a brand-new web interface. View your dashboard, browse activities, and capture new ones without leaving your browser.',
  },
  {
    date: '2025-12-10',
    title: 'Export Your Data',
    category: 'New Feature',
    description: 'Export your activities as Markdown or JSON. Useful for sharing accomplishments in documents, emails, or performance review tools.',
  },
  {
    date: '2025-12-01',
    title: 'Folio Launch',
    category: 'Announcement',
    description: 'Folio is here! A local-first career tracker that helps you capture and organize your professional accomplishments as they happen.',
  },
];

function formatChangelogDate(dateStr) {
  return new Date(dateStr + 'T00:00:00').toLocaleDateString('en-US', {
    month: 'long',
    day: 'numeric',
    year: 'numeric',
  });
}

function renderChangelogEntry(entry) {
  const categoryClass = entry.category.toLowerCase().replace(/\s+/g, '-');
  return `
    <div class="whatsnew-entry">
      <div class="whatsnew-date">${formatChangelogDate(entry.date)}</div>
      <div class="whatsnew-body">
        <div class="whatsnew-heading">
          <span class="whatsnew-category ${categoryClass}">${escapeHtml(entry.category)}</span>
          <h3 class="whatsnew-title">${escapeHtml(entry.title)}</h3>
        </div>
        <p class="whatsnew-description">${escapeHtml(entry.description)}</p>
      </div>
    </div>
  `;
}

function loadWhatsNew() {
  const container = $('#whatsnew-list');
  if (!container) return;
  container.innerHTML = CHANGELOG.map(renderChangelogEntry).join('');
}

// --- Heatmap / Timeline ---
async function loadTimeline() {
  const months = parseInt($('#heatmap-months').value) || 6;
  try {
    const [data, timeline, insights] = await Promise.all([
      api.getHeatmap(months),
      api.getTimeline(7),
      api.getInsights(null, 7),
    ]);
    state.heatmapData = data;
    renderHeatmap(data);
    renderInsights(insights);
    renderTimelineDays(timeline);
  } catch (err) {
    console.error('Failed to load timeline:', err);
  }
}

function renderHeatmap(data) {
  const container = $('#heatmap-container');
  if (!container) return;

  const days = data.days || [];
  if (days.length === 0) {
    container.innerHTML = '<p style="color: var(--text-muted); font-size: 13px;">No activity data yet</p>';
    return;
  }

  // Build a map of date -> count
  const countMap = {};
  let maxCount = 0;
  for (const d of days) {
    countMap[d.date] = d.count;
    if (d.count > maxCount) maxCount = d.count;
  }

  // Generate calendar grid: weeks as columns, days as rows
  const months = parseInt($('#heatmap-months').value) || 6;
  const endDate = new Date();
  const startDate = new Date();
  startDate.setDate(startDate.getDate() - (months * 30));
  // Align to Sunday
  startDate.setDate(startDate.getDate() - startDate.getDay());

  const dayLabels = ['', 'Mon', '', 'Wed', '', 'Fri', ''];
  let html = '<div class="heatmap-grid">';

  // Day labels column
  html += '<div class="heatmap-labels">';
  for (const label of dayLabels) {
    html += `<div class="heatmap-day-label">${label}</div>`;
  }
  html += '</div>';

  // Week columns
  const current = new Date(startDate);
  let lastMonth = -1;
  let monthHtml = '<div class="heatmap-months">';
  monthHtml += '<div style="width:28px"></div>'; // spacer for labels

  while (current <= endDate) {
    html += '<div class="heatmap-week">';
    for (let day = 0; day < 7; day++) {
      const dateStr = current.toISOString().split('T')[0];
      const count = countMap[dateStr] || 0;
      const level = maxCount === 0 ? 0 : Math.min(4, Math.ceil((count / maxCount) * 4));
      const isFuture = current > endDate;

      html += `<div class="heatmap-cell${isFuture ? ' future' : ''}" data-level="${isFuture ? '' : level}" data-date="${dateStr}" data-count="${count}" onclick="selectHeatmapDay('${dateStr}')"></div>`;
      current.setDate(current.getDate() + 1);
    }
    html += '</div>';

    // Track month labels
    if (current.getMonth() !== lastMonth) {
      const monthName = new Date(current).toLocaleDateString('en-US', { month: 'short' });
      monthHtml += `<div class="heatmap-month-label">${monthName}</div>`;
      lastMonth = current.getMonth();
    }
  }

  monthHtml += '</div>';
  html += '</div>';

  container.innerHTML = monthHtml + html;
}

async function selectHeatmapDay(dateStr) {
  state.selectedDate = dateStr;

  // Highlight selected day
  $$('.heatmap-cell.selected').forEach(c => c.classList.remove('selected'));
  $$(`.heatmap-cell[data-date="${dateStr}"]`).forEach(c => c.classList.add('selected'));

  // Load that day's activities and insights in parallel
  try {
    const [timeline, insights] = await Promise.all([
      api.getTimeline(1, dateStr),
      api.getInsights(dateStr),
    ]);
    renderInsights(insights);
    renderTimelineDays(timeline);
  } catch (err) {
    console.error('Failed to load day:', err);
  }
}

function renderInsights(data) {
  const container = $('#timeline-insights');
  if (!container || data.total === 0) {
    if (container) container.classList.add('hidden');
    return;
  }
  container.classList.remove('hidden');

  // Stats
  $('#insight-stat-total').querySelector('.insights-stat-value').textContent = data.total;
  $('#insight-stat-projects').querySelector('.insights-stat-value').textContent = data.top_projects.length;
  $('#insight-stat-streak').querySelector('.insights-stat-value').textContent = data.streak || 0;
  $('#insight-stat-high').querySelector('.insights-stat-value').textContent = data.importance.high;

  // Importance doughnut
  drawDoughnut('doughnut-importance', [
    { label: 'High', value: data.importance.high, color: '#f43f5e' },
    { label: 'Medium', value: data.importance.medium, color: '#f59e0b' },
    { label: 'Low', value: data.importance.low, color: '#22c55e' },
  ], data.total);

  // Source doughnut
  const sourceColors = {
    git: '#f97316', github: '#e2e8f0', linear: '#6366f1', claude_code: '#d4a574',
    manual: '#22c55e', jira: '#3b82f6',
  };
  const sourceSlices = Object.entries(data.by_source)
    .sort((a, b) => b[1] - a[1])
    .map(([src, count]) => ({
      label: src.replace(/_/g, ' '),
      value: count,
      color: sourceColors[src] || '#94a3b8',
    }));
  drawDoughnut('doughnut-source', sourceSlices, data.total);

  // Top projects bar chart
  const projContainer = $('#insights-top-projects');
  if (data.top_projects.length === 0) {
    projContainer.innerHTML = '<p class="setup-empty">No project data</p>';
  } else {
    const maxCount = data.top_projects[0].count;
    projContainer.innerHTML = data.top_projects.map(p => {
      const pct = maxCount > 0 ? (p.count / maxCount) * 100 : 0;
      return `
        <div class="insights-proj-row">
          <span class="insights-proj-name" title="${escapeHtml(p.project)}">${escapeHtml(p.project)}</span>
          <div class="insights-proj-bar-wrap">
            <div class="insights-proj-bar" style="width:${pct}%"></div>
          </div>
          <span class="insights-proj-count">${p.count}</span>
        </div>`;
    }).join('');
  }

  // Highlights
  const hlContainer = $('#insights-highlights');
  if (data.highlights && data.highlights.length > 0) {
    hlContainer.innerHTML = data.highlights.map(h => {
      const source = h.source || 'manual';
      return `
        <div class="insights-highlight-item" data-id="${escapeHtml(h.id)}">
          <div class="activity-source-icon ${source}" style="width:24px;height:24px;font-size:10px;border-radius:5px;">${sourceIcon(source)}</div>
          <div class="insights-highlight-info">
            <div class="insights-highlight-title">${escapeHtml(h.title)}</div>
            <div class="insights-highlight-meta">${h.project ? escapeHtml(h.project) : ''} &middot; ${timeAgo(h.timestamp)}</div>
          </div>
        </div>`;
    }).join('');
  } else {
    hlContainer.innerHTML = '<p style="color: var(--text-muted); font-size: 13px;">No high-impact activities in this period</p>';
  }

  // Insight callouts
  const calloutsContainer = $('#insights-callouts');
  if (data.insights && data.insights.length > 0) {
    calloutsContainer.innerHTML = data.insights.map(i => {
      const icon = i.type === 'positive' ? '&#9650;' : i.type === 'suggestion' ? '&#9888;' : '&#9679;';
      return `<div class="insight-callout ${i.type}">${icon} ${escapeHtml(i.text)}</div>`;
    }).join('');
  } else {
    calloutsContainer.innerHTML = '';
  }
}

function drawDoughnut(canvasId, slices, total) {
  const canvas = document.getElementById(canvasId);
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  const w = canvas.width;
  const h = canvas.height;
  const cx = w / 2;
  const cy = h / 2;
  const outerR = Math.min(cx, cy) - 4;
  const innerR = outerR * 0.6;

  ctx.clearRect(0, 0, w, h);

  if (total === 0) return;

  let startAngle = -Math.PI / 2;
  for (const slice of slices) {
    if (slice.value === 0) continue;
    const sweep = (slice.value / total) * Math.PI * 2;

    ctx.beginPath();
    ctx.arc(cx, cy, outerR, startAngle, startAngle + sweep);
    ctx.arc(cx, cy, innerR, startAngle + sweep, startAngle, true);
    ctx.closePath();
    ctx.fillStyle = slice.color;
    ctx.fill();

    startAngle += sweep;
  }

  // Center label
  const label = document.getElementById(canvasId + '-label');
  if (label) {
    label.innerHTML = `<strong>${total}</strong><br><span>total</span>`;
  }

  // Determine filter type from canvas ID
  const filterType = canvasId.includes('importance') ? 'importance' : 'source';

  // Legend with clickable items
  const legend = document.getElementById(canvasId + '-legend');
  if (legend) {
    legend.innerHTML = slices
      .filter(s => s.value > 0)
      .map(s => {
        const filterVal = s.label.toLowerCase().replace(/\s/g, '_');
        const pct = Math.round((s.value / total) * 100);
        return `<div class="doughnut-legend-item clickable" onclick="filterTimelineActivities('${filterType}', '${filterVal}')" title="Click to filter">
          <span class="doughnut-legend-dot" style="background:${s.color}"></span>
          ${escapeHtml(s.label)} <span class="doughnut-legend-val">${s.value} (${pct}%)</span>
        </div>`;
      }).join('');
  }
}

// Filter the timeline activity list by importance or source
function filterTimelineActivities(type, value) {
  // Toggle: if already filtering by this value, clear the filter
  const activeFilter = state.timelineFilter;
  if (activeFilter && activeFilter.type === type && activeFilter.value === value) {
    state.timelineFilter = null;
  } else {
    state.timelineFilter = { type, value };
  }

  const filter = state.timelineFilter;

  // Update legend active states
  $$('.doughnut-legend-item').forEach(item => item.classList.remove('active-filter'));
  if (filter) {
    $$('.doughnut-legend-item').forEach(item => {
      if (item.onclick && item.onclick.toString().includes(`'${filter.type}', '${filter.value}'`)) {
        item.classList.add('active-filter');
      }
    });
  }

  // Show/hide filter indicator
  let indicator = $('#timeline-filter-indicator');
  if (!indicator) {
    const detail = $('#timeline-detail');
    const ind = document.createElement('div');
    ind.id = 'timeline-filter-indicator';
    ind.className = 'timeline-filter-indicator hidden';
    detail.insertBefore(ind, detail.querySelector('.timeline-day, #timeline-days'));
    indicator = ind;
  }

  if (filter) {
    const label = filter.value.replace(/_/g, ' ');
    indicator.innerHTML = `Showing: <strong>${label}</strong> <button class="timeline-filter-clear" onclick="filterTimelineActivities('${filter.type}', '${filter.value}')">&times; Clear</button>`;
    indicator.classList.remove('hidden');
  } else {
    indicator.classList.add('hidden');
  }

  // Apply filter to visible activity items
  $$('#timeline-days .activity-item').forEach(item => {
    if (!filter) {
      item.style.display = '';
      return;
    }
    const matches = filter.type === 'importance'
      ? item.querySelector('.importance-badge')?.textContent.trim().toLowerCase() === filter.value
      : item.dataset.source === filter.value;
    item.style.display = matches ? '' : 'none';
  });
}

function renderTimelineDays(data) {
  const container = $('#timeline-days');
  if (!container) return;

  const days = data.days || [];
  if (days.length === 0) {
    container.innerHTML = '<p style="color: var(--text-muted); font-size: 13px; padding: 12px 0;">No activities for this period</p>';
    return;
  }

  let html = '';
  for (const day of days) {
    const dateLabel = new Date(day.date + 'T12:00:00').toLocaleDateString('en-US', {
      weekday: 'long', month: 'long', day: 'numeric', year: 'numeric'
    });

    // Group by source
    const bySource = {};
    for (const a of day.activities) {
      const src = a.source || 'manual';
      if (!bySource[src]) bySource[src] = [];
      bySource[src].push(a);
    }

    // Source summary chips
    const sourceChips = Object.entries(bySource)
      .sort((a, b) => b[1].length - a[1].length)
      .map(([src, items]) =>
        `<button class="timeline-source-chip" data-source="${src}" onclick="filterTimelineDay(this, '${day.date}', '${src}')">
          <span class="activity-source-icon ${src}" style="width:18px;height:18px;font-size:9px;border-radius:4px;">${sourceIcon(src)}</span>
          ${sourceLabel(src)} <strong>${items.length}</strong>
        </button>`
      ).join('');

    html += `<div class="timeline-day" data-day="${day.date}">`;
    html += `<div class="timeline-day-header">`;
    html += `<span class="timeline-date">${dateLabel}</span>`;
    html += `<span class="timeline-count">${day.count} activities</span>`;
    html += `</div>`;
    html += `<div class="timeline-source-chips">
      <button class="timeline-source-chip active" onclick="filterTimelineDay(this, '${day.date}', 'all')">All <strong>${day.count}</strong></button>
      ${sourceChips}
    </div>`;
    html += `<div class="timeline-day-activities" data-day-list="${day.date}">`;

    for (const a of day.activities) {
      const source = a.source || 'manual';
      const importance = a.importance || 'medium';
      html += `
        <div class="activity-item" data-id="${escapeHtml(a.id)}" data-source="${source}">
          <div class="activity-source-icon ${source}">${sourceIcon(source)}</div>
          <div class="activity-info">
            <div class="activity-title">${escapeHtml(a.title)}</div>
            <div class="activity-meta">
              <span>${sourceLabel(source)}</span>
              <span>${new Date(a.timestamp).toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' })}</span>
              ${a.project ? `<span>${escapeHtml(a.project)}</span>` : ''}
            </div>
          </div>
          <span class="importance-badge ${importance}">${importance}</span>
        </div>`;
    }

    html += '</div></div>';
  }

  container.innerHTML = html;
}

function filterTimelineDay(btn, date, source) {
  // Update active chip
  const chips = btn.parentElement;
  chips.querySelectorAll('.timeline-source-chip').forEach(c => c.classList.remove('active'));
  btn.classList.add('active');

  // Filter activities
  const list = document.querySelector(`[data-day-list="${date}"]`);
  if (!list) return;
  list.querySelectorAll('.activity-item').forEach(item => {
    if (source === 'all' || item.dataset.source === source) {
      item.style.display = '';
    } else {
      item.style.display = 'none';
    }
  });
}

// --- Accomplishments ---
async function loadAccomplishments() {
  try {
    const accomplishments = await api.listAccomplishments();

    if (accomplishments.length === 0) {
      $('#accomplish-list').innerHTML = '';
      $('#accomplish-empty').classList.remove('hidden');
    } else {
      $('#accomplish-empty').classList.add('hidden');
      $('#accomplish-list').innerHTML = accomplishments.map(renderAccomplishmentItem).join('');
    }
  } catch (err) {
    console.error('Failed to load accomplishments:', err);
  }
}

function renderAccomplishmentItem(acc) {
  const skills = (acc.skills || []).map(s => `<span class="detail-tag">${escapeHtml(s)}</span>`).join('');
  const themes = (acc.themes || []).map(t => `<span class="detail-tag">${escapeHtml(t)}</span>`).join('');

  return `
    <div class="accomplish-card card" style="margin-bottom: 12px; cursor: default;">
      <div class="accomplish-header">
        <h4 style="font-size: 16px; font-weight: 600; margin-bottom: 8px;">${escapeHtml(acc.title)}</h4>
        ${acc.employer ? `<span class="detail-tag">${escapeHtml(acc.employer)}</span>` : ''}
        ${acc.role ? `<span class="detail-tag">${escapeHtml(acc.role)}</span>` : ''}
      </div>
      ${acc.situation ? `<div class="star-field"><span class="star-label">S</span> ${escapeHtml(acc.situation)}</div>` : ''}
      ${acc.task ? `<div class="star-field"><span class="star-label">T</span> ${escapeHtml(acc.task)}</div>` : ''}
      ${acc.action ? `<div class="star-field"><span class="star-label">A</span> ${escapeHtml(acc.action)}</div>` : ''}
      ${acc.result ? `<div class="star-field"><span class="star-label">R</span> ${escapeHtml(acc.result)}</div>` : ''}
      ${skills || themes ? `<div class="accomplish-tags" style="margin-top: 10px;">${skills}${themes}</div>` : ''}
      ${acc.generated_bullets && acc.generated_bullets.length > 0 ? `
        <div style="margin-top: 12px; padding-top: 12px; border-top: 1px solid var(--border);">
          <div class="detail-field-label">Resume Bullets</div>
          <ul style="padding-left: 18px; color: var(--text-secondary); font-size: 13px;">
            ${acc.generated_bullets.map(b => `<li>${escapeHtml(b)}</li>`).join('')}
          </ul>
        </div>
      ` : ''}
    </div>
  `;
}

// --- Promote Flow ---
async function showPromoteForm(activityId) {
  try {
    const activity = await api.getActivity(activityId);
    closeModal();

    const html = `
      <div class="detail-header">
        <h3>Promote to Accomplishment</h3>
        <p style="color: var(--text-secondary); margin-top: 4px; font-size: 13px;">
          Turn "${escapeHtml(activity.title)}" into a polished STAR accomplishment
        </p>
      </div>
      <form id="promote-form" onsubmit="handlePromoteSubmit(event, '${escapeHtml(activity.id)}')">
        <div class="form-group">
          <label for="promote-title">Title</label>
          <input type="text" id="promote-title" value="${escapeHtml(activity.title)}" required>
        </div>
        <div class="form-group">
          <label for="promote-situation">Situation <span class="optional">(context & background)</span></label>
          <textarea id="promote-situation" rows="2" placeholder="What was happening? What was the context?"></textarea>
        </div>
        <div class="form-group">
          <label for="promote-task">Task <span class="optional">(what you were asked to do)</span></label>
          <textarea id="promote-task" rows="2" placeholder="What was the goal or challenge?"></textarea>
        </div>
        <div class="form-group">
          <label for="promote-action">Action <span class="optional">(what you did)</span></label>
          <textarea id="promote-action" rows="2" placeholder="What specific steps did you take?">${escapeHtml(activity.description || '')}</textarea>
        </div>
        <div class="form-group">
          <label for="promote-result">Result <span class="optional">(quantified outcome)</span></label>
          <textarea id="promote-result" rows="2" placeholder="What was the measurable impact?">${escapeHtml((activity.metadata && activity.metadata.impact) || '')}</textarea>
        </div>
        <div class="form-row">
          <div class="form-group">
            <label for="promote-skills">Skills <span class="optional">(comma-separated)</span></label>
            <input type="text" id="promote-skills" placeholder="e.g. Rust, API design, mentoring">
          </div>
          <div class="form-group">
            <label for="promote-themes">Themes <span class="optional">(comma-separated)</span></label>
            <input type="text" id="promote-themes" placeholder="e.g. reliability, performance">
          </div>
        </div>
        <div class="form-actions">
          <button type="submit" class="btn btn-primary">Create Accomplishment</button>
          <button type="button" class="btn btn-secondary" onclick="closePromoteModal()">Cancel</button>
        </div>
        <div id="promote-success" class="success-message hidden">
          Accomplishment created!
        </div>
        <div id="promote-error" class="error-message hidden">
          Failed to create accomplishment.
        </div>
      </form>
    `;

    $('#promote-body').innerHTML = html;
    $('#promote-modal').classList.remove('hidden');
  } catch (err) {
    console.error('Failed to load activity for promotion:', err);
  }
}

function closePromoteModal() {
  $('#promote-modal').classList.add('hidden');
}

async function handlePromoteSubmit(e, activityId) {
  e.preventDefault();

  const splitTrim = (val) => val ? val.split(',').map(s => s.trim()).filter(Boolean) : [];

  const data = {
    title: $('#promote-title').value.trim(),
    situation: $('#promote-situation').value.trim() || undefined,
    task: $('#promote-task').value.trim() || undefined,
    action: $('#promote-action').value.trim() || undefined,
    result: $('#promote-result').value.trim() || undefined,
    skills: splitTrim($('#promote-skills').value),
    themes: splitTrim($('#promote-themes').value),
    activity_ids: [activityId],
  };

  try {
    await api.createAccomplishment(data);
    const msg = $('#promote-success');
    msg.classList.remove('hidden');
    setTimeout(() => {
      closePromoteModal();
      if (state.currentView === 'accomplish') loadAccomplishments();
    }, 1500);
  } catch (err) {
    console.error('Failed to promote:', err);
    $('#promote-error').classList.remove('hidden');
  }
}

// --- Claude View ---
async function loadClaude() {
  const days = parseInt($('#claude-days').value) || 30;
  try {
    const [projects, heatmap] = await Promise.all([
      api.getClaudeProjects(days),
      api.getClaudeHeatmap(days),
    ]);
    // Store raw data for filtering
    state.claudeProjects = projects;
    state.claudeHeatmap = heatmap;
    applyClaudeTagFilter();
  } catch (err) {
    console.error('Failed to load Claude data:', err);
  }
}

function formatHours(minutes) {
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

function applyClaudeTagFilter() {
  const tag = state.claudeTagFilter || 'all';
  const projects = state.claudeProjects;
  const heatmap = state.claudeHeatmap;
  if (!projects || !heatmap) return;

  // Compute estimated hours per tag for toggle labels (5 min per prompt estimate)
  const workPrompts = projects.projects.filter(p => !p.is_personal).reduce((s, p) => s + p.total_prompts, 0);
  const personalPrompts = projects.projects.filter(p => p.is_personal).reduce((s, p) => s + p.total_prompts, 0);
  const allPrompts = workPrompts + personalPrompts;

  $$('#claude-tag-toggle .toggle-btn').forEach(btn => {
    const val = btn.dataset.value;
    const prompts = val === 'work' ? workPrompts : val === 'personal' ? personalPrompts : allPrompts;
    const estH = Math.floor((prompts * 5) / 60);
    const label = val.charAt(0).toUpperCase() + val.slice(1);
    btn.innerHTML = `${label} <span class="toggle-hours">~${estH}h</span>`;
  });

  // Filter projects
  const filtered = {
    ...projects,
    projects: tag === 'all'
      ? projects.projects
      : projects.projects.filter(p => (tag === 'personal') === p.is_personal),
    total_projects: 0,
    total_sessions: 0,
  };
  filtered.total_projects = filtered.projects.length;
  filtered.total_sessions = filtered.projects.reduce((s, p) => s + p.sessions, 0);

  // Filter heatmap
  const filteredHeatmap = {
    ...heatmap,
    projects: tag === 'all'
      ? heatmap.projects
      : heatmap.projects.filter(p => {
          const projData = projects.projects.find(pp => pp.project === p.project);
          return projData ? (tag === 'personal') === projData.is_personal : true;
        }),
  };

  renderClaudeStats(filtered);
  renderClaudeHeatmap(filteredHeatmap);
  renderClaudeProjects(filtered);
}

function renderClaudeStats(data) {
  const container = $('#claude-stats');
  const totalPrompts = data.projects.reduce((sum, p) => sum + p.total_prompts, 0);
  const totalMinutes = data.projects.reduce((sum, p) => sum + p.total_minutes, 0);
  const hours = Math.floor(totalMinutes / 60);
  const mins = totalMinutes % 60;
  // Active time = gap between prompts. Estimated total adds ~5 min per prompt
  // for reading responses, testing code, reviewing diffs
  const estTotalMins = Math.round(totalPrompts * 5);
  const estHours = Math.floor(estTotalMins / 60);
  const days = parseInt($('#claude-days').value) || 30;
  const perDay = (estTotalMins / days / 60).toFixed(1);

  container.innerHTML = `
    <div class="stat-card">
      <div class="stat-value">${data.total_projects}</div>
      <div class="stat-label">Projects</div>
    </div>
    <div class="stat-card">
      <div class="stat-value">${data.total_sessions}</div>
      <div class="stat-label">Sessions</div>
    </div>
    <div class="stat-card">
      <div class="stat-value">${totalPrompts.toLocaleString()}</div>
      <div class="stat-label">Total Prompts</div>
    </div>
    <div class="stat-card">
      <div class="stat-value">~${estHours}h</div>
      <div class="stat-label">Est. Time with Claude</div>
      <div class="stat-sub">~${perDay}h/day &middot; ${formatHours(totalMinutes)} active</div>
    </div>
  `;
}

function renderClaudeHeatmap(data) {
  const container = $('#claude-heatmap');
  const projects = data.projects || [];

  if (projects.length === 0) {
    container.innerHTML = '<p style="color: var(--text-muted); font-size: 13px;">No Claude Code data yet</p>';
    return;
  }

  // Build date range
  const days = data.days || 30;
  const dates = [];
  const now = new Date();
  for (let i = days - 1; i >= 0; i--) {
    const d = new Date(now);
    d.setDate(d.getDate() - i);
    dates.push(d.toISOString().split('T')[0]);
  }

  // Find global max for color scaling
  let globalMax = 0;
  for (const p of projects) {
    for (const d of p.days || []) {
      if (d.prompts > globalMax) globalMax = d.prompts;
    }
  }

  // Take top N projects to keep it readable
  const topProjects = projects.slice(0, 15);

  // Render grid
  let html = '<div class="claude-heatmap-grid">';

  // Date header (show every 7th date)
  html += '<div class="claude-hm-row claude-hm-header"><div class="claude-hm-label"></div>';
  for (let i = 0; i < dates.length; i++) {
    if (i % 7 === 0) {
      const d = new Date(dates[i] + 'T12:00:00');
      html += `<div class="claude-hm-date-label">${d.toLocaleDateString('en-US', {month: 'short', day: 'numeric'})}</div>`;
    } else {
      html += '<div class="claude-hm-date-label"></div>';
    }
  }
  html += '</div>';

  // Project rows
  for (const p of topProjects) {
    const dayMap = {};
    for (const d of p.days || []) {
      dayMap[d.date] = d.prompts;
    }

    const tag = p.project;
    html += `<div class="claude-hm-row">`;
    html += `<div class="claude-hm-label" title="${escapeHtml(tag)}">${escapeHtml(tag)}</div>`;

    for (const date of dates) {
      const count = dayMap[date] || 0;
      const level = globalMax === 0 ? 0 : Math.min(4, Math.ceil((count / globalMax) * 4));
      html += `<div class="claude-hm-cell" data-level="${count > 0 ? level : 0}" data-date="${date}" data-project="${escapeHtml(tag)}" data-count="${count}"></div>`;
    }
    html += '</div>';
  }

  html += '</div>';
  if (projects.length > 15) {
    html += `<p style="font-size: 12px; color: var(--text-muted); margin-top: 8px;">Showing top 15 of ${projects.length} projects</p>`;
  }

  container.innerHTML = html;
}

function renderClaudeProjects(data) {
  const container = $('#claude-projects');
  const projects = data.projects || [];

  if (projects.length === 0) {
    container.innerHTML = '<p style="color: var(--text-muted); font-size: 13px;">No Claude Code data yet. Run "folio sync --source claude" to import.</p>';
    return;
  }

  const maxPrompts = Math.max(...projects.map(p => p.total_prompts));

  container.innerHTML = projects.map(p => {
    const pct = maxPrompts > 0 ? (p.total_prompts / maxPrompts) * 100 : 0;
    const tag = p.is_personal ? '<span class="claude-tag personal">personal</span>' : '<span class="claude-tag work">work</span>';
    const hours = Math.floor(p.total_minutes / 60);
    const mins = p.total_minutes % 60;
    const timeStr = hours > 0 ? `${hours}h ${mins}m` : `${mins}m`;

    return `
      <div class="claude-project-item">
        <div class="claude-project-info">
          <div class="claude-project-name">
            ${escapeHtml(p.project)} ${tag}
          </div>
          <div class="claude-project-meta">
            ${p.sessions} sessions &middot; ${p.total_prompts} prompts &middot; ~${timeStr} &middot; ${p.days_active} days active
          </div>
        </div>
        <div class="claude-project-bar-wrap">
          <div class="claude-project-bar" style="width: ${pct}%"></div>
        </div>
        <div class="claude-project-count">${p.total_prompts}</div>
      </div>
    `;
  }).join('');
}

async function showClaudeHeatmapDetail(project, date, promptCount) {
  const detail = $('#claude-heatmap-detail');
  detail.classList.remove('hidden');

  const d = new Date(date + 'T12:00:00');
  const dateLabel = d.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric', year: 'numeric' });

  // Search for activities matching this project and date
  try {
    const activities = await api.listActivities({ limit: 200 });
    const matching = activities.filter(a =>
      a.source === 'claude_code' &&
      a.timestamp.startsWith(date) &&
      (a.project === project || (a.metadata && a.metadata.project_path && a.metadata.project_path.includes('/' + project)))
    );

    let html = `<div class="timeline-day-header">
      <span class="timeline-date">${escapeHtml(project)} — ${dateLabel}</span>
      <span class="timeline-count">${promptCount} prompts across ${matching.length} sessions</span>
    </div>`;

    if (matching.length > 0) {
      html += '<div class="timeline-day-activities">';
      for (const a of matching) {
        const prompts = (a.metadata && a.metadata.prompt_count) || '?';
        const mins = (a.metadata && a.metadata.duration_minutes) || 0;
        html += `
          <div class="activity-item" data-id="${escapeHtml(a.id)}">
            <div class="activity-source-icon claude_code">CC</div>
            <div class="activity-info">
              <div class="activity-title">${escapeHtml(a.title)}</div>
              <div class="activity-meta">
                <span>${prompts} prompts</span>
                <span>${mins > 0 ? mins + ' min' : ''}</span>
                <span>${new Date(a.timestamp).toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' })}</span>
              </div>
            </div>
            <span class="importance-badge ${a.importance}">${a.importance}</span>
          </div>`;
      }
      html += '</div>';
    } else {
      html += `<p style="color: var(--text-muted); font-size: 13px;">${promptCount} prompts in this session (click the activity list to see details)</p>`;
    }

    detail.innerHTML = html;
  } catch (err) {
    detail.innerHTML = `<p>${escapeHtml(project)} — ${dateLabel}: ${promptCount} prompts</p>`;
  }
}

function filterClaudeProjects() {
  const query = ($('#claude-project-filter').value || '').toLowerCase();
  $$('#claude-projects .claude-project-item').forEach(item => {
    const name = item.querySelector('.claude-project-name')?.textContent.toLowerCase() || '';
    item.style.display = name.includes(query) ? '' : 'none';
  });
}

// --- Setup View ---
async function loadSetup() {
  try {
    const config = await api.getConfig();
    state.config = config;

    // Show welcome banner if not configured
    const welcome = $('#setup-welcome');
    if (!config.config_exists || Object.keys(config.email_tags || {}).length === 0) {
      welcome.classList.remove('hidden');
    } else {
      welcome.classList.add('hidden');
    }

    renderSetupDirs(config.scan_dirs || []);
    renderSetupEmails(config.email_tags || {});
    renderSetupIntegrations(config.integrations || {});
  } catch (err) {
    console.error('Failed to load setup:', err);
  }
}

function renderSetupDirs(dirs) {
  const container = $('#setup-dirs');
  if (dirs.length === 0) {
    container.innerHTML = '<p class="setup-empty">No scan directories configured</p>';
    return;
  }

  container.innerHTML = dirs.map(d => `
    <div class="setup-dir-item">
      <div class="setup-dir-info">
        <span class="setup-dir-path">${escapeHtml(d.path)}</span>
        <span class="setup-dir-meta">${d.exists ? `${d.repo_count} repos` : 'directory not found'}</span>
      </div>
      <button class="setup-remove-btn" onclick="handleRemoveDir('${escapeHtml(d.path)}')" title="Remove">&times;</button>
    </div>
  `).join('');
}

function renderSetupEmails(emailTags) {
  const container = $('#setup-emails');
  const entries = Object.entries(emailTags);

  if (entries.length === 0) {
    container.innerHTML = '<p class="setup-empty">No email identities mapped. Click "Detect Emails" to scan your repos.</p>';
    return;
  }

  container.innerHTML = entries.map(([email, tag]) => `
    <div class="setup-email-item">
      <span class="setup-email-addr">${escapeHtml(email)}</span>
      <select class="select-input setup-tag-select" onchange="handleEmailTagChange('${escapeHtml(email)}', this.value)">
        <option value="personal" ${tag === 'personal' ? 'selected' : ''}>Personal</option>
        <option value="work" ${tag === 'work' ? 'selected' : ''}>Work</option>
      </select>
    </div>
  `).join('');
}

function renderSetupIntegrations(integrations) {
  const container = $('#setup-integrations');
  const items = [
    { key: 'git', label: 'Git', desc: 'Local git repositories', configured: integrations.git?.enabled },
    { key: 'github', label: 'GitHub', desc: 'Pull requests and issues', configured: integrations.github?.configured },
    { key: 'linear', label: 'Linear', desc: 'Issue tracking', configured: integrations.linear?.configured },
    { key: 'claude_code', label: 'Claude Code', desc: 'AI coding sessions', configured: integrations.claude_code?.configured },
  ];

  container.innerHTML = items.map(item => `
    <div class="setup-integration-item">
      <div class="setup-integration-status ${item.configured ? 'configured' : 'not-configured'}"></div>
      <div class="setup-integration-info">
        <span class="setup-integration-label">${item.label}</span>
        <span class="setup-integration-desc">${item.desc}</span>
      </div>
      <span class="setup-integration-badge">${item.configured ? 'Configured' : 'Not configured'}</span>
    </div>
  `).join('');
}

async function handleDiscoverDirs() {
  try {
    const btn = event.target;
    btn.textContent = 'Discovering...';
    btn.disabled = true;

    const discovery = await api.discoverConfig();

    // Show discovered dirs that aren't already configured
    const container = $('#setup-dirs');
    let html = '';

    for (const d of discovery.dirs) {
      const icon = d.already_configured ? '&check;' : '+';
      const cls = d.already_configured ? 'setup-dir-item' : 'setup-dir-item setup-dir-suggested';
      html += `
        <div class="${cls}">
          <div class="setup-dir-info">
            <span class="setup-dir-path">${escapeHtml(d.path)}</span>
            <span class="setup-dir-meta">${d.repo_count} repos (${d.repo_names.slice(0, 3).join(', ')}${d.repo_names.length > 3 ? '...' : ''})</span>
          </div>
          ${d.already_configured
            ? `<button class="setup-remove-btn" onclick="handleRemoveDir('${escapeHtml(d.path)}')">&times;</button>`
            : `<button class="btn btn-secondary" style="padding: 4px 12px; font-size: 12px;" onclick="handleAddDiscoveredDir('${escapeHtml(d.path)}')">Add</button>`
          }
        </div>`;
    }

    container.innerHTML = html || '<p class="setup-empty">No directories with git repos found</p>';
    btn.textContent = 'Discover Directories';
    btn.disabled = false;
  } catch (err) {
    console.error('Discovery failed:', err);
  }
}

async function handleAddDiscoveredDir(path) {
  const config = state.config;
  const currentDirs = (config.scan_dirs || []).map(d => d.path);
  if (!currentDirs.includes(path)) {
    currentDirs.push(path);
  }
  try {
    await api.updateConfig({ scan_dirs: currentDirs });
    await loadSetup();
  } catch (err) {
    console.error('Failed to add dir:', err);
  }
}

async function handleAddDir() {
  const input = $('#setup-add-dir');
  const path = input.value.trim();
  if (!path) return;

  const config = state.config;
  const currentDirs = (config.scan_dirs || []).map(d => d.path);
  currentDirs.push(path);

  try {
    await api.updateConfig({ scan_dirs: currentDirs });
    input.value = '';
    await loadSetup();
  } catch (err) {
    console.error('Failed to add dir:', err);
  }
}

async function handleRemoveDir(path) {
  const config = state.config;
  const currentDirs = (config.scan_dirs || []).map(d => d.path).filter(d => d !== path);
  try {
    await api.updateConfig({ scan_dirs: currentDirs });
    await loadSetup();
  } catch (err) {
    console.error('Failed to remove dir:', err);
  }
}

async function handleDetectEmails() {
  try {
    const btn = event.target;
    btn.textContent = 'Detecting...';
    btn.disabled = true;

    const discovery = await api.discoverConfig();
    const currentTags = state.config?.email_tags || {};

    // Merge discovered with current
    const newTags = { ...currentTags };
    for (const e of discovery.emails) {
      if (!newTags[e.email]) {
        newTags[e.email] = e.current_tag || e.suggested_tag;
      }
    }

    await api.updateConfig({ email_tags: newTags });
    await loadSetup();
    btn.textContent = 'Detect Emails from Repos';
    btn.disabled = false;
  } catch (err) {
    console.error('Failed to detect emails:', err);
  }
}

async function handleEmailTagChange(email, tag) {
  const currentTags = state.config?.email_tags || {};
  currentTags[email] = tag;
  try {
    await api.updateConfig({ email_tags: currentTags });
  } catch (err) {
    console.error('Failed to update email tag:', err);
  }
}

async function handleSyncNow() {
  const status = $('#setup-sync-status');
  status.textContent = 'This may take a moment. Run "folio sync" from the terminal for full sync.';
  // Note: We can't run sync from the web server directly since it's a CLI operation.
  // For now, guide the user to use the CLI.
  status.textContent = 'Run "folio sync" in your terminal, then refresh this page to see results.';
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
    case 'timeline':
      loadTimeline();
      break;
    case 'claude':
      loadClaude();
      break;
    case 'accomplish':
      loadAccomplishments();
      break;
    case 'search':
      setTimeout(() => $('#search-input').focus(), 100);
      break;
    case 'whatsnew':
      loadWhatsNew();
      break;
    case 'setup':
      loadSetup();
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
  $('#filter-source').addEventListener('change', loadActivities);
  $('#filter-importance').addEventListener('change', loadActivities);
  $('#filter-project').addEventListener('change', loadActivities);
  $('#btn-refresh').addEventListener('click', loadActivities);

  // Heatmap controls
  $('#heatmap-months').addEventListener('change', loadTimeline);
  $('#claude-days').addEventListener('change', loadClaude);

  // Claude tag toggle
  $$('#claude-tag-toggle .toggle-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      $$('#claude-tag-toggle .toggle-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      state.claudeTagFilter = btn.dataset.value;
      applyClaudeTagFilter();
    });
  });

  // Claude heatmap click handler
  document.addEventListener('click', (e) => {
    const cell = e.target.closest('.claude-hm-cell');
    if (!cell) return;
    const date = cell.dataset.date;
    const project = cell.dataset.project;
    const count = cell.dataset.count;
    if (!date || count === '0') return;

    // Highlight selected
    $$('.claude-hm-cell.selected').forEach(c => c.classList.remove('selected'));
    cell.classList.add('selected');

    showClaudeHeatmapDetail(project, date, parseInt(count));
  });

  // Floating tooltip for heatmap cells
  const tooltip = document.createElement('div');
  tooltip.className = 'heatmap-tooltip hidden';
  document.body.appendChild(tooltip);

  document.addEventListener('mouseover', (e) => {
    const cell = e.target.closest('.heatmap-cell, .claude-hm-cell');
    if (!cell) { tooltip.classList.add('hidden'); return; }

    const count = cell.dataset.count || '0';
    const date = cell.dataset.date;
    const project = cell.dataset.project;
    if (!date || count === '0') { tooltip.classList.add('hidden'); return; }

    const d = new Date(date + 'T12:00:00');
    const dateLabel = d.toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' });
    const unit = cell.classList.contains('claude-hm-cell') ? 'prompts' : 'activities';
    const projectLine = project ? `<div class="heatmap-tooltip-project">${project}</div>` : '';

    tooltip.innerHTML = `${projectLine}<strong>${count}</strong> ${unit}<div class="heatmap-tooltip-date">${dateLabel}</div>`;
    tooltip.classList.remove('hidden');

    const rect = cell.getBoundingClientRect();
    tooltip.style.left = (rect.left + rect.width / 2 - tooltip.offsetWidth / 2) + 'px';
    tooltip.style.top = (rect.top - tooltip.offsetHeight - 8 + window.scrollY) + 'px';
  });

  document.addEventListener('mouseout', (e) => {
    if (e.target.closest('.heatmap-cell, .claude-hm-cell')) {
      tooltip.classList.add('hidden');
    }
  });

  // Initial load — detect first launch
  checkHealth().then(async (ok) => {
    if (!ok) return;
    try {
      const stats = await api.getStats();
      if (stats.total_activities === 0) {
        switchView('setup');
      } else {
        loadDashboard();
      }
    } catch {
      loadDashboard();
    }
  });

  // Periodic health check
  setInterval(checkHealth, 30000);
}

document.addEventListener('DOMContentLoaded', init);
