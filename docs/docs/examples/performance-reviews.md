---
id: performance-reviews
title: Performance Reviews
sidebar_label: Performance Reviews
sidebar_position: 4
---

# Performance Reviews

Use Folio to compile compelling performance review documentation.

## Quarterly Review Workflow

### Step 1: Gather Your Accomplishments

At the end of each quarter, review what you've captured:

```bash
# See all activities
folio list --limit 100 --full

# Get an overview
folio stats
```

### Step 2: Identify Key Themes

Group your accomplishments into categories that align with your company's review criteria. Common themes:

| Theme | What to Include |
|-------|-----------------|
| **Technical Excellence** | Architecture, optimization, complex debugging |
| **Delivery & Execution** | Features shipped, deadlines met, quality |
| **Impact & Results** | Metrics, business outcomes, user improvements |
| **Leadership** | Mentoring, leading projects, driving decisions |
| **Collaboration** | Cross-team work, code reviews, knowledge sharing |

### Step 3: Write Your Self-Review

Pull specific examples from your captures for each theme.

---

## Sample Performance Review Sections

### Technical Excellence

**From your captures:**
```bash
folio capture "Designed and implemented event-driven order system" \
  --impact "Enabled real-time order tracking and 10x throughput" \
  --importance high

folio capture "Optimized database queries across catalog service" \
  --impact "Reduced average query time from 500ms to 50ms"

folio capture "Established API versioning strategy" \
  --impact "Enabled backward-compatible API evolution"
```

**Review text:**
> This quarter I focused on architectural improvements that enhanced our system's scalability and maintainability. Key accomplishments include:
>
> - **Designed event-driven order system** - Created an event-based architecture that enabled real-time order tracking for customers and increased our throughput capacity by 10x, preparing us for the holiday traffic surge.
>
> - **Optimized catalog database queries** - Identified and fixed inefficient query patterns, reducing average query time from 500ms to 50ms (90% improvement). This directly improved page load times for all product pages.
>
> - **Established API versioning strategy** - Proposed and implemented a versioning approach that allows us to evolve our API without breaking existing clients. This unblocked the mobile team's development.

---

### Delivery & Execution

**From your captures:**
```bash
folio capture "Shipped user preferences feature" \
  --project "settings-page" \
  --impact "On time delivery, 100% test coverage"

folio capture "Completed payments refactor ahead of schedule" \
  --project "payments" \
  --impact "Delivered 2 weeks early with zero bugs in production"

folio capture "Fixed 15 bugs during hardening week" \
  --importance medium
```

**Review text:**
> I consistently delivered high-quality work on schedule:
>
> - **User preferences feature** - Delivered the complete settings page on schedule with 100% test coverage. Received positive feedback from users about the new notification controls.
>
> - **Payments refactor** - Completed the payments system refactor 2 weeks ahead of schedule. The new system has been running in production for 6 weeks with zero bugs or incidents.
>
> - **Bug fixes** - Resolved 15 bugs during our Q3 hardening week, including 3 critical issues that had been affecting user experience.

---

### Impact & Results

**From your captures:**
```bash
folio capture "Launched new checkout flow" \
  --impact "Conversion rate increased 8%, $200K additional revenue/month" \
  --importance high

folio capture "Reduced failed payments by 15%" \
  --impact "Recovered approximately $50K/month in lost revenue" \
  --importance high

folio capture "Improved page load time by 60%" \
  --impact "Bounce rate decreased by 12%"
```

**Review text:**
> My work this quarter had significant measurable business impact:
>
> - **New checkout flow** - Led the redesign of our checkout experience, resulting in an 8% increase in conversion rate. This translates to approximately $200K in additional monthly revenue.
>
> - **Payment reliability improvements** - Redesigned our payment retry system, reducing failed transactions by 15% and recovering approximately $50K per month in previously lost revenue.
>
> - **Performance optimization** - Improved page load time by 60%, which correlated with a 12% decrease in bounce rate across the site.

---

### Leadership & Mentorship

