use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::types::{Activity, ActivitySource, ActivityType, Importance};

const GITHUB_API_BASE: &str = "https://api.github.com";

/// GitHub API client for fetching PRs, issues, and activity
pub struct GitHubClient {
    client: Client,
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubPullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub user: GitHubUser,
    pub additions: Option<u64>,
    pub deletions: Option<u64>,
    pub changed_files: Option<u64>,
    pub commits: Option<u64>,
    pub base: GitHubBranch,
}

#[derive(Debug, Deserialize)]
pub struct GitHubBranch {
    pub repo: GitHubRepo,
}

#[derive(Debug, Deserialize)]
pub struct GitHubRepo {
    pub name: String,
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub user: GitHubUser,
    pub labels: Vec<GitHubLabel>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubLabel {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubReview {
    pub id: u64,
    pub user: GitHubUser,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub pull_request_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub created_at: DateTime<Utc>,
    pub repo: GitHubEventRepo,
    pub payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct GitHubEventRepo {
    pub name: String,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .user_agent("folio-cli/0.2.0")
                .build()
                .expect("Failed to create HTTP client"),
            token,
        }
    }

    fn request(&self, url: &str) -> Result<reqwest::blocking::Response> {
        let mut req = self.client.get(url);

        if let Some(ref token) = self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        req.header("Accept", "application/vnd.github.v3+json")
            .send()
            .with_context(|| format!("Failed to fetch {}", url))
    }

    /// Get the authenticated user
    pub fn get_user(&self) -> Result<GitHubUser> {
        let resp = self.request(&format!("{}/user", GITHUB_API_BASE))?;
        resp.json().context("Failed to parse user response")
    }

    /// Fetch merged PRs for a repository
    pub fn get_merged_prs(
        &self,
        owner: &str,
        repo: &str,
        days_back: u32,
    ) -> Result<Vec<GitHubPullRequest>> {
        let since = Utc::now() - chrono::Duration::days(days_back as i64);
        let url = format!(
            "{}/repos/{}/{}/pulls?state=closed&sort=updated&direction=desc&per_page=100",
            GITHUB_API_BASE, owner, repo
        );

        let resp = self.request(&url)?;
        let prs: Vec<GitHubPullRequest> = resp.json().context("Failed to parse PRs response")?;

        // Filter to merged PRs within timeframe
        Ok(prs
            .into_iter()
            .filter(|pr| pr.merged_at.map(|merged| merged > since).unwrap_or(false))
            .collect())
    }

    /// Fetch a single PR with full details
    pub fn get_pr(&self, owner: &str, repo: &str, number: u64) -> Result<GitHubPullRequest> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            GITHUB_API_BASE, owner, repo, number
        );
        let resp = self.request(&url)?;
        resp.json().context("Failed to parse PR response")
    }

    /// Fetch issues closed by the user
    pub fn get_closed_issues(
        &self,
        owner: &str,
        repo: &str,
        days_back: u32,
    ) -> Result<Vec<GitHubIssue>> {
        let since = Utc::now() - chrono::Duration::days(days_back as i64);
        let url = format!(
            "{}/repos/{}/{}/issues?state=closed&sort=updated&direction=desc&per_page=100",
            GITHUB_API_BASE, owner, repo
        );

        let resp = self.request(&url)?;
        let issues: Vec<GitHubIssue> = resp.json().context("Failed to parse issues response")?;

        Ok(issues
            .into_iter()
            .filter(|issue| {
                issue
                    .closed_at
                    .map(|closed| closed > since)
                    .unwrap_or(false)
            })
            .collect())
    }

    /// Fetch user's recent activity events
    pub fn get_user_events(&self, username: &str) -> Result<Vec<GitHubEvent>> {
        let url = format!("{}/users/{}/events?per_page=100", GITHUB_API_BASE, username);
        let resp = self.request(&url)?;
        resp.json().context("Failed to parse events response")
    }

    /// Fetch reviews submitted by user
    pub fn get_user_reviews(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<Vec<GitHubReview>> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}/reviews",
            GITHUB_API_BASE, owner, repo, pr_number
        );
        let resp = self.request(&url)?;
        resp.json().context("Failed to parse reviews response")
    }

    /// Convert a merged PR to an activity
    pub fn pr_to_activity(&self, pr: &GitHubPullRequest) -> Activity {
        let importance = self.infer_pr_importance(pr);

        Activity {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::Github,
            activity_type: ActivityType::PrMerged,
            timestamp: pr.merged_at.unwrap_or(pr.updated_at),
            title: format!("Merged PR #{}: {}", pr.number, pr.title),
            description: pr.body.clone(),
            project: Some(pr.base.repo.name.clone()),
            employer: None,
            importance,
            metadata: serde_json::json!({
                "pr_number": pr.number,
                "pr_url": pr.html_url,
                "additions": pr.additions,
                "deletions": pr.deletions,
                "changed_files": pr.changed_files,
                "commits": pr.commits,
                "repo": pr.base.repo.full_name,
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Convert an issue to an activity
    pub fn issue_to_activity(&self, issue: &GitHubIssue, repo_name: &str) -> Activity {
        let importance = self.infer_issue_importance(issue);

        Activity {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::Github,
            activity_type: ActivityType::IssueClosed,
            timestamp: issue.closed_at.unwrap_or(issue.updated_at),
            title: format!("Closed issue #{}: {}", issue.number, issue.title),
            description: issue.body.clone(),
            project: Some(repo_name.to_string()),
            employer: None,
            importance,
            metadata: serde_json::json!({
                "issue_number": issue.number,
                "issue_url": issue.html_url,
                "labels": issue.labels.iter().map(|l| &l.name).collect::<Vec<_>>(),
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn infer_pr_importance(&self, pr: &GitHubPullRequest) -> Importance {
        let title_lower = pr.title.to_lowercase();
        let lines = pr.additions.unwrap_or(0) + pr.deletions.unwrap_or(0);

        if title_lower.contains("breaking")
            || title_lower.contains("major")
            || title_lower.contains("feat")
            || lines > 500
        {
            return Importance::High;
        }

        if title_lower.contains("typo")
            || title_lower.contains("fix:")
            || title_lower.contains("chore:")
            || lines < 50
        {
            return Importance::Low;
        }

        Importance::Medium
    }

    fn infer_issue_importance(&self, issue: &GitHubIssue) -> Importance {
        let labels: Vec<&str> = issue.labels.iter().map(|l| l.name.as_str()).collect();

        if labels
            .iter()
            .any(|l| l.contains("critical") || l.contains("security") || l.contains("bug"))
        {
            return Importance::High;
        }

        if labels
            .iter()
            .any(|l| l.contains("enhancement") || l.contains("feature"))
        {
            return Importance::Medium;
        }

        Importance::Low
    }
}

/// Parse a GitHub URL to extract owner and repo
pub fn parse_github_url(url: &str) -> Option<(String, String, Option<u64>)> {
    let url = url::Url::parse(url).ok()?;

    if url.host_str() != Some("github.com") {
        return None;
    }

    let segments: Vec<&str> = url.path_segments()?.collect();

    if segments.len() >= 2 {
        let owner = segments[0].to_string();
        let repo = segments[1].to_string();

        // Check for PR or issue number
        let number = if segments.len() >= 4 && (segments[2] == "pull" || segments[2] == "issues") {
            segments[3].parse().ok()
        } else {
            None
        };

        return Some((owner, repo, number));
    }

    None
}
