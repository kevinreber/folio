# Folio Architecture

## Overview

Folio is structured as a layered pipeline: **Collection → Storage → Enrichment → Synthesis → Output**. Each layer is modular and independently useful, so you get value even before all layers are built.

## System Architecture

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
│  Links back to source activities                                    │
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
│  │   CLI    │ │   TUI    │ │ Web UI   │ │   MCP    │               │
│  │ (quick)  │ │ (review) │ │ (export) │ │ (Claude) │               │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘               │
└─────────────────────────────────────────────────────────────────────┘
```

## Data Model

### Activity (Raw Events)

```typescript
interface Activity {
  id: string;
  source: 'git' | 'github' | 'linear' | 'jira' | 'calendar' | 'screen_capture' | 'manual';
  timestamp: Date;
  type: 'commit' | 'pr_merged' | 'pr_reviewed' | 'issue_closed' | 'meeting' |
        'deploy' | 'incident' | 'screen_capture' | 'manual_entry';

  // Source-specific metadata
  raw: {
    // Git: commit hash, files changed, lines added/removed, branch
    // GitHub: PR number, title, description, review comments, files
    // Linear: issue key, title, story points, project
    // Screen capture: screenshot path, OCR text
    // Calendar: attendees, duration, meeting title
    [key: string]: unknown;
  };

  // Derived/computed
  project?: string;        // Auto-detected or user-tagged
  employer: string;        // Which job this belongs to
  importance?: 'low' | 'medium' | 'high';  // Auto-scored or manual
}
```

### Accomplishment (Enriched & Packaged)

```typescript
interface Accomplishment {
  id: string;
  title: string;           // "Payment retry system redesign"

  // STAR framework fields (user-provided context — the valuable stuff)
  situation?: string;      // Context/background
  task?: string;           // What you were asked to do
  action?: string;         // What you actually did
  result?: string;         // Impact, metrics, outcome

  // Structured impact data
  metrics?: {
    type: 'percentage' | 'absolute' | 'currency' | 'time';
    value: number;
    description: string;   // "reduced failed transactions by 15%"
  }[];

  // Linking
  activityIds: string[];   // Source activities that make up this accomplishment

  // Categorization
  skills: string[];        // ["TypeScript", "System Design", "Payment Systems"]
  themes: string[];        // ["reliability", "cost-savings", "performance"]

  // Job context
  employer: string;
  role: string;
  timeframe: { start: Date; end: Date };

  // Output cache
  generatedBullets?: string[];   // Cached resume bullets
  generatedStory?: string;       // Cached STAR story
}
```

### Employment (Job Context)

```typescript
interface Employment {
  id: string;
  company: string;
  role: string;
  startDate: Date;
  endDate?: Date;

  // For context in generation
  teamSize?: number;
  techStack?: string[];
  domain?: string;         // "fintech", "healthcare", etc.
}
```

## Data Collection Details

### Tier 1: Local Git (Zero Config)

The git collector watches local repositories for activity. No API keys needed.

**What it captures:**
- Commits (message, files changed, insertions/deletions, timestamp)
- Branch creation/merges
- Tags

**How it works:**
- On-demand scan: `folio scan /path/to/repo`
- Background watcher: monitors `.git` directories for changes
- Deduplication by commit hash

**Detection triggers:**
- Merge to main/master branch with significant changes (configurable threshold)
- Tag creation (likely a release)
- High-churn commits (many files changed)

### Tier 2: API Integrations (Personal API Keys)

These require a one-time API key setup. Keys are stored locally in the Folio config.

#### GitHub
- **Setup:** Personal Access Token with `repo` scope
- **Captures:** PRs authored, PRs reviewed, issues closed, release notes
- **Enrichment:** Auto-links to local git commits via SHA

#### Linear
- **Setup:** Personal API key from Linear settings
- **Captures:** Issues completed, project membership, cycle contributions

#### GitLab
- **Setup:** Personal Access Token
- **Captures:** Same as GitHub

### Tier 3: Link Enrichment

When you manually capture an accomplishment, paste a URL and Folio auto-fetches context:

```
> folio capture
  Title: Payment retry system redesign
  Link: https://github.com/company/repo/pull/4521

  [Auto-fetched from GitHub]
  PR: "Implement idempotent payment retry with exponential backoff"
  Files changed: 23 | +1,247 / -312
  Reviews: 3 approvals
  Merged: 2025-01-15
