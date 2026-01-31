# Folio Features Guide

This document provides comprehensive documentation for all Folio features, including examples and usage instructions.

## Table of Contents

1. [Core Commands](#core-commands)
2. [Search & Discovery](#search--discovery)
3. [Editing & Management](#editing--management)
4. [Integrations](#integrations)
5. [AI-Powered Features](#ai-powered-features)
6. [Export & Import](#export--import)
7. [Interactive TUI](#interactive-tui)
8. [Servers (API & MCP)](#servers-api--mcp)
9. [Background Watcher](#background-watcher)
10. [Configuration](#configuration)

---

## Core Commands

### Capture

Capture a new accomplishment or activity manually.

```bash
# Basic capture
folio capture "Implemented new authentication system"

# With impact and project
folio capture "Reduced API latency by 50%" \
  --impact "Improved user experience and reduced server costs" \
  --project "backend-api" \
  --importance high

# With employer context
folio capture "Led migration to Kubernetes" \
  --employer "Acme Corp" \
  --project "infrastructure" \
  --impact "Reduced deployment time from 4 hours to 15 minutes"
```

**Options:**
- `-i, --impact <TEXT>` - Impact or result of the accomplishment
- `-p, --project <NAME>` - Project this relates to
- `-e, --employer <NAME>` - Employer/company context
- `--importance <LEVEL>` - Importance level: low, medium, high (default: medium)

### List

List recent activities.

```bash
# List last 10 activities (default)
folio list

# List more activities
folio list --limit 50

# Show full details
folio list --full
```

### Show

Show detailed information about a specific activity.

```bash
# Show by full ID
folio show a1b2c3d4-e5f6-7890-abcd-ef1234567890

# Show by partial ID (first 8 characters)
folio show a1b2c3d4
```

### Delete

Delete an activity.

```bash
# Delete with confirmation
folio delete a1b2c3d4

# Delete without confirmation
folio delete a1b2c3d4 --force
```

### Stats

Show statistics about your captured activities.

```bash
folio stats
```

Output includes:
- Total activities
- Breakdown by importance (high, medium, low)
- Breakdown by source (git, github, manual, etc.)
- Breakdown by project
- Date range of activities

---

## Search & Discovery

### Search

Search activities with fuzzy matching.

```bash
# Basic search
folio search "authentication"

# Search with filters
folio search "api" --project backend --limit 10

# Filter by importance
folio search "bug" --importance high
```

**Options:**
- `-l, --limit <N>` - Maximum number of results (default: 20)
- `-p, --project <NAME>` - Filter by project name
- `--importance <LEVEL>` - Filter by importance level

---

## Editing & Management

### Edit

Edit an existing activity.

```bash
# Interactive edit mode
folio edit a1b2c3d4

# Direct edit with specific fields
folio edit a1b2c3d4 --title "Updated title" --impact "New impact"

# Change importance
folio edit a1b2c3d4 --importance high

# Update project
folio edit a1b2c3d4 --project "new-project"
```

**Options:**
- `-t, --title <TEXT>` - New title
- `-i, --impact <TEXT>` - New impact
- `-p, --project <NAME>` - New project
- `-e, --employer <NAME>` - New employer
- `--importance <LEVEL>` - New importance level

### Promote

Convert an activity into a polished accomplishment with STAR format.

```bash
# Auto-generate STAR story from activity
folio promote a1b2c3d4

# Interactive mode for detailed STAR story
folio promote a1b2c3d4 --interactive
```

The interactive mode will guide you through:
1. **Situation** - Context and background
2. **Task** - Your specific responsibility
3. **Action** - Steps you took
4. **Result** - Quantified outcomes

---

## Integrations

### Sync

Sync activities from external sources.

```bash
# Sync from all configured sources
folio sync

# Sync only from git
folio sync --source git --days 30

# Sync from GitHub
folio sync --source github --days 14

# Sync from Linear
folio sync --source linear --days 7

# Sync a specific repository
folio sync --source git --repo ~/projects/my-app

# Dry run - see what would be synced
folio sync --source git --dry-run
```

**Options:**
- `-s, --source <SOURCE>` - Source to sync from: git, github, linear, or all
- `-d, --days <N>` - Number of days to look back (default: 30)
- `-r, --repo <PATH>` - Specific repository path (for git)
- `--dry-run` - Show what would be synced without actually syncing

### Git Integration

Folio automatically scans your local git repositories for commits.

**Detected information:**
- Commit message
- Files changed
- Lines added/deleted
- Author information
- Project name (from directory)

**Importance inference:**
- **High**: Breaking changes, major features, security fixes, 500+ lines
- **Medium**: Regular features, bug fixes, 50-500 lines
- **Low**: Typos, formatting, documentation, <50 lines

### GitHub Integration

Connect to GitHub to pull PRs and issues.

```bash
# Configure GitHub token
folio config github.token ghp_xxxxxxxxxxxx

# Enable GitHub integration
folio config github.enabled true

# Add repositories to track
# (Edit ~/.folio/config.toml manually)
```

**Tracked activities:**
- Merged pull requests
- Closed issues
- Code reviews (coming soon)

### Linear Integration

Connect to Linear for issue tracking.

```bash
# Configure Linear API key
folio config linear.api_key lin_api_xxxxxxxxxxxx

# Enable Linear integration
folio config linear.enabled true
```

---

## AI-Powered Features

### Resume Bullets

Generate polished resume bullet points from your activities.

```bash
# Export as resume bullets
folio export --bullets

# Save to file
folio export --bullets --output resume-bullets.md
```

**Bullet styles:**
- **Impact-focused**: Emphasizes quantified results
- **Technical**: Highlights technologies and architecture
- **Leadership**: Focuses on collaboration and mentorship
- **Concise**: Short, punchy statements

### STAR Story Builder

Build behavioral interview stories using the STAR framework.

```bash
# Promote activity to accomplishment with STAR format
folio promote a1b2c3d4 --interactive
```

### Auto-Tagging

Activities are automatically tagged with:
- **Technical skills**: Languages, frameworks, tools
- **Soft skills**: Leadership, communication, problem-solving
- **Themes**: Backend, frontend, infrastructure, etc.
- **Suggested importance**: Based on content analysis

### Digest

Generate summaries of your activities.

```bash
# Weekly digest (default)
folio digest

# Daily digest
folio digest daily

# Monthly digest
folio digest monthly

# Output as markdown
folio digest weekly --markdown
```

### Performance Review

Generate a performance review summary.

```bash
# 6-month review (default)
folio review

# 12-month review
folio review --months 12
```

### Job Matching

Match your experience against job descriptions.

```bash
# Interactive - paste job description
folio match "Senior Software Engineer"

# From file
folio match "Backend Developer" --file job-description.txt

# With inline description
folio match "Full Stack Engineer" --description "Looking for someone with React, Node.js..."
```

Output includes:
- Overall match score (percentage)
- Matched requirements
- Preferred qualifications met
- Gaps to address
- Suggested talking points
- Activities to highlight in the interview

---

## Export & Import

### Export

Export your data in various formats.

```bash
# Export as markdown (default)
folio export

# Export as JSON
folio export --format json

# Export as YAML
folio export --format yaml

# Export to file
folio export --format markdown --output career-history.md

# Export as brag document
folio export --brag

# Export as resume bullets
folio export --bullets
```

**Formats:**
- `markdown` / `md` - Human-readable markdown
- `json` - Structured JSON
- `yaml` / `yml` - YAML format

### Import

Import activities from files.

```bash
# Import from JSON
folio import activities.json

# Import from YAML
folio import activities.yaml

# Dry run - see what would be imported
folio import activities.json --dry-run
```

---

## Interactive TUI

Launch the terminal user interface for browsing and managing activities.

```bash
folio tui
# or
folio ui
```

**Keyboard shortcuts:**
| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Tab` | Next tab |
| `Shift+Tab` | Previous tab |
| `/` | Search |
| `d` | Delete selected |
| `r` | Refresh |
| `Esc` | Clear search |
| `?` | Show help |
| `q` | Quit |

**Tabs:**
1. **Activities** - Browse and view activity details
2. **Accomplishments** - View polished accomplishments
3. **Stats** - Overview statistics

---

## Servers (API & MCP)

### REST API Server

Start a REST API server for integrations.

```bash
# Start on default port (3000)
folio serve

# Custom host and port
folio serve --host 0.0.0.0 --port 8080
```

**Endpoints:**
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/activities` | List activities |
| POST | `/api/activities` | Create activity |
| GET | `/api/activities/:id` | Get activity |
| DELETE | `/api/activities/:id` | Delete activity |
| GET | `/api/activities/search?q=query` | Search activities |
| GET | `/api/stats` | Get statistics |
| GET | `/api/export?format=json` | Export data |

### MCP Server

Start a Model Context Protocol server for AI integration (e.g., with Claude).

```bash
folio serve --mcp

# Custom port
folio serve --mcp --port 3001
```

**Available tools:**
- `folio_list_activities` - List recent activities
- `folio_search` - Search activities
- `folio_get_activity` - Get activity details
- `folio_create_activity` - Create new activity
- `folio_generate_bullets` - Generate resume bullets
- `folio_generate_digest` - Generate activity digest
- `folio_export` - Export activities

---

## Background Watcher

Automatically monitor git repositories for new commits.

```bash
# Run single scan
folio watch

# Run as daemon (continuous monitoring)
folio watch --daemon
```

**Configuration:**
```toml
# ~/.folio/config.toml
[watcher]
enabled = true
interval_seconds = 300  # Scan every 5 minutes
notifications = true    # Desktop notifications for new activities
```

---

## Configuration

### Config Command

Manage Folio configuration.

```bash
# List all configuration keys
folio config --list

# Get a value
folio config github.token

# Set a value
folio config github.token ghp_xxxxxxxxxxxx

# Run interactive setup wizard
folio config --init
```

### Configuration File

Located at `~/.folio/config.toml`:

```toml
[general]
default_employer = "My Company"
default_project = "main-product"
git_email = "me@example.com"
color = true
date_format = "%Y-%m-%d"

[git]
enabled = true
scan_dirs = [
    "~/code",
    "~/projects",
    "~/work"
]
max_depth = 3
days_back = 30
min_lines_changed = 10

[github]
enabled = true
token = "ghp_xxxxxxxxxxxx"
repositories = [
    "owner/repo1",
    "owner/repo2"
]
days_back = 30

[linear]
enabled = false
api_key = "lin_api_xxxxxxxxxxxx"
teams = ["TEAM1", "TEAM2"]
days_back = 30

[ai]
enabled = false
provider = "openai"
api_key = "sk-xxxxxxxxxxxx"
model = "gpt-4"
temperature = 0.7

[export]
default_format = "markdown"
output_dir = "~/.folio/exports"

[watcher]
enabled = false
interval_seconds = 300
notifications = true
```

### Environment Variables

Folio also reads from environment variables:
- `GITHUB_TOKEN` - GitHub personal access token
- `LINEAR_API_KEY` - Linear API key
- `OPENAI_API_KEY` - OpenAI API key (for AI features)
- `JIRA_URL`, `JIRA_API_TOKEN`, `JIRA_EMAIL` - Jira configuration

---

## Examples

### Daily Workflow

```bash
# Morning: Sync from git
folio sync --source git --days 1

# Throughout the day: Capture accomplishments
folio capture "Fixed critical bug in payment processing" \
  --impact "Prevented $10k in lost revenue" \
  --importance high

# End of day: Review
folio list --limit 5
```

### Weekly Review

```bash
# Generate weekly digest
folio digest weekly --markdown > weekly-review.md

# Export for brag doc
folio export --brag --output brag-doc.md
```

### Interview Prep

```bash
# Match against job description
folio match "Senior Backend Engineer" --file job.txt

# Generate resume bullets
folio export --bullets

# Review key accomplishments
folio search "leadership" --importance high
```

### Setting Up Integrations

```bash
# 1. Initialize configuration
folio config --init

# 2. Add GitHub token
folio config github.token ghp_xxxxxxxxxxxx
folio config github.enabled true

# 3. Sync from GitHub
folio sync --source github --days 30

# 4. Start background watcher
folio watch --daemon
```
