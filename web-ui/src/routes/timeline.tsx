import { createRoute } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { useState, useMemo, useCallback } from "react";
import { rootRoute } from "./__root";
import { heatmapQuery, timelineQuery, insightsQuery } from "../api/queries";
import { ActivityItem } from "../components/ActivityItem";
import { DoughnutChart, DoughnutLegend } from "../components/DoughnutChart";
import { sourceIcon, sourceLabel } from "../components/helpers";

export const timelineRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/timeline",
  component: TimelinePage,
});

const SOURCE_COLORS: Record<string, string> = {
  git: "#f97316",
  github: "#e2e8f0",
  linear: "#6366f1",
  claude_code: "#d4a574",
  manual: "#22c55e",
  jira: "#3b82f6",
};

function TimelinePage() {
  const [months, setMonths] = useState(6);
  const [selectedDate, setSelectedDate] = useState<string | undefined>();
  const [filter, setFilter] = useState<{ type: string; value: string } | null>(null);

  const { data: heatmapData } = useQuery(heatmapQuery(months));
  const { data: timelineData } = useQuery(
    selectedDate ? timelineQuery(1, selectedDate) : timelineQuery(7)
  );
  const { data: insightsData } = useQuery(
    selectedDate ? insightsQuery(selectedDate) : insightsQuery(undefined, 7)
  );

  const handleFilter = useCallback(
    (type: string, value: string) => {
      setFilter((prev) =>
        prev?.type === type && prev?.value === value ? null : { type, value }
      );
    },
    []
  );

  return (
    <section className="view active">
      <header className="view-header">
        <h2>Timeline</h2>
        <p className="subtitle">Activity heatmap and daily breakdown</p>
        <div className="view-controls">
          <select className="select-input" value={months} onChange={(e) => setMonths(Number(e.target.value))}>
            <option value={3}>Last 3 months</option>
            <option value={6}>Last 6 months</option>
            <option value={12}>Last 12 months</option>
          </select>
        </div>
      </header>

      <div className="card" style={{ marginBottom: 24 }}>
        <h3>Activity Heatmap</h3>
        <Heatmap data={heatmapData} months={months} selectedDate={selectedDate} onSelectDate={setSelectedDate} />
        <div className="heatmap-legend">
          <span className="heatmap-legend-label">Less</span>
          {[0, 1, 2, 3, 4].map((level) => (
            <span key={level} className="heatmap-cell" data-level={level} />
          ))}
          <span className="heatmap-legend-label">More</span>
        </div>
      </div>

      {insightsData && insightsData.total > 0 && (
        <InsightsPanel data={insightsData} onFilter={handleFilter} activeFilter={filter} />
      )}

      <div className="card">
        <h3>Daily Breakdown</h3>
        {filter && (
          <div className="timeline-filter-indicator">
            Showing: <strong>{filter.value.replace(/_/g, " ")}</strong>{" "}
            <button className="timeline-filter-clear" onClick={() => setFilter(null)}>&times; Clear</button>
          </div>
        )}
        <TimelineDays data={timelineData} filter={filter} />
      </div>
    </section>
  );
}