```

### Tier 4: Triggered Screen Capture

For data that lives in systems without API access (Jira Server, Slack, dashboards).

**Flow:**
1. User hits keyboard shortcut (e.g., `Cmd+Shift+F`)
2. Screenshot of current screen is captured
3. OCR extracts text from screenshot
4. Quick annotation prompt appears
5. Screenshot + OCR text + annotation stored as activity

**Privacy controls:**
- User explicitly triggers each capture
- App allowlist/blocklist (only capture from work apps)
- Review before storing
- Sensitive pattern redaction (optional)

### Tier 5: Background Detection (Smart Nudges)

The git watcher can prompt you when it detects significant work:

```
🔔 Folio: You just merged a 500-line PR to main
   "Implement idempotent payment retry with exponential backoff"
   → Capture this accomplishment? [y/n/later]
```

**Trigger conditions (configurable):**
- PR merged with > N lines changed
- Issue closed that was tagged high priority
- Deploy to production detected
- End of week with uncaptured activity

## Enrichment Workflows

### Immediate Enrichment (While Fresh)

Triggered by significant events detected in Tier 5:

```
Folio: You just merged PR #4521. Quick capture:
> What was the business impact?  "Reduced payment failures by 15%"
> Any metrics?                   "$50K/month recovered"
> What would've happened without this?  "Continued losing transactions"
```

### Batch Enrichment (Weekly Review)

```
> folio review --week

This week you had:
  12 commits to payments/
  3 PRs merged
  5 issues closed

Existing accomplishments: 1 (Payment retry system)

Gaps detected:
  - 7 commits to auth/ with no accomplishment — combine into something?
  - Issue PROJ-234 "Fix race condition in checkout" closed — worth capturing?
```

### LLM-Assisted Tagging

When accomplishments are captured, an LLM can auto-suggest:
- **Skills:** Based on files changed, tech mentioned, PR description
- **Themes:** reliability, performance, cost-savings, developer-experience, etc.
- **Importance:** Based on scope of changes, number of reviewers, etc.

## Synthesis Engine

### Resume Bullet Generator

Input: An accomplishment with context and metrics.

Output: Multiple bullet variations:

```
Strong (has metrics):
• Designed and implemented idempotent payment retry system, reducing failed
  transaction rate by 15% and recovering ~$50K/month in previously lost revenue

Medium (has context but no hard metrics):
• Redesigned payment retry logic to handle idempotency and exponential backoff,
  improving transaction reliability for the checkout flow

Weak (minimal context — flag for enrichment):
• Implemented payment retry system improvements
  ⚠️ Add metrics to strengthen this bullet
```

### STAR Story Generator

Input: An accomplishment with STAR fields populated.

Output: A structured interview-ready story:

```
SITUATION: Our payment system was losing ~$50K/month due to failed retry
logic that didn't handle idempotency properly. Customer complaints about
double-charges were increasing.

TASK: I was asked to redesign the retry mechanism to be idempotent and
reliable, while ensuring zero double-charges during the migration.

ACTION: I designed an idempotent retry system with exponential backoff,
implemented a migration strategy that ran old and new systems in parallel
for 2 weeks, and set up monitoring dashboards to track failure rates in
real-time.

RESULT: Failed transaction rate dropped 15%, recovering ~$50K/month.
Zero double-charges during migration. The pattern was adopted by 3 other
teams for their own retry logic.
```

### Performance Review Summary

Input: All accomplishments for a given time period.

Output: Grouped, themed summary:

```
Q4 2025 Summary — Acme Corp, Senior Engineer

