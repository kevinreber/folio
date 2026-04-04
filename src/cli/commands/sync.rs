use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

use crate::config::Config;
use crate::db::Database;
use crate::integrations::{
    claude_code::ClaudeCodeConfig, git::GitScanConfig, ClaudeCodeScanner, GitHubClient, GitScanner,
    LinearClient,
};
use crate::turso::{SessionRecord, TursoDB};
use crate::types::Activity;

pub fn run(source: Option<String>, days: u32, repo: Option<PathBuf>, dry_run: bool) -> Result<()> {
    let config = Config::load()?;
    let db = Database::open()?;

    let source = source.unwrap_or_else(|| "all".to_string());
    let mut total_imported = 0;

    println!("{} Syncing from {}", "↻".cyan().bold(), source);
    println!("{}", "─".repeat(50).dimmed());

    match source.as_str() {
        "git" | "all" => {
            let count = sync_git(&config, &db, days, repo.clone(), dry_run)?;
            total_imported += count;
        }
        _ => {}
    }

    if source == "github" || source == "all" {
        if config.github.enabled && config.github.token.is_some() {
            let count = sync_github(&config, &db, days, dry_run)?;
            total_imported += count;
        } else if source == "github" {
            println!(
                "{}",
                "GitHub not configured. Run `folio config github.token <token>`".yellow()
            );
        }
    }

    if source == "linear" || source == "all" {
        if config.linear.enabled && config.linear.api_key.is_some() {
            let count = sync_linear(&config, &db, days, dry_run)?;
            total_imported += count;
        } else if source == "linear" {
            println!(
                "{}",
                "Linear not configured. Run `folio config linear.api_key <key>`".yellow()
            );
        }
    }

    if source == "claude" || source == "all" {
        let count = sync_claude_code(&db, days, dry_run)?;
        total_imported += count;
    }

    println!();
    println!("{}", "─".repeat(50).dimmed());

    if dry_run {
        println!(
            "{} Would import {} activities (dry run)",
            "✓".yellow().bold(),
            total_imported
        );
    } else {
        println!(
            "{} Imported {} activities",
            "✓".green().bold(),
            total_imported
        );
    }

    Ok(())
}

fn sync_git(
    config: &Config,
    db: &Database,
    days: u32,
    repo: Option<PathBuf>,
    dry_run: bool,
) -> Result<usize> {
    println!();
    println!("{}", "Git repositories".yellow().bold());

    let (scan_dirs, author_email, min_lines) = if let Some(r) = repo {
        // When scanning an explicit repo, don't filter by email or line count
        (vec![r], None, 1)
    } else if !config.git.email_tags.is_empty() {
        // Device-based scanning: scan all commits, tag by email
        (
            config.git.scan_dirs.clone(),
            None,
            config.git.min_lines_changed,
        )
    } else {
        // Legacy: filter by single email
        (
            config.git.scan_dirs.clone(),
            config.general.git_email.clone(),
            config.git.min_lines_changed,
        )
    };

    let git_config = GitScanConfig {
        scan_dirs,
        max_depth: config.git.max_depth,
        days_back: days,
        author_email,
        min_lines_changed: min_lines,
        email_tags: config.git.email_tags.clone(),
    };

    let scanner = GitScanner::new(git_config);

    // Discover repos
    let repos = scanner.discover_repos()?;
    println!("  Found {} repositories", repos.len());

    // Scan for commits
    let activities = scanner.scan_all()?;
    println!("  Found {} commits in last {} days", activities.len(), days);

    if dry_run {
        for activity in activities.iter().take(5) {
            println!("    {} {}", "•".dimmed(), activity.title);
        }
        if activities.len() > 5 {
            println!("    {} ... and {} more", "".dimmed(), activities.len() - 5);
        }
        return Ok(activities.len());
    }

    // Import new activities
    let mut imported = 0;
    for activity in activities {
        // Check if already exists (by commit SHA)
        let sha = activity
            .metadata
            .get("commit_sha")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if !sha.is_empty() && !db.activity_exists_by_metadata("commit_sha", sha)? {
            db.insert_activity(&activity)?;
            imported += 1;
        }
    }

    println!("  {} Imported {} new commits", "✓".green(), imported);
    Ok(imported)
}

