---
id: index
title: Usage Examples
sidebar_label: Overview
sidebar_position: 1
---

# Usage Examples

Real-world examples of how to use Folio effectively in your daily workflow.

## Example Categories

### [Daily Workflow](/docs/examples/daily-workflow)

Build habits for capturing accomplishments as they happen:
- End-of-day capture routine
- Quick wins and bug fixes
- Meeting outcomes
- Code review contributions

### [Interview Preparation](/docs/examples/interview-prep)

Use your captured accomplishments to prepare for interviews:
- Finding your best stories
- Quantifying impact
- Building STAR narratives
- Role-specific filtering

### [Performance Reviews](/docs/examples/performance-reviews)

Compile your accomplishments for performance reviews:
- Quarterly summaries
- Theme-based grouping
- Impact highlighting
- Gap identification

---

## Quick Reference

Here are some common patterns you'll use frequently:

### Capture Patterns

```bash
# Quick capture (just the title)
folio capture "Fixed critical auth bug"

# With impact
folio capture "Optimized search" --impact "3x faster queries"

# Full context
folio capture "Led API redesign" \
  --impact "Reduced latency 40%, enabled mobile app" \
  --project "api-v2" \
  --employer "TechCorp" \
  --importance high
```

### Viewing Activities

```bash
# Recent activities
folio list

# More activities
folio list --limit 50

# Full details
folio list --full

# Specific activity
folio show abc123
```

### Understanding Your Data

```bash
# Overall statistics
folio stats
```

---

## Tips for Effective Capturing

### 1. Capture Early, Enrich Later

```bash
# Right after shipping, capture the basics
folio capture "Shipped user dashboard redesign"

# Later, add context when you have metrics
# (enrichment feature coming soon)
```

### 2. Use Consistent Project Names

```bash
# Good: consistent naming
folio capture "Added caching layer" --project "payments-api"
folio capture "Fixed race condition" --project "payments-api"
folio capture "Added retry logic" --project "payments-api"

# Avoid: inconsistent naming
folio capture "Added caching" --project "Payments"
folio capture "Fixed bug" --project "payments api"
folio capture "Added retry" --project "payment-service"
```

### 3. Include Quantifiable Impact

```bash
# Good: specific metrics
folio capture "Database optimization" \
  --impact "Reduced query time from 2s to 200ms for product catalog"

# Better: include scope
folio capture "Database optimization" \
  --impact "Reduced query time by 90% (2s to 200ms), affecting 50K daily requests"

# Avoid: vague impact
folio capture "Database optimization" --impact "Made it faster"
```

### 4. Use Importance Levels Strategically

```bash
# High: Major features, significant impact, leadership
folio capture "Led migration to Kubernetes" --importance high

# Medium: Regular features, improvements (default)
folio capture "Added pagination to user list"

# Low: Bug fixes, small improvements, routine work
folio capture "Fixed typo in error message" --importance low
```

---

## What to Capture

Not sure what's worth capturing? Here are examples by category:

### Technical Achievements
- Shipped new features
- Performance optimizations
- Architecture improvements
- Technical debt reduction
- Security fixes

### Impact & Metrics
- Cost reductions
- Speed improvements
- Reliability increases
- User experience improvements

### Leadership & Collaboration
- Led projects or initiatives
- Mentored team members
- Drove technical decisions
- Cross-team coordination
- Onboarded new engineers

### Problem Solving
- Debugged difficult issues
- Resolved production incidents
- Fixed long-standing bugs
- Created tools that helped the team

### Learning & Growth
- Learned new technologies
- Gave presentations or demos
- Wrote documentation
- Contributed to open source
