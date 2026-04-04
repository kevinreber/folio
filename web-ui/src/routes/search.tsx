import { useState, useCallback, useRef } from 'react'
import { api } from '../lib/api'
import type { Activity } from '../lib/api'
import { ActivityItem } from '../components/ActivityItem'

export function SearchPage() {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<Activity[]>([])
  const [searched, setSearched] = useState(false)
  const timeoutRef = useRef<ReturnType<typeof setTimeout>>(undefined)

  const doSearch = useCallback((q: string) => {
    if (!q.trim()) {
      setResults([])
      setSearched(false)
      return
    }
    clearTimeout(timeoutRef.current)
    timeoutRef.current = setTimeout(async () => {
      try {
        const data = await api.searchActivities(q)
        setResults(data)
        setSearched(true)
      } catch (err) {
        console.error('Search failed:', err)
      }
    }, 300)
  }, [])

  return (
    <section className="view active">
      <header className="view-header">
        <h2>Search</h2>
      </header>

      <div className="search-bar">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input
          type="text"
          placeholder="Search activities by title, description, or project..."
          autoFocus
          value={query}
          onChange={(e) => {
            setQuery(e.target.value)
            doSearch(e.target.value)
          }}
        />
      </div>

      {results.length > 0 ? (
        <div className="activity-list">
          {results.map((a) => <ActivityItem key={a.id} activity={a} />)}
        </div>
      ) : (
        <div className="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" opacity="0.4">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          {searched ? (
            <>
              <p>No results for "{query}"</p>
              <p className="muted">Try a different search term</p>
            </>
          ) : (
            <>
              <p>Search your activities</p>
              <p className="muted">Find activities by keywords in title, description, or project name</p>
            </>
          )}
        </div>
      )}
    </section>
  )
}