fn sync_github(config: &Config, db: &Database, days: u32, dry_run: bool) -> Result<usize> {
    println!();
    println!("{}", "GitHub".yellow().bold());

    let token = config.github.token.clone().unwrap();
    let client = GitHubClient::new(Some(token));

    // Get user info
    let user = client.get_user()?;
    println!("  Authenticated as: {}", user.login);

    let mut all_activities = Vec::new();

    // Fetch from configured repositories
    for repo_spec in &config.github.repositories {
        let parts: Vec<&str> = repo_spec.split('/').collect();
        if parts.len() != 2 {
            continue;
        }

        let (owner, repo) = (parts[0], parts[1]);
        println!("  Fetching from {}/{}...", owner, repo);

        // Get merged PRs
        if let Ok(prs) = client.get_merged_prs(owner, repo, days) {
            for pr in prs {
                if pr.user.login == user.login {
                    all_activities.push(client.pr_to_activity(&pr));
                }
            }
        }

        // Get closed issues
        if let Ok(issues) = client.get_closed_issues(owner, repo, days) {
            for issue in issues {
                if issue.user.login == user.login {
                    all_activities.push(client.issue_to_activity(&issue, repo));
                }
            }
        }
    }

    println!("  Found {} activities", all_activities.len());

    if dry_run {
        for activity in all_activities.iter().take(5) {
            println!("    {} {}", "•".dimmed(), activity.title);
        }
        return Ok(all_activities.len());
    }

    // Import new activities
    let mut imported = 0;
    for activity in all_activities {
        let pr_number = activity
            .metadata
            .get("pr_number")
            .and_then(|v| v.as_u64())
            .map(|n| n.to_string())
            .unwrap_or_default();

        if pr_number.is_empty() || !db.activity_exists_by_metadata("pr_number", &pr_number)? {
            db.insert_activity(&activity)?;
            imported += 1;
        }
    }

    println!("  {} Imported {} new activities", "✓".green(), imported);
    Ok(imported)
}

fn sync_claude_code(db: &Database, days: u32, dry_run: bool) -> Result<usize> {
    println!();
    println!("{}", "Claude Code".yellow().bold());

    let config = ClaudeCodeConfig {
        days_back: days,
        ..Default::default()
    };

    if !config.history_path.exists() {
        println!(
            "  {}",
            "No history.jsonl found. Claude Code history not available.".dimmed()
        );
        return Ok(0);
    }

    let scanner = ClaudeCodeScanner::new(config);
    let activities = scanner.scan()?;
    println!(
        "  Found {} sessions in last {} days",
        activities.len(),
        days
    );

    if dry_run {
        for activity in activities.iter().take(5) {
            println!("    {} {}", "•".dimmed(), activity.title);
        }
        if activities.len() > 5 {
            println!("    {} ... and {} more", "".dimmed(), activities.len() - 5);
        }
        return Ok(activities.len());
    }

    // Connect to Turso for dual-write
    let rt = tokio::runtime::Runtime::new()?;
    let turso = rt.block_on(TursoDB::connect())?;

    // Import new activities
    let mut imported = 0;
    for activity in &activities {
        let session_id = activity
            .metadata
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if session_id.is_empty() {
            continue;
        }

        // Write to local Folio DB (backward compat)
        if !db.activity_exists_by_metadata("session_id", session_id)? {
            db.insert_activity(activity)?;
        }

        // Write to Turso
        let record = SessionRecord {
            session_id: session_id.to_string(),
            date: activity.timestamp.format("%Y-%m-%d").to_string(),
            project: activity.project.clone().unwrap_or_default(),
            project_path: activity
                .metadata
                .get("project_path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            prompt_count: activity
                .metadata
                .get("prompt_count")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            duration_minutes: activity
                .metadata
                .get("duration_minutes")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            is_personal: activity
                .metadata
                .get("is_personal")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            device_id: String::new(), // filled by TursoDB
            first_prompt_at: activity.timestamp.to_rfc3339(),
            last_prompt_at: activity.updated_at.to_rfc3339(),
            metadata: Some(activity.metadata.to_string()),
            created_at: None,
        };
        rt.block_on(turso.write_session(&record))?;
        imported += 1;
    }

    rt.block_on(turso.sync())?;

    println!("  {} Imported {} new sessions", "✓".green(), imported);
    Ok(imported)
}

fn sync_linear(config: &Config, db: &Database, days: u32, dry_run: bool) -> Result<usize> {
    println!();
    println!("{}", "Linear".yellow().bold());

    let api_key = config.linear.api_key.clone().unwrap();
    let client = LinearClient::new(api_key);

    // Get user info
    let viewer = client.get_viewer()?;
    println!("  Authenticated as: {}", viewer.name);

    // Get completed issues
    let issues = client.get_completed_issues(days)?;
    println!("  Found {} completed issues", issues.len());

    let activities: Vec<Activity> = issues
        .iter()
        .map(|issue| client.issue_to_activity(issue))
        .collect();

    if dry_run {
        for activity in activities.iter().take(5) {
            println!("    {} {}", "•".dimmed(), activity.title);
        }
        return Ok(activities.len());
    }

    // Import new activities
    let mut imported = 0;
    for activity in activities {
        let issue_id = activity
            .metadata
            .get("issue_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if !issue_id.is_empty() && !db.activity_exists_by_metadata("issue_id", issue_id)? {
            db.insert_activity(&activity)?;
            imported += 1;
        }
    }

    println!("  {} Imported {} new activities", "✓".green(), imported);
    Ok(imported)
}
