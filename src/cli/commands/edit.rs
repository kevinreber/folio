use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Write};

use crate::db::Database;
use crate::types::Importance;

pub fn run(
    id: String,
    title: Option<String>,
    impact: Option<String>,
    project: Option<String>,
    employer: Option<String>,
    importance: Option<String>,
) -> Result<()> {
    let db = Database::open()?;

    // Find the activity
    let activity = db
        .get_activity(&id)?
        .or_else(|| db.get_activity_by_partial_id(&id).ok().flatten())
        .with_context(|| format!("No activity found matching '{}'", id))?;

    // Check if any edits were provided
    let has_edits = title.is_some() || impact.is_some() || project.is_some()
        || employer.is_some() || importance.is_some();

    if !has_edits {
        // Interactive edit mode
        println!("{}", "Interactive edit mode".bold());
        println!("{}", "─".repeat(40).dimmed());
        println!("Current values (press Enter to keep, or type new value):\n");

        let new_title = prompt_edit("Title", &activity.title)?;
        let new_impact = prompt_edit_optional("Impact",
            activity.metadata.get("impact").and_then(|v| v.as_str()))?;
        let new_project = prompt_edit_optional("Project", activity.project.as_deref())?;
        let new_employer = prompt_edit_optional("Employer", activity.employer.as_deref())?;
        let new_importance = prompt_edit("Importance", &activity.importance.to_string())?;

        // Apply updates
        db.update_activity(
            &activity.id,
            Some(&new_title),
            new_impact.as_deref(),
            new_project.as_deref(),
            new_employer.as_deref(),
            Some(&new_importance),
        )?;

        println!();
        println!("{} Updated activity {}", "✓".green().bold(), &activity.id[..8]);
    } else {
        // Direct edit with provided values
        db.update_activity(
            &activity.id,
            title.as_deref(),
            impact.as_deref(),
            project.as_deref(),
            employer.as_deref(),
            importance.as_deref(),
        )?;

        println!("{} Updated activity {}", "✓".green().bold(), &activity.id[..8]);

        if let Some(t) = title {
            println!("  {} {}", "Title:".dimmed(), t);
        }
        if let Some(i) = impact {
            println!("  {} {}", "Impact:".dimmed(), i);
        }
        if let Some(p) = project {
            println!("  {} {}", "Project:".dimmed(), p);
        }
        if let Some(e) = employer {
            println!("  {} {}", "Employer:".dimmed(), e);
        }
        if let Some(i) = importance {
            println!("  {} {}", "Importance:".dimmed(), i);
        }
    }

    Ok(())
}

fn prompt_edit(label: &str, current: &str) -> Result<String> {
    print!("{} [{}]: ", label.cyan(), current);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(current.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn prompt_edit_optional(label: &str, current: Option<&str>) -> Result<Option<String>> {
    let display = current.unwrap_or("(none)");
    print!("{} [{}]: ", label.cyan(), display);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(current.map(String::from))
    } else if trimmed == "-" {
        // Use "-" to clear the value
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}
