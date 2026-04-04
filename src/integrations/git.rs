use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use git2::{Commit, Repository};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::types::{Activity, ActivitySource, ActivityType, Importance};

/// Configuration for git scanning
#[derive(Debug, Clone)]
pub struct GitScanConfig {
    /// Directories to scan for git repos
    pub scan_dirs: Vec<PathBuf>,
    /// Maximum depth to scan for repos
    pub max_depth: usize,
    /// Number of days to look back for commits
    pub days_back: u32,
    /// Author email to filter commits (None = all commits)
    pub author_email: Option<String>,
    /// Minimum lines changed to consider significant
    pub min_lines_changed: usize,
    /// Email-to-tag mapping (e.g., "user@gmail.com" -> "personal")
    pub email_tags: HashMap<String, String>,
}

impl Default for GitScanConfig {
    fn default() -> Self {
        Self {
            scan_dirs: vec![],
            max_depth: 3,
            days_back: 30,
            author_email: None,
            min_lines_changed: 10,
            email_tags: HashMap::new(),
        }
    }
}

/// Git repository scanner for extracting commits as activities
pub struct GitScanner {
    config: GitScanConfig,
}

impl GitScanner {
    pub fn new(config: GitScanConfig) -> Self {
        Self { config }
    }

    /// Discover git repositories in configured directories
    pub fn discover_repos(&self) -> Result<Vec<PathBuf>> {
        let mut repos = Vec::new();
        let mut seen = HashSet::new();

        for scan_dir in &self.config.scan_dirs {
            if !scan_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(scan_dir)
                .max_depth(self.config.max_depth)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.ends_with(".git") && path.is_dir() {
                    if let Some(parent) = path.parent() {
                        let canonical = parent
                            .canonicalize()
                            .unwrap_or_else(|_| parent.to_path_buf());
                        if !seen.contains(&canonical) {
                            seen.insert(canonical.clone());
                            repos.push(canonical);
                        }
                    }
                }
            }
        }

