import { useState } from 'react'
import type { Activity, UpdateActivityRequest } from '../lib/api'
import { api } from '../lib/api'
import { formatDate, sourceLabel } from '../lib/utils'

interface Props {
  activity: Activity
  onClose: () => void
  onUpdated: () => void
}

export function ActivityModal({ activity, onClose, onUpdated }: Props) {
  const [editing, setEditing] = useState(false)

  if (editing) {
    return (
      <EditForm
        activity={activity}
        onCancel={() => setEditing(false)}
        onSaved={() => {
          setEditing(false)
          onUpdated()
        }}
      />
    )
  }

  const impact = activity.metadata?.impact

  return (
    <>
      <div className="detail-header">
        <div className="detail-meta" style={{ marginBottom: 10 }}>
          <span className={`importance-badge ${activity.importance}`}>{activity.importance}</span>
          <span className="detail-tag">{sourceLabel(activity.source)}</span>
          {activity.project && <span className="detail-tag">{activity.project}</span>}
        </div>
        <h3>{activity.title}</h3>
        <div style={{ fontSize: 12, color: 'var(--text-muted)', marginTop: 4 }}>
          {formatDate(activity.timestamp)}
        </div>
      </div>

      {activity.description && (
        <div className="detail-field">
          <div className="detail-field-label">Description</div>
          <div className="detail-field-value">{activity.description}</div>
        </div>
      )}

      {impact && (
        <div className="detail-field">
          <div className="detail-field-label">Impact</div>
          <div className="detail-field-value">{impact}</div>
        </div>
      )}

      {activity.employer && (
        <div className="detail-field">
          <div className="detail-field-label">Employer</div>
          <div className="detail-field-value">{activity.employer}</div>
        </div>
      )}

      <div className="detail-field">
        <div className="detail-field-label">Activity ID</div>
        <div className="detail-field-value" style={{ fontFamily: 'monospace', fontSize: 12, color: 'var(--text-muted)' }}>
          {activity.id}
        </div>
      </div>

      <div className="detail-actions">
        <button className="btn btn-primary" onClick={() => setEditing(true)}>Edit</button>
        <button className="btn btn-secondary" onClick={onClose}>Close</button>
        <button
          className="btn btn-secondary"
          style={{ color: 'var(--high)' }}
          onClick={async () => {
            if (!confirm('Delete this activity? This cannot be undone.')) return
            await api.deleteActivity(activity.id)
            onUpdated()
            onClose()
          }}
        >
          Delete
        </button>
      </div>
    </>
  )
}

function EditForm({ activity, onCancel, onSaved }: { activity: Activity; onCancel: () => void; onSaved: () => void }) {
  const [title, setTitle] = useState(activity.title)
  const [impact, setImpact] = useState(activity.metadata?.impact ?? '')
  const [project, setProject] = useState(activity.project ?? '')
  const [employer, setEmployer] = useState(activity.employer ?? '')
  const [importance, setImportance] = useState(activity.importance)
  const [error, setError] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      const data: UpdateActivityRequest = {
        title: title.trim() || undefined,
        impact: impact.trim() || undefined,
        project: project.trim() || undefined,
        employer: employer.trim() || undefined,
        importance,
      }
      await api.updateActivity(activity.id, data)
      onSaved()
    } catch {
      setError(true)
    }
  }

  return (
    <>
      <div className="detail-header">
        <h3>Edit Activity</h3>
      </div>
      <form className="edit-form" onSubmit={handleSubmit}>
        <div className="form-group">
          <label htmlFor="edit-title">Title</label>
          <input id="edit-title" type="text" value={title} onChange={(e) => setTitle(e.target.value)} required />
        </div>
        <div className="form-group">
          <label htmlFor="edit-impact">Impact <span className="optional">(optional)</span></label>
          <input id="edit-impact" type="text" value={impact} onChange={(e) => setImpact(e.target.value)} />
        </div>
        <div className="form-row">
          <div className="form-group">
            <label htmlFor="edit-project">Project <span className="optional">(optional)</span></label>
            <input id="edit-project" type="text" value={project} onChange={(e) => setProject(e.target.value)} />
          </div>
          <div className="form-group">
            <label htmlFor="edit-importance">Importance</label>
            <select id="edit-importance" className="select-input" value={importance} onChange={(e) => setImportance(e.target.value as 'high' | 'medium' | 'low')}>
              <option value="high">High</option>
              <option value="medium">Medium</option>
              <option value="low">Low</option>
            </select>
          </div>
        </div>
        <div className="form-group">
          <label htmlFor="edit-employer">Employer <span className="optional">(optional)</span></label>
          <input id="edit-employer" type="text" value={employer} onChange={(e) => setEmployer(e.target.value)} />
        </div>
        <div className="form-actions">
          <button type="submit" className="btn btn-primary">Save Changes</button>
          <button type="button" className="btn btn-secondary" onClick={onCancel}>Cancel</button>
        </div>
        {error && <div className="error-message">Failed to save changes. Please try again.</div>}
      </form>
    </>
  )
}
