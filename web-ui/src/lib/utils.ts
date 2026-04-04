export function timeAgo(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = Math.floor((now.getTime() - date.getTime()) / 1000)

  if (diff < 60) return 'just now'
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
  if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
}

export function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  })
}

const SOURCE_ICONS: Record<string, string> = {
  git: 'G',
  github: 'GH',
  linear: 'LN',
  jira: 'JR',
  manual: 'M',
  screen_capture: 'SC',
  active_window: 'AW',
  calendar: 'CA',
  transcript: 'TR',
  voice_note: 'VN',
  meeting: 'MT',
  browser: 'BR',
}

export function sourceIcon(source: string): string {
  return SOURCE_ICONS[source] ?? source.charAt(0).toUpperCase()
}

export function sourceLabel(source: string): string {
  return source.replace(/_/g, ' ')
}
