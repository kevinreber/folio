import { createRoute } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { rootRoute } from "./__root";
import { statsQuery, activitiesQuery } from "../api/queries";
import { ActivityItem } from "../components/ActivityItem";
import { sourceLabel } from "../components/helpers";

export const dashboardRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: DashboardPage,
});

function DashboardPage() {
  const stats = useQuery(statsQuery);
  const recent = useQuery(activitiesQuery({ limit: 8 }));

  const s = stats.data;
  const bySource = s?.by_source || {};
  const max = Math.max(...Object.values(bySource), 0);
  const sorted = Object.entries(bySource).sort((a, b) => b[1] - a[1]);

  return (
    <section className="view active">
      <header className="view-header">
        <h2>Dashboard</h2>
        <p className="subtitle">Your career activity at a glance</p>
      </header>

      <div className="stats-grid">
        <div className="stat-card">
          <div className="stat-value">{s?.total_activities ?? "--"}</div>
          <div className="stat-label">Total Activities</div>
        </div>
        <div className="stat-card high">
          <div className="stat-value">{s?.by_importance.high ?? "--"}</div>
          <div className="stat-label">High Impact</div>
        </div>
        <div className="stat-card medium">
          <div className="stat-value">{s?.by_importance.medium ?? "--"}</div>
          <div className="stat-label">Medium Impact</div>
        </div>
        <div className="stat-card low">
          <div className="stat-value">{s?.by_importance.low ?? "--"}</div>
          <div className="stat-label">Low Impact</div>
        </div>
      </div>

      <div className="dashboard-grid">
        <div className="card">
          <h3>Activity by Source</h3>
          <div className="chart-container">
            {sorted.length === 0 ? (
              <p style={{ color: "var(--text-muted)", fontSize: 13 }}>No data yet</p>
            ) : (
              sorted.map(([source, count]) => {
                const pct = max > 0 ? (count / max) * 100 : 0;
                return (
                  <div className="chart-bar" key={source}>
                    <span className="chart-bar-label">{sourceLabel(source)}</span>
                    <div className="chart-bar-track">
                      <div className="chart-bar-fill" style={{ width: `${pct}%` }} />
                    </div>
                    <span className="chart-bar-value">{count}</span>
                  </div>
                );
              })
            )}
          </div>
        </div>
        <div className="card">
          <h3>Recent Activity</h3>
          <div className="activity-list compact">
            {recent.data && recent.data.length > 0 ? (
              recent.data.map((a) => <ActivityItem key={a.id} activity={a} />)
            ) : (
              <p style={{ color: "var(--text-muted)", fontSize: 13, padding: "12px 0" }}>
                No activities yet. Capture your first one!
              </p>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}
