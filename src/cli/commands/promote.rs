use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Write};

use crate::ai::{AutoTagger, BulletGenerator, StarBuilder};
use crate::db::Database;
use crate::types::Metric;

pub fn run(id: String, interactive: bool) -> Result<()> {
    let db = Database::open()?;

    // Find the activity
    let activity = db
        .get_activity(&id)?
        .or_else(|| db.get_activity_by_partial_id(&id).ok().flatten())
        .with_context(|| format!("No activity found matching '{}'", id))?;

    println!("{}", "Promoting activity to accomplishment".bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();
    println!("{} {}", "Activity:".cyan(), activity.title);
    println!();

    // Get tags for the activity
    let tagger = AutoTagger::new();
    let tags = tagger.tag(&activity);

    println!(
        "{} {}",
        "Detected skills:".dimmed(),
        tags.technical_skills.join(", ")
    );
    println!("{} {}", "Detected themes:".dimmed(), tags.themes.join(", "));
    println!();

    // Build STAR story
    let star_builder = StarBuilder::new();

    let (situation, task, action, result) = if interactive {
        // Interactive prompts
        println!("{}", "Let's build your STAR story:".yellow().bold());
        println!();

        let prompts = star_builder.get_prompts();

        println!("{}", "SITUATION".cyan().bold());
        println!("{}", prompts.situation[0].dimmed());
        let situation = prompt_multiline()?;

        println!();
        println!("{}", "TASK".cyan().bold());
        println!("{}", prompts.task[0].dimmed());
        let task = prompt_multiline()?;

        println!();
        println!("{}", "ACTION".cyan().bold());
        println!("{}", prompts.action[0].dimmed());
        let action = prompt_multiline()?;

        println!();
        println!("{}", "RESULT".cyan().bold());
        println!("{}", prompts.result[0].dimmed());
        let result = prompt_multiline()?;

        (situation, task, action, result)
    } else {
        // Auto-generate from activity
        let draft = star_builder.draft_from_activity(&activity);

        let situation = draft.situation_hints.first().cloned().unwrap_or_else(|| {
            format!(
                "While working on {}",
                activity.project.as_deref().unwrap_or("the project")
            )
        });

        let task = draft
            .task_hints
            .first()
            .cloned()
            .unwrap_or_else(|| activity.title.clone());

        let action = draft
            .action_hints
            .first()
            .cloned()
            .unwrap_or_else(|| activity.description.clone().unwrap_or_default());

        let result = draft.result_hints.first().cloned().unwrap_or_else(|| {
            activity
                .metadata
                .get("impact")
                .and_then(|v| v.as_str())
                .unwrap_or("Successfully completed the task")
                .to_string()
        });

        println!("{}", "Auto-generated STAR story:".yellow());
        println!();
        println!("{} {}", "Situation:".cyan(), situation);
        println!("{} {}", "Task:".cyan(), task);
        println!("{} {}", "Action:".cyan(), action);
        println!("{} {}", "Result:".cyan(), result);
        println!();

        (situation, task, action, result)
    };

    // Collect metrics
    let mut metrics = Vec::new();
    if let Some(lines) = activity
        .metadata
        .get("lines_changed")
        .and_then(|v| v.as_u64())
    {
        if lines > 50 {
            metrics.push(Metric {
                metric_type: "absolute".to_string(),
                value: lines as f64,
                description: "lines of code".to_string(),
            });
        }
    }

    // Build the story
    let story = star_builder.build_story(
        situation,
        task,
        action,
        result,
        metrics,
        tags.technical_skills.clone(),
    );

    // Generate bullets
    let _bullet_gen = BulletGenerator::new();
    let accomplishment = star_builder.to_accomplishment(&story, vec![activity.id.clone()]);

    // Save accomplishment
    db.insert_accomplishment(&accomplishment)?;

    println!("{}", "─".repeat(50).dimmed());
    println!("{} Created accomplishment", "✓".green().bold());
    println!("  {} {}", "ID:".dimmed(), &accomplishment.id[..8]);
    println!();

    // Show generated bullets
    if !accomplishment.generated_bullets.is_empty() {
        println!("{}", "Generated resume bullets:".yellow().bold());
        for bullet in &accomplishment.generated_bullets {
            println!("  {} {}", "•".green(), bullet);
        }
        println!();
    }

    // Show the narrative
    if let Some(ref story) = accomplishment.generated_story {
        println!("{}", "Full narrative:".yellow().bold());
        println!("{}", story.dimmed());
    }

    Ok(())
}

fn prompt_multiline() -> Result<String> {
    print!("{}", "> ".dimmed());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}
