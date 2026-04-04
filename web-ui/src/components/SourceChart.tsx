import { sourceLabel } from '../lib/utils'

export function SourceChart({ bySource }: { bySource: Record<string, number> }) {
  const entries = Object.entries(bySource).sort((a, b) => b[1] - a[1])

  if (entries.length === 0) {
    return <p style={{ color: 'var(--text-muted)', fontSize: 13 }}>No data yet</p>
  }

  const max = Math.max(...entries.map(([, v]) => v))

  return (
    <div className="chart-container">
      {entries.map(([source, count]) => {
        const pct = max > 0 ? (count / max) * 100 : 0
        return (
          <div className="chart-bar" key={source}>
            <span className="chart-bar-label">{sourceLabel(source)}</span>
            <div className="chart-bar-track">
              <div className="chart-bar-fill" style={{ width: `${pct}%` }} />
            </div>
            <span className="chart-bar-value">{count}</span>
          </div>
        )
      })}
    </div>
  )
}
