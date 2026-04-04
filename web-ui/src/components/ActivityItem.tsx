import type { Activity } from "../api/types";
import { sourceIcon, sourceLabel, timeAgo } from "./helpers";

export function ActivityItem({
  activity,
  onClick,
}: {
  activity: Activity;
  onClick?: (id: string) => void;
}) {
  const source = activity.source || "manual";
  const importance = activity.importance || "medium";

  return (
    <div
      className="activity-item"
      onClick={() => onClick?.(activity.id)}
    >
      <div className={`activity-source-icon ${source}`}>
        {sourceIcon(source)}
      </div>
      <div className="activity-info">
        <div className="activity-title">{activity.title}</div>
        <div className="activity-meta">
          <span>{sourceLabel(source)}</span>
          <span>{timeAgo(activity.timestamp)}</span>
          {activity.project && <span>{activity.project}</span>}
        </div>
      </div>
      <span className={`importance-badge ${importance}`}>{importance}</span>
    </div>
  );
}
