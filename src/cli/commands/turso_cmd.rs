use anyhow::Result;
use colored::Colorize;

use crate::cli::TursoAction;
use crate::turso::{ActivityRecord, SessionRecord, SummaryRecord, TursoDB};

pub fn run(action: TursoAction) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_async(action))
}

async fn run_async(action: TursoAction) -> Result<()> {
    match action {
        TursoAction::Init => {
            let turso = TursoDB::connect().await?;
            let (sessions, activities, summaries, goals) = turso.stats().await?;
            println!("{} Turso database initialized", "ok".green());
            println!(
                "  {} sessions, {} activities, {} summaries, {} goals",
                sessions, activities, summaries, goals
            );
            Ok(())
        }

        TursoAction::WriteSession {
            session_id,
            date,
            project,
            project_path,
            prompt_count,
            duration_minutes,
            is_personal,
            first_prompt_at,
            last_prompt_at,
            metadata,
        } => {
            let turso = TursoDB::connect().await?;
            let record = SessionRecord {
                session_id,
                date,
                project: project.clone(),
                project_path,
                prompt_count,
                duration_minutes,
                is_personal,
                device_id: String::new(), // filled by TursoDB
                first_prompt_at,
                last_prompt_at,
                metadata,
                created_at: None,
            };
            turso.write_session(&record).await?;
            turso.sync().await?;
            println!(
                "{} Wrote session: {} ({} prompts)",
                "ok".green(),
                project,
                prompt_count
            );
            Ok(())
        }

        TursoAction::WriteActivity {
            date,
            category,
            source,
            title,
            detail,
            url,
            metadata,
            tag,
        } => {
            let turso = TursoDB::connect().await?;
            turso
                .write_activity(&ActivityRecord {
                    date,
                    category: category.clone(),
                    source: source.clone(),
                    title: title.clone(),
                    detail,
                    url,
                    metadata,
                    tag,
                })
                .await?;
            turso.sync().await?;
            println!(
                "{} Wrote activity: [{}] {} - {}",
                "ok".green(),
                source,
                category,
                title
            );
            Ok(())
        }

        TursoAction::WriteSummary {
            date,
            period,
            summary_type,
            content,
            metadata,
        } => {
            let turso = TursoDB::connect().await?;
            turso
                .write_summary(&SummaryRecord {
                    date: date.clone(),
                    period: period.clone(),
                    summary_type: summary_type.clone(),
                    content,
                    metadata,
                })
                .await?;
            turso.sync().await?;
            println!(
                "{} Wrote {} {} summary for {}",
                "ok".green(),
                period,
                summary_type,
                date
            );
            Ok(())
        }

        TursoAction::Query {
            date,
            start,
            end,
            source,
            category,
            tag,
        } => {
            let turso = TursoDB::connect().await?;
            let results = turso
                .query_activities(
                    date.as_deref(),
                    start.as_deref(),
                    end.as_deref(),
                    source.as_deref(),
                    category.as_deref(),
                    tag.as_deref(),
                )
                .await?;
            println!("{}", serde_json::to_string_pretty(&results)?);
            eprintln!("\n({} rows)", results.len());
            Ok(())
        }

        TursoAction::QuerySummaries {
            date,
            period,
            summary_type,
        } => {
            let turso = TursoDB::connect().await?;
            let results = turso
                .query_summaries(date.as_deref(), period.as_deref(), summary_type.as_deref())
                .await?;
            println!("{}", serde_json::to_string_pretty(&results)?);
            eprintln!("\n({} rows)", results.len());
            Ok(())
        }

        TursoAction::Sync => {
            let turso = TursoDB::connect().await?;
            turso.sync().await?;
            println!("{} Synced to Turso", "ok".green());
            Ok(())
        }

        TursoAction::Stats => {
            let turso = TursoDB::connect().await?;
            let (sessions, activities, summaries, goals) = turso.stats().await?;
            println!("Turso Database Stats");
            println!("  Device:     {}", turso.device_id);
            println!("  Sessions:   {}", sessions);
            println!("  Activities: {}", activities);
            println!("  Summaries:  {}", summaries);
            println!("  Goals:      {}", goals);
            Ok(())
        }
    }
}
