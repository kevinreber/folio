import { Link } from '@tanstack/react-router'
import type { Activity } from '../lib/api'
import { sourceIcon, sourceLabel, timeAgo } from '../lib/utils'

export function ActivityItem({ activity, compact }: { activity: Activity; compact?: boolean }) {
  const source = activity.source || 'manual'
  const importance = activity.importance || 'medium'

  return (
    <Link
      to="/activities/$id"
      params={{ id: activity.id }}
      className={`activity-item ${compact ? 'compact' : ''}`}
    >
      <div className={`activity-source-icon ${source}`}>{sourceIcon(source)}</div>
      <div className="activity-info">
        <div className="activity-title">{activity.title}</div>
        <div className="activity-meta">
          <span>{sourceLabel(source)}</span>
          <span>{timeAgo(activity.timestamp)}</span>
          {activity.project && <span>{activity.project}</span>}
        </div>
      </div>
      <span className={`importance-badge ${importance}`}>{importance}</span>
    </Link>
  )
}
