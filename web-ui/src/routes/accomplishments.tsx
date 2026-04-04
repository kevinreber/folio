import { createRoute } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { rootRoute } from "./__root";
import { accomplishmentsQuery } from "../api/queries";
import type { Accomplishment } from "../api/types";

export const accomplishmentsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/accomplishments",
  component: AccomplishmentsPage,
});

function AccomplishmentsPage() {
  const { data: accomplishments = [] } = useQuery(accomplishmentsQuery);

  return (
    <section className="view active">
      <header className="view-header">
        <h2>Accomplishments</h2>
        <p className="subtitle">Promoted activities with STAR stories and resume bullets</p>
      </header>

      {accomplishments.length === 0 ? (
        <div className="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" opacity="0.4">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
            <polyline points="22 4 12 14.01 9 11.01" />
          </svg>
          <p>No accomplishments yet</p>
          <p className="muted">Promote activities from the Activities view to create polished accomplishments</p>
        </div>
      ) : (
        <div>
          {accomplishments.map((acc) => (
            <AccomplishmentCard key={acc.id} accomplishment={acc} />
          ))}
        </div>
      )}
    </section>
  );
}

function AccomplishmentCard({ accomplishment: acc }: { accomplishment: Accomplishment }) {
  return (
    <div className="accomplish-card card" style={{ marginBottom: 12 }}>
      <div className="accomplish-header">
        <h4 style={{ fontSize: 16, fontWeight: 600, marginBottom: 8 }}>{acc.title}</h4>
        {acc.employer && <span className="detail-tag">{acc.employer}</span>}
        {acc.role && <span className="detail-tag">{acc.role}</span>}
      </div>
      {acc.situation && <div className="star-field"><span className="star-label">S</span> {acc.situation}</div>}
      {acc.task && <div className="star-field"><span className="star-label">T</span> {acc.task}</div>}
      {acc.action && <div className="star-field"><span className="star-label">A</span> {acc.action}</div>}
      {acc.result && <div className="star-field"><span className="star-label">R</span> {acc.result}</div>}
      {((acc.skills?.length ?? 0) > 0 || (acc.themes?.length ?? 0) > 0) && (
        <div className="accomplish-tags" style={{ marginTop: 10 }}>
          {acc.skills?.map((s) => <span key={s} className="detail-tag">{s}</span>)}
          {acc.themes?.map((t) => <span key={t} className="detail-tag">{t}</span>)}
        </div>
      )}
      {acc.generated_bullets && acc.generated_bullets.length > 0 && (
        <div style={{ marginTop: 12, paddingTop: 12, borderTop: "1px solid var(--border)" }}>
          <div className="detail-field-label">Resume Bullets</div>
          <ul style={{ paddingLeft: 18, color: "var(--text-secondary)", fontSize: 13 }}>
            {acc.generated_bullets.map((b, i) => <li key={i}>{b}</li>)}
          </ul>
        </div>
      )}
    </div>
  );
}
