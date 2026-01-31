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
            println!();
            println!("{}", "Config file location:".yellow());
            println!("  {}", Config::config_path()?.display());
            println!();
            println!("{}", "Common settings:".yellow());
            println!("  general.default_employer      Default company for new entries");
            println!("  general.git_email             Your git email (for filtering commits)");
            println!("  git.enabled                   Enable git scanning");
            println!("  github.token                  GitHub personal access token");
            println!("  linear.api_key                Linear API key");
            println!("  ai.api_key                    OpenAI API key");
        }
    }

    Ok(())
}

/// Initialize config with interactive wizard
pub fn init() -> Result<()> {
    use std::io::{self, Write};

    println!("{}", "Folio Configuration Wizard".bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    let mut config = Config::load()?;

    // Git email
    let default_email = crate::integrations::git::get_git_user_email().unwrap_or_default();
    println!("{}", "Git Configuration".yellow().bold());
    print!("Git email [{}]: ", default_email.dimmed());
    io::stdout().flush()?;
    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim();
    if !email.is_empty() {
        config.general.git_email = Some(email.to_string());
    } else if !default_email.is_empty() {
        config.general.git_email = Some(default_email);
    }

    // Default employer
    println!();
    print!("Default employer (optional): ");
    io::stdout().flush()?;
    let mut employer = String::new();
    io::stdin().read_line(&mut employer)?;
    let employer = employer.trim();
    if !employer.is_empty() {
        config.general.default_employer = Some(employer.to_string());
    }

    // GitHub token
    println!();
    println!("{}", "GitHub Integration".yellow().bold());
    print!("GitHub token (optional, for PR tracking): ");
    io::stdout().flush()?;
    let mut token = String::new();
    io::stdin().read_line(&mut token)?;
    let token = token.trim();
    if !token.is_empty() {
        config.github.token = Some(token.to_string());
        config.github.enabled = true;
    }

    // Watched directories
    println!();
    println!("{}", "Git Watching".yellow().bold());
    println!("Default directories to scan: {:?}", config.git.scan_dirs);
    print!("Add additional directory (or Enter to skip): ");
    io::stdout().flush()?;
    let mut dir = String::new();
    io::stdin().read_line(&mut dir)?;
    let dir = dir.trim();
    if !dir.is_empty() {
        config.git.scan_dirs.push(std::path::PathBuf::from(dir));
    }

    // Save config
    config.save()?;

    println!();
    println!("{} Configuration saved!", "✓".green().bold());
    println!("  {}", Config::config_path()?.display());
    println!();
    println!("{}", "Next steps:".yellow());
    println!("  folio capture \"Your accomplishment\"   - Capture your first activity");
    println!("  folio sync                            - Sync from git/GitHub");
    println!("  folio config --list                   - View all settings");

    Ok(())
}
