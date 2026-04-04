import { createRoute } from "@tanstack/react-router";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState, useCallback } from "react";
import { rootRoute } from "./__root";
import { activitiesQuery, activityQuery } from "../api/queries";
import { ActivityItem } from "../components/ActivityItem";
import { Modal } from "../components/Modal";
import { api } from "../api/client";
import type { Activity } from "../api/types";
import { formatDate, sourceLabel } from "../components/helpers";

export const activitiesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/activities",
  component: ActivitiesPage,
});

function ActivitiesPage() {
  const [importance, setImportance] = useState("");
  const [project, setProject] = useState("");
  const [sourceFilter, setSourceFilter] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [editing, setEditing] = useState(false);

  const queryClient = useQueryClient();
  const { data: activities = [], refetch } = useQuery(
    activitiesQuery({
      limit: 200,
      importance: importance || undefined,
      project: project || undefined,
    })
  );

  const filtered = sourceFilter
    ? activities.filter((a) => a.source === sourceFilter)
    : activities;

  const sources = [...new Set(activities.map((a) => a.source).filter(Boolean))].sort();
  const projects = [...new Set(activities.map((a) => a.project).filter(Boolean))].sort();

  const { data: detail } = useQuery({
    ...activityQuery(selectedId!),
    enabled: !!selectedId,
  });

  const handleDelete = useCallback(
    async (id: string) => {
      if (!confirm("Delete this activity? This cannot be undone.")) return;
      await api.deleteActivity(id);
      setSelectedId(null);
      queryClient.invalidateQueries({ queryKey: ["activities"] });
      queryClient.invalidateQueries({ queryKey: ["stats"] });
    },
    [queryClient]
  );

  return (
    <section className="view active">
      <header className="view-header">
        <h2>Activities</h2>
        <div className="view-controls">
          <select className="select-input" value={sourceFilter} onChange={(e) => setSourceFilter(e.target.value)}>
            <option value="">All Sources</option>
            {sources.map((s) => (
              <option key={s} value={s}>{sourceLabel(s)}</option>
            ))}
          </select>
          <select className="select-input" value={importance} onChange={(e) => setImportance(e.target.value)}>
            <option value="">All Importance</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
          </select>
          <select className="select-input" value={project} onChange={(e) => setProject(e.target.value)}>
            <option value="">All Projects</option>
            {projects.map((p) => (
              <option key={p} value={p}>{p}</option>
            ))}
          </select>
          <button className="btn btn-secondary" title="Refresh" onClick={() => refetch()}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="23 4 23 10 17 10" />
              <polyline points="1 20 1 14 7 14" />
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
            </svg>
          </button>
        </div>
      </header>

      {filtered.length === 0 ? (
        <div className="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" opacity="0.4">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
          <p>No activities found</p>
          <p className="muted">Capture your first activity to get started</p>
        </div>
      ) : (
        <div className="activity-list">
          {filtered.map((a) => (
            <ActivityItem key={a.id} activity={a} onClick={setSelectedId} />
          ))}
        </div>
      )}

      <Modal open={!!selectedId} onClose={() => { setSelectedId(null); setEditing(false); }}>
        {detail && !editing && (
          <ActivityDetail
            activity={detail}
            onEdit={() => setEditing(true)}
            onDelete={() => handleDelete(detail.id)}
            onClose={() => setSelectedId(null)}
          />
        )}
        {detail && editing && (
          <EditForm
            activity={detail}
            onCancel={() => setEditing(false)}
            onSaved={() => {
              setEditing(false);
              queryClient.invalidateQueries({ queryKey: ["activity", detail.id] });
              queryClient.invalidateQueries({ queryKey: ["activities"] });
            }}
          />
        )}
      </Modal>
    </section>
  );
}

function ActivityDetail({
  activity,
  onEdit,
  onDelete,
  onClose,
}: {
  activity: Activity;
  onEdit: () => void;
  onDelete: () => void;
  onClose: () => void;
}) {
  const impact = activity.metadata?.impact as string | undefined;

  return (
    <>
      <div className="detail-header">
        <div className="detail-meta" style={{ marginBottom: 10 }}>
          <span className={`importance-badge ${activity.importance}`}>{activity.importance}</span>
          <span className="detail-tag">{sourceLabel(activity.source)}</span>
          {activity.project && <span className="detail-tag">{activity.project}</span>}
        </div>
        <h3>{activity.title}</h3>
        <div style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 4 }}>
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
        <div className="detail-field-value" style={{ fontFamily: "monospace", fontSize: 12, color: "var(--text-muted)" }}>
          {activity.id}
        </div>
      </div>
      <div className="detail-actions-section">
        <div className="detail-actions">
          <button className="btn btn-secondary" onClick={onEdit}>Edit</button>
          <button className="btn btn-secondary" onClick={onClose}>Close</button>
          <button className="btn btn-secondary" style={{ color: "var(--high)" }} onClick={onDelete}>Delete</button>
        </div>
      </div>
    </>
  );
}

function EditForm({
  activity,
  onCancel,
  onSaved,
}: {
  activity: Activity;
  onCancel: () => void;
  onSaved: () => void;
}) {
  const impact = (activity.metadata?.impact as string) || "";
  const [title, setTitle] = useState(activity.title);
  const [impactVal, setImpactVal] = useState(impact);
  const [projectVal, setProjectVal] = useState(activity.project || "");
  const [employerVal, setEmployerVal] = useState(activity.employer || "");
  const [importanceVal, setImportanceVal] = useState(activity.importance);
  const [error, setError] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await api.updateActivity(activity.id, {
        title: title.trim() || undefined,
        impact: impactVal.trim() || undefined,
        project: projectVal.trim() || undefined,
        employer: employerVal.trim() || undefined,
        importance: importanceVal,
      });
      onSaved();
    } catch {
      setError(true);
    }
  };

  return (
    <>
      <div className="detail-header"><h3>Edit Activity</h3></div>
      <form className="edit-form" onSubmit={handleSubmit}>
        <div className="form-group">
          <label>Title</label>
          <input type="text" value={title} onChange={(e) => setTitle(e.target.value)} required />
        </div>
        <div className="form-group">
          <label>Impact <span className="optional">(optional)</span></label>
          <input type="text" value={impactVal} onChange={(e) => setImpactVal(e.target.value)} />
        </div>
        <div className="form-row">
          <div className="form-group">
            <label>Project <span className="optional">(optional)</span></label>
            <input type="text" value={projectVal} onChange={(e) => setProjectVal(e.target.value)} />
          </div>
          <div className="form-group">
            <label>Importance</label>
            <select className="select-input" value={importanceVal} onChange={(e) => setImportanceVal(e.target.value as "high" | "medium" | "low")}>
              <option value="high">High</option>
              <option value="medium">Medium</option>
              <option value="low">Low</option>
            </select>
          </div>
        </div>
        <div className="form-group">
          <label>Employer <span className="optional">(optional)</span></label>
          <input type="text" value={employerVal} onChange={(e) => setEmployerVal(e.target.value)} />
        </div>
        <div className="form-actions">
          <button type="submit" className="btn btn-primary">Save Changes</button>
          <button type="button" className="btn btn-secondary" onClick={onCancel}>Cancel</button>
        </div>
        {error && <div className="error-message">Failed to save changes. Please try again.</div>}
      </form>
    </>
  );
}
