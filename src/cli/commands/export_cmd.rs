use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

use crate::db::Database;
use crate::export::{markdown, ExportFormat, Exporter};

pub fn run(format: String, output: Option<PathBuf>, brag: bool, bullets: bool) -> Result<()> {
    let db = Database::open()?;
    let activities = db.list_activities(None)?;
    let accomplishments = db.list_accomplishments()?;

    if activities.is_empty() && accomplishments.is_empty() {
        println!("{}", "No data to export.".dimmed());
        return Ok(());
    }

    let content = if brag {
        // Export as brag document
        let export_data = crate::export::ExportData {
            exported_at: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            activities: activities.clone(),
            accomplishments: accomplishments.clone(),
            metadata: crate::export::ExportMetadata {
                total_activities: activities.len(),
                total_accomplishments: accomplishments.len(),
                date_range: None,
                projects: vec![],
                employers: vec![],
            },
        };
        markdown::export_brag_doc(&export_data)?
    } else if bullets {
        // Export as resume bullets
        let export_data = crate::export::ExportData {
            exported_at: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            activities: activities.clone(),
            accomplishments: accomplishments.clone(),
            metadata: crate::export::ExportMetadata {
                total_activities: activities.len(),
                total_accomplishments: accomplishments.len(),
                date_range: None,
                projects: vec![],
                employers: vec![],
            },
        };
        markdown::export_resume_bullets(&export_data)?
    } else {
        // Regular export
        let export_format: ExportFormat = format.parse()?;
        Exporter::export(activities, accomplishments, export_format)?
    };

    // Output to file or stdout
    if let Some(path) = output {
        std::fs::write(&path, &content)?;
        println!("{} Exported to {}", "✓".green().bold(), path.display());
        println!("  {} bytes written", content.len());
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// Import activities from a file
pub fn import(path: PathBuf, dry_run: bool) -> Result<()> {
    use crate::export::Importer;

    println!("{} {}", "Importing from".cyan(), path.display());

    let activities = Importer::from_file(&path)?;

    if activities.is_empty() {
        println!("{}", "No activities found in file.".dimmed());
        return Ok(());
    }

    println!("Found {} activities", activities.len());

    if dry_run {
        println!();
        println!(
            "{}",
            "Dry run - activities that would be imported:".yellow()
        );
        for activity in &activities {
            println!("  {} {}", "•".dimmed(), activity.title);
        }
        println!();
        println!("{}", "Run without --dry-run to actually import.".dimmed());
    } else {
        let db = Database::open()?;
        let mut imported = 0;

        for activity in activities {
            if db.insert_activity(&activity).is_ok() {
                imported += 1;
            }
        }

        println!("{} Imported {} activities", "✓".green().bold(), imported);
    }

    Ok(())
}
