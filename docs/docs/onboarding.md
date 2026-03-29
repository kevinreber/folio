---
id: onboarding
title: New User Onboarding Guide
sidebar_label: Onboarding Guide
sidebar_position: 3
---

# New User Onboarding Guide

Welcome to Folio! This guide will walk you through everything you need to get started tracking your career accomplishments. By the end, you'll have Folio installed, understand the core concepts, and have captured your first few accomplishments.

## What is Folio?

Folio is a **local-first career accomplishment tracker** for developers. It helps you:

- **Capture** what you do while it's fresh in your mind
- **Organize** your work by project, impact, and importance
- **Generate** polished outputs for resumes, interviews, and performance reviews

All your data stays on your machine in a simple SQLite database that you own and control.

---

## Step 1: Install Folio

### Quick Install (Recommended)

Run this one-liner to install Folio:

```bash
curl -sSf https://raw.githubusercontent.com/kevinreber/folio/main/install.sh | sh
```

This will:
- Check if Rust is installed (and install it if needed)
- Clone and build Folio
- Install the binary to `~/.local/bin/`

### Manual Install

If you prefer manual installation:

```bash
# Clone the repository
git clone https://github.com/kevinreber/folio.git
cd folio

# Build the release binary
cargo build --release

# Copy to your PATH
cp target/release/folio ~/.local/bin/
```

### Verify Installation

```bash
folio --version
```

You should see something like:
```
folio 0.2.0
```

---

## Step 2: Your First Capture

The core of Folio is the `capture` command. Let's add your first accomplishment:

```bash
folio capture "Installed Folio to start tracking my career accomplishments"
```

That's it! You've captured your first activity. Let's see it:

```bash
folio list
```

Output:
```
┌──────────────┬────────────┬─────────────────────────────────────────────────────────┬─────────┬────────────┐
│ ID           │ Date       │ Title                                                   │ Project │ Importance │
├──────────────┼────────────┼─────────────────────────────────────────────────────────┼─────────┼────────────┤
│ a1b2c3d4     │ 2025-01-30 │ Installed Folio to start tracking my career accomp...   │         │ medium     │
└──────────────┴────────────┴─────────────────────────────────────────────────────────┴─────────┴────────────┘
```

---

## Step 3: Add More Context

Basic captures are fine, but adding context makes your accomplishments more valuable later. Let's try a richer capture:

```bash
folio capture "Reduced API response time by optimizing database queries" \
  --impact "Improved p95 latency from 800ms to 200ms, affecting 1M daily requests" \
  --project "backend-api" \
  --importance high
```

### Available Options

| Option | Description | Example |
|--------|-------------|---------|
| `--impact` | Business or technical impact | `"Saved $5K/month in cloud costs"` |
| `--project` | Project name for grouping | `"checkout-v2"` |
| `--employer` | Company name | `"Acme Corp"` |
| `--importance` | Priority level | `high`, `medium`, `low` |

---

## Step 4: Explore Your Data

### List Activities

```bash
# Show recent activities
folio list

# Show more activities
folio list --limit 50

# Show full details
folio list --full
```

### View a Specific Activity

Copy the ID from `folio list` and view full details:

```bash
folio show a1b2c3d4
```

Output:
```
╭────────────────────────────────────────────────────────────────╮
│ Reduced API response time by optimizing database queries        │
╰────────────────────────────────────────────────────────────────╯

ID:         a1b2c3d4e5f6g7h8...
Date:       2025-01-30 14:32:15
Source:     manual
Type:       manual_entry

Project:    backend-api
Importance: HIGH

Impact:
  Improved p95 latency from 800ms to 200ms, affecting 1M daily requests
```

### Search Activities

Find specific accomplishments using fuzzy search:

```bash
folio search "API"
folio search "database" --importance high
```

### View Statistics

Get an overview of your captured work:

```bash
folio stats
```

---

## Step 5: Try the Interactive UI

Folio includes a terminal-based UI for browsing and managing activities:

```bash
folio tui
```

Navigate with arrow keys, press `Enter` to view details, and `q` to quit.

---

## Step 6: Use the Web Dashboard

If you prefer a graphical interface, Folio includes a built-in web dashboard:

```bash
folio serve --open
```

This starts the server and opens `http://127.0.0.1:3000` in your browser. The dashboard lets you:

- **View stats** at a glance — totals, importance breakdown, activity by source
- **Browse and filter** activities by importance or project
- **Search** across all your activities
- **Capture new activities** using a form instead of CLI commands

The dashboard is great for non-technical users or when you want a visual overview of your career data.

---

## Practical Examples

Here are real-world examples to help you get started:

### Example 1: Bug Fix

```bash
folio capture "Fixed null pointer exception in user authentication" \
  --impact "Resolved bug affecting 5% of login attempts" \
  --project "auth-service" \
  --importance medium
```

### Example 2: Feature Launch

```bash
folio capture "Launched new user dashboard with real-time analytics" \
  --impact "Increased user engagement by 25% based on initial metrics" \
  --project "dashboard-v2" \
  --importance high
```

