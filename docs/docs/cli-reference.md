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

## Coming Soon

These commands are planned for future releases:

### scan

Scan a git repository for activities:

```bash
folio scan /path/to/repo
folio scan .              # Current directory
folio scan ~/work/repos/  # Multiple repos
```

### generate

Generate resume bullets and STAR stories:

```bash
folio generate bullets --employer "Acme Corp"
folio generate stories --top 5
```

### review

Interactive weekly review:

```bash
folio review --week
folio review --month
```

### export

Export your accomplishments:

```bash
folio export --format markdown
folio export --format json --employer "Acme Corp"
```
