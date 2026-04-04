const CHANGELOG = [
  {
    date: '2026-02-14',
    title: 'Weekly and Monthly Digests',
    category: 'New Feature',
    description: 'You can now generate weekly or monthly summaries of your career activities. Great for preparing for 1-on-1s or performance reviews.',
  },
  {
    date: '2026-02-10',
    title: 'Search Your Activities',
    category: 'New Feature',
    description: 'Quickly find any past activity using the new search page. Search by title, description, or project name to locate exactly what you need.',
  },
  {
    date: '2026-02-05',
    title: 'Edit and Delete Activities',
    category: 'Improvement',
    description: 'You can now edit activity details or remove entries you no longer need, directly from the activity detail view.',
  },
  {
    date: '2026-01-28',
    title: 'Activity Importance Levels',
    category: 'New Feature',
    description: 'Tag your activities as High, Medium, or Low importance so you can focus on the accomplishments that matter most.',
  },
  {
    date: '2026-01-20',
    title: 'Dashboard Overview',
    category: 'New Feature',
    description: 'A new dashboard gives you a snapshot of your total activities, broken down by importance and source, plus your most recent entries.',
  },
  {
    date: '2026-01-15',
    title: 'Filter Activities by Project',
    category: 'Improvement',
    description: 'The activities page now lets you filter by project and importance level, making it easier to review work for a specific team or initiative.',
  },
  {
    date: '2026-01-08',
    title: 'GitHub Integration',
    category: 'New Feature',
    description: 'Folio can now automatically pull in your contributions from GitHub, including pull requests and code reviews, so you never lose track of your work.',
  },
  {
    date: '2025-12-18',
    title: 'Web Interface Launched',
    category: 'New Feature',
    description: 'Access Folio from your browser with a brand-new web interface. View your dashboard, browse activities, and capture new ones without leaving your browser.',
  },
  {
    date: '2025-12-10',
    title: 'Export Your Data',
    category: 'New Feature',
    description: 'Export your activities as Markdown or JSON. Useful for sharing accomplishments in documents, emails, or performance review tools.',
  },
  {
    date: '2025-12-01',
    title: 'Folio Launch',
    category: 'Announcement',
    description: 'Folio is here! A local-first career tracker that helps you capture and organize your professional accomplishments as they happen.',
  },
]

function formatChangelogDate(dateStr: string): string {
  return new Date(dateStr + 'T00:00:00').toLocaleDateString('en-US', {
    month: 'long',
    day: 'numeric',
    year: 'numeric',
  })
}

export function WhatsNewPage() {
  return (
    <section className="view active">
      <header className="view-header">
        <h2>What's New</h2>
        <p className="subtitle">Latest features and improvements in Folio</p>
      </header>

      <div className="whatsnew-list">
        {CHANGELOG.map((entry) => {
          const categoryClass = entry.category.toLowerCase().replace(/\s+/g, '-')
          return (
            <div className="whatsnew-entry" key={entry.date + entry.title}>
              <div className="whatsnew-date">{formatChangelogDate(entry.date)}</div>
              <div className="whatsnew-body">
                <div className="whatsnew-heading">
                  <span className={`whatsnew-category ${categoryClass}`}>{entry.category}</span>
                  <h3 className="whatsnew-title">{entry.title}</h3>
                </div>
                <p className="whatsnew-description">{entry.description}</p>
              </div>
            </div>
          )
        })}
      </div>
    </section>
  )
}