        Ok(repos)
    }

    /// Scan a single repository for commits
    pub fn scan_repo(&self, repo_path: &Path) -> Result<Vec<Activity>> {
        let repo = Repository::open(repo_path)
            .with_context(|| format!("Failed to open repository at {:?}", repo_path))?;

        let mut activities = Vec::new();
        let cutoff = Utc::now() - chrono::Duration::days(self.config.days_back as i64);

        // Get the revwalk
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        // Get project name from repo path
        let project_name = repo_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        for oid in revwalk {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;

            let commit_time = Self::commit_time(&commit);
            if commit_time < cutoff {
                break; // Commits are in time order, so we can stop
            }

            // Filter by author if configured
            if let Some(ref email) = self.config.author_email {
                if let Some(author_email) = commit.author().email() {
                    if !author_email.eq_ignore_ascii_case(email) {
                        continue;
                    }
                }
            }

            // Calculate lines changed
            let stats = self.get_commit_stats(&repo, &commit);
            if stats.lines_changed < self.config.min_lines_changed {
                continue;
            }

            let activity = self.commit_to_activity(&commit, &project_name, stats);
            activities.push(activity);
        }

        Ok(activities)
    }

    /// Scan all discovered repositories
    pub fn scan_all(&self) -> Result<Vec<Activity>> {
        let repos = self.discover_repos()?;
        let mut all_activities = Vec::new();

        for repo_path in repos {
            match self.scan_repo(&repo_path) {
                Ok(activities) => all_activities.extend(activities),
                Err(e) => {
                    eprintln!("Warning: Failed to scan {:?}: {}", repo_path, e);
                }
            }
        }

        // Sort by timestamp, newest first
        all_activities.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(all_activities)
    }

    fn commit_time(commit: &Commit) -> DateTime<Utc> {
        let time = commit.time();
        Utc.timestamp_opt(time.seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now)
    }

    fn get_commit_stats(&self, repo: &Repository, commit: &Commit) -> CommitStats {
        let mut stats = CommitStats::default();

        // Get the tree for this commit
        let tree = match commit.tree() {
            Ok(t) => t,
            Err(_) => return stats,
        };

        // Get parent tree (if exists)
        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

        // Calculate diff
        let diff = match repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None) {
            Ok(d) => d,
            Err(_) => return stats,
        };

        if let Ok(diff_stats) = diff.stats() {
            stats.files_changed = diff_stats.files_changed();
            stats.insertions = diff_stats.insertions();
            stats.deletions = diff_stats.deletions();
            stats.lines_changed = stats.insertions + stats.deletions;
        }

        stats
    }

    fn commit_to_activity(&self, commit: &Commit, project: &str, stats: CommitStats) -> Activity {
        let message = commit.message().unwrap_or("No message").to_string();
        let first_line = message.lines().next().unwrap_or(&message).to_string();

        // Determine importance based on commit message and stats
        let importance = self.infer_importance(&message, &stats);

        // Look up email tag (work/personal)
        let author_sig = commit.author();
        let author_email = author_sig.email().unwrap_or("Unknown");
        let identity_tag = self
            .config
            .email_tags
            .iter()
            .find(|(email, _)| email.eq_ignore_ascii_case(author_email))
            .map(|(_, tag)| tag.clone());

        let activity = Activity {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::Git,
            activity_type: ActivityType::Commit,
            timestamp: Self::commit_time(commit),
            title: first_line.clone(),
            description: if message.lines().count() > 1 {
                Some(message.clone())
            } else {
                None
            },
            project: Some(project.to_string()),
            employer: None,
            importance,
            metadata: serde_json::json!({
                "commit_sha": commit.id().to_string(),
                "author_name": commit.author().name().unwrap_or("Unknown"),
                "author_email": author_email,
                "identity_tag": identity_tag.as_deref().unwrap_or("unknown"),
                "files_changed": stats.files_changed,
                "insertions": stats.insertions,
                "deletions": stats.deletions,
                "lines_changed": stats.lines_changed,
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        activity
    }

    fn infer_importance(&self, message: &str, stats: &CommitStats) -> Importance {
        let msg_lower = message.to_lowercase();

        // High importance indicators
        if msg_lower.contains("breaking")
            || msg_lower.contains("major")
            || msg_lower.contains("critical")
            || msg_lower.contains("security")
            || msg_lower.contains("feat:")
            || msg_lower.contains("feature:")
            || stats.lines_changed > 500
        {
            return Importance::High;
        }

        // Low importance indicators
        if msg_lower.contains("typo")
            || msg_lower.contains("fix typo")
            || msg_lower.contains("whitespace")
            || msg_lower.contains("format")
            || msg_lower.contains("chore:")
            || msg_lower.contains("docs:")
            || stats.lines_changed < 20
        {
            return Importance::Low;
        }

        Importance::Medium
    }
}

#[derive(Debug, Default)]
struct CommitStats {
    files_changed: usize,
    insertions: usize,
    deletions: usize,
    lines_changed: usize,
}

/// Scan a specific git repository
pub fn scan_repo(
    path: &Path,
    days_back: u32,
    author_email: Option<String>,
) -> Result<Vec<Activity>> {
    let config = GitScanConfig {
        scan_dirs: vec![path.to_path_buf()],
        days_back,
        author_email,
        min_lines_changed: 1, // Don't filter for explicit scans
        ..Default::default()
    };

    let scanner = GitScanner::new(config);
    scanner.scan_repo(path)
}

/// Get git configuration (user email)
pub fn get_git_user_email() -> Option<String> {
    git2::Config::open_default()
        .ok()
        .and_then(|config| config.get_string("user.email").ok())
}

/// Discover unique author emails across git repos in given directories.
/// Returns a list of (email, vec_of_repo_names) tuples.
pub fn discover_emails(
    scan_dirs: &[PathBuf],
    max_depth: usize,
) -> Result<Vec<(String, Vec<String>)>> {
    let scanner = GitScanner::new(GitScanConfig {
        scan_dirs: scan_dirs.to_vec(),
        max_depth,
        days_back: 90,
        author_email: None,
        min_lines_changed: 0,
        email_tags: HashMap::new(),
    });

    let repos = scanner.discover_repos()?;
    let mut email_repos: HashMap<String, HashSet<String>> = HashMap::new();
    let cutoff = Utc::now() - chrono::Duration::days(90);

    for repo_path in repos {
        let repo_name = repo_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let repo = match Repository::open(&repo_path) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let mut revwalk = match repo.revwalk() {
            Ok(r) => r,
            Err(_) => continue,
        };
        let _ = revwalk.push_head();
        revwalk.set_sorting(git2::Sort::TIME).ok();

        for oid in revwalk.take(200) {
            let oid = match oid {
                Ok(o) => o,
                Err(_) => break,
            };
            let commit = match repo.find_commit(oid) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let commit_time = Utc
                .timestamp_opt(commit.time().seconds(), 0)
                .single()
                .unwrap_or_else(Utc::now);
            if commit_time < cutoff {
                break;
            }

            let author = commit.author();
            if let Some(email) = author.email() {
                let email_lower = email.to_lowercase();
                email_repos
                    .entry(email_lower)
                    .or_default()
                    .insert(repo_name.clone());
            }
        }
    }

    let mut results: Vec<(String, Vec<String>)> = email_repos
        .into_iter()
        .map(|(email, repos)| {
            let mut repos: Vec<String> = repos.into_iter().collect();
            repos.sort();
            (email, repos)
        })
        .collect();
    results.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(results)
}

/// Discover git repos in given directories and return their paths.
pub fn discover_repos_in(scan_dirs: &[PathBuf], max_depth: usize) -> Result<Vec<PathBuf>> {
    let scanner = GitScanner::new(GitScanConfig {
        scan_dirs: scan_dirs.to_vec(),
        max_depth,
        ..Default::default()
    });
    scanner.discover_repos()
}
