# Folio

> Your career changelog. Capture, enrich, and package your professional accomplishments — own your narrative across your entire career.

## What is Folio?

Folio is a local-first career accomplishment tracker that helps developers capture what they've done, enrich it with impact and context, and package it into interview-ready stories, resume bullets, and performance review summaries.

**The problem:** You do impactful work every day but forget the specifics by review/interview time. Even when you remember, translating "I fixed the thing" into "Reduced API latency by 40% through query optimization, impacting 2M daily requests" is hard.

**The solution:** A prompted work journal that enriches itself when possible — capturing accomplishments while they're fresh and packaging them when you need them.

## How It's Different

| Tool | What it does | Gap |
|------|-------------|-----|
| **Glean** | Enterprise search/knowledge management | Company-owned, finds info but doesn't synthesize YOUR narrative |
| **Notion/Brag docs** | Manual accomplishment logs | Requires discipline; you have to remember to write |
| **Lattice/15Five** | Performance review tools | Company-owned, review-cycle focused |
| **RescueTime** | Time tracking | Tracks time, not impact or accomplishments |
| **LinkedIn** | Static professional profile | Manual, not activity-driven |

**Folio's positioning:** Glean helps you find what your company knows. Folio helps you articulate what YOU'VE done — and own that narrative across your entire career.

## Key Principles

- **Individual-owned, portable across jobs** — Your career history, not your employer's
- **Local-first** — Your career narrative isn't sitting on someone else's servers
- **Capture → Synthesis** — Not just search, but packaged output (resume bullets, STAR stories)
- **Developer-depth** — Git, PRs, deploys, incidents at a level general tools don't reach
- **Prompted, not passive** — Capture context while it's fresh through smart nudges

## Architecture Overview

See [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) for the full technical design, including data models, collection strategies, enrichment workflows, and synthesis engine details.

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  Data Collection │ ──▶ │ Enrichment Layer │ ──▶ │ Synthesis Engine │
│  (Git, Manual,   │     │ (Clustering,     │     │ (Resume bullets, │
│   Link Enrichment│     │  Impact prompts, │     │  STAR stories,   │
│   Screen Capture)│     │  LLM tagging)    │     │  Review summaries)│
└─────────────────┘     └──────────────────┘     └─────────────────┘
        │                        │                        │
        ▼                        ▼                        ▼
   ┌──────────┐          ┌──────────────┐         ┌────────────┐
   │ Activity │          │Accomplishment│         │  Interfaces │
   │  Store   │          │    Store     │         │ CLI/TUI/MCP │
   │ (SQLite) │          │   (SQLite)   │         └────────────┘
   └──────────┘          └──────────────┘
```

## Data Collection Strategy

Folio takes a pragmatic, tiered approach to data collection:

### Tier 1: Zero Config (Works Everywhere)
- **Local git repos** — Commits, branches, diffs
- **Manual entry** — The fallback that always works

### Tier 2: Personal API Keys (5-10 min setup)
- **GitHub PAT** — PRs, issues, code review comments
- **Linear API key** — Issues you've worked on
- **GitLab PAT** — Same as GitHub

### Tier 3: Link Enrichment (Paste & Enrich)
- Paste a GitHub PR URL → auto-fetch title, description, files changed
- Paste a Jira/Linear URL → auto-fetch issue details (if accessible)

### Tier 4: Triggered Screen Capture (Optional)
- Keyboard shortcut → screenshot + OCR + annotation prompt
- Captures anything on screen (Jira ticket, dashboard metrics, Slack message)
- You control exactly what gets captured

### Tier 5: Background Detection (Smart Nudges)
- Git watcher detects significant merges → prompts you to capture
- Weekly review of git activity — "Anything worth capturing?"

## Planned Output Formats

- **Resume bullets** — Quantified, action-oriented accomplishment statements
- **STAR stories** — Situation, Task, Action, Result formatted for interviews
- **Performance review summaries** — Grouped by theme/quarter
- **Career timeline** — Visual overview of your professional journey
- **JSON/Markdown export** — For use in other tools

## Tech Stack

- **Rust** — Fast, reliable CLI with rich terminal output
- **SQLite** — Local-first data storage
- **Ratatui** — Interactive TUI for browsing activities
- **Axum** — REST API and MCP server
- **git2** — Native Git integration

## Installation

```bash
# Build from source
cargo build --release

# Install globally
cargo install --path .
```

## Quick Start

```bash
# Initialize configuration
folio config --init

# Capture your first accomplishment
folio capture "Implemented user authentication" \
  --impact "Reduced login time by 50%" \
  --project "backend-api" \
  --importance high

# Sync from git repositories
folio sync --source git --days 30

# View your activities
folio list

# Generate a weekly digest
folio digest weekly

# Export as brag document
folio export --brag --output brag-doc.md

# Launch interactive TUI
folio tui
```

## Commands

| Command | Description |
|---------|-------------|
| `capture` | Capture a new accomplishment |
| `list` | List recent activities |
| `show` | Show activity details |
| `search` | Search activities with fuzzy matching |
| `edit` | Edit an existing activity |
| `delete` | Delete an activity |
| `promote` | Convert activity to STAR-formatted accomplishment |
| `sync` | Sync from Git, GitHub, or Linear |
| `export` | Export as markdown, JSON, YAML, or resume bullets |
| `import` | Import activities from file |
| `digest` | Generate daily/weekly/monthly digest |
| `review` | Generate performance review summary |
| `match` | Match experience against job description |
| `tui` | Launch interactive terminal UI |
| `serve` | Start REST API or MCP server |
| `watch` | Background git monitoring daemon |
| `config` | Manage configuration |
| `stats` | Show activity statistics |

## Documentation

- [Full Features Guide](./docs/FEATURES.md) — Detailed documentation with examples
- [Architecture](./ARCHITECTURE.md) — Technical design and data models

## Status

✅ **Implemented Features:**
- Core CLI with capture, list, show, delete, stats
- Search with fuzzy matching
- Edit and promote commands
- Git repository scanning
- GitHub and Linear integrations
- STAR story builder
- Resume bullet generator
- Auto-tagging (skills, themes)
- Weekly/monthly digests
- Performance review generator
- Job description matching
- Export (Markdown, JSON, YAML)
- Interactive TUI
- REST API server
- MCP server for AI integration
- Background git watcher
- Configuration management

## License

MIT
