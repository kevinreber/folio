use anyhow::Result;
use colored::Colorize;

use crate::db::Database;
use crate::ai::{DigestGenerator, digest::DigestPeriod};

pub fn run(period: String, markdown: bool) -> Result<()> {
    let db = Database::open()?;
    let activities = db.list_activities(None)?;

    if activities.is_empty() {
        println!("{}", "No activities to generate digest from.".dimmed());
        println!("{}", "Capture some activities first with `folio capture`".dimmed());
        return Ok(());
    }

    let digest_period = match period.to_lowercase().as_str() {
        "daily" | "day" | "today" => DigestPeriod::Daily,
        "weekly" | "week" => DigestPeriod::Weekly,
        "monthly" | "month" => DigestPeriod::Monthly,
        "quarterly" | "quarter" => DigestPeriod::Quarterly,
        "yearly" | "year" => DigestPeriod::Yearly,
        _ => DigestPeriod::Weekly,
    };

    let generator = DigestGenerator::new();
    let digest = generator.generate(&activities, digest_period, None);

    if markdown {
        println!("{}", generator.to_markdown(&digest));
    } else {
        // Pretty terminal output
        let period_name = match digest_period {
            DigestPeriod::Daily => "Daily",
            DigestPeriod::Weekly => "Weekly",
            DigestPeriod::Monthly => "Monthly",
            DigestPeriod::Quarterly => "Quarterly",
            DigestPeriod::Yearly => "Yearly",
            DigestPeriod::Custom => "Custom",
        };

        println!("{} {}", period_name.bold(), "Digest".bold());
        println!("{}", "─".repeat(50).dimmed());
        println!();

        // Period
        println!(
            "{} {} to {}",
            "Period:".cyan(),
            digest.start_date.format("%Y-%m-%d"),
            digest.end_date.format("%Y-%m-%d")
        );
        println!();

        // Summary
        println!("{}", digest.summary);
        println!();

        // Stats
        println!("{}", "Statistics".yellow().bold());
        println!("  {} {}", "Total activities:".dimmed(), digest.stats.total_activities);
        println!(
            "  {} {} high, {} medium, {} low",
            "By importance:".dimmed(),
            digest.stats.high_importance.to_string().red(),
            digest.stats.medium_importance.to_string().yellow(),
            digest.stats.low_importance.to_string().dimmed()
        );
        println!("  {} {}", "Projects touched:".dimmed(), digest.stats.projects_touched);
        println!();

        // Highlights
        if !digest.highlights.is_empty() {
            println!("{}", "Highlights".yellow().bold());
            for highlight in &digest.highlights {
                println!("  {} {}", "•".green(), highlight.title);
                if let Some(ref impact) = highlight.impact {
                    println!("    {}", impact.dimmed());
                }
            }
            println!();
        }

        // Skills
        if !digest.skills_used.is_empty() {
            println!("{}", "Skills Used".yellow().bold());
            println!("  {}", digest.skills_used.join(", ").dimmed());
            println!();
        }

        // By Project
        if digest.by_project.len() > 1 || !digest.by_project.contains_key("(No Project)") {
            println!("{}", "By Project".yellow().bold());
            for (project, activities) in &digest.by_project {
                println!("  {} ({} activities)", project.cyan(), activities.len());
            }
        }
    }

    Ok(())
}

/// Generate a performance review summary
pub fn review(months: u32) -> Result<()> {
    let db = Database::open()?;
    let activities = db.list_activities(None)?;

    if activities.is_empty() {
        println!("{}", "No activities to generate review from.".dimmed());
        return Ok(());
    }

    let summary = crate::ai::digest::generate_performance_review(&activities, months);
    println!("{}", summary);

    Ok(())
}
