import { Link, useRouterState } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { api } from '../lib/api'

const NAV_ITEMS = [
  {
    to: '/' as const,
    label: 'Dashboard',
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <rect x="3" y="3" width="7" height="7" rx="1" />
        <rect x="14" y="3" width="7" height="7" rx="1" />
        <rect x="3" y="14" width="7" height="7" rx="1" />
        <rect x="14" y="14" width="7" height="7" rx="1" />
      </svg>
    ),
  },
  {
    to: '/activities' as const,
    label: 'Activities',
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
      </svg>
    ),
  },
  {
    to: '/search' as const,
    label: 'Search',
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
    ),
  },
  {
    to: '/capture' as const,
    label: 'Capture',
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="16" />
        <line x1="8" y1="12" x2="16" y2="12" />
      </svg>
    ),
  },
  {
    to: '/whatsnew' as const,
    label: "What's New",
    icon: (
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M12 20h9" />
        <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
      </svg>
    ),
  },
]

export function Sidebar() {
  const routerState = useRouterState()
  const currentPath = routerState.location.pathname
  const [connected, setConnected] = useState(false)

  useEffect(() => {
    const check = () => {
      api.health().then(() => setConnected(true)).catch(() => setConnected(false))
    }
    check()
    const interval = setInterval(check, 30000)
    return () => clearInterval(interval)
  }, [])

  return (
    <nav id="sidebar">
      <div className="sidebar-header">
        <h1 className="logo">folio</h1>
        <span className="logo-tagline">career tracker</span>
      </div>
      <ul className="nav-links">
        {NAV_ITEMS.map((item) => (
          <li key={item.to}>
            <Link
              to={item.to}
              className={`nav-link ${currentPath === item.to ? 'active' : ''}`}
            >
              {item.icon}
              {item.label}
            </Link>
          </li>
        ))}
      </ul>
      <div className="sidebar-footer">
        <div className={`status-dot ${connected ? 'connected' : 'disconnected'}`} />
        <span>{connected ? 'API Connected' : 'Disconnected'}</span>
      </div>
    </nav>
  )
}
