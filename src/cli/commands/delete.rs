use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Write};

use crate::db::Database;

pub fn run(id: String, force: bool) -> Result<()> {
    let db = Database::open()?;

    // Find the activity first
    let activity = db
        .get_activity(&id)?
        .or_else(|| db.get_activity_by_partial_id(&id).ok().flatten())
        .with_context(|| format!("No activity found matching '{}'", id))?;

    // Confirm deletion unless --force
    if !force {
        println!("{}", "About to delete:".yellow());
        println!("  {} {}", "ID:".dimmed(), &activity.id[..8]);
        println!("  {} {}", "Title:".dimmed(), activity.title);
        println!();

        print!("{}", "Are you sure? [y/N] ".yellow());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Cancelled.".dimmed());
            return Ok(());
        }
    }

    // Delete
    let deleted = db.delete_activity(&activity.id)?;

    if deleted {
        println!(
            "{} Deleted activity {}",
            "✓".green().bold(),
            &activity.id[..8]
        );
    } else {
        println!("{} Activity not found", "✗".red().bold());
    }

    Ok(())
}
