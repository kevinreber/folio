use anyhow::Result;
use colored::Colorize;

use crate::config::Config;

pub fn run(key: Option<String>, value: Option<String>, list: bool) -> Result<()> {
    let mut config = Config::load()?;

    if list {
        // List all config keys
        println!("{}", "Configuration Keys".bold());
        println!("{}", "─".repeat(50).dimmed());
        println!();

        for key in Config::list_keys() {
            let current = config.get(key).unwrap_or_else(|| "(not set)".to_string());
            let display = if key.contains("token") || key.contains("key") || key.contains("api_key")
            {
                if current == "(not set)" {
                    current
                } else {
                    format!("{}...", &current[..current.len().min(8)])
                }
            } else {
                current
            };
            println!("  {} = {}", key.cyan(), display.dimmed());
        }
        println!();
        println!(
            "{}",
            "Use `folio config <key> <value>` to set a value".dimmed()
        );
        return Ok(());
    }

    match (key, value) {
        (Some(k), Some(v)) => {
            // Set a value
            config.set(&k, &v)?;
            config.save()?;

            println!("{} Set {} = {}", "✓".green().bold(), k.cyan(), v);
        }
        (Some(k), None) => {
            // Get a value
            match config.get(&k) {
                Some(v) => {
                    // Mask sensitive values
                    let display =
                        if k.contains("token") || k.contains("key") || k.contains("api_key") {
                            format!("{}...", &v[..v.len().min(8)])
                        } else {
                            v
                        };
                    println!("{} = {}", k.cyan(), display);
                }
                None => {
                    println!("{} Key '{}' not found", "✗".red(), k);
                    println!();
                    println!("Available keys:");
                    for key in Config::list_keys() {
                        println!("  {}", key.dimmed());
                    }
                }
            }
        }
        (None, _) => {
            // Show help
            println!("{}", "Folio Configuration".bold());
            println!();
            println!("{}", "Usage:".yellow());
            println!("  folio config --list           List all config keys");
            println!("  folio config <key>            Get a config value");
            println!("  folio config <key> <value>    Set a config value");
            println!("  folio config --init           Interactive setup wizard");
            println!();
            println!("{}", "Config file location:".yellow());
            println!("  {}", Config::config_path()?.display());
            println!();
            println!("{}", "Common settings:".yellow());
            println!("  general.default_employer      Default company for new entries");
            println!("  general.git_email             Your git email (for filtering commits)");
            println!("  git.scan_dirs                 Directories to scan for git repos");
            println!("  git.email_tags                Email-to-tag mapping (email=tag,...)");
            println!("  github.token                  GitHub personal access token");
            println!("  linear.api_key                Linear API key");
        }
    }

    Ok(())
}

