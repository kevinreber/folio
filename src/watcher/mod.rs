use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time;

use crate::config::Config;
use crate::db::Database;
use crate::integrations::{GitScanner, git::GitScanConfig};
use crate::types::Activity;

/// Background watcher that monitors git repositories for new commits
pub struct Watcher {
    config: WatcherConfig,
    db: Arc<Mutex<Database>>,
    seen_commits: Arc<Mutex<HashSet<String>>>,
    last_scan: Arc<Mutex<DateTime<Utc>>>,
}

#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Directories to watch for git repos
    pub watch_dirs: Vec<PathBuf>,
    /// Scan interval in seconds
    pub interval_seconds: u64,
    /// Author email to filter commits
    pub author_email: Option<String>,
    /// Minimum lines changed to notify
    pub min_lines_changed: usize,
    /// Enable desktop notifications
    pub notifications: bool,
    /// Callback for new activities
    pub on_activity: Option<fn(&Activity)>,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            watch_dirs: vec![],
            interval_seconds: 300, // 5 minutes
            author_email: crate::integrations::git::get_git_user_email(),
            min_lines_changed: 10,
            notifications: true,
            on_activity: None,
        }
    }
}

impl Watcher {
    pub fn new(config: WatcherConfig, db: Database) -> Self {
        Self {
            config,
            db: Arc::new(Mutex::new(db)),
            seen_commits: Arc::new(Mutex::new(HashSet::new())),
            last_scan: Arc::new(Mutex::new(Utc::now())),
        }
    }

    /// Load configuration from app config
    pub fn from_config(app_config: &Config, db: Database) -> Self {
        let config = WatcherConfig {
            watch_dirs: app_config.git.scan_dirs.clone(),
            interval_seconds: app_config.watcher.interval_seconds,
            author_email: app_config.general.git_email.clone(),
            min_lines_changed: app_config.git.min_lines_changed,
            notifications: app_config.watcher.notifications,
            on_activity: None,
        };

        Self::new(config, db)
    }

    /// Start the watcher (blocking)
    pub async fn run(&self) -> Result<()> {
        println!("Starting Folio watcher...");
        println!("Watching directories: {:?}", self.config.watch_dirs);
        println!("Scan interval: {} seconds", self.config.interval_seconds);

        let mut interval = time::interval(time::Duration::from_secs(self.config.interval_seconds));

        // Initial scan to populate seen commits
        self.initial_scan().await?;

        loop {
            interval.tick().await;
            if let Err(e) = self.scan_and_capture().await {
                eprintln!("Watcher scan error: {}", e);
            }
        }
    }

    /// Run a single scan
    pub async fn scan_once(&self) -> Result<Vec<Activity>> {
        self.scan_and_capture().await
    }

    /// Initial scan to populate seen commits without creating activities
    async fn initial_scan(&self) -> Result<()> {
        let git_config = GitScanConfig {
            scan_dirs: self.config.watch_dirs.clone(),
            max_depth: 3,
            days_back: 7, // Only look at recent commits for initial scan
            author_email: self.config.author_email.clone(),
            min_lines_changed: 0, // Get all commits for seen list
        };

        let scanner = GitScanner::new(git_config);
        let activities = scanner.scan_all()?;

        let mut seen = self.seen_commits.lock().await;
        for activity in activities {
            if let Some(sha) = activity.metadata.get("commit_sha").and_then(|v| v.as_str()) {
                seen.insert(sha.to_string());
            }
        }

        println!("Initial scan complete. Tracking {} existing commits.", seen.len());
        Ok(())
    }

    /// Scan for new commits and create activities
    async fn scan_and_capture(&self) -> Result<Vec<Activity>> {
        let git_config = GitScanConfig {
            scan_dirs: self.config.watch_dirs.clone(),
            max_depth: 3,
            days_back: 1, // Only look at recent commits
            author_email: self.config.author_email.clone(),
            min_lines_changed: self.config.min_lines_changed,
        };

        let scanner = GitScanner::new(git_config);
        let activities = scanner.scan_all()?;

        let mut new_activities = Vec::new();
        let mut seen = self.seen_commits.lock().await;
        let db = self.db.lock().await;

        for activity in activities {
            let sha = activity.metadata.get("commit_sha")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if !sha.is_empty() && !seen.contains(&sha) {
                // New commit detected!
                seen.insert(sha.clone());

                // Save to database
                if let Err(e) = db.insert_activity(&activity) {
                    eprintln!("Failed to save activity: {}", e);
                    continue;
                }

                // Notify
                if self.config.notifications {
                    self.notify(&activity);
                }

                // Callback
                if let Some(callback) = self.config.on_activity {
                    callback(&activity);
                }

                new_activities.push(activity);
            }
        }

        if !new_activities.is_empty() {
            println!("Captured {} new activities", new_activities.len());
        }

        // Update last scan time
        *self.last_scan.lock().await = Utc::now();

        Ok(new_activities)
    }

    fn notify(&self, activity: &Activity) {
        // Print to console for now
        // In a full implementation, this would send desktop notifications
        println!("\n📝 New activity captured!");
        println!("   Title: {}", activity.title);
        if let Some(project) = &activity.project {
            println!("   Project: {}", project);
        }
        println!("   Importance: {}", activity.importance);
        println!();
    }

    /// Get watcher status
    pub async fn status(&self) -> WatcherStatus {
        let seen = self.seen_commits.lock().await;
        let last_scan = *self.last_scan.lock().await;

        WatcherStatus {
            running: true,
            watched_dirs: self.config.watch_dirs.clone(),
            interval_seconds: self.config.interval_seconds,
            commits_tracked: seen.len(),
            last_scan,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WatcherStatus {
    pub running: bool,
    pub watched_dirs: Vec<PathBuf>,
    pub interval_seconds: u64,
    pub commits_tracked: usize,
    pub last_scan: DateTime<Utc>,
}

/// Run the watcher as a background daemon
pub async fn run_daemon(config: Config) -> Result<()> {
    let db = Database::open()?;
    let watcher = Watcher::from_config(&config, db);
    watcher.run().await
}

/// Scan once without running as daemon
pub async fn scan_once(config: Config) -> Result<Vec<Activity>> {
    let db = Database::open()?;
    let watcher = Watcher::from_config(&config, db);
    watcher.scan_once().await
}
