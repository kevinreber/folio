---
id: getting-started
title: Getting Started
sidebar_label: Getting Started
sidebar_position: 2
---

# Getting Started

Get up and running with Folio in just a few minutes.

## Prerequisites

- **Rust** (1.70 or later) - [Install Rust](https://rustup.rs/)

## Installation

### From Source

Clone the repository and build:

```bash
git clone https://github.com/kevinreber/folio.git
cd folio
cargo build --release
```

The binary will be at `./target/release/folio`. You can move it to your PATH:

```bash
# Linux/macOS
sudo cp target/release/folio /usr/local/bin/

# Or add to your local bin
cp target/release/folio ~/.local/bin/
```

### Verify Installation

```bash
folio --version
```

## Your First Accomplishment

Let's capture your first professional accomplishment:

```bash
folio capture "Set up Folio for career tracking"
```

That's it! Your accomplishment is now saved locally.

## Adding More Context

For more meaningful entries, add impact and context:

```bash
folio capture "Reduced API response time" \
  --impact "40% latency reduction affecting 2M daily requests" \
  --project "backend-optimization" \
  --employer "Acme Corp" \
  --importance high
```

## Viewing Your Accomplishments

### List Recent Activities

```bash
folio list
```

Output:
```
┌──────────────┬────────────┬───────────────────────────────────┬──────────────────────┬────────────┐
│ ID           │ Date       │ Title                             │ Project              │ Importance │
├──────────────┼────────────┼───────────────────────────────────┼──────────────────────┼────────────┤
│ a1b2c3d4     │ 2025-01-30 │ Reduced API response time         │ backend-optimization │ high       │
│ e5f6g7h8     │ 2025-01-30 │ Set up Folio for career tracking  │                      │ medium     │
└──────────────┴────────────┴───────────────────────────────────┴──────────────────────┴────────────┘

Showing 2 of 2 activities
```

### View Full Details

```bash
folio show a1b2c3d4
```

Output:
```
╭────────────────────────────────────────────────────────────────╮
│ Reduced API response time                                       │
╰────────────────────────────────────────────────────────────────╯

ID:         a1b2c3d4e5f6g7h8...
Date:       2025-01-30 14:32:15
Source:     manual
Type:       manual_entry

Project:    backend-optimization
Employer:   Acme Corp
Importance: HIGH

Impact:
  40% latency reduction affecting 2M daily requests

Suggested next steps:
  • Add STAR context (Situation, Task, Action, Result)
  • Link related git commits or PRs
  • Add specific metrics if available
```

### View Statistics

```bash
folio stats
```

Output:
```
╭────────────────────────────────────────╮
│          Folio Statistics              │
╰────────────────────────────────────────╯

Total Activities: 2

By Importance:
  High:   1 (50%)
  Medium: 1 (50%)
  Low:    0 (0%)

By Source:
  manual: 2

By Project:
  backend-optimization: 1
  (no project):         1

Date Range: 2025-01-30 to 2025-01-30

Needs Attention:
  Missing impact:      1
  Missing description: 0
```

## Where Data is Stored

Folio stores all your data locally in a SQLite database:

```
~/.folio/folio.db
```

This file contains your entire career history and can be backed up or moved between machines.

## Next Steps

Now that you've captured your first accomplishments:

1. **[Read the CLI Reference](./cli-reference)** - Learn all available commands
2. **[Explore Examples](./examples/)** - See real-world usage patterns
3. **Build a habit** - Capture accomplishments as they happen, not months later

:::tip Best Practice
The best time to capture an accomplishment is right after it happens, when the details are fresh. Even a quick capture with just a title is better than nothing — you can enrich it later.
:::
