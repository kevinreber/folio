use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{Activity, Importance};

/// Generator for periodic digests and summaries
pub struct DigestGenerator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Digest {
    pub period: DigestPeriod,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub summary: String,
    pub highlights: Vec<DigestHighlight>,
    pub stats: DigestStats,
    pub by_project: HashMap<String, Vec<ActivitySummary>>,
    pub skills_used: Vec<String>,
    pub themes: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DigestPeriod {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestHighlight {
    pub activity_id: String,
    pub title: String,
    pub impact: Option<String>,
    pub importance: Importance,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DigestStats {
    pub total_activities: usize,
    pub high_importance: usize,
    pub medium_importance: usize,
    pub low_importance: usize,
    pub projects_touched: usize,
    pub by_type: HashMap<String, usize>,
    pub by_source: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySummary {
    pub id: String,
    pub title: String,
    pub importance: Importance,
    pub timestamp: DateTime<Utc>,
}

impl Default for DigestGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl DigestGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a digest for a specific period
    pub fn generate(
        &self,
        activities: &[Activity],
        period: DigestPeriod,
        end_date: Option<DateTime<Utc>>,
    ) -> Digest {
        let end = end_date.unwrap_or_else(Utc::now);
        let start = self.calculate_start_date(&end, period);

        // Filter activities within the period
        let period_activities: Vec<&Activity> = activities
            .iter()
            .filter(|a| a.timestamp >= start && a.timestamp <= end)
            .collect();

        let stats = self.calculate_stats(&period_activities);
        let highlights = self.extract_highlights(&period_activities);
        let by_project = self.group_by_project(&period_activities);
        let (skills_used, themes) = self.extract_skills_and_themes(&period_activities);
        let summary = self.generate_summary(&period_activities, period, &stats);

        Digest {
            period,
            start_date: start,
            end_date: end,
            summary,
            highlights,
            stats,
            by_project,
            skills_used,
            themes,
        }
    }

    /// Generate a weekly digest (convenience method)
    pub fn weekly(&self, activities: &[Activity]) -> Digest {
        self.generate(activities, DigestPeriod::Weekly, None)
    }

    /// Generate a monthly digest (convenience method)
    pub fn monthly(&self, activities: &[Activity]) -> Digest {
        self.generate(activities, DigestPeriod::Monthly, None)
    }

    fn calculate_start_date(&self, end: &DateTime<Utc>, period: DigestPeriod) -> DateTime<Utc> {
        match period {
            DigestPeriod::Daily => *end - Duration::days(1),
            DigestPeriod::Weekly => *end - Duration::weeks(1),
            DigestPeriod::Monthly => *end - Duration::days(30),
            DigestPeriod::Quarterly => *end - Duration::days(90),
            DigestPeriod::Yearly => *end - Duration::days(365),
            DigestPeriod::Custom => *end - Duration::weeks(1), // Default to weekly for custom
        }
    }

    fn calculate_stats(&self, activities: &[&Activity]) -> DigestStats {
        let mut stats = DigestStats::default();
        let mut projects = std::collections::HashSet::new();

        for activity in activities {
            stats.total_activities += 1;

            match activity.importance {
                Importance::High => stats.high_importance += 1,
                Importance::Medium => stats.medium_importance += 1,
                Importance::Low => stats.low_importance += 1,
            }

            *stats
                .by_type
                .entry(activity.activity_type.to_string())
                .or_insert(0) += 1;
            *stats
                .by_source
                .entry(activity.source.to_string())
                .or_insert(0) += 1;

            if let Some(project) = &activity.project {
                projects.insert(project.clone());
            }
        }

        stats.projects_touched = projects.len();
        stats
    }

    fn extract_highlights(&self, activities: &[&Activity]) -> Vec<DigestHighlight> {
        let mut highlights: Vec<DigestHighlight> = activities
            .iter()
            .filter(|a| matches!(a.importance, Importance::High))
            .map(|a| DigestHighlight {
                activity_id: a.id.clone(),
                title: a.title.clone(),
                impact: a
                    .metadata
                    .get("impact")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                importance: a.importance.clone(),
            })
            .collect();

        // Sort by timestamp, newest first
        highlights.sort_by(|a, b| {
            let a_activity = activities.iter().find(|act| act.id == a.activity_id);
            let b_activity = activities.iter().find(|act| act.id == b.activity_id);
            match (a_activity, b_activity) {
                (Some(a), Some(b)) => b.timestamp.cmp(&a.timestamp),
                _ => std::cmp::Ordering::Equal,
            }
        });

        // Limit to top 5 highlights
        highlights.truncate(5);
        highlights
    }

    fn group_by_project(&self, activities: &[&Activity]) -> HashMap<String, Vec<ActivitySummary>> {
        let mut grouped: HashMap<String, Vec<ActivitySummary>> = HashMap::new();

        for activity in activities {
            let project = activity
                .project
                .clone()
                .unwrap_or_else(|| "(No Project)".to_string());

            grouped.entry(project).or_default().push(ActivitySummary {
                id: activity.id.clone(),
                title: activity.title.clone(),
                importance: activity.importance.clone(),
                timestamp: activity.timestamp,
            });
        }

        // Sort each project's activities by timestamp
        for summaries in grouped.values_mut() {
            summaries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        }

        grouped
    }

    fn extract_skills_and_themes(&self, activities: &[&Activity]) -> (Vec<String>, Vec<String>) {
        use crate::ai::AutoTagger;

        let tagger = AutoTagger::new();
        let owned_activities: Vec<Activity> = activities.iter().map(|a| (*a).clone()).collect();
        let tag_result = tagger.tag_batch(&owned_activities);

        (tag_result.technical_skills, tag_result.themes)
    }

    fn generate_summary(
        &self,
        _activities: &[&Activity],
        period: DigestPeriod,
        stats: &DigestStats,
    ) -> String {
        let period_name = match period {
            DigestPeriod::Daily => "day",
            DigestPeriod::Weekly => "week",
            DigestPeriod::Monthly => "month",
            DigestPeriod::Quarterly => "quarter",
            DigestPeriod::Yearly => "year",
            DigestPeriod::Custom => "period",
        };

        if stats.total_activities == 0 {
            return format!("No activities captured this {}.", period_name);
        }

        let mut summary_parts = Vec::new();

        // Activity count
        summary_parts.push(format!(
            "You captured {} {} this {}.",
            stats.total_activities,
            if stats.total_activities == 1 {
                "activity"
            } else {
                "activities"
            },
            period_name
        ));

        // Highlights
        if stats.high_importance > 0 {
            summary_parts.push(format!(
                "{} {} high-importance.",
                stats.high_importance,
                if stats.high_importance == 1 {
                    "was"
                } else {
                    "were"
                }
            ));
        }

        // Projects
        if stats.projects_touched > 1 {
            summary_parts.push(format!(
                "You worked across {} projects.",
                stats.projects_touched
            ));
        }

        // Top activity types
        if let Some((top_type, count)) = stats.by_type.iter().max_by_key(|(_, c)| *c) {
            if *count > 1 {
                summary_parts.push(format!(
                    "Most common activity: {} ({} times).",
                    top_type, count
                ));
            }
        }

        summary_parts.join(" ")
    }

    /// Format digest as markdown
    pub fn to_markdown(&self, digest: &Digest) -> String {
        let mut md = String::new();

        // Header
        let period_name = match digest.period {
            DigestPeriod::Daily => "Daily",
            DigestPeriod::Weekly => "Weekly",
            DigestPeriod::Monthly => "Monthly",
            DigestPeriod::Quarterly => "Quarterly",
            DigestPeriod::Yearly => "Yearly",
            DigestPeriod::Custom => "Custom",
        };

        md.push_str(&format!("# {} Digest\n\n", period_name));
        md.push_str(&format!(
            "**Period:** {} to {}\n\n",
            digest.start_date.format("%Y-%m-%d"),
            digest.end_date.format("%Y-%m-%d")
        ));

        // Summary
        md.push_str("## Summary\n\n");
        md.push_str(&digest.summary);
        md.push_str("\n\n");

        // Stats
        md.push_str("## Statistics\n\n");
        md.push_str(&format!(
            "- **Total Activities:** {}\n",
            digest.stats.total_activities
        ));
        md.push_str(&format!(
            "- **High Importance:** {}\n",
            digest.stats.high_importance
        ));
        md.push_str(&format!(
            "- **Medium Importance:** {}\n",
            digest.stats.medium_importance
        ));
        md.push_str(&format!(
            "- **Low Importance:** {}\n",
            digest.stats.low_importance
        ));
        md.push_str(&format!(
            "- **Projects:** {}\n",
            digest.stats.projects_touched
        ));
        md.push('\n');

        // Highlights
        if !digest.highlights.is_empty() {
            md.push_str("## Highlights\n\n");
            for highlight in &digest.highlights {
                md.push_str(&format!("- **{}**", highlight.title));
                if let Some(impact) = &highlight.impact {
                    md.push_str(&format!(" - {}", impact));
                }
                md.push('\n');
            }
            md.push('\n');
        }

        // By Project
        if !digest.by_project.is_empty() {
            md.push_str("## By Project\n\n");
            for (project, activities) in &digest.by_project {
                md.push_str(&format!("### {}\n\n", project));
                for activity in activities {
                    let importance_marker = match activity.importance {
                        Importance::High => "🔴",
                        Importance::Medium => "🟡",
                        Importance::Low => "⚪",
                    };
                    md.push_str(&format!("- {} {}\n", importance_marker, activity.title));
                }
                md.push('\n');
            }
        }

        // Skills
        if !digest.skills_used.is_empty() {
            md.push_str("## Skills Used\n\n");
            md.push_str(&digest.skills_used.join(", "));
            md.push_str("\n\n");
        }

        // Themes
        if !digest.themes.is_empty() {
            md.push_str("## Themes\n\n");
            md.push_str(&digest.themes.join(", "));
            md.push('\n');
        }

        md
    }
}

/// Generate a performance review summary from activities
pub fn generate_performance_review(activities: &[Activity], period_months: u32) -> String {
    let generator = DigestGenerator::new();
    let end = Utc::now();
    let start = end - Duration::days((period_months * 30) as i64);

    let period_activities: Vec<&Activity> = activities
        .iter()
        .filter(|a| a.timestamp >= start && a.timestamp <= end)
        .collect();

    if period_activities.is_empty() {
        return "No activities found for the review period.".to_string();
    }

    let stats = generator.calculate_stats(&period_activities);
    let highlights = generator.extract_highlights(&period_activities);

    let mut review = String::new();

    review.push_str("# Performance Review Summary\n\n");
    review.push_str(&format!("## Overview ({} months)\n\n", period_months));
    review.push_str(&format!(
        "During this period, I completed {} significant activities across {} projects. ",
        stats.total_activities, stats.projects_touched
    ));
    review.push_str(&format!(
        "{} were high-impact contributions.\n\n",
        stats.high_importance
    ));

    if !highlights.is_empty() {
        review.push_str("## Key Accomplishments\n\n");
        for (i, highlight) in highlights.iter().enumerate() {
            review.push_str(&format!("{}. **{}**", i + 1, highlight.title));
            if let Some(impact) = &highlight.impact {
                review.push_str(&format!("\n   - Impact: {}", impact));
            }
            review.push_str("\n\n");
        }
    }

    review
}
