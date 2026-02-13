---
id: architecture
title: Architecture
sidebar_label: Architecture
sidebar_position: 5
---

# Architecture

Folio is structured as a layered pipeline: **Collection → Storage → Enrichment → Synthesis → Output**. Each layer is modular and independently useful.

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         DATA COLLECTION                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │   Git    │ │  GitHub  │ │  Linear  │ │  Screen  │ │  Manual  │  │
│  │  (local) │ │   API    │ │   API    │ │ Capture  │ │  Entry   │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘  │
│       └────────────┴────────────┴────────────┴────────────┘        │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      ACTIVITY STORE (SQLite)                         │
│  Raw events with source, timestamp, metadata, employer context      │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     ENRICHMENT LAYER                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐      │
│  │ Auto-clustering │  │ Impact Prompts  │  │  LLM Tagging    │      │
│  │ (related work)  │  │ ("Add metrics?")│  │ (categories)    │      │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘      │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    ACCOMPLISHMENT STORE (SQLite)                     │
│  Enriched, clustered accomplishments with impact/context            │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     SYNTHESIS ENGINE                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐      │
│  │ Resume Bullet   │  │  STAR Story     │  │ Review Summary  │      │
│  │   Generator     │  │   Generator     │  │   Generator     │      │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘      │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        INTERFACES                                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐               │
│  │   CLI    │ │   TUI    │ │  Web UI  │ │   MCP    │               │
│  │ (quick)  │ │ (review) │ │(browser) │ │ (Claude) │               │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘               │
└─────────────────────────────────────────────────────────────────────┘
```

## Data Models

### Activity (Raw Events)

Activities are raw events captured from various sources:

```rust
pub struct Activity {
    pub id: String,                    // UUID
    pub source: ActivitySource,        // git, github, linear, manual, etc.
    pub activity_type: ActivityType,   // commit, pr_merged, manual_entry, etc.
    pub timestamp: DateTime<Utc>,
    pub title: String,
    pub description: Option<String>,
    pub project: Option<String>,
    pub employer: Option<String>,
    pub importance: Importance,        // low, medium, high
    pub metadata: Option<String>,      // JSON for source-specific data
}
```

### Activity Sources

| Source | Description |
|--------|-------------|
| `git` | Local git repository activity |
| `github` | GitHub API (PRs, issues, reviews) |
| `linear` | Linear API (issues, projects) |
| `jira` | Jira tickets |
| `screen_capture` | Screenshot with OCR |
| `active_window` | Active application tracking |
| `calendar` | Calendar events (ICS) |
| `transcript` | Meeting transcripts (VTT, SRT) |
| `voice_note` | Audio recordings |
| `meeting` | Meeting summaries (Otter, Loom, etc.) |
| `browser` | Browser history |
| `manual` | Manual entry via CLI or web UI |

### Activity Types

| Type | Description |
|------|-------------|
| `commit` | Git commit |
| `pr_merged` | Pull request merged |
| `pr_reviewed` | Code review completed |
| `issue_closed` | Issue or ticket closed |
| `deploy` | Production deployment |
| `incident` | Incident response |
| `manual_entry` | Manual capture |
| `window_session` | Active application time |
| `browser_session` | Website/documentation research |
| `calendar_event` | Meeting or event |
| `transcript_segment` | Meeting transcript segment |
| `voice_note` | Audio note |
| `meeting_summary` | Meeting summary |
| `action_item` | Task extracted from meetings |
| `work_session` | Grouped activity session |

### Accomplishment (Enriched)

Accomplishments are enriched, packaged versions of activities:

```rust
pub struct Accomplishment {
    pub id: String,
    pub title: String,

    // STAR framework fields
    pub situation: Option<String>,
    pub task: Option<String>,
    pub action: Option<String>,
    pub result: Option<String>,

    // Structured impact
    pub metrics: Vec<Metric>,

    // Linking
    pub activity_ids: Vec<String>,

    // Categorization
    pub skills: Vec<String>,
    pub themes: Vec<String>,

    // Job context
    pub employer: String,
    pub role: Option<String>,
    pub timeframe_start: DateTime<Utc>,
    pub timeframe_end: DateTime<Utc>,

    // Cached outputs
    pub generated_bullets: Option<Vec<String>>,
    pub generated_story: Option<String>,
}
```

## Storage

### SQLite Database

All data is stored in a single SQLite database at `~/.folio/folio.db`:

| Table | Purpose |
|-------|---------|
| `activities` | Raw events from all sources |
| `accomplishments` | Enriched accomplishments |
| `activity_accomplishment_links` | Many-to-many relationships |
| `employment` | Job history |

### File Storage

Additional files at `~/.folio/`:
- `captures/` - Screenshots and attachments
- `exports/` - Generated reports

### Portability

Your career history is:
- A single SQLite file + captures directory
- Easily backed up or moved between machines
- No cloud dependency
- Can be encrypted at rest

## Data Collection Tiers

### Tier 1: Zero Config (Local Git)

Works everywhere without setup:
- Scans local git repositories
- Captures commits, branches, merges
- Deduplicates by commit hash

### Tier 2: API Integrations

Requires one-time API key setup:
- **GitHub** - PRs authored, reviews, issues
- **Linear** - Issues completed, projects
- **GitLab** - Same as GitHub

### Tier 3: Link Enrichment

Paste a URL, auto-fetch context:
- GitHub PR URL → title, files changed, reviews
- Jira/Linear URL → issue details

### Tier 4: Screen Capture

For data without API access:
- Keyboard shortcut triggers capture
- OCR extracts text
- User annotates and confirms

### Tier 5: Smart Nudges

Background detection prompts capture:
- Git watcher detects significant merges
- Weekly review of uncaptured activity

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust 2021 |
| CLI Framework | clap 4 |
| Database | SQLite (rusqlite) |
| TUI | ratatui + crossterm |
| Web Server | axum + tower-http |
| Web UI | Vanilla HTML/CSS/JS (embedded via rust-embed) |
| Git Integration | git2 |
| HTTP Client | reqwest |
| Serialization | serde + serde_json |
| Date/Time | chrono |
| Output | colored, tabled |

### Web UI Architecture

The web dashboard is a self-contained SPA embedded into the binary at compile time using `rust-embed`. This means:

- **No separate build step** — the HTML/CSS/JS files in `web-ui/` are compiled into the Rust binary
- **No runtime dependencies** — no Node.js, npm, or separate server process
- **Single binary distribution** — `folio serve` serves both the API and the dashboard
- **SPA routing** — unmatched routes fall back to `index.html` for client-side navigation

The frontend communicates with the backend exclusively through the REST API (`/api/*` endpoints). The same API is available for other clients (scripts, integrations, MCP).

## Development Roadmap

### Phase 1: Core Capture
- [x] CLI with manual capture
- [x] SQLite storage
- [x] Local git scanning
- [x] Basic bullet generation

### Phase 2: Smart Enrichment
- [x] GitHub API integration
- [x] Link enrichment
- [x] LLM-powered generation
- [x] Weekly review/digest command

### Phase 3: Automation
- [x] Background git watcher
- [x] Screen capture
- [x] Activity tracking (active window, browser, idle detection)
- [x] TUI for review

### Phase 4: Synthesis
- [x] STAR story generator
- [x] Review summary generator
- [x] MCP server for Claude
- [x] Export formats (markdown, JSON, YAML)

### Phase 5: Polish (Current)
- [x] Web UI dashboard
- [x] Job description matching
- [ ] Multi-device sync
- [ ] Interview prep mode
