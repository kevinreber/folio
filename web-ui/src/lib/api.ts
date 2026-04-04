const API_BASE = window.location.origin + '/api'

export interface Activity {
  id: string
  title: string
  description?: string
  source: string
  importance: 'high' | 'medium' | 'low'
  project?: string
  employer?: string
  timestamp: string
  metadata?: { impact?: string; [key: string]: unknown }
}

export interface Stats {
  total_activities: number
  by_importance: { high: number; medium: number; low: number }
  by_source: Record<string, number>
  projects_count: number
}

export interface CreateActivityRequest {
  title: string
  description?: string
  impact?: string
  importance?: string
  project?: string
  employer?: string
}

export interface UpdateActivityRequest {
  title?: string
  impact?: string
  project?: string
  employer?: string
  importance?: string
}

async function get<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`)
  if (!res.ok) throw new Error(`API error: ${res.status}`)
  return res.json()
}

async function post<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  if (!res.ok) throw new Error(`API error: ${res.status}`)
  return res.json()
}

async function put<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  if (!res.ok) throw new Error(`API error: ${res.status}`)
  return res.json()
}

async function del(path: string): Promise<void> {
  const res = await fetch(`${API_BASE}${path}`, { method: 'DELETE' })
  if (!res.ok) throw new Error(`API error: ${res.status}`)
}

export const api = {
  health: () => get<{ status: string; version: string }>('/health'),

  listActivities: (params?: { limit?: number; importance?: string; project?: string }) => {
    const qs = new URLSearchParams()
    if (params?.limit) qs.set('limit', String(params.limit))
    if (params?.importance) qs.set('importance', params.importance)
    if (params?.project) qs.set('project', params.project)
    const query = qs.toString()
    return get<Activity[]>(`/activities${query ? '?' + query : ''}`)
  },

  getActivity: (id: string) => get<Activity>(`/activities/${id}`),

  createActivity: (data: CreateActivityRequest) => post<Activity>('/activities', data),

  updateActivity: (id: string, data: UpdateActivityRequest) => put<Activity>(`/activities/${id}`, data),

  deleteActivity: (id: string) => del(`/activities/${id}`),

  searchActivities: (q: string, limit = 50) =>
    get<Activity[]>(`/activities/search?q=${encodeURIComponent(q)}&limit=${limit}`),

  getStats: () => get<Stats>('/stats'),
}
