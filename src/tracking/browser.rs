//! Browser context capture module
//!
//! Tracks browser activity including URLs visited (with privacy controls),
//! research sessions, and documentation browsing.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{Activity, ActivitySource, ActivityType, Importance};

/// Configuration for browser tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// Enable browser tracking
    pub enabled: bool,
    /// Track only domains (not full URLs) for privacy
    pub domains_only: bool,
    /// Domains to always exclude
    pub excluded_domains: Vec<String>,
    /// Domains to always include (whitelist mode)
    pub included_domains: Vec<String>,
    /// Use whitelist mode (only track included_domains)
    pub whitelist_mode: bool,
    /// Minimum time on a page to track (seconds)
    pub min_duration_secs: u32,
    /// Categories for known sites
    pub site_categories: HashMap<String, String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        let mut site_categories = HashMap::new();
        // Development
        site_categories.insert("github.com".to_string(), "development".to_string());
        site_categories.insert("gitlab.com".to_string(), "development".to_string());
        site_categories.insert("stackoverflow.com".to_string(), "development".to_string());
        site_categories.insert("docs.rs".to_string(), "documentation".to_string());
        site_categories.insert(
            "developer.mozilla.org".to_string(),
            "documentation".to_string(),
        );
        site_categories.insert("rust-lang.org".to_string(), "documentation".to_string());
        // Communication
        site_categories.insert("slack.com".to_string(), "communication".to_string());
        site_categories.insert("discord.com".to_string(), "communication".to_string());
        site_categories.insert("mail.google.com".to_string(), "communication".to_string());
        // Productivity
        site_categories.insert("notion.so".to_string(), "documentation".to_string());
        site_categories.insert("linear.app".to_string(), "project_management".to_string());
        site_categories.insert(
            "jira.atlassian.com".to_string(),
            "project_management".to_string(),
        );

        Self {
            enabled: false,
            domains_only: true,
            excluded_domains: vec![
                "google.com/search".to_string(),
                "facebook.com".to_string(),
                "twitter.com".to_string(),
                "instagram.com".to_string(),
                "reddit.com".to_string(),
                "youtube.com".to_string(),
                "netflix.com".to_string(),
            ],
            included_domains: vec![
                "github.com".to_string(),
                "gitlab.com".to_string(),
                "stackoverflow.com".to_string(),
                "docs.rs".to_string(),
                "crates.io".to_string(),
                "developer.mozilla.org".to_string(),
                "rust-lang.org".to_string(),
            ],
            whitelist_mode: false,
            min_duration_secs: 30,
            site_categories,
        }
    }
}

/// Represents a browser session on a single page/domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSession {
    pub domain: String,
    pub path: Option<String>,
    pub title: Option<String>,
    pub category: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_secs: i64,
}

/// Browser activity tracker
pub struct BrowserTracker {
    config: BrowserConfig,
    current_session: Option<BrowserSessionBuilder>,
    sessions: Vec<BrowserSession>,
}

impl BrowserTracker {
    pub fn new(config: BrowserConfig) -> Self {
        Self {
            config,
            current_session: None,
            sessions: Vec::new(),
        }
    }

    /// Process a new URL/title observation
    pub fn observe(
        &mut self,
        url: Option<&str>,
        title: Option<&str>,
    ) -> Result<Option<BrowserSession>> {
        let domain = url.and_then(extract_domain);

        // Check if this should be tracked
        let should_track = match &domain {
            Some(d) => self.should_track_domain(d),
            None => false,
        };

        if !should_track {
            // End current session if any
            return Ok(self.end_current_session());
        }

        let domain = domain.unwrap();
        let path = if self.config.domains_only {
            None
        } else {
            url.and_then(extract_path)
        };
        let category = self.categorize_domain(&domain);

        match &mut self.current_session {
            None => {
                // Start new session
                self.current_session = Some(BrowserSessionBuilder::new(
                    domain,
                    path,
                    title.map(|s| s.to_string()),
                    category,
                ));
                Ok(None)
            }
            Some(session) => {
                if session.domain == domain && (self.config.domains_only || session.path == path) {
                    // Same page, update title if changed
                    if title.is_some() {
                        session.title = title.map(|s| s.to_string());
                    }
                    Ok(None)
                } else {
                    // Different page, end current and start new
                    let ended = self.end_current_session();
                    self.current_session = Some(BrowserSessionBuilder::new(
                        domain,
                        path,
                        title.map(|s| s.to_string()),
                        category,
                    ));
                    Ok(ended)
                }
            }
        }
    }

    /// End the current session
    fn end_current_session(&mut self) -> Option<BrowserSession> {
        if let Some(builder) = self.current_session.take() {
            let session = builder.build();
            if session.duration_secs >= self.config.min_duration_secs as i64 {
                self.sessions.push(session.clone());
                return Some(session);
            }
        }
        None
    }

    /// Flush any pending session
    pub fn flush(&mut self) -> Option<BrowserSession> {
        self.end_current_session()
    }

    /// Check if a domain should be tracked
    fn should_track_domain(&self, domain: &str) -> bool {
        let domain_lower = domain.to_lowercase();

        // Check exclusions first
        for excluded in &self.config.excluded_domains {
            if domain_lower.contains(&excluded.to_lowercase()) {
                return false;
            }
        }

        // If whitelist mode, only track included domains
        if self.config.whitelist_mode {
            for included in &self.config.included_domains {
                if domain_lower.contains(&included.to_lowercase()) {
                    return true;
                }
            }
            return false;
        }

        true
    }

