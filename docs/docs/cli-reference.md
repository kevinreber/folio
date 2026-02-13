---
id: cli-reference
title: CLI Reference
sidebar_label: CLI Reference
sidebar_position: 3
---

# CLI Reference

Complete reference for all Folio commands.

## Global Options

```bash
folio [OPTIONS] <COMMAND>

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

---

## capture

Add a new accomplishment or activity to your career log.

### Usage

```bash
folio capture <TITLE> [OPTIONS]
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<TITLE>` | Yes | Title or description of the accomplishment |

### Options

| Option | Short | Description | Example |
|--------|-------|-------------|---------|
| `--impact <TEXT>` | `-i` | Impact or outcome of the work | `--impact "Reduced costs by 30%"` |
| `--project <NAME>` | `-p` | Project or initiative name | `--project "payments-v2"` |
| `--employer <NAME>` | `-e` | Company or employer name | `--employer "Acme Corp"` |
| `--importance <LEVEL>` | | Priority level: `low`, `medium`, `high` | `--importance high` |

### Examples

```bash
# Basic capture
folio capture "Shipped new user onboarding flow"

# With impact
folio capture "Optimized database queries" \
  --impact "Reduced page load time by 60%"

# Full details
folio capture "Led migration to microservices" \
  --impact "Improved deployment frequency from weekly to daily" \
  --project "platform-modernization" \
  --employer "TechCo" \
  --importance high
```

---

## list

View your recent activities.

### Usage

```bash
folio list [OPTIONS]
```

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--limit <N>` | `-l` | Number of activities to show | `10` |
| `--full` | `-f` | Show full details (verbose output) | `false` |

### Examples

```bash
# Show last 10 activities
folio list

# Show last 25 activities
folio list --limit 25

# Show with full details
folio list --full

# Combine options
folio list -l 5 -f
```

### Output Format

The default output is a formatted table:

```
┌──────────────┬────────────┬──────────────────────────┬─────────────┬────────────┐
│ ID           │ Date       │ Title                    │ Project     │ Importance │
├──────────────┼────────────┼──────────────────────────┼─────────────┼────────────┤
│ a1b2c3d4     │ 2025-01-30 │ Shipped new feature      │ frontend    │ high       │
│ e5f6g7h8     │ 2025-01-29 │ Fixed production bug     │ backend     │ medium     │
│ i9j0k1l2     │ 2025-01-28 │ Code review for teammate │             │ low        │
└──────────────┴────────────┴──────────────────────────┴─────────────┴────────────┘

Showing 3 of 15 activities
```

---

## show

View detailed information about a specific activity.

### Usage

```bash
folio show <ID>
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<ID>` | Yes | Activity ID (full or partial match) |

### Examples

```bash
# Full ID
folio show a1b2c3d4-e5f6-g7h8-i9j0-k1l2m3n4o5p6

# Partial ID (if unique)
folio show a1b2
```

### Output

```
╭────────────────────────────────────────────────────────────────╮
│ Shipped new user onboarding flow                                │
╰────────────────────────────────────────────────────────────────╯

ID:         a1b2c3d4-e5f6-g7h8-i9j0-k1l2m3n4o5p6
Date:       2025-01-30 09:15:22
Source:     manual
Type:       manual_entry

Project:    frontend
Employer:   Acme Corp
Importance: HIGH

Impact:
  Increased user activation rate by 25%

Suggested next steps:
  • Add STAR context (Situation, Task, Action, Result)
  • Link related git commits or PRs
  • Add specific metrics if available
```

---

## stats

View aggregated statistics about your captured activities.

### Usage

```bash
folio stats
```

### Output

```
╭────────────────────────────────────────╮
│          Folio Statistics              │
╰────────────────────────────────────────╯

Total Activities: 47

By Importance:
  High:   12 (25.5%)
  Medium: 28 (59.6%)
  Low:    7  (14.9%)

By Source:
  manual:  35
  git:     12

By Project:
  payments:    15
  frontend:    12
  backend:      8
  (no project): 12

Date Range: 2024-06-15 to 2025-01-30

Needs Attention:
  Missing impact:      8
  Missing description: 3
```

---

## delete

Remove an activity from your career log.

### Usage

```bash
folio delete <ID> [OPTIONS]
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<ID>` | Yes | Activity ID to delete |

