use anyhow::Result;
use colored::Colorize;

use crate::db::Database;

pub fn run() -> Result<()> {
    let db = Database::open()?;

    let total = db.count_activities()?;
    let activities = db.list_activities(None)?;

    println!("{}", "Folio Statistics".bold());
    println!("{}", "─".repeat(40).dimmed());
    println!();

    println!("{} {}", "Total activities:".cyan(), total);

    if total == 0 {
        println!();
        println!("{}", "Get started with:".dimmed());
        println!(
            "  {}",
            "folio capture \"Your accomplishment\" --impact \"The result\"".dimmed()
        );
        return Ok(());
    }

    // Count by importance
    let high = activities
        .iter()
        .filter(|a| matches!(a.importance, crate::types::Importance::High))
        .count();
    let medium = activities
        .iter()
        .filter(|a| matches!(a.importance, crate::types::Importance::Medium))
        .count();
    let low = activities
        .iter()
        .filter(|a| matches!(a.importance, crate::types::Importance::Low))
        .count();

    println!();
    println!("{}", "By importance:".cyan());
    println!("  {} {}", "High:".red(), high);
    println!("  {} {}", "Medium:".yellow(), medium);
    println!("  {} {}", "Low:".dimmed(), low);

    // Count by source
    let by_source = activities
        .iter()
        .fold(std::collections::HashMap::new(), |mut acc, a| {
            *acc.entry(a.source.to_string()).or_insert(0) += 1;
            acc
        });

    println!();
    println!("{}", "By source:".cyan());
    for (source, count) in by_source {
        println!("  {} {}", format!("{}:", source).dimmed(), count);
    }

    // Count by project
    let by_project: std::collections::HashMap<String, i32> =
        activities
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, a| {
                let project = a.project.clone().unwrap_or_else(|| "(none)".to_string());
                *acc.entry(project).or_insert(0) += 1;
                acc
            });

    if by_project.len() > 1 || !by_project.contains_key("(none)") {
        println!();
        println!("{}", "By project:".cyan());
        for (project, count) in by_project {
            println!("  {} {}", format!("{}:", project).dimmed(), count);
        }
    }

    // Date range
    if let (Some(first), Some(last)) = (activities.last(), activities.first()) {
        println!();
        println!("{}", "Date range:".cyan());
        println!(
            "  {} to {}",
            first.timestamp.format("%Y-%m-%d"),
            last.timestamp.format("%Y-%m-%d")
        );
    }

    // Activities missing impact
    let missing_impact = activities
        .iter()
        .filter(|a| a.metadata.get("impact").is_none() && a.description.is_none())
        .count();

    if missing_impact > 0 {
        println!();
        println!(
            "{} {} activities missing impact/description",
            "⚠".yellow(),
            missing_impact
        );
        println!(
            "  {}",
            "Consider enriching these for stronger resume bullets".dimmed()
        );
    }

    Ok(())
}