function Heatmap({
  data,
  months,
  selectedDate,
  onSelectDate,
}: {
  data: { days: { date: string; count: number }[] } | undefined;
  months: number;
  selectedDate?: string;
  onSelectDate: (date: string) => void;
}) {
  const grid = useMemo(() => {
    if (!data?.days) return null;

    const countMap: Record<string, number> = {};
    let maxCount = 0;
    for (const d of data.days) {
      countMap[d.date] = d.count;
      if (d.count > maxCount) maxCount = d.count;
    }

    const endDate = new Date();
    const startDate = new Date();
    startDate.setDate(startDate.getDate() - months * 30);
    startDate.setDate(startDate.getDate() - startDate.getDay());

    const weeks: { date: string; count: number; level: number; future: boolean }[][] = [];
    const current = new Date(startDate);

    while (current <= endDate) {
      const week: typeof weeks[number] = [];
      for (let day = 0; day < 7; day++) {
        const dateStr = current.toISOString().split("T")[0];
        const count = countMap[dateStr] || 0;
        const level = maxCount === 0 ? 0 : Math.min(4, Math.ceil((count / maxCount) * 4));
        week.push({ date: dateStr, count, level, future: current > endDate });
        current.setDate(current.getDate() + 1);
      }
      weeks.push(week);
    }

    return { weeks, maxCount };
  }, [data, months]);

  if (!grid) {
    return <p style={{ color: "var(--text-muted)", fontSize: 13 }}>No activity data yet</p>;
  }

  const dayLabels = ["", "Mon", "", "Wed", "", "Fri", ""];

  return (
    <div className="heatmap-container">
      <div className="heatmap-grid">
        <div className="heatmap-labels">
          {dayLabels.map((label, i) => (
            <div key={i} className="heatmap-day-label">{label}</div>
          ))}
        </div>
        {grid.weeks.map((week, wi) => (
          <div key={wi} className="heatmap-week">
            {week.map((day) => (
              <div
                key={day.date}
                className={`heatmap-cell${day.future ? " future" : ""}${day.date === selectedDate ? " selected" : ""}`}
                data-level={day.future ? "" : day.level}
                title={`${day.date}: ${day.count} activities`}
                onClick={() => onSelectDate(day.date)}
              />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}

function InsightsPanel({
  data,
  onFilter,
  activeFilter,
}: {
  data: NonNullable<ReturnType<typeof insightsQuery>["queryKey"]> extends unknown ? {
    total: number;
    importance: { high: number; medium: number; low: number };
    by_source: Record<string, number>;
    top_projects: { project: string; count: number }[];
    highlights: { id: string; title: string; source: string; project?: string; timestamp: string; importance: string }[];
    insights: { type: string; text: string }[];
    streak: number;
  } : never;
  onFilter: (type: string, value: string) => void;
  activeFilter: { type: string; value: string } | null;
}) {
  const importanceSlices = [
    { label: "High", value: data.importance.high, color: "#f43f5e" },
    { label: "Medium", value: data.importance.medium, color: "#f59e0b" },
    { label: "Low", value: data.importance.low, color: "#22c55e" },
  ];

  const sourceSlices = Object.entries(data.by_source)
    .sort((a, b) => b[1] - a[1])
    .map(([src, count]) => ({
      label: src.replace(/_/g, " "),
      value: count,
      color: SOURCE_COLORS[src] || "#94a3b8",
    }));

  return (
    <>
      <div className="insights-row-1">
        <div className="card insights-stats-card">
          <div className="insights-stat-grid">
            <div className="insights-stat">
              <div className="insights-stat-value">{data.total}</div>
              <div className="insights-stat-label">Activities</div>
            </div>
            <div className="insights-stat">
              <div className="insights-stat-value">{data.top_projects.length}</div>
              <div className="insights-stat-label">Projects</div>
            </div>
            <div className="insights-stat">
              <div className="insights-stat-value">{data.streak || 0}</div>
              <div className="insights-stat-label">Day Streak</div>
            </div>
            <div
              className="insights-stat clickable"
              onClick={() => onFilter("importance", "high")}
              title="Click to filter"
            >
              <div className="insights-stat-value">{data.importance.high}</div>
              <div className="insights-stat-label">High Impact</div>
            </div>
          </div>
          {data.insights.length > 0 && (
            <div className="insights-callouts">
              {data.insights.map((i, idx) => {
                const icon = i.type === "positive" ? "\u25B2" : i.type === "suggestion" ? "\u26A0" : "\u25CF";
                return (
                  <div key={idx} className={`insight-callout ${i.type}`}>
                    {icon} {i.text}
                  </div>
                );
              })}
            </div>
          )}
        </div>
        <div className="card insights-chart-card">
          <h3>Impact</h3>
          <DoughnutChart slices={importanceSlices} total={data.total} />
          <DoughnutLegend
            slices={importanceSlices}
            total={data.total}
            onFilter={(val) => onFilter("importance", val)}
          />
        </div>
        <div className="card insights-chart-card">
          <h3>Sources</h3>
          <DoughnutChart slices={sourceSlices} total={data.total} />
          <DoughnutLegend
            slices={sourceSlices}
            total={data.total}
            onFilter={(val) => onFilter("source", val)}
          />
        </div>
      </div>

      <div className="insights-row-2">
        <div className="card">
          <h3>Top Projects</h3>
          {data.top_projects.length === 0 ? (
            <p className="setup-empty">No project data</p>
          ) : (
            data.top_projects.map((p) => {
              const maxCount = data.top_projects[0].count;
              const pct = maxCount > 0 ? (p.count / maxCount) * 100 : 0;
              return (
                <div key={p.project} className="insights-proj-row">
                  <span className="insights-proj-name" title={p.project}>{p.project}</span>
                  <div className="insights-proj-bar-wrap">
                    <div className="insights-proj-bar" style={{ width: `${pct}%` }} />
                  </div>
                  <span className="insights-proj-count">{p.count}</span>
                </div>
              );
            })
          )}
        </div>
        <div className="card">
          <h3>High Impact Highlights</h3>
          {data.highlights.length > 0 ? (
            data.highlights.map((h) => (
              <div key={h.id} className="insights-highlight-item">
                <div
                  className={`activity-source-icon ${h.source}`}
                  style={{ width: 24, height: 24, fontSize: 10, borderRadius: 5 }}
                >
                  {sourceIcon(h.source)}
                </div>
                <div className="insights-highlight-info">
                  <div className="insights-highlight-title">{h.title}</div>
                  <div className="insights-highlight-meta">
                    {h.project || ""} &middot; {new Date(h.timestamp).toLocaleDateString()}
                  </div>
                </div>
              </div>
            ))
          ) : (
            <p style={{ color: "var(--text-muted)", fontSize: 13 }}>No high-impact activities in this period</p>
          )}
        </div>
      </div>
    </>
  );
}

function TimelineDays({
  data,
  filter,
}: {
  data: { days: { date: string; count: number; activities: { id: string; title: string; source: string; importance: string; project?: string; timestamp: string }[] }[] } | undefined;
  filter: { type: string; value: string } | null;
}) {
  const [daySourceFilters, setDaySourceFilters] = useState<Record<string, string>>({});

  const days = data?.days || [];
  if (days.length === 0) {
    return <p style={{ color: "var(--text-muted)", fontSize: 13, padding: "12px 0" }}>No activities for this period</p>;
  }

  return (
    <div>
      {days.map((day) => {
        const bySource: Record<string, typeof day.activities> = {};
        for (const a of day.activities) {
          const src = a.source || "manual";
          if (!bySource[src]) bySource[src] = [];
          bySource[src].push(a);
        }

        const activeSourceFilter = daySourceFilters[day.date] || "all";

        const filteredActivities = day.activities.filter((a) => {
          if (filter) {
            const matches =
              filter.type === "importance"
                ? a.importance === filter.value
                : a.source === filter.value;
            if (!matches) return false;
          }
          if (activeSourceFilter !== "all" && a.source !== activeSourceFilter) return false;
          return true;
        });

        const dateLabel = new Date(day.date + "T12:00:00").toLocaleDateString("en-US", {
          weekday: "long",
          month: "long",
          day: "numeric",
          year: "numeric",
        });

        return (
          <div key={day.date} className="timeline-day">
            <div className="timeline-day-header">
              <span className="timeline-date">{dateLabel}</span>
              <span className="timeline-count">{day.count} activities</span>
            </div>
            <div className="timeline-source-chips">
              <button
                className={`timeline-source-chip ${activeSourceFilter === "all" ? "active" : ""}`}
                onClick={() => setDaySourceFilters((p) => ({ ...p, [day.date]: "all" }))}
              >
                All <strong>{day.count}</strong>
              </button>
              {Object.entries(bySource)
                .sort((a, b) => b[1].length - a[1].length)
                .map(([src, items]) => (
                  <button
                    key={src}
                    className={`timeline-source-chip ${activeSourceFilter === src ? "active" : ""}`}
                    onClick={() => setDaySourceFilters((p) => ({ ...p, [day.date]: src }))}
                  >
                    <span className={`activity-source-icon ${src}`} style={{ width: 18, height: 18, fontSize: 9, borderRadius: 4 }}>
                      {sourceIcon(src)}
                    </span>
                    {sourceLabel(src)} <strong>{items.length}</strong>
                  </button>
                ))}
            </div>
            <div className="timeline-day-activities">
              {filteredActivities.map((a) => (
                <ActivityItem key={a.id} activity={a as never} />
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}