**From your captures:**
```bash
folio capture "Mentored 2 junior engineers on backend development" \
  --impact "Both engineers now handling production deploys independently"

folio capture "Led technical design for API v2" \
  --project "api-v2" \
  --impact "Created design doc reviewed by 8 engineers, now in implementation"

folio capture "Gave tech talk on testing best practices" \
  --impact "50+ attendees, positive feedback"
```

**Review text:**
> I contributed to team growth and technical direction:
>
> - **Mentorship** - Provided ongoing mentorship to two junior engineers, focusing on backend development practices, code review skills, and production readiness. Both are now confidently handling production deployments independently.
>
> - **Technical leadership** - Led the technical design for our API v2 initiative. Created a comprehensive design document that was reviewed by 8 engineers and is now guiding our implementation work.
>
> - **Knowledge sharing** - Presented a tech talk on testing best practices to 50+ engineers. The talk resulted in measurable improvements in test coverage across several teams.

---

### Collaboration

**From your captures:**
```bash
folio capture "Coordinated with mobile team on API contract" \
  --project "mobile-api" \
  --impact "Unblocked mobile app development"

folio capture "Reviewed 45 PRs this quarter" \
  --impact "Average review turnaround under 4 hours"

folio capture "Helped resolve production incident for orders team" \
  --impact "Identified root cause, prevented similar issues"
```

**Review text:**
> I actively contributed to cross-team success:
>
> - **Cross-team coordination** - Worked closely with the mobile team to define API contracts for new features. This collaboration unblocked their development and ensured smooth integration.
>
> - **Code review** - Reviewed 45 pull requests this quarter with an average turnaround time under 4 hours. Focused on providing constructive feedback that helps teammates learn and improve.
>
> - **Incident support** - Assisted the orders team during a production incident, helping identify the root cause and implementing preventive measures. Wrote a runbook to help with future incidents.

---

## Quantifying Your Impact

Folio helps you gather the numbers that make reviews compelling:

```bash
folio stats
```

Use the output to quantify:
- Number of features/projects completed
- High vs. medium vs. low priority work
- Projects you contributed to
- Date range of your work

### Turning Stats Into Statements

| Folio Data | Review Statement |
|------------|-----------------|
| "15 high-importance activities" | "Led or significantly contributed to 15 high-impact initiatives" |
| "5 projects" | "Contributed across 5 different projects and teams" |
| "Date range: Jan-Mar" | "Maintained consistent delivery throughout the quarter" |
| "Missing impact: 2" | "Achieved measurable impact on the vast majority of my work" |

---

## Creating Goals From Gaps

Use Folio stats to identify areas for growth:

```bash
folio stats
```

**Example insights:**
- Few high-importance items → Set goal to take on more impactful projects
- All same project → Set goal for cross-team contributions
- Low importance dominates → Focus on graduating to larger scope work
- Missing impact metrics → Get better at capturing outcomes

---

## Review Prep Checklist

1. **Run `folio list --limit 100 --full`** to see all your work
2. **Run `folio stats`** to see patterns and metrics
3. **Group by theme** (technical, delivery, impact, leadership, collaboration)
4. **Pull 3-5 examples per theme** from your captures
5. **Expand each with specific metrics** and context
6. **Connect to business outcomes** where possible
7. **Identify 2-3 areas for growth** based on gaps

---

## Tips for Strong Reviews

### 1. Lead With Impact
Always start with the outcome, then explain what you did:

> ❌ "I implemented a new caching layer using Redis"
>
> ✅ "Reduced page load time by 40% by implementing a Redis caching layer"

### 2. Use Specific Numbers
Vague claims are forgettable; specific metrics are memorable:

> ❌ "Improved performance significantly"
>
> ✅ "Improved query performance from 500ms to 50ms (90% reduction)"

### 3. Show Breadth AND Depth
Include examples of:
- Deep technical work (complexity)
- Broad impact (scope)
- Soft skills (collaboration, leadership)

### 4. Be Honest About Challenges
Mention what you learned from difficulties:

> "The payments migration was more complex than estimated. I learned to build in more time for edge cases and created a checklist for future migrations."

### 5. Connect to Company Goals
Frame accomplishments in terms of what the company values:

> "This work directly supported our Q3 goal of improving customer satisfaction scores."
