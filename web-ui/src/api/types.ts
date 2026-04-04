// API response types matching the Axum backend

export interface Activity {
  id: string;
  title: string;
  description?: string;
  source: string;
  importance: "high" | "medium" | "low";
  project?: string;
  employer?: string;
  timestamp: string;
  metadata?: Record<string, unknown>;
}

export interface Stats {
  total_activities: number;
  by_importance: {
    high: number;
    medium: number;
    low: number;
  };
  by_source: Record<string, number>;
  projects_count: number;
}

export interface HeatmapDay {
  date: string;
  count: number;
}

export interface HeatmapData {
  days: HeatmapDay[];
}

export interface TimelineDay {
  date: string;
  count: number;
  activities: Activity[];
}

export interface TimelineData {
  days: TimelineDay[];
}

export interface InsightCallout {
  type: "positive" | "suggestion" | "neutral";
  text: string;
}

export interface InsightsData {
  total: number;
  importance: { high: number; medium: number; low: number };
  by_source: Record<string, number>;
  top_projects: { project: string; count: number }[];
  highlights: Activity[];
  insights: InsightCallout[];
  streak: number;
}

export interface Accomplishment {
  id: string;
  title: string;
  situation?: string;
  task?: string;
  action?: string;
  result?: string;
  employer?: string;
  role?: string;
  skills?: string[];
  themes?: string[];
  generated_bullets?: string[];
  activity_ids?: string[];
}

export interface ClaudeProject {
  project: string;
  sessions: number;
  total_prompts: number;
  total_minutes: number;
  days_active: number;
  is_personal: boolean;
}

export interface ClaudeProjectsData {
  projects: ClaudeProject[];
  total_projects: number;
  total_sessions: number;
}

export interface ClaudeHeatmapProject {
  project: string;
  days: { date: string; prompts: number }[];
}

export interface ClaudeHeatmapData {
  projects: ClaudeHeatmapProject[];
  days: number;
}

export interface ScanDir {
  path: string;
  exists: boolean;
  repo_count: number;
}

export interface ConfigData {
  config_exists: boolean;
  scan_dirs: ScanDir[];
  email_tags: Record<string, string>;
  integrations: Record<string, { enabled?: boolean; configured?: boolean }>;
}

export interface DiscoveryData {
  dirs: {
    path: string;
    repo_count: number;
    repo_names: string[];
    already_configured: boolean;
  }[];
  emails: {
    email: string;
    current_tag?: string;
    suggested_tag: string;
  }[];
}

export interface CreateActivityPayload {
  title: string;
  description?: string;
  impact?: string;
  importance?: string;
  project?: string;
  employer?: string;
}

export interface UpdateActivityPayload {
  title?: string;
  impact?: string;
  project?: string;
  employer?: string;
  importance?: string;
}

export interface CreateAccomplishmentPayload {
  title: string;
  situation?: string;
  task?: string;
  action?: string;
  result?: string;
  skills?: string[];
  themes?: string[];
  activity_ids?: string[];
}
