use anyhow::Result;
use colored::Colorize;

use crate::db::Database;
use crate::types::{Activity, Importance};

pub fn run(
    title: String,
    impact: Option<String>,
    project: Option<String>,
    employer: Option<String>,
    importance: String,
) -> Result<()> {
    let db = Database::open()?;

    let importance: Importance = importance.parse().unwrap_or(Importance::Medium);

    let mut activity = Activity::new_manual(&title).with_importance(importance);

    if let Some(impact) = impact {
        activity = activity.with_impact(&impact);
        activity.description = Some(impact);
    }

    if let Some(project) = project {
        activity = activity.with_project(project);
    }

    if let Some(employer) = employer {
        activity = activity.with_employer(employer);
    }

    let id = activity.id.clone();
    db.insert_activity(&activity)?;

    println!("{} Captured activity", "✓".green().bold());
    println!("  {} {}", "ID:".dimmed(), &id[..8]);
    println!("  {} {}", "Title:".dimmed(), title);

    if let Some(ref desc) = activity.description {
        println!("  {} {}", "Impact:".dimmed(), desc);
    }

    if let Some(ref proj) = activity.project {
        println!("  {} {}", "Project:".dimmed(), proj);
    }

    Ok(())
}
