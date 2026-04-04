import { createRoute } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { useState, useMemo } from "react";
import { rootRoute } from "./__root";
import { claudeProjectsQuery, claudeHeatmapQuery } from "../api/queries";
import type { ClaudeProject, ClaudeHeatmapProject } from "../api/types";
import { formatHours } from "../components/helpers";

export const claudeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/claude",
  component: ClaudePage,
});

function ClaudePage() {
  const [days, setDays] = useState(30);
  const [tagFilter, setTagFilter] = useState<"all" | "work" | "personal">("all");
  const [projectFilter, setProjectFilter] = useState("");

  const { data: projectsData } = useQuery(claudeProjectsQuery(days));
  const { data: heatmapData } = useQuery(claudeHeatmapQuery(days));

  const filteredProjects = useMemo(() => {
    if (!projectsData) return [];
    let projects = projectsData.projects;
    if (tagFilter !== "all") {
      projects = projects.filter((p) => (tagFilter === "personal") === p.is_personal);
    }
    return projects;
  }, [projectsData, tagFilter]);

  const filteredHeatmap = useMemo(() => {
    if (!heatmapData || !projectsData) return [];
    if (tagFilter === "all") return heatmapData.projects;
    return heatmapData.projects.filter((p) => {
      const projData = projectsData.projects.find((pp) => pp.project === p.project);
      return projData ? (tagFilter === "personal") === projData.is_personal : true;
    });
  }, [heatmapData, projectsData, tagFilter]);

  // Compute estimated hours per tag
  const workPrompts = projectsData?.projects.filter((p) => !p.is_personal).reduce((s, p) => s + p.total_prompts, 0) || 0;
  const personalPrompts = projectsData?.projects.filter((p) => p.is_personal).reduce((s, p) => s + p.total_prompts, 0) || 0;
  const allPrompts = workPrompts + personalPrompts;

  const toggleLabels = {
    all: `All ~${Math.floor((allPrompts * 5) / 60)}h`,
    work: `Work ~${Math.floor((workPrompts * 5) / 60)}h`,
    personal: `Personal ~${Math.floor((personalPrompts * 5) / 60)}h`,
  };

  // Stats
  const totalPrompts = filteredProjects.reduce((sum, p) => sum + p.total_prompts, 0);
  const totalMinutes = filteredProjects.reduce((sum, p) => sum + p.total_minutes, 0);
  const estTotalMins = Math.round(totalPrompts * 5);
  const estHours = Math.floor(estTotalMins / 60);
  const perDay = (estTotalMins / days / 60).toFixed(1);

  // Filter projects by text search
  const displayProjects = projectFilter
    ? filteredProjects.filter((p) => p.project.toLowerCase().includes(projectFilter.toLowerCase()))
    : filteredProjects;

  return (
    <section className="view active">
      <header className="view-header">
        <h2>Claude Code Usage</h2>
        <p className="subtitle">Where you've been using AI-assisted coding</p>
        <div className="view-controls">
          <div className="toggle-group">
            {(["all", "work", "personal"] as const).map((val) => (
              <button
                key={val}
                className={`toggle-btn ${tagFilter === val ? "active" : ""}`}
                onClick={() => setTagFilter(val)}
              >
                {toggleLabels[val]}
              </button>
            ))}
          </div>
          <select className="select-input" value={days} onChange={(e) => setDays(Number(e.target.value))}>
            <option value={7}>Last 7 days</option>
            <option value={30}>Last 30 days</option>
            <option value={90}>Last 90 days</option>
          </select>
        </div>
      </header>

      <div className="claude-stats-row">
        <div className="stat-card">
          <div className="stat-value">{filteredProjects.length}</div>
          <div className="stat-label">Projects</div>
        </div>
        <div className="stat-card">
          <div className="stat-value">{filteredProjects.reduce((s, p) => s + p.sessions, 0)}</div>
          <div className="stat-label">Sessions</div>
        </div>
        <div className="stat-card">
          <div className="stat-value">{totalPrompts.toLocaleString()}</div>
          <div className="stat-label">Total Prompts</div>
        </div>
        <div className="stat-card">
          <div className="stat-value">~{estHours}h</div>
          <div className="stat-label">Est. Time with Claude</div>
          <div className="stat-sub">~{perDay}h/day &middot; {formatHours(totalMinutes)} active</div>
        </div>
      </div>

      <div className="claude-main-grid">
        <div className="card">
          <h3>Project Usage Heatmap</h3>
          <p style={{ fontSize: 12, color: "var(--text-muted)", marginBottom: 14 }}>
            Prompt volume by project and day — darker = more prompts.
          </p>
          <ClaudeHeatmap projects={filteredHeatmap} days={days} />
        </div>

        <div className="card claude-projects-card">
          <h3>Projects by Usage</h3>
          <div className="claude-projects-filter">
            <input
              type="text"
              placeholder="Filter projects..."
              className="setup-input"
              value={projectFilter}
              onChange={(e) => setProjectFilter(e.target.value)}
            />
          </div>
          <div className="claude-projects-scroll">
            <ProjectsList projects={displayProjects} />
          </div>
        </div>
      </div>
    </section>
  );
}

