import { useEffect, useState, useCallback } from 'react'
import { api } from '../lib/api'
import type { Activity } from '../lib/api'
import { subscribeToUpdates } from '../lib/sse'
import { ActivityItem } from '../components/ActivityItem'

export function ActivitiesPage() {
  const [activities, setActivities] = useState<Activity[]>([])
  const [importance, setImportance] = useState('')
  const [project, setProject] = useState('')
  const [projects, setProjects] = useState<string[]>([])

  const load = useCallback(() => {
    api.listActivities({
      limit: 100,
      importance: importance || undefined,
      project: project || undefined,
    }).then((data) => {
      setActivities(data)
      const uniqueProjects = [...new Set(data.map((a) => a.project).filter(Boolean))] as string[]
      setProjects(uniqueProjects)
    }).catch(console.error)
  }, [importance, project])

  useEffect(() => {
    load()
    return subscribeToUpdates(load)
  }, [load])

  return (
    <section className="view active">
      <header className="view-header">
        <h2>Activities</h2>
        <div className="view-controls">
          <select className="select-input" value={importance} onChange={(e) => setImportance(e.target.value)}>
            <option value="">All Importance</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
          </select>
          <select className="select-input" value={project} onChange={(e) => setProject(e.target.value)}>
            <option value="">All Projects</option>
            {projects.map((p) => <option key={p} value={p}>{p}</option>)}
          </select>
          <button className="btn btn-secondary" title="Refresh" onClick={load}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="23 4 23 10 17 10" />
              <polyline points="1 20 1 14 7 14" />
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
            </svg>
          </button>
        </div>
      </header>

      {activities.length > 0 ? (
        <div className="activity-list">
          {activities.map((a) => <ActivityItem key={a.id} activity={a} />)}
        </div>
      ) : (
        <div className="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" opacity="0.4">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
          <p>No activities found</p>
          <p className="muted">Capture your first activity to get started</p>
        </div>
      )}
    </section>
  )
}
