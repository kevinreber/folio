use anyhow::{Context, Result};
use colored::Colorize;

use crate::db::Database;
use crate::types::Activity;

pub fn run(id: String) -> Result<()> {
    let db = Database::open()?;

    // Try exact match first, then partial
    let activity = db
        .get_activity(&id)?
        .or_else(|| db.get_activity_by_partial_id(&id).ok().flatten())
        .with_context(|| format!("No activity found matching '{}'", id))?;

    print_activity_detail(&activity);

    Ok(())
}

fn print_activity_detail(activity: &Activity) {
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", activity.title.bold());
    println!("{}", "─".repeat(60).dimmed());

    println!();
    println!("{} {}", "ID:".cyan(), activity.id);
    println!(
        "{} {}",
        "Captured:".cyan(),
        activity.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("{} {}", "Source:".cyan(), activity.source);
    println!("{} {}", "Type:".cyan(), activity.activity_type);
    println!(
        "{} {}",
        "Importance:".cyan(),
        format_importance(&activity.importance)
    );

    println!();

    if let Some(ref desc) = activity.description {
        println!("{}", "Description:".cyan());
        println!("  {}", desc);
        println!();
    }

    if let Some(ref project) = activity.project {
        println!("{} {}", "Project:".cyan(), project);
    }

    if let Some(ref employer) = activity.employer {
        println!("{} {}", "Employer:".cyan(), employer);
    }

    // Show impact from metadata if present
    if let Some(impact) = activity.metadata.get("impact") {
        if let Some(impact_str) = impact.as_str() {
            println!();
            println!("{}", "Impact:".cyan());
            println!("  {}", impact_str.green());
        }
    }

    // Show any other metadata
    if activity.metadata.as_object().map(|o| o.len()).unwrap_or(0) > 1
        || (activity.metadata.as_object().map(|o| o.len()).unwrap_or(0) == 1
            && activity.metadata.get("impact").is_none())
    {
        println!();
        println!("{}", "Metadata:".cyan());
        for (key, value) in activity.metadata.as_object().unwrap_or(&serde_json::Map::new()) {
            if key != "impact" {
                println!("  {}: {}", key, value);
            }
        }
    }

    println!();
    println!("{}", "─".repeat(60).dimmed());

    // Hints for next actions
    println!();
    println!("{}", "Next steps:".dimmed());
    println!(
        "  {} {}",
        "•".dimmed(),
        "Add to an accomplishment: folio promote <id>".dimmed()
    );
    println!(
        "  {} {}",
        "•".dimmed(),
        format!("Delete: folio delete {}", &activity.id[..8]).dimmed()
    );
}

fn format_importance(importance: &crate::types::Importance) -> String {
    match importance {
        crate::types::Importance::High => "high".red().bold().to_string(),
        crate::types::Importance::Medium => "medium".yellow().to_string(),
        crate::types::Importance::Low => "low".dimmed().to_string(),
    }
}
