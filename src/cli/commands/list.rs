use anyhow::Result;
use colored::Colorize;
use tabled::{settings::Style, Table, Tabled};

use crate::db::Database;
use crate::types::Activity;

#[derive(Tabled)]
struct ActivityRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Date")]
    date: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Project")]
    project: String,
    #[tabled(rename = "Importance")]
    importance: String,
}

impl From<&Activity> for ActivityRow {
    fn from(activity: &Activity) -> Self {
        Self {
            id: activity.id[..8].to_string(),
            date: activity.timestamp.format("%Y-%m-%d").to_string(),
            title: truncate(&activity.title, 40),
            project: activity.project.clone().unwrap_or_else(|| "-".to_string()),
            importance: format_importance(&activity.importance),
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn format_importance(importance: &crate::types::Importance) -> String {
    match importance {
        crate::types::Importance::High => "high".red().to_string(),
        crate::types::Importance::Medium => "medium".yellow().to_string(),
        crate::types::Importance::Low => "low".dimmed().to_string(),
    }
}

pub fn run(limit: u32, full: bool) -> Result<()> {
    let db = Database::open()?;
    let activities = db.list_activities(Some(limit))?;

    if activities.is_empty() {
        println!("{}", "No activities captured yet.".dimmed());
        println!(
            "{}",
            "Use `folio capture \"Your accomplishment\"` to get started.".dimmed()
        );
        return Ok(());
    }

    if full {
        for activity in &activities {
            print_activity_full(activity);
            println!();
        }
    } else {
        let rows: Vec<ActivityRow> = activities.iter().map(ActivityRow::from).collect();
        let table = Table::new(rows).with(Style::rounded()).to_string();
        println!("{}", table);
    }

    let total = db.count_activities()?;
    if total > limit as i64 {
        println!(
            "{}",
            format!("Showing {} of {} activities", limit, total).dimmed()
        );
    }

    Ok(())
}

fn print_activity_full(activity: &Activity) {
    println!("{} {}", "ID:".bold(), &activity.id[..8]);
    println!("{} {}", "Title:".bold(), activity.title);
    println!(
        "{} {}",
        "Date:".bold(),
        activity.timestamp.format("%Y-%m-%d %H:%M")
    );
    println!("{} {}", "Source:".bold(), activity.source);
    println!("{} {}", "Type:".bold(), activity.activity_type);
    println!("{} {}", "Importance:".bold(), activity.importance);

    if let Some(ref desc) = activity.description {
        println!("{} {}", "Description:".bold(), desc);
    }

    if let Some(ref project) = activity.project {
        println!("{} {}", "Project:".bold(), project);
    }

    if let Some(ref employer) = activity.employer {
        println!("{} {}", "Employer:".bold(), employer);
    }

    // Show impact from metadata if present
    if let Some(impact) = activity.metadata.get("impact") {
        if let Some(impact_str) = impact.as_str() {
            println!("{} {}", "Impact:".bold(), impact_str);
        }
    }
}