/// Initialize config with interactive wizard
pub fn init() -> Result<()> {
    use std::io::{self, Write};
    use std::path::PathBuf;

    println!();
    println!("{}", "  Welcome to Folio  ".bold().on_bright_blue().white());
    println!();
    println!("Let's set up your career tracker. This wizard will auto-discover");
    println!("your git repos, detect your identities, and configure syncing.");
    println!("{}", "─".repeat(60).dimmed());

    let mut config = Config::load()?;

    // --- Step 1: Discover scan directories ---
    println!();
    println!(
        "{} {}",
        "Step 1:".yellow().bold(),
        "Scan Directories".bold()
    );
    println!(
        "{}",
        "Which directories contain your git repositories?".dimmed()
    );
    println!();

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let candidate_dirs = vec![
        home.join("Documents").join("code"),
        home.join("code"),
        home.join("projects"),
        home.join("work"),
        home.join("dev"),
        home.join("src"),
        home.join("repos"),
    ];

    // Find dirs that exist and contain git repos
    let mut found_dirs: Vec<(PathBuf, usize)> = Vec::new();
    for dir in &candidate_dirs {
        if dir.exists() {
            let repos = crate::integrations::git::discover_repos_in(std::slice::from_ref(dir), 3)
                .unwrap_or_default();
            if !repos.is_empty() {
                found_dirs.push((dir.clone(), repos.len()));
            }
        }
    }

    if found_dirs.is_empty() {
        println!(
            "  {} No git repos found in common directories.",
            "!".yellow()
        );
    } else {
        println!("  {} Found repos in:", "✓".green());
        for (i, (dir, count)) in found_dirs.iter().enumerate() {
            println!(
                "    {} {} ({} repos)",
                format!("[{}]", i + 1).cyan(),
                dir.display(),
                count
            );
        }
    }

    println!();
    print!("  Enter directories to scan (comma-separated, Enter to accept found dirs): ");
    io::stdout().flush()?;
    let mut dirs_input = String::new();
    io::stdin().read_line(&mut dirs_input)?;
    let dirs_input = dirs_input.trim();

    if dirs_input.is_empty() {
        config.git.scan_dirs = found_dirs.iter().map(|(d, _)| d.clone()).collect();
    } else {
        config.git.scan_dirs = dirs_input
            .split(',')
            .map(|s| PathBuf::from(s.trim()))
            .filter(|p| p.exists())
            .collect();
    }

    if config.git.scan_dirs.is_empty() {
        println!(
            "  {} No valid directories selected. You can set them later with:",
            "!".yellow()
        );
        println!(
            "    {}",
            "folio config git.scan_dirs /path/to/code".dimmed()
        );
    } else {
        let total_repos: usize = config
            .git
            .scan_dirs
            .iter()
            .map(|d| {
                crate::integrations::git::discover_repos_in(std::slice::from_ref(d), 3)
                    .unwrap_or_default()
                    .len()
            })
            .sum();
        println!(
            "  {} Will scan {} directories ({} repos)",
            "✓".green(),
            config.git.scan_dirs.len(),
            total_repos
        );
    }

    // --- Step 2: Discover git emails ---
    println!();
    println!("{} {}", "Step 2:".yellow().bold(), "Git Identities".bold());
    println!(
        "{}",
        "Folio scans ALL commits on your device, then tags them by email.".dimmed()
    );
    println!();

    let emails = if !config.git.scan_dirs.is_empty() {
        println!("  Scanning for git emails...");
        crate::integrations::git::discover_emails(&config.git.scan_dirs, 3).unwrap_or_default()
    } else {
        vec![]
    };

    if emails.is_empty() {
        println!("  {} No emails found in repos.", "!".yellow());
    } else {
        println!("  {} Found {} email identities:", "✓".green(), emails.len());
        println!();

        let personal_domains = [
            "gmail.com",
            "outlook.com",
            "hotmail.com",
            "yahoo.com",
            "proton.me",
            "icloud.com",
            "me.com",
        ];

        for (email, repos) in &emails {
            let suggested = if personal_domains.iter().any(|d| email.ends_with(d)) {
                "personal"
            } else {
                "work"
            };
            let repos_str = if repos.len() <= 3 {
                repos.join(", ")
            } else {
                format!("{}, ... +{} more", repos[..2].join(", "), repos.len() - 2)
            };

            println!("    {} (found in: {})", email.cyan(), repos_str.dimmed());
            print!("    Tag as [{}]: ", suggested.yellow());
            io::stdout().flush()?;
            let mut tag_input = String::new();
            io::stdin().read_line(&mut tag_input)?;
            let tag = tag_input.trim();
            let tag = if tag.is_empty() { suggested } else { tag };

            if tag != "skip" && tag != "ignore" {
                config.git.email_tags.insert(email.clone(), tag.to_string());
            }
            println!();
        }
    }

    // --- Step 3: Default employer ---
    if config.git.email_tags.values().any(|v| v == "work") {
        println!("{} {}", "Step 3:".yellow().bold(), "Employer".bold());
        let current = config
            .general
            .default_employer
            .as_deref()
            .unwrap_or("(none)");
        print!("  Default employer [{}]: ", current.dimmed());
        io::stdout().flush()?;
        let mut employer = String::new();
        io::stdin().read_line(&mut employer)?;
        let employer = employer.trim();
        if !employer.is_empty() {
            config.general.default_employer = Some(employer.to_string());
        }
        println!();
    }

    // --- Step 4: Integrations ---
    println!(
        "{} {}",
        "Step 4:".yellow().bold(),
        "Integrations (optional)".bold()
    );
    println!();

    // GitHub
    if config.github.token.is_some() {
        println!("  {} GitHub: already configured", "✓".green());
    } else {
        print!("  GitHub token (Enter to skip): ");
        io::stdout().flush()?;
        let mut token = String::new();
        io::stdin().read_line(&mut token)?;
        let token = token.trim();
        if !token.is_empty() {
            config.github.token = Some(token.to_string());
            config.github.enabled = true;
            println!("  {} GitHub: configured", "✓".green());
        } else {
            println!("  {} GitHub: skipped", "─".dimmed());
        }
    }

    // Claude Code
    let history_path = home.join(".claude").join("history.jsonl");
    if history_path.exists() {
        println!("  {} Claude Code: history.jsonl found", "✓".green());
    } else {
        println!(
            "  {} Claude Code: no history.jsonl (install Claude Code to enable)",
            "─".dimmed()
        );
    }

    // --- Save ---
    config.save()?;

    // --- Summary ---
    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", "  Setup Complete!  ".bold().on_green().white());
    println!();
    println!("  {}", "Configuration saved to:".dimmed());
    println!("    {}", Config::config_path()?.display());
    println!();

    println!("  {}", "Scan directories:".dimmed());
    for dir in &config.git.scan_dirs {
        println!("    {}", dir.display().to_string().cyan());
    }

    if !config.git.email_tags.is_empty() {
        println!();
        println!("  {}", "Email identities:".dimmed());
        for (email, tag) in &config.git.email_tags {
            let tag_colored = match tag.as_str() {
                "personal" => tag.green().to_string(),
                "work" => tag.yellow().to_string(),
                _ => tag.dimmed().to_string(),
            };
            println!("    {} = {}", email, tag_colored);
        }
    }

    println!();
    println!("{}", "Next steps:".yellow().bold());
    println!("  folio sync                  Sync git + Claude Code activity");
    println!("  folio serve --open          Launch the web dashboard");
    println!("  folio list                  View imported activities");
    println!("  folio config --list         View all settings");

    Ok(())
}
