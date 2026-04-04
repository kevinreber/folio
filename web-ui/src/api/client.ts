import type {
  Activity,
  Stats,
  HeatmapData,
  TimelineData,
  InsightsData,
  Accomplishment,
  ClaudeProjectsData,
  ClaudeHeatmapData,
  ConfigData,
  DiscoveryData,
  CreateActivityPayload,
  UpdateActivityPayload,
  CreateAccomplishmentPayload,
} from "./types";

const API_BASE =
  (import.meta as Record<string, Record<string, string>>).env?.VITE_API_URL ||
  window.location.origin + "/api";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, init);
  if (!res.ok) throw new Error(`API error: ${res.status}`);
  return res.json();
}

function get<T>(path: string) {
  return request<T>(path);
}

function post<T>(path: string, body: unknown) {
  return request<T>(path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

function put<T>(path: string, body: unknown) {
  return request<T>(path, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
}

function del(path: string) {
  return fetch(`${API_BASE}${path}`, { method: "DELETE" }).then((res) => {
    if (!res.ok) throw new Error(`API error: ${res.status}`);
  });
}

export const api = {
  health: () => get<{ status: string; version: string }>("/health"),

  listActivities: (params?: {
    limit?: number;
    importance?: string;
    project?: string;
  }) => {
    const qs = new URLSearchParams();
    if (params?.limit) qs.set("limit", String(params.limit));
    if (params?.importance) qs.set("importance", params.importance);
    if (params?.project) qs.set("project", params.project);
    const query = qs.toString();
    return get<Activity[]>(`/activities${query ? "?" + query : ""}`);
  },

  getActivity: (id: string) => get<Activity>(`/activities/${id}`),

  createActivity: (data: CreateActivityPayload) =>
    post<Activity>("/activities", data),

  updateActivity: (id: string, data: UpdateActivityPayload) =>
    put<Activity>(`/activities/${id}`, data),

  deleteActivity: (id: string) => del(`/activities/${id}`),

  searchActivities: (q: string, limit = 50) =>
    get<Activity[]>(
      `/activities/search?q=${encodeURIComponent(q)}&limit=${limit}`
    ),

  getStats: () => get<Stats>("/stats"),

  getHeatmap: (months = 6) => get<HeatmapData>(`/heatmap?months=${months}`),

  getTimeline: (days = 7, date?: string) => {
    const params = date ? `date=${date}&days=1` : `days=${days}`;
    return get<TimelineData>(`/timeline?${params}`);
  },

  getInsights: (date?: string, days = 7) => {
    const params = date ? `date=${date}` : `days=${days}`;
    return get<InsightsData>(`/insights?${params}`);
  },

  listAccomplishments: () => get<Accomplishment[]>("/accomplishments"),

  createAccomplishment: (data: CreateAccomplishmentPayload) =>
    post<Accomplishment>("/accomplishments", data),

  getClaudeProjects: (days = 30) =>
    get<ClaudeProjectsData>(`/claude/projects?days=${days}`),

  getClaudeHeatmap: (days = 90) =>
    get<ClaudeHeatmapData>(`/claude/heatmap?days=${days}`),

  getConfig: () => get<ConfigData>("/config"),

  updateConfig: (data: Partial<{ scan_dirs: string[]; email_tags: Record<string, string> }>) =>
    post<ConfigData>("/config", data),

  discoverConfig: () => get<DiscoveryData>("/config/discover"),
};
