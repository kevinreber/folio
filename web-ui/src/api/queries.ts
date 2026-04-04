import { queryOptions } from "@tanstack/react-query";
import { api } from "./client";

export const statsQuery = queryOptions({
  queryKey: ["stats"],
  queryFn: () => api.getStats(),
  staleTime: 30_000,
});

export const activitiesQuery = (params?: {
  limit?: number;
  importance?: string;
  project?: string;
}) =>
  queryOptions({
    queryKey: ["activities", params],
    queryFn: () => api.listActivities(params),
    staleTime: 30_000,
  });

export const activityQuery = (id: string) =>
  queryOptions({
    queryKey: ["activity", id],
    queryFn: () => api.getActivity(id),
    staleTime: 60_000,
  });

export const searchQuery = (q: string) =>
  queryOptions({
    queryKey: ["search", q],
    queryFn: () => api.searchActivities(q),
    enabled: q.length > 0,
    staleTime: 15_000,
  });

export const heatmapQuery = (months = 6) =>
  queryOptions({
    queryKey: ["heatmap", months],
    queryFn: () => api.getHeatmap(months),
    staleTime: 60_000,
  });

export const timelineQuery = (days = 7, date?: string) =>
  queryOptions({
    queryKey: ["timeline", days, date],
    queryFn: () => api.getTimeline(days, date),
    staleTime: 30_000,
  });

export const insightsQuery = (date?: string, days = 7) =>
  queryOptions({
    queryKey: ["insights", date, days],
    queryFn: () => api.getInsights(date, days),
    staleTime: 30_000,
  });

export const accomplishmentsQuery = queryOptions({
  queryKey: ["accomplishments"],
  queryFn: () => api.listAccomplishments(),
  staleTime: 30_000,
});

export const claudeProjectsQuery = (days = 30) =>
  queryOptions({
    queryKey: ["claude-projects", days],
    queryFn: () => api.getClaudeProjects(days),
    staleTime: 60_000,
  });

export const claudeHeatmapQuery = (days = 90) =>
  queryOptions({
    queryKey: ["claude-heatmap", days],
    queryFn: () => api.getClaudeHeatmap(days),
    staleTime: 60_000,
  });

export const configQuery = queryOptions({
  queryKey: ["config"],
  queryFn: () => api.getConfig(),
  staleTime: 60_000,
});

export const healthQuery = queryOptions({
  queryKey: ["health"],
  queryFn: () => api.health(),
  refetchInterval: 30_000,
  staleTime: 10_000,
  retry: 1,
});
