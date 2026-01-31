---
id: daily-workflow
title: Daily Workflow
sidebar_label: Daily Workflow
sidebar_position: 2
---

# Daily Workflow Examples

Build effective habits for capturing accomplishments as they happen. The best time to capture is when the work is fresh in your mind.

## End-of-Day Routine

### The 5-Minute Capture

At the end of each workday, spend 5 minutes capturing what you did:

```bash
# What did I ship today?
folio capture "Deployed new checkout flow to production" \
  --project "checkout-v2" \
  --importance high

# What bugs did I fix?
folio capture "Fixed cart total calculation bug" \
  --impact "Users were being overcharged on multi-item orders" \
  --project "checkout-v2"

# What did I learn or figure out?
folio capture "Debugged intermittent test failures" \
  --impact "Found race condition in test setup, reduced CI flakiness by 50%"

# Check your progress
folio list --limit 5
```

### Sample Day Captures

Here's what a typical developer's day might look like:

```bash
# Morning: Started the feature
folio capture "Implemented user preferences API endpoint" \
  --project "settings-page"

# Midday: Code review
folio capture "Reviewed auth refactoring PR" \
  --impact "Caught potential security issue before merge" \
  --importance medium

# Afternoon: Fixed an issue
folio capture "Fixed timezone handling in scheduling" \
  --impact "Resolved bug affecting users in non-US timezones" \
  --importance medium

# End of day: Finished the feature
folio capture "Completed settings page frontend" \
  --project "settings-page" \
  --impact "Users can now customize notification preferences"
```

---

## Capturing Different Types of Work

### Feature Development

```bash
# Starting a feature
folio capture "Started payment retry system redesign" \
  --project "payments" \
  --importance high

# Completing milestones
folio capture "Completed idempotency layer for payments" \
  --project "payments" \
  --impact "Prevents duplicate charges on network failures"

# Shipping
folio capture "Shipped payment retry system to production" \
  --project "payments" \
  --impact "Reduced failed transactions by 15%, recovering ~$50K/month" \
  --importance high
```

### Bug Fixes

```bash
# Quick fix
folio capture "Fixed null pointer in user search" \
  --importance low

# Critical fix
folio capture "Fixed production outage in order processing" \
  --impact "Restored service in 30 minutes, prevented $10K in lost orders" \
  --importance high

# Complex debugging
folio capture "Resolved memory leak in background workers" \
  --impact "Fixed issue causing daily restarts, improved stability" \
  --project "infrastructure"
```

### Code Reviews

```bash
# Thorough review
folio capture "Reviewed 800-line database migration PR" \
  --impact "Identified 3 potential issues before production deploy"

# Mentoring through review
folio capture "Pair-programmed with junior dev on their first PR" \
  --impact "Helped them understand our testing patterns"

# Security-focused review
folio capture "Security review of auth changes" \
  --importance high \
  --impact "Caught SQL injection vulnerability"
```

### Meetings & Collaboration

```bash
# Architecture discussion
folio capture "Led architecture discussion for API v2" \
  --project "api-v2" \
  --impact "Team aligned on microservices approach"

# Cross-team coordination
folio capture "Synced with mobile team on API requirements" \
  --project "mobile-api" \
  --impact "Defined contract for new endpoints"

# Incident response
folio capture "Participated in production incident response" \
  --impact "Helped identify root cause in 20 minutes"
```

---

## Weekly Patterns

### Monday: Plan the Week

```bash
# After sprint planning
folio capture "Sprint planning - taking on payments refactor" \
  --project "payments" \
  --importance medium
```

### Friday: Review the Week

```bash
# See what you accomplished
folio list --limit 20

# Check your stats
folio stats

# Add any missed captures
folio capture "Helped onboard new team member" \
  --impact "Reduced their ramp-up time with pair programming"
```

---

## Real-World Capture Examples

### Infrastructure Work

```bash
folio capture "Migrated CI/CD from Jenkins to GitHub Actions" \
  --project "developer-experience" \
  --impact "Build times reduced from 15 minutes to 5 minutes" \
  --importance high

folio capture "Set up staging environment auto-deployment" \
  --project "developer-experience" \
  --impact "Developers can now test changes without manual deploys"

folio capture "Implemented log aggregation with Datadog" \
  --project "observability" \
  --impact "Reduced incident investigation time by 70%"
```

### Performance Optimization

```bash
folio capture "Optimized product listing query" \
  --project "catalog" \
  --impact "Page load time reduced from 3s to 800ms" \
  --importance high

folio capture "Added Redis caching for user sessions" \
  --project "auth" \
  --impact "Reduced database load by 40%"

folio capture "Implemented image lazy loading" \
  --project "frontend" \
  --impact "Initial page load 50% faster on mobile"
```

### Team Contributions

```bash
folio capture "Created shared testing utilities library" \
  --project "developer-experience" \
  --impact "Adopted by 5 teams, reduced test boilerplate by 60%"

folio capture "Wrote runbook for production deployments" \
  --impact "Reduced deployment errors and onboarding time"

folio capture "Presented tech talk on event-driven architecture" \
  --impact "Team adopted patterns for 2 new services"
```

---

## Tips for Consistency

### 1. Set a Daily Reminder

Add a recurring reminder at the end of your workday to spend 5 minutes on Folio captures.

### 2. Capture Before Context Switching

When you finish a task and are about to start something new, take 30 seconds to capture what you just did.

### 3. Keep a Scratch Pad

Throughout the day, jot quick notes. At day's end, turn them into proper captures:

```
Notes:
- Fixed auth bug
- Reviewed Jake's PR
- Synced with mobile team

→ Convert to folio captures
```

### 4. Don't Overthink It

A basic capture is better than no capture:

```bash
# This is perfectly fine!
folio capture "Fixed login issue"

# You can always add more detail later
```
