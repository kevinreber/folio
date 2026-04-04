import { createRoute } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import { useState, useDeferredValue } from "react";
import { rootRoute } from "./__root";
import { searchQuery } from "../api/queries";
import { ActivityItem } from "../components/ActivityItem";

export const searchRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/search",
  component: SearchPage,
});

function SearchPage() {
  const [query, setQuery] = useState("");
  const deferredQuery = useDeferredValue(query);

  const { data: results = [], isFetching } = useQuery(searchQuery(deferredQuery));

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
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          autoFocus
        />
      </div>

      {deferredQuery && results.length > 0 && (
        <div className="activity-list">
          {results.map((a) => (
            <ActivityItem key={a.id} activity={a} />
          ))}
        </div>
      )}

      {deferredQuery && results.length === 0 && !isFetching && (
        <div className="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" opacity="0.4">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <p>No results for &ldquo;{deferredQuery}&rdquo;</p>
          <p className="muted">Try a different search term</p>
        </div>
      )}

      {!deferredQuery && (
        <div className="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" opacity="0.4">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <p>Search your activities</p>
          <p className="muted">Find activities by keywords in title, description, or project name</p>
        </div>
      )}
    </section>
  );
}
