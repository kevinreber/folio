use anyhow::Result;
use colored::Colorize;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::db::Database;
use crate::types::Importance;

pub fn run(
    query: String,
    limit: u32,
    project: Option<String>,
    importance: Option<String>,
) -> Result<()> {
    let db = Database::open()?;
    let activities = db.list_activities(None)?;

    if activities.is_empty() {
        println!("{}", "No activities found.".dimmed());
        return Ok(());
    }

    let matcher = SkimMatcherV2::default();
    let query_lower = query.to_lowercase();

    // Filter and score activities
    let mut scored: Vec<_> = activities
        .iter()
        .filter_map(|a| {
            // Apply project filter
            if let Some(ref p) = project {
                if !a
                    .project
                    .as_ref()
                    .map(|ap| ap.to_lowercase().contains(&p.to_lowercase()))
                    .unwrap_or(false)
                {
                    return None;
                }
            }

            // Apply importance filter
            if let Some(ref imp) = importance {
                let filter_importance: Importance = imp.parse().unwrap_or(Importance::Medium);
                if a.importance != filter_importance {
                    return None;
                }
            }

            // Calculate fuzzy match score
            let title_score = matcher.fuzzy_match(&a.title, &query).unwrap_or(0);
            let desc_score = a
                .description
                .as_ref()
                .and_then(|d| matcher.fuzzy_match(d, &query))
                .unwrap_or(0);
            let project_score = a
                .project
                .as_ref()
                .and_then(|p| matcher.fuzzy_match(p, &query))
                .unwrap_or(0);

            let total_score = title_score + desc_score / 2 + project_score / 2;

            // Also check for exact substring matches
            let has_match = a.title.to_lowercase().contains(&query_lower)
                || a.description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
                || a.project
                    .as_ref()
                    .map(|p| p.to_lowercase().contains(&query_lower))
                    .unwrap_or(false);

            if total_score > 0 || has_match {
                Some((
                    a,
                    if has_match {
                        total_score + 100
                    } else {
                        total_score
                    },
                ))
            } else {
                None
            }
        })
        .collect();

    // Sort by score descending
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    // Limit results
    let results: Vec<_> = scored.into_iter().take(limit as usize).collect();

    if results.is_empty() {
        println!("{}", format!("No activities matching '{}'", query).dimmed());
        return Ok(());
    }

    println!(
        "{} {} {}",
        "Found".green().bold(),
        results.len(),
        format!("results for '{}':", query).green()
    );
    println!();

    for (activity, _score) in &results {
        let importance_marker = match activity.importance {
            Importance::High => "●".red(),
            Importance::Medium => "●".yellow(),
            Importance::Low => "●".dimmed(),
        };

        println!(
            "{} {} {}",
            importance_marker,
            format!("[{}]", &activity.id[..8]).dimmed(),
            highlight_match(&activity.title, &query)
        );

        println!(
            "  {} {}",
            "Date:".dimmed(),
            activity.timestamp.format("%Y-%m-%d")
        );

        if let Some(ref project) = activity.project {
            println!("  {} {}", "Project:".dimmed(), project);
        }

        if let Some(ref desc) = activity.description {
            let truncated = if desc.len() > 80 {
                format!("{}...", &desc[..77])
            } else {
                desc.clone()
            };
            println!("  {}", truncated.dimmed());
        }

        println!();
    }

    Ok(())
}

fn highlight_match(text: &str, query: &str) -> String {
    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    if let Some(pos) = text_lower.find(&query_lower) {
        let before = &text[..pos];
        let matched = &text[pos..pos + query.len()];
        let after = &text[pos + query.len()..];
        format!("{}{}{}", before, matched.yellow().bold(), after)
    } else {
        text.to_string()
    }
}
