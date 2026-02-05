---
id: intro
title: Introduction to Folio
sidebar_label: Introduction
slug: /
sidebar_position: 1
---

# Folio

> Your career changelog. Capture, enrich, and package your professional accomplishments — own your narrative across your entire career.

## What is Folio?

Folio is a **local-first career accomplishment tracker** that helps developers capture what they've done, enrich it with impact and context, and package it into interview-ready stories, resume bullets, and performance review summaries.

### The Problem

You do impactful work every day but forget the specifics by review/interview time. Even when you remember, translating "I fixed the thing" into "Reduced API latency by 40% through query optimization, impacting 2M daily requests" is hard.

### The Solution

A prompted work journal that enriches itself when possible — capturing accomplishments while they're fresh and packaging them when you need them.

## Key Features

- **Capture** your professional work while it's fresh (commits, PRs, manual entries)
- **Enrich** that data with impact metrics and context
- **Package** accomplishments into interview-ready content (resume bullets, STAR stories, performance review summaries)

## Quick Example

```bash
# Capture an accomplishment
folio capture "Shipped payment retry system" \
  --impact "15% fewer failed transactions" \
  --project "payments" \
  --importance high

# View your recent activities
folio list

# Get detailed stats
folio stats
```

## Key Principles

| Principle | Description |
|-----------|-------------|
| **Individual-owned** | Your career history, not your employer's — portable across jobs |
| **Local-first** | Your career narrative isn't sitting on someone else's servers |
| **Capture → Synthesis** | Not just search, but packaged output (resume bullets, STAR stories) |
| **Developer-depth** | Git, PRs, deploys, incidents at a level general tools don't reach |
| **Prompted, not passive** | Capture context while it's fresh through smart nudges |

## How It's Different

| Tool | What it does | Gap |
|------|-------------|-----|
| **Glean** | Enterprise search/knowledge management | Company-owned, finds info but doesn't synthesize YOUR narrative |
| **Notion/Brag docs** | Manual accomplishment logs | Requires discipline; you have to remember to write |
| **Lattice/15Five** | Performance review tools | Company-owned, review-cycle focused |
| **RescueTime** | Time tracking | Tracks time, not impact or accomplishments |

**Folio's positioning:** Glean helps you find what your company knows. Folio helps you articulate what YOU'VE done — and own that narrative across your entire career.

## Next Steps

- [Getting Started](/docs/getting-started) - Install Folio and capture your first accomplishment
- [Onboarding Guide](/docs/onboarding) - Step-by-step guide with practical examples
- [CLI Reference](/docs/cli-reference) - Complete command documentation
- [Examples](/docs/examples/) - Real-world usage patterns