RELIABILITY & INFRASTRUCTURE
• Payment retry redesign: 15% failure reduction, $50K/month recovered
• Circuit breaker implementation: 99.9% → 99.95% uptime

DEVELOPER EXPERIENCE
• CI pipeline optimization: 40% faster build times
• Shared testing utilities adopted by 5 teams

LEADERSHIP
• Led technical design for payments v2 architecture
• Mentored 2 junior engineers through onboarding
```

## Interfaces

### CLI (Quick Capture)

```bash
# Quick manual capture
folio capture "Shipped payment retry system" --impact "15% fewer failures"

# Scan git repos
folio scan ~/work/repos/

# Generate resume bullets
folio generate bullets --employer "Acme Corp"

# Generate STAR stories
folio generate stories --top 5

# Weekly review
folio review --week

# Export
folio export --format markdown --employer "Acme Corp"
```

### TUI (Interactive Review)

An Ink-based terminal UI for:
- Reviewing and enriching captured activities
- Promoting activities to accomplishments
- Adding STAR context interactively
- Previewing generated outputs

### MCP Server (Claude Integration)

Expose Folio data via MCP so Claude can:
- Query your accomplishments during resume writing
- Suggest improvements to bullet points
- Help prepare for specific interview questions
- Generate tailored content for specific job descriptions

### Web UI (Future)

A local web interface for:
- Visual career timeline
- Drag-and-drop accomplishment organization
- PDF/DOCX resume export
- Dashboard with stats and gaps

## Storage

### SQLite Database

Single file at `~/.folio/folio.db` containing:
- `activities` — Raw events from all sources
- `accomplishments` — Enriched, packaged accomplishments
- `employment` — Job history
- `activity_accomplishment_links` — Many-to-many relationship
- `config` — API keys (encrypted), preferences, thresholds
- `generation_cache` — Cached LLM outputs to avoid regeneration

### File Storage

Screenshots and attachments at `~/.folio/captures/`:
- Screen captures with OCR text
- Exported reports

### Portability

The entire career history is:
- A single SQLite file + captures directory
- Easily backed up, moved between machines
- No cloud dependency
- Encrypted at rest (optional)

## LLM Integration

Folio uses LLMs for:
1. **Auto-tagging** — Skills, themes, importance scoring
2. **Bullet generation** — Converting raw accomplishments to polished bullets
3. **Story generation** — Creating STAR-format interview stories
4. **Clustering** — Identifying related activities that form a single accomplishment

### Provider Options

| Provider | Pros | Cons |
|----------|------|------|
| **Local (Ollama)** | Private, free, offline | Lower quality, slower |
| **Claude API** | High quality, good at writing | Costs money, data leaves machine |
| **OpenAI API** | High quality | Costs money, data leaves machine |
| **Via MCP** | Uses existing Claude session | Requires Claude desktop |

User configures their preference. All LLM calls are optional — the tool works without them, just with less polish.

## Development Phases

### Phase 1: Core Capture (MVP)
- [ ] CLI with manual capture command
- [ ] Local git scanning
- [ ] SQLite storage
- [ ] Basic resume bullet generation (template-based, no LLM)

### Phase 2: Smart Enrichment
- [ ] GitHub API integration
- [ ] Link enrichment (paste URL → auto-fetch)
- [ ] LLM-powered bullet/story generation
- [ ] Weekly review command

### Phase 3: Nudges & Automation
- [ ] Background git watcher with notifications
- [ ] Triggered screen capture
- [ ] Auto-clustering of related activities
- [ ] TUI for interactive review

### Phase 4: Synthesis & Export
- [ ] STAR story generator
- [ ] Performance review summary generator
- [ ] MCP server for Claude integration
- [ ] Markdown/PDF/DOCX export

### Phase 5: Polish
- [ ] Web UI with career timeline
- [ ] Multi-device sync (optional, encrypted)
- [ ] Job description matching ("tailor resume for this role")
- [ ] Interview prep mode ("practice talking about this accomplishment")
