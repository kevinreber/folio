use anyhow::Result;
use colored::Colorize;

use crate::config::Config;

pub fn run(daemon: bool) -> Result<()> {
    let config = Config::load()?;

    if !config.git.enabled {
        println!("{}", "Git watching is disabled in config.".yellow());
        println!("{}", "Enable with: folio config git.enabled true".dimmed());
        return Ok(());
    }

    if config.git.scan_dirs.is_empty() {
        println!("{}", "No directories configured to watch.".yellow());
        println!("{}", "Add directories in ~/.folio/config.toml".dimmed());
        return Ok(());
    }

    println!("{}", "Starting Folio watcher...".cyan());
    println!("{}", "─".repeat(50).dimmed());
    println!();
    println!("{}", "Watching directories:".yellow());
    for dir in &config.git.scan_dirs {
        if dir.exists() {
            println!("  {} {}", "✓".green(), dir.display());
        } else {
            println!("  {} {} (not found)", "✗".red(), dir.display());
        }
    }
    println!();
    println!(
        "{} {} seconds",
        "Scan interval:".dimmed(),
        config.watcher.interval_seconds
    );
    println!(
        "{} {}",
        "Notifications:".dimmed(),
        if config.watcher.notifications {
            "enabled"
        } else {
            "disabled"
        }
    );
    println!();

    if daemon {
        println!("{}", "Running as daemon... Press Ctrl+C to stop.".dimmed());
    } else {
        println!("{}", "Running single scan...".dimmed());
    }

    // Run the watcher
    let rt = tokio::runtime::Runtime::new()?;

    if daemon {
        rt.block_on(crate::watcher::run_daemon(config))?;
    } else {
        let activities = rt.block_on(crate::watcher::scan_once(config))?;
        println!();
        if activities.is_empty() {
            println!("{}", "No new activities found.".dimmed());
        } else {
            println!(
                "{} Found {} new activities",
                "✓".green().bold(),
                activities.len()
            );
            for activity in &activities {
                println!("  {} {}", "•".dimmed(), activity.title);
            }
        }
    }

    Ok(())
}