function ClaudeHeatmap({
  projects,
  days: numDays,
}: {
  projects: ClaudeHeatmapProject[];
  days: number;
}) {
  if (projects.length === 0) {
    return <p style={{ color: "var(--text-muted)", fontSize: 13 }}>No Claude Code data yet</p>;
  }

  const dates: string[] = [];
  const now = new Date();
  for (let i = numDays - 1; i >= 0; i--) {
    const d = new Date(now);
    d.setDate(d.getDate() - i);
    dates.push(d.toISOString().split("T")[0]);
  }

  let globalMax = 0;
  for (const p of projects) {
    for (const d of p.days || []) {
      if (d.prompts > globalMax) globalMax = d.prompts;
    }
  }

  const topProjects = projects.slice(0, 15);

  return (
    <div className="claude-heatmap-grid">
      <div className="claude-hm-row claude-hm-header">
        <div className="claude-hm-label" />
        {dates.map((date, i) => {
          if (i % 7 === 0) {
            const d = new Date(date + "T12:00:00");
            return (
              <div key={date} className="claude-hm-date-label">
                {d.toLocaleDateString("en-US", { month: "short", day: "numeric" })}
              </div>
            );
          }
          return <div key={date} className="claude-hm-date-label" />;
        })}
      </div>
      {topProjects.map((p) => {
        const dayMap: Record<string, number> = {};
        for (const d of p.days || []) dayMap[d.date] = d.prompts;

        return (
          <div key={p.project} className="claude-hm-row">
            <div className="claude-hm-label" title={p.project}>{p.project}</div>
            {dates.map((date) => {
              const count = dayMap[date] || 0;
              const level = globalMax === 0 ? 0 : Math.min(4, Math.ceil((count / globalMax) * 4));
              return (
                <div
                  key={date}
                  className="claude-hm-cell"
                  data-level={count > 0 ? level : 0}
                  title={`${p.project} - ${date}: ${count} prompts`}
                />
              );
            })}
          </div>
        );
      })}
      {projects.length > 15 && (
        <p style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 8 }}>
          Showing top 15 of {projects.length} projects
        </p>
      )}
    </div>
  );
}

function ProjectsList({ projects }: { projects: ClaudeProject[] }) {
  if (projects.length === 0) {
    return (
      <p style={{ color: "var(--text-muted)", fontSize: 13 }}>
        No Claude Code data yet. Run "folio sync --source claude" to import.
      </p>
    );
  }

  const maxPrompts = Math.max(...projects.map((p) => p.total_prompts));

  return (
    <>
      {projects.map((p) => {
        const pct = maxPrompts > 0 ? (p.total_prompts / maxPrompts) * 100 : 0;
        const timeStr = formatHours(p.total_minutes);

        return (
          <div key={p.project} className="claude-project-item">
            <div className="claude-project-info">
              <div className="claude-project-name">
                {p.project}{" "}
                <span className={`claude-tag ${p.is_personal ? "personal" : "work"}`}>
                  {p.is_personal ? "personal" : "work"}
                </span>
              </div>
              <div className="claude-project-meta">
                {p.sessions} sessions &middot; {p.total_prompts} prompts &middot; ~{timeStr} &middot; {p.days_active} days active
              </div>
            </div>
            <div className="claude-project-bar-wrap">
              <div className="claude-project-bar" style={{ width: `${pct}%` }} />
            </div>
            <div className="claude-project-count">{p.total_prompts}</div>
          </div>
        );
      })}
    </>
  );
}
