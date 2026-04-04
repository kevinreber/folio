import { useEffect, useState } from 'react'
import { api } from '../lib/api'
import type { Activity, Stats } from '../lib/api'
import { subscribeToUpdates } from '../lib/sse'
import { StatsGrid } from '../components/StatsGrid'
import { SourceChart } from '../components/SourceChart'
import { ActivityItem } from '../components/ActivityItem'

export function DashboardPage() {
  const [stats, setStats] = useState<Stats | null>(null)
  const [recent, setRecent] = useState<Activity[]>([])

  const load = () => {
    Promise.all([api.getStats(), api.listActivities({ limit: 8 })]).then(([s, a]) => {
      setStats(s)
      setRecent(a)
    }).catch(console.error)
  }

  useEffect(() => {
    load()
    return subscribeToUpdates(load)
  }, [])

  return (
    <section className="view active">
      <header className="view-header">
        <h2>Dashboard</h2>
        <p className="subtitle">Your career activity at a glance</p>
      </header>

      {stats && <StatsGrid stats={stats} />}

      <div className="dashboard-grid">
        <div className="card">
          <h3>Activity by Source</h3>
          {stats && <SourceChart bySource={stats.by_source} />}
        </div>
        <div className="card">
          <h3>Recent Activity</h3>
          <div className="activity-list compact">
            {recent.length > 0
              ? recent.map((a) => <ActivityItem key={a.id} activity={a} compact />)
              : <p style={{ color: 'var(--text-muted)', fontSize: 13, padding: '12px 0' }}>No activities yet. Capture your first one!</p>
            }
          </div>
        </div>
      </div>
    </section>
  )
}
