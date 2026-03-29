//! Unified background daemon combining git watcher + activity tracker

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;

use crate::cli::DaemonAction;
use crate::config::Config;
use crate::db::Database;

const PID_FILE: &str = "daemon.pid";
const LOG_FILE: &str = "daemon.log";
const PLIST_LABEL: &str = "com.folio.daemon";

/// Get the folio data directory (~/.folio/)
fn folio_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let dir = home.join(".folio");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn pid_file_path() -> Result<PathBuf> {
    Ok(folio_dir()?.join(PID_FILE))
}

fn log_file_path() -> Result<PathBuf> {
    Ok(folio_dir()?.join(LOG_FILE))
}

fn plist_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let dir = home.join("Library/LaunchAgents");
    fs::create_dir_all(&dir)?;
    Ok(dir.join(format!("{PLIST_LABEL}.plist")))
}

fn write_pid_file() -> Result<()> {
    let pid = std::process::id();
    fs::write(pid_file_path()?, pid.to_string())?;
    Ok(())
}

fn read_pid_file() -> Result<Option<u32>> {
    let path = pid_file_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&path)?;
    Ok(contents.trim().parse::<u32>().ok())
}

fn remove_pid_file() -> Result<()> {
    let path = pid_file_path()?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

fn is_process_alive(pid: u32) -> bool {
    // Use kill -0 to check if process exists without signaling it
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn send_signal(pid: u32, signal: &str) -> bool {
    Command::new("kill")
        .args([signal, &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn run_action(action: DaemonAction) -> Result<()> {
    match action {
        DaemonAction::Start => daemon_start(),
        DaemonAction::Stop => daemon_stop(),
        DaemonAction::Status => daemon_status(),
        DaemonAction::Run => daemon_run(),
        DaemonAction::Install => daemon_install(),
        DaemonAction::Uninstall => daemon_uninstall(),
    }
}

/// Start the daemon as a background process
fn daemon_start() -> Result<()> {
    // Check if already running
    if let Some(pid) = read_pid_file()? {
        if is_process_alive(pid) {
            println!("{} Daemon is already running (PID: {})", "●".green(), pid);
            return Ok(());
        }
        // Stale PID file
        remove_pid_file()?;
    }

    let exe = std::env::current_exe().context("Could not determine folio binary path")?;
    let log_path = log_file_path()?;

    let log_out = fs::File::create(&log_path)
        .with_context(|| format!("Could not create log file: {}", log_path.display()))?;
    let log_err = log_out.try_clone()?;

    let child = Command::new(exe)
        .args(["daemon", "run"])
        .stdin(Stdio::null())
        .stdout(log_out)
        .stderr(log_err)
        .spawn()
        .context("Failed to start daemon process")?;

    println!("{} Daemon started (PID: {})", "●".green(), child.id());
    println!("  {} {}", "Log:".dimmed(), log_path.display());
    println!("  {} folio daemon status", "Check with:".dimmed());
    println!("  {} folio daemon stop", "Stop with:".dimmed());

    Ok(())
}

/// Stop the running daemon
fn daemon_stop() -> Result<()> {
    let pid = match read_pid_file()? {
        Some(pid) if is_process_alive(pid) => pid,
        Some(_) => {
            remove_pid_file()?;
            println!(
                "{} Daemon is not running (stale PID file cleaned up)",
                "●".red()
            );
            return Ok(());
        }
        None => {
            println!("{} Daemon is not running", "●".red());
            return Ok(());
        }
    };

    println!("Stopping daemon (PID: {})...", pid);
    if send_signal(pid, "-TERM") {
        // Wait briefly for graceful shutdown
        for _ in 0..10 {
            if !is_process_alive(pid) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        // Force kill if still alive
        if is_process_alive(pid) {
            send_signal(pid, "-KILL");
        }
    }

    remove_pid_file()?;
    println!("{} Daemon stopped", "●".red());
    Ok(())
}

/// Show daemon status
fn daemon_status() -> Result<()> {
    let config = Config::load()?;

    println!("{}", "Folio Daemon Status".cyan().bold());
    println!("{}", "─".repeat(40).dimmed());

    match read_pid_file()? {
        Some(pid) if is_process_alive(pid) => {
            println!(
                "  {} {} (PID: {})",
                "Status:".dimmed(),
                "running".green(),
                pid
            );
        }
        Some(_) => {
            remove_pid_file()?;
            println!(
                "  {} {}",
                "Status:".dimmed(),
                "not running (stale PID cleaned)".red()
            );
        }
        None => {
            println!("  {} {}", "Status:".dimmed(), "not running".red());
        }
    }

    // Check launchd installation
    let plist = plist_path()?;
    if plist.exists() {
        println!(
            "  {} {}",
            "Launchd:".dimmed(),
            "installed (auto-starts on login)".green()
        );
    } else {
        println!("  {} {}", "Launchd:".dimmed(), "not installed".yellow());
    }

    println!();
    println!("{}", "Subsystems".cyan());
    println!("{}", "─".repeat(40).dimmed());

    // Git watcher status
    println!(
        "  {} {}",
        "Git watcher:".dimmed(),
        if config.git.enabled {
            format!(
                "enabled ({} dirs, {}s interval)",
                config.git.scan_dirs.len(),
                config.watcher.interval_seconds
            )
            .green()
            .to_string()
        } else {
            "disabled".red().to_string()
        }
    );

    // Activity tracker status
    println!(
        "  {} {}",
        "Activity tracker:".dimmed(),
        if config.active_window.enabled {
            format!(
                "enabled ({}s poll interval)",
                config.active_window.poll_interval_secs
            )
            .green()
            .to_string()
        } else {
            "disabled".red().to_string()
        }
    );

    // Log file
    let log_path = log_file_path()?;
    if log_path.exists() {
        let metadata = fs::metadata(&log_path)?;
        let size_kb = metadata.len() / 1024;
        println!();
        println!(
            "  {} {} ({}KB)",
            "Log file:".dimmed(),
            log_path.display(),
            size_kb
        );

        // Show last few lines of log
        if metadata.len() > 0 {
            let content = fs::read_to_string(&log_path)?;
            let lines: Vec<&str> = content.lines().collect();
            let start = lines.len().saturating_sub(3);
            if !lines[start..].is_empty() {
                println!("  {}", "Recent log:".dimmed());
                for line in &lines[start..] {
                    println!("    {}", line.dimmed());
                }
            }
        }
    }

    Ok(())
}

/// Run the daemon in the foreground (called by `daemon start` and launchd)
fn daemon_run() -> Result<()> {
    let config = Config::load()?;
    let db = Database::open()?;

    // Write PID file
    write_pid_file()?;

    // Ensure PID file is cleaned up on exit
    let cleanup = PidFileGuard;

    println!("Folio daemon starting...");

    let git_enabled = config.git.enabled && !config.git.scan_dirs.is_empty();
    let tracker_enabled = config.active_window.enabled;

    if !git_enabled && !tracker_enabled {
        println!(
            "{}",
            "Warning: Both git watcher and activity tracker are disabled.".yellow()
        );
        println!("Enable at least one in ~/.folio/config.toml");
        drop(cleanup);
        return Ok(());
    }

    if git_enabled {
        println!(
            "  Git watcher: enabled ({} dirs)",
            config.git.scan_dirs.len()
        );
    }
    if tracker_enabled {
        println!("  Activity tracker: enabled");
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { run_daemon_async(config, db, git_enabled, tracker_enabled).await })?;

    drop(cleanup);
    Ok(())
}

async fn run_daemon_async(
    config: Config,
    db: Database,
    git_enabled: bool,
    tracker_enabled: bool,
) -> Result<()> {
    let db = Arc::new(tokio::sync::Mutex::new(db));

    let mut tasks: Vec<tokio::task::JoinHandle<Result<()>>> = Vec::new();

    // Spawn git watcher task
    if git_enabled {
        let watcher_config = config.clone();
        tasks.push(tokio::spawn(async move {
            run_git_watcher(watcher_config).await
        }));
    }

    // Spawn activity tracker task
    if tracker_enabled {
        let tracker_config = config.clone();
        let tracker_db = db.clone();
        tasks.push(tokio::spawn(async move {
            run_activity_tracker(tracker_config, tracker_db).await
        }));
    }

    // Wait for shutdown signal or any task to complete
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("\nReceived shutdown signal, stopping daemon...");
        }
        result = async {
            // Also handle SIGTERM
            let mut sigterm = tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::terminate()
            ).expect("Failed to register SIGTERM handler");
            sigterm.recv().await
        } => {
            let _ = result;
            println!("\nReceived SIGTERM, stopping daemon...");
        }
    }

    // Abort all tasks
    for task in &tasks {
        task.abort();
    }

    Ok(())
}

async fn run_git_watcher(config: Config) -> Result<()> {
    // The watcher opens its own DB connection internally via from_config
    let db = Database::open()?;
    let watcher = crate::watcher::Watcher::from_config(&config, db);
    watcher.run().await
}

async fn run_activity_tracker(config: Config, db: Arc<tokio::sync::Mutex<Database>>) -> Result<()> {
    use crate::tracking::ActiveWindowTracker;

    let mut tracker = ActiveWindowTracker::new(config.active_window.clone());
    let poll_interval = config.active_window.poll_interval_secs;

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(poll_interval));

    loop {
        interval.tick().await;
        if let Ok(Some(session)) = tracker.poll() {
            let activity = tracker.session_to_activity(&session);
            let db_guard = db.lock().await;
            if let Err(e) = db_guard.insert_activity(&activity) {
                eprintln!("Failed to save tracked activity: {}", e);
            } else {
                println!(
                    "Tracked: {} - {} ({} secs)",
                    session.app_name, session.category, session.duration_secs
                );
            }
        }
    }
}

/// RAII guard that removes the PID file when dropped
struct PidFileGuard;

impl Drop for PidFileGuard {
    fn drop(&mut self) {
        let _ = remove_pid_file();
    }
}

/// Install macOS launchd plist for auto-start
fn daemon_install() -> Result<()> {
    #[cfg(not(target_os = "macos"))]
    {
        anyhow::bail!("Launchd installation is only supported on macOS");
    }

    #[cfg(target_os = "macos")]
    {
        let exe = std::env::current_exe().context("Could not determine folio binary path")?;
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let plist = plist_path()?;
        let log_path = log_file_path()?;

        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{PLIST_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>daemon</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}</string>
    <key>StandardErrorPath</key>
    <string>{}</string>
    <key>WorkingDirectory</key>
    <string>{}</string>
</dict>
</plist>"#,
            exe.display(),
            log_path.display(),
            log_path.display(),
            home.display(),
        );

        fs::write(&plist, plist_content)?;

        // Load the plist
        let status = Command::new("launchctl")
            .args(["load", &plist.to_string_lossy()])
            .status()
            .context("Failed to run launchctl")?;

        if status.success() {
            println!("{} Daemon installed and started", "✓".green().bold());
            println!("  {} {}", "Plist:".dimmed(), plist.display());
            println!("  {}", "The daemon will auto-start on login.".dimmed());
        } else {
            println!("{} Plist written but launchctl load failed", "!".yellow());
            println!("  Try manually: launchctl load {}", plist.display());
        }

        Ok(())
    }
}

/// Uninstall the launchd plist
fn daemon_uninstall() -> Result<()> {
    #[cfg(not(target_os = "macos"))]
    {
        anyhow::bail!("Launchd installation is only supported on macOS");
    }

    #[cfg(target_os = "macos")]
    {
        let plist = plist_path()?;

        if !plist.exists() {
            println!("{} Daemon is not installed", "●".yellow());
            return Ok(());
        }

        // Unload the plist
        let _ = Command::new("launchctl")
            .args(["unload", &plist.to_string_lossy()])
            .status();

        fs::remove_file(&plist)?;

        // Also stop the daemon if running
        if let Some(pid) = read_pid_file()? {
            if is_process_alive(pid) {
                send_signal(pid, "-TERM");
            }
            remove_pid_file()?;
        }

        println!("{} Daemon uninstalled", "✓".green().bold());
        println!(
            "  {}",
            "The daemon will no longer auto-start on login.".dimmed()
        );

        Ok(())
    }
}