    /// Categorize a domain
    fn categorize_domain(&self, domain: &str) -> String {
        let domain_lower = domain.to_lowercase();

        for (pattern, category) in &self.config.site_categories {
            if domain_lower.contains(&pattern.to_lowercase()) {
                return category.clone();
            }
        }

        // Default categorization based on common patterns
        if domain_lower.contains("docs")
            || domain_lower.contains("documentation")
            || domain_lower.contains("wiki")
            || domain_lower.contains("readme")
        {
            return "documentation".to_string();
        }

        if domain_lower.contains("github")
            || domain_lower.contains("gitlab")
            || domain_lower.contains("bitbucket")
        {
            return "development".to_string();
        }

        if domain_lower.contains("stackoverflow")
            || domain_lower.contains("stackexchange")
            || domain_lower.contains("dev.to")
        {
            return "research".to_string();
        }

        "browsing".to_string()
    }

    /// Get all sessions
    pub fn sessions(&self) -> &[BrowserSession] {
        &self.sessions
    }

    /// Take and clear all sessions
    pub fn take_sessions(&mut self) -> Vec<BrowserSession> {
        std::mem::take(&mut self.sessions)
    }

    /// Convert a browser session to an activity
    pub fn session_to_activity(&self, session: &BrowserSession) -> Activity {
        let now = Utc::now();

        let title = session
            .title
            .clone()
            .unwrap_or_else(|| format!("Browsed {}", session.domain));

        Activity {
            id: uuid::Uuid::new_v4().to_string(),
            source: ActivitySource::Browser,
            activity_type: ActivityType::BrowserSession,
            timestamp: session.start_time,
            title,
            description: Some(format!(
                "Spent {} on {}",
                format_duration(session.duration_secs),
                session.domain
            )),
            project: None, // Will be detected by ProjectDetector
            employer: None,
            importance: self.session_importance(session),
            metadata: serde_json::json!({
                "domain": session.domain,
                "path": session.path,
                "category": session.category,
                "duration_secs": session.duration_secs,
                "start_time": session.start_time.to_rfc3339(),
                "end_time": session.end_time.to_rfc3339(),
            }),
            created_at: now,
            updated_at: now,
        }
    }

    /// Calculate importance based on session characteristics
    fn session_importance(&self, session: &BrowserSession) -> Importance {
        // Development and documentation browsing is more important
        match session.category.as_str() {
            "development" | "documentation" | "research" => {
                if session.duration_secs > 300 {
                    Importance::High
                } else {
                    Importance::Medium
                }
            }
            "project_management" => Importance::Medium,
            _ => Importance::Low,
        }
    }

    /// Get summary by category
    pub fn get_category_summary(&self) -> HashMap<String, Duration> {
        let mut summary: HashMap<String, Duration> = HashMap::new();

        for session in &self.sessions {
            *summary.entry(session.category.clone()).or_default() +=
                Duration::seconds(session.duration_secs);
        }

        summary
    }
}

/// Builder for browser sessions
struct BrowserSessionBuilder {
    domain: String,
    path: Option<String>,
    title: Option<String>,
    category: String,
    start_time: DateTime<Utc>,
}

impl BrowserSessionBuilder {
    fn new(domain: String, path: Option<String>, title: Option<String>, category: String) -> Self {
        Self {
            domain,
            path,
            title,
            category,
            start_time: Utc::now(),
        }
    }

    fn build(self) -> BrowserSession {
        let end_time = Utc::now();
        let duration_secs = (end_time - self.start_time).num_seconds();

        BrowserSession {
            domain: self.domain,
            path: self.path,
            title: self.title,
            category: self.category,
            start_time: self.start_time,
            end_time,
            duration_secs,
        }
    }
}

/// Extract domain from URL
fn extract_domain(url: &str) -> Option<String> {
    let url = url.trim();

    // Handle protocol
    let without_protocol = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    // Extract domain (up to first /)
    let domain = without_protocol.split('/').next()?;

    // Remove port if present
    let domain = domain.split(':').next()?;

    if domain.is_empty() {
        None
    } else {
        Some(domain.to_string())
    }
}

/// Extract path from URL
fn extract_path(url: &str) -> Option<String> {
    let url = url.trim();

    let without_protocol = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);

    // Find first /
    if let Some(idx) = without_protocol.find('/') {
        let path = &without_protocol[idx..];
        // Remove query string and fragment
        let path = path.split('?').next().unwrap_or(path);
        let path = path.split('#').next().unwrap_or(path);
        if path != "/" {
            return Some(path.to_string());
        }
    }

    None
}

/// Format duration in human-readable format
fn format_duration(secs: i64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://github.com/user/repo"),
            Some("github.com".to_string())
        );
        assert_eq!(
            extract_domain("http://localhost:3000/api"),
            Some("localhost".to_string())
        );
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            extract_path("https://github.com/user/repo"),
            Some("/user/repo".to_string())
        );
        assert_eq!(
            extract_path("https://github.com/user/repo?tab=code"),
            Some("/user/repo".to_string())
        );
        assert_eq!(extract_path("https://github.com/"), None);
    }

    #[test]
    fn test_categorize_domain() {
        let tracker = BrowserTracker::new(BrowserConfig::default());

        assert_eq!(tracker.categorize_domain("github.com"), "development");
        assert_eq!(
            tracker.categorize_domain("stackoverflow.com"),
            "development"
        );
        assert_eq!(tracker.categorize_domain("docs.rs"), "documentation");
    }

    #[test]
    fn test_should_track_domain() {
        let tracker = BrowserTracker::new(BrowserConfig::default());

        assert!(tracker.should_track_domain("github.com"));
        assert!(!tracker.should_track_domain("facebook.com"));
    }
}
