import type { Stats } from '../lib/api'

export function StatsGrid({ stats }: { stats: Stats }) {
  return (
    <div className="stats-grid">
      <div className="stat-card">
        <div className="stat-value">{stats.total_activities}</div>
        <div className="stat-label">Total Activities</div>
      </div>
      <div className="stat-card high">
        <div className="stat-value">{stats.by_importance.high}</div>
        <div className="stat-label">High Impact</div>
      </div>
      <div className="stat-card medium">
        <div className="stat-value">{stats.by_importance.medium}</div>
        <div className="stat-label">Medium Impact</div>
      </div>
      <div className="stat-card low">
        <div className="stat-value">{stats.by_importance.low}</div>
        <div className="stat-label">Low Impact</div>
      </div>
    </div>
  )
}
