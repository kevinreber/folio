use anyhow::Result;
use colored::Colorize;
use std::io::{self, Read};

use crate::ai::JobMatcher;
use crate::db::Database;

pub fn run(
    job_title: String,
    description: Option<String>,
    file: Option<std::path::PathBuf>,
) -> Result<()> {
    let db = Database::open()?;
    let activities = db.list_activities(None)?;

    if activities.is_empty() {
        println!("{}", "No activities to match against.".dimmed());
        println!(
            "{}",
            "Capture some activities first with `folio capture`".dimmed()
        );
        return Ok(());
    }

    // Get job description from file, argument, or stdin
    let job_description = if let Some(path) = file {
        std::fs::read_to_string(&path)?
    } else if let Some(desc) = description {
        desc
    } else {
        println!(
            "{}",
            "Paste the job description (end with Ctrl+D or Ctrl+Z):".yellow()
        );
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    if job_description.trim().is_empty() {
        println!("{}", "No job description provided.".yellow());
        return Ok(());
    }

    println!();
    println!("{} {}", "Matching against:".cyan(), job_title);
    println!("{}", "─".repeat(50).dimmed());

    let matcher = JobMatcher::new();
    let job = matcher.parse_job_description(job_title.clone(), &job_description);
    let result = matcher.match_activities(&job, &activities);

    // Overall score
    let score_pct = (result.overall_score * 100.0) as u32;
    let score_color = if score_pct >= 70 {
        "green"
    } else if score_pct >= 50 {
        "yellow"
    } else {
        "red"
    };

    println!();
    println!(
        "{} {}%",
        "Overall Match Score:".bold(),
        match score_color {
            "green" => score_pct.to_string().green().bold(),
            "yellow" => score_pct.to_string().yellow().bold(),
            _ => score_pct.to_string().red().bold(),
        }
    );
    println!();

    // Requirements
    if !result.matched_requirements.is_empty() {
        println!("{}", "Requirements".yellow().bold());
        for req in &result.matched_requirements {
            let status = if req.matched {
                "✓".green()
            } else {
                "✗".red()
            };
            println!("  {} {}", status, req.requirement);
            if req.matched {
                let confidence = (req.confidence * 100.0) as u32;
                println!("    {} {}% confidence", "".dimmed(), confidence);
            }
        }
        println!();
    }

    // Preferred qualifications
    let matched_preferred: Vec<_> = result
        .matched_preferred
        .iter()
        .filter(|p| p.matched)
        .collect();
    if !matched_preferred.is_empty() {
        println!("{}", "Preferred Qualifications Met".yellow().bold());
        for pref in matched_preferred {
            println!("  {} {}", "✓".green(), pref.requirement);
        }
        println!();
    }

    // Gaps
    if !result.gaps.is_empty() {
        println!("{}", "Gaps to Address".red().bold());
        for gap in &result.gaps {
            println!("  {} {}", "•".red(), gap);
        }
        println!();
    }

    // Talking points
    if !result.talking_points.is_empty() {
        println!("{}", "Suggested Talking Points".green().bold());
        for point in &result.talking_points {
            println!("  {} {}", "•".cyan(), point.topic);
            println!("    {}", point.point.dimmed());
        }
        println!();
    }

    // Recommended activities
    if !result.recommended_activities.is_empty() {
        println!("{}", "Activities to Highlight".yellow().bold());
        println!("{}", "Review these before your interview:".dimmed());
        for id in &result.recommended_activities {
            if let Some(activity) = activities.iter().find(|a| &a.id == id) {
                println!("  {} [{}] {}", "•".dimmed(), &id[..8], activity.title);
            }
        }
    }

    Ok(())
}
