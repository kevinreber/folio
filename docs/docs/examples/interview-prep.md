---
id: interview-prep
title: Interview Preparation
sidebar_label: Interview Prep
sidebar_position: 3
---

# Interview Preparation

Use your captured accomplishments to prepare compelling interview stories.

## Finding Your Best Stories

### Review Your High-Impact Work

Start by looking at your most significant accomplishments:

```bash
# See all your activities
folio list --limit 50 --full

# Look at your statistics to find patterns
folio stats
```

### Identify Strong Stories

Look for accomplishments that have:

1. **Quantifiable impact** - Numbers make stories memorable
2. **Technical complexity** - Demonstrates your skills
3. **Business value** - Shows you understand the bigger picture
4. **Leadership/ownership** - Highlights soft skills

---

## Building STAR Stories

Take your captures and expand them into STAR format (Situation, Task, Action, Result).

### Example: Payment Retry System

**Original capture:**
```bash
folio capture "Shipped payment retry system" \
  --impact "Reduced failed transactions by 15%, recovering ~$50K/month" \
  --project "payments" \
  --importance high
```

**Expanded to STAR format:**

> **Situation:** Our payment system was losing approximately $50K per month due to failed retry logic that didn't handle idempotency properly. Customers were complaining about failed transactions, and in some cases, duplicate charges.
>
> **Task:** I was tasked with redesigning the retry mechanism to be idempotent and reliable, ensuring zero duplicate charges during the migration to the new system.
>
> **Action:** I designed an idempotent retry system using unique transaction IDs and exponential backoff. I implemented a shadow mode that ran both old and new systems in parallel for 2 weeks to validate correctness. I also set up real-time monitoring dashboards to track failure rates.
>
> **Result:** Failed transaction rate dropped by 15%, recovering approximately $50K per month in previously lost revenue. Zero duplicate charges occurred during the migration. The pattern was later adopted by three other teams for their retry logic.

### Example: Performance Optimization

**Original capture:**
```bash
folio capture "Optimized database queries for product catalog" \
  --impact "Reduced page load from 3s to 800ms" \
  --project "catalog" \
  --importance high
```

**Expanded to STAR format:**

> **Situation:** Our product catalog page was taking 3+ seconds to load, causing poor user experience and higher bounce rates. The page was loading 200K products, and the database queries weren't optimized for the access patterns.
>
> **Task:** Reduce page load time to under 1 second without changing the product data model or requiring a complete rewrite.
>
> **Action:** I analyzed query execution plans and identified missing indexes. I implemented database connection pooling and query result caching with Redis. I also added pagination to limit initial data loads.
>
> **Result:** Page load time dropped from 3 seconds to 800ms (73% improvement). Database CPU usage decreased by 40%, and user engagement metrics improved by 15%.

---

## Interview Question Prep

### "Tell me about a technically challenging project"

Search your captures for high-importance technical work:

```bash
folio list --full
```

Look for entries with:
- Technical complexity (architecture, optimization, debugging)
- Problem-solving elements
- Measurable outcomes

**Example story from capture:**
```bash
folio capture "Debugged and fixed distributed deadlock in order processing" \
  --impact "Resolved issue causing 5% of orders to fail silently" \
  --importance high
```

→ Expand into a story about identifying the issue, your debugging approach, and the fix.

### "Tell me about a time you showed leadership"

Look for entries involving:
- Leading projects or initiatives
- Mentoring teammates
- Driving technical decisions

**Example:**
```bash
folio capture "Led team through production incident response" \
  --impact "Coordinated 5 engineers to restore service in 30 minutes"
```

### "Tell me about a time you improved a process"

Look for developer experience and automation work:

**Example:**
```bash
folio capture "Automated deployment process" \
  --impact "Reduced deployment time from 2 hours to 15 minutes" \
  --project "developer-experience"
```

### "Tell me about a failure and what you learned"

Even negative outcomes are valuable stories:

```bash
folio capture "Deployed feature with caching bug" \
  --impact "Caused 2-hour incident, implemented better testing as result"
```

---

## Preparing for Specific Roles

### For Senior/Staff Positions

Focus on captures that show:
- System design and architecture decisions
- Cross-team impact
- Mentoring and leadership
- Technical strategy

```bash
# Examples to look for:
folio capture "Designed event-driven architecture for order system" \
  --impact "Enabled 10x throughput increase and real-time order tracking"

folio capture "Mentored 3 junior engineers on distributed systems" \
  --impact "All promoted within 18 months"

folio capture "Drove adoption of TypeScript across frontend teams" \
  --impact "Reduced production bugs by 30%"
```

### For Management Positions

Focus on people and process:

```bash
folio capture "Led team of 4 engineers on payments rewrite" \
  --impact "Delivered 2 weeks early, reduced transaction failures by 15%"

folio capture "Established code review standards for the team" \
  --impact "Review turnaround time reduced from 2 days to 4 hours"
```

### For Specific Tech Stacks

Filter mentally by project or technology mentioned in captures:

```bash
folio list --full
# Then search for relevant technologies in the output
```

---

## Creating Your Interview Prep Sheet

Use Folio to build a prep document:

### Step 1: Export Your Top Stories

```bash
# View your best work
folio list --limit 30 --full
```

### Step 2: Categorize Stories

Group by interview question type:
- Technical challenges (3-5 stories)
- Leadership/collaboration (2-3 stories)
- Failure/learning (1-2 stories)
- Impact/results (3-5 stories)

### Step 3: Expand to STAR Format

For each story, write out the full STAR narrative.

### Step 4: Practice

Use your captures as talking points and practice telling each story out loud.

---

## Quick Stats for Interviews

When asked about your experience, Folio stats can help quantify:

```bash
folio stats
```

This helps you say things like:
- "Over the past year, I shipped X features across Y projects"
- "I focused primarily on the payments and infrastructure domains"
- "About 30% of my work was high-impact initiatives"

---

## Tips for Interview Success

### 1. Have 5-7 Strong Stories Ready

Not every capture is an interview story. Select your best ones that:
- Have clear, quantifiable impact
- Demonstrate relevant skills
- Are interesting to tell

### 2. Know Your Metrics

From your captures, be ready to cite specific numbers:
- "Reduced latency by 40%"
- "Saved $50K per month"
- "Improved deployment frequency from weekly to daily"

### 3. Connect to Business Value

Transform technical accomplishments into business terms:

```bash
# Technical capture
folio capture "Implemented caching layer"

# Interview framing
"I implemented a caching layer that reduced our infrastructure
costs by $2K per month while improving user experience with
faster page loads."
```

### 4. Show Growth Over Time

Your Folio history shows progression:
- Earlier captures: Individual contributor work
- Later captures: Larger scope, more leadership

Use this to tell a growth story in interviews.