### Options

| Option | Description |
|--------|-------------|
| `--force` | Skip confirmation prompt |

### Examples

```bash
# With confirmation
folio delete a1b2c3d4

# Skip confirmation
folio delete a1b2c3d4 --force
```

### Confirmation Prompt

```
Are you sure you want to delete this activity?

  Title: Shipped new feature
  Date:  2025-01-30

Delete? [y/N]
```

---

## serve

Start the Folio server with the web dashboard and REST API.

### Usage

```bash
folio serve [OPTIONS]
```

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--host <HOST>` | `-H` | Host to bind to | `127.0.0.1` |
| `--port <PORT>` | `-p` | Port to listen on | `3000` |
| `--mcp` | | Start MCP server instead of REST API | `false` |
| `--open` | | Open the dashboard in your default browser | `false` |

### Examples

```bash
# Start server with web dashboard
folio serve

# Start and open browser automatically
folio serve --open

# Use a custom port
folio serve --port 8080

# Bind to all interfaces (for LAN access)
folio serve --host 0.0.0.0

# Start MCP server for Claude integration
folio serve --mcp
```

### Web Dashboard

When running without `--mcp`, the server serves a web dashboard at the root URL alongside the REST API. The dashboard includes:

- **Dashboard** — stats overview with activity counts by importance and source
- **Activities** — filterable list of all activities
- **Search** — full-text search across titles, descriptions, and projects
- **Capture** — form to create new activities from the browser

### REST API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/health` | Health check |
| `GET` | `/api/activities` | List activities |
| `POST` | `/api/activities` | Create activity |
| `GET` | `/api/activities/:id` | Get activity by ID |
| `PUT` | `/api/activities/:id` | Update activity |
| `DELETE` | `/api/activities/:id` | Delete activity |
| `GET` | `/api/activities/search?q=` | Search activities |
| `GET` | `/api/stats` | Get statistics |
| `GET` | `/api/export?format=` | Export data (json) |

---

## sync

Sync activities from external sources.

### Usage

```bash
folio sync [OPTIONS]
```

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--source <SOURCE>` | `-s` | Source to sync from (git, github, linear, all) | all |
| `--days <N>` | `-d` | Number of days to look back | `30` |
| `--repo <PATH>` | `-r` | Specific repository path (for git) | |
| `--dry-run` | | Show what would be synced | `false` |

### Examples

```bash
# Sync from all sources
folio sync

# Sync just git commits from last 7 days
folio sync --source git --days 7

# Sync a specific repo
folio sync --source git --repo ~/work/my-project

# Preview what would be synced
folio sync --dry-run
```

---

## export

Export activities and accomplishments.

### Usage

```bash
folio export [OPTIONS]
```

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--format <FORMAT>` | `-f` | Export format (markdown, json, yaml) | `markdown` |
| `--output <FILE>` | `-o` | Output file (stdout if not specified) | |
| `--brag` | | Export as brag document | `false` |
| `--bullets` | | Export as resume bullets | `false` |

### Examples

```bash
# Export as markdown to stdout
folio export

# Export as JSON to a file
folio export --format json --output activities.json

# Generate a brag document
folio export --brag --output brag-doc.md
```

---

## promote

Promote an activity to a polished STAR-format accomplishment.

### Usage

```bash
folio promote <ID> [OPTIONS]
```

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--interactive` | `-i` | Interactive mode for detailed STAR story |

### Examples

```bash
# Promote an activity
folio promote a1b2c3d4

# Interactive STAR story building
folio promote a1b2c3d4 --interactive
```

---

## digest

Generate a summary of your activities over a time period.

### Usage

```bash
folio digest [PERIOD] [OPTIONS]
```

### Arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `[PERIOD]` | Time period: daily, weekly, monthly, quarterly, yearly | `weekly` |

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--markdown` | `-m` | Output as markdown |

### Examples

```bash
# Weekly digest
folio digest

# Monthly summary in markdown
folio digest monthly --markdown
```

---

## review

Generate a performance review summary.

### Usage

```bash
folio review [OPTIONS]
```

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--months <N>` | `-m` | Number of months to include | `6` |

### Examples

```bash
# Last 6 months
folio review

# Last year
folio review --months 12
```