### Example 3: Code Review

```bash
folio capture "Reviewed critical payment processing PR" \
  --impact "Caught race condition that could cause duplicate charges" \
  --importance medium
```

### Example 4: Infrastructure Work

```bash
folio capture "Migrated CI/CD pipeline from Jenkins to GitHub Actions" \
  --impact "Reduced build times from 15 minutes to 4 minutes" \
  --project "devops" \
  --importance high
```

### Example 5: Team Contribution

```bash
folio capture "Onboarded two new engineers to the backend team" \
  --impact "Reduced their ramp-up time through pair programming sessions"
```

### Example 6: Performance Optimization

```bash
folio capture "Implemented Redis caching for product catalog" \
  --impact "Reduced database load by 60%, improved page load by 2 seconds" \
  --project "catalog-service" \
  --importance high
```

### Example 7: Incident Response

```bash
folio capture "Led incident response for production database outage" \
  --impact "Restored service in 45 minutes, implemented monitoring to prevent recurrence" \
  --importance high
```

### Example 8: Documentation

```bash
folio capture "Created comprehensive API documentation" \
  --impact "Reduced support tickets from partner teams by 40%" \
  --project "api-docs"
```

---

## Building Good Habits

### The 5-Minute Daily Routine

At the end of each day, take 5 minutes to capture what you did:

```bash
# What did I ship?
folio capture "Deployed feature X"

# What bugs did I fix?
folio capture "Fixed issue with Y"

# What did I contribute to?
folio capture "Reviewed PR for Z"

# Check your progress
folio list --limit 5
```

### Weekly Review

Every Friday, review your week:

```bash
# See this week's activities
folio list --limit 20

# Check your stats
folio stats

# Fill in anything you missed
folio capture "Thing I forgot to capture earlier"
```

---

## Common Commands Reference

| Command | What it does | Example |
|---------|--------------|---------|
| `capture` | Add a new accomplishment | `folio capture "Did something great"` |
| `list` | View recent activities | `folio list --limit 20` |
| `show` | View activity details | `folio show abc123` |
| `search` | Find activities | `folio search "authentication"` |
| `edit` | Modify an activity | `folio edit abc123 --impact "New impact"` |
| `delete` | Remove an activity | `folio delete abc123` |
| `stats` | View statistics | `folio stats` |
| `export` | Export data | `folio export --format json` |
| `tui` | Open terminal UI | `folio tui` |
| `serve` | Start web dashboard + API | `folio serve --open` |
| `sync` | Sync from git/GitHub/Linear | `folio sync --source git` |
| `digest` | Generate activity summary | `folio digest weekly` |
| `import-transcript` | Import a transcript file | `folio import-transcript meeting.vtt` |
| `import-meeting` | Import a meeting summary | `folio import-meeting notes.json --source otter` |
| `import-calendar` | Import calendar events | `folio import-calendar calendar.ics` |
| `voice` | Record a voice note | `folio voice --duration 60` |
| `screen-capture` | Capture the screen | `folio screen-capture --title "Bug"` |
| `track` | Start activity tracking | `folio track` |
| `daemon` | Background daemon (watch + track) | `folio daemon start` |

---

## Tips for Success

### 1. Capture Now, Enrich Later

Don't wait to write the perfect entry. A quick capture is better than no capture:

```bash
# Good enough!
folio capture "Fixed login bug"

# You can always add more detail later
```

### 2. Include Numbers When Possible

Quantified impact is more powerful:

```bash
# Vague
folio capture "Made the app faster"

# Better
folio capture "Improved app performance" \
  --impact "Reduced load time from 5s to 1.5s (70% improvement)"
```

### 3. Use Consistent Project Names

This helps you filter and group later:

```bash
# Good: consistent naming
folio capture "Added caching" --project "payments-api"
folio capture "Fixed timeout" --project "payments-api"
folio capture "Added logging" --project "payments-api"
```

### 4. Capture Soft Skills Too

Technical work isn't the only thing that matters:

```bash
folio capture "Mentored junior developer on testing best practices"
folio capture "Led architecture decision meeting for new service"
folio capture "Wrote runbook that reduced on-call burden"
```

---

## Where Your Data Lives

All your data is stored locally:

```
~/.folio/folio.db    # SQLite database with all your activities
~/.folio/config.toml # Configuration file (optional)
```

You can back up your career history by copying `~/.folio/folio.db` to another location.

---

## Getting Help

```bash
# General help
folio --help

# Help for a specific command
folio capture --help
folio list --help
```

---

## Next Steps

Now that you're set up:

1. **Start capturing** - Add 3-5 accomplishments from this week
2. **Build the habit** - Set a daily reminder to spend 5 minutes on captures
3. **Explore more features** - Check out [CLI Reference](/docs/cli-reference) for all commands
4. **Learn workflows** - See [Daily Workflow Examples](/docs/examples/daily-workflow) for patterns

Happy tracking!
